
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate::MigrateDatabase, Sqlite};
use sqlx::Row;
use std::error::Error;
use std::fs;
use csv::{ReaderBuilder, StringRecord};
use rand::seq::SliceRandom;

pub const DB_URL: &str = "sqlite:data/words_database.db";


#[derive(Debug)]
#[allow(dead_code)]
pub enum WordField {
    Id,
    Expression,
    Reading,
    Meaning,
    Tags(TagLevel),
}

#[derive(Debug)]
pub enum TagLevel {
    N1,
    N2,
    N3,
    N4,
    N5,
}

impl TagLevel {
    pub fn to_string(&self) -> String {
        match self {
            TagLevel::N1 => "n1".to_string(),
            TagLevel::N2 => "n2".to_string(),
            TagLevel::N3 => "n3".to_string(),
            TagLevel::N4 => "n4".to_string(),
            TagLevel::N5 => "n5".to_string(),
        }
    }
    pub fn from_string(tag: &str) -> Option<TagLevel> {
        match tag {
            "n1" => Some(TagLevel::N1),
            "n2" => Some(TagLevel::N2),
            "n3" => Some(TagLevel::N3),
            "n4" => Some(TagLevel::N4),
            "n5" => Some(TagLevel::N5),
            _ => None,
        }
    }
}

pub enum FamiliarityField {
    Id,
    PracticeTime,
    Familiar,
    UserMark,
}

impl FamiliarityField {
    pub fn to_string(&self) -> String {
        match self {
            FamiliarityField::Id => "id".to_string(),
            FamiliarityField::PracticeTime => "practice_time".to_string(),
            FamiliarityField::Familiar => "familiar".to_string(),
            FamiliarityField::UserMark => "user_mark".to_string(),
        }
    }
}

impl WordField {
    pub fn to_string(&self) -> String {
        match self {
            WordField::Id => "id".to_string(),
            WordField::Expression => "expression".to_string(),
            WordField::Reading => "reading".to_string(),
            WordField::Meaning => "meaning".to_string(),
            WordField::Tags(tag_level) => { 
                tag_level.to_string()
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct WordRecord {
    pub id: i64,
    pub expression: String,
    pub reading: String,
    pub meaning: String,
    pub tags: String,
}

pub fn load_csv_to_word_records(
    file_path: &str,
    records: &mut Vec<WordRecord>,
    tag: &str,
) -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().from_path(file_path)?;
    for result in rdr.records() {
        let record = result?;
        // Assuming CSV columns: expression, reading, meaning, tags
        if record.len() < 4 {
            continue;
        }
        records.push(WordRecord {
            id: 0, // ID will be auto-incremented by the database
            expression: record[0].to_string(),
            reading: record[1].to_string(),
            meaning: record[2].to_string(),
            tags: tag.to_string(),
        });
    }
    Ok(())
}


pub async fn reset_database(db_url: &str) -> Result<(), sqlx::Error> {
    // Remove the database file if it exists
    if Sqlite::database_exists(db_url).await? {
        Sqlite::drop_database(db_url).await?;
    }

    // Remove the -shm and -wal files if they exist
    let shm_file = format!("{}-shm", db_url);
    let wal_file = format!("{}-wal", db_url);

    if fs::metadata(&shm_file).is_ok() {
        fs::remove_file(&shm_file)?;
    }

    if fs::metadata(&wal_file).is_ok() {
        fs::remove_file(&wal_file)?;
    }

    // Create a new database
    Sqlite::create_database(db_url).await?;
    Ok(())
}

pub async fn create_table(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS words (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            expression TEXT NOT NULL,
            reading TEXT NOT NULL,
            meaning TEXT NOT NULL,
            tags TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS familiarity (
            id INTEGER PRIMARY KEY REFERENCES words(id) ON DELETE CASCADE,
            practice_time INTEGER NOT NULL,
            familiar BOOLEAN NOT NULL,
            user_mark BOOLEAN NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn bulk_insert_words(pool: &sqlx::SqlitePool, records: Vec<WordRecord>) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    for record in records {
        sqlx::query(
            r#"
            INSERT INTO words (expression, reading, meaning, tags)
            VALUES (?, ?, ?, ?)
            "#,)
            .bind(&record.expression)
            .bind(&record.reading)
            .bind(&record.meaning)
            .bind(&record.tags)
            .execute(&mut *transaction)
            .await?;
    }

    transaction.commit().await?;
    Ok(())
}

pub async fn find_word_ids(pool: &sqlx::SqlitePool, field: WordField, value: &str) -> Result<Vec<i64>, sqlx::Error> {
    
    let query = match field {
        WordField::Id => {
            println!("Please use find_word_by_ids instead of find_word_ids");
            return Ok(vec![]);
        }
        _ => format!("SELECT id FROM words WHERE {} = ?", field.to_string()),
    };

    let rows = sqlx::query(&query)
        .bind(value)
        .fetch_all(pool)
        .await?;

    // Collect all IDs into a Vec<i64>
    Ok(rows.iter().map(|row| row.get::<i64, _>("id")).collect())

}

pub async fn find_word_by_ids(pool: &sqlx::SqlitePool, ids: Vec<i64>) -> Result<Vec<WordRecord>, sqlx::Error> {
    let mut records = Vec::new();
    for id in ids {
        let rows = sqlx::query("SELECT * FROM words WHERE id = ?")
            .bind(id)
            .fetch_all(pool)
            .await?;
        for row in rows {
            records.push(WordRecord {
                id: row.get::<i64, _>("id"),
                expression: row.get::<String, _>("expression"),
                reading: row.get::<String, _>("reading"),
                meaning: row.get::<String, _>("meaning"),
                tags: row.get::<String, _>("tags"),
            });
        }
    }
    Ok(records)
}

pub async fn get_unfamiliar_word_ids(
    pool: &sqlx::SqlitePool,
    tag: TagLevel,
    num: usize,
    random: bool,
) -> Result<Vec<i64>, sqlx::Error> {
    // Query for word ids with the given tag that are either not in familiarity or familiar is false
    let tag_str = tag.to_string();
    let rows = sqlx::query(
        r#"
        SELECT w.id
        FROM words w
        LEFT JOIN familiarity f ON w.id = f.id
        WHERE w.tags = ? AND (f.id IS NULL OR f.familiar = 0)
        "#,
    )
    .bind(&tag_str)
    .fetch_all(pool)
    .await?;

    let mut ids: Vec<i64> = rows.iter().map(|row| row.get::<i64, _>("id")).collect();

    if random {
        let mut rng = rand::rng();
        ids.shuffle(&mut rng);
    }

    Ok(ids.into_iter().take(num).collect())
}

pub async fn get_unfamiliar_words(
    pool: &sqlx::SqlitePool,
    tag: TagLevel,
    num: usize,
    random: bool,
) -> Result<Vec<WordRecord>, sqlx::Error> {
    // Query for word ids with the given tag that are either not in familiarity or familiar is false
    let tag_str = tag.to_string();
    let query = if random {
        format!(
            r#"
            SELECT w.*
            FROM words w
            LEFT JOIN familiarity f ON w.id = f.id
            WHERE w.tags = ? AND (f.id IS NULL OR f.familiar = 0)
            ORDER BY RANDOM()
            LIMIT {}
            "#,
            num
        )
    } else {
        format!(
            r#"
            SELECT w.*
            FROM words w
            LEFT JOIN familiarity f ON w.id = f.id
            WHERE w.tags = ? AND (f.id IS NULL OR f.familiar = 0)
            LIMIT {}
            "#,
            num
        )
    };
    

    let rows = sqlx::query(&query)
        .bind(&tag_str)
        .fetch_all(pool)
        .await?;

    let records = rows
        .into_iter()
        .map(|row| WordRecord {
            id: row.get::<i64, _>("id"),
            expression: row.get::<String, _>("expression"),
            reading: row.get::<String, _>("reading"),
            meaning: row.get::<String, _>("meaning"),
            tags: row.get::<String, _>("tags"),
        })
        .collect();
    eprintln!("get_unfamiliar_words function called");
    Ok(records)
}

// this function is not needed, update_familiarity will handle both insert and update
pub async fn increment_practice_time(pool: &sqlx::SqlitePool, word_id: i64) -> Result<(), sqlx::Error> {
    // Try to update
    let result = sqlx::query("UPDATE familiarity SET practice_time = practice_time + 1 WHERE id = ?")
        .bind(word_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        // If no row was updated, insert a new one with practice_time = 1 and familiar = false (0)
        sqlx::query("INSERT INTO familiarity (id, practice_time, familiar) VALUES (?, 1, 0)")
            .bind(word_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn update_familiar(pool: &sqlx::SqlitePool, word_id: i64, familiar: bool) -> Result<(), sqlx::Error> {
    let result = sqlx::query("UPDATE familiarity SET familiar = ?, practice_time = practice_time + 1 WHERE id = ?")
        .bind(familiar)
        .bind(word_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        // If no row was updated, insert a new one
        sqlx::query("INSERT INTO familiarity (id, practice_time, familiar) VALUES (?, 1, ?)")
            .bind(word_id)
            .bind(familiar)
            .execute(pool)
            .await?;
    }


    Ok(())
}


pub async fn mark_word(pool: &sqlx::SqlitePool, word_id: i64) -> Result<(), sqlx::Error> {
    // Attempt to update the user_mark for an existing record.
    let result = sqlx::query("UPDATE familiarity SET user_mark = 1 WHERE id = ?")
        .bind(word_id)
        .execute(pool)
        .await?;

    // If no record was updated, we need to insert one.
    if result.rows_affected() == 0 {
        // Insert a new record with default values for practice/familiarity,
        // but with `user_mark` explicitly set to true (1).
        sqlx::query(
            "INSERT INTO familiarity (id, practice_time, familiar, user_mark) VALUES (?, 0, 0, 1)",
        )
        .bind(word_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn return_marked_words(
    pool: &sqlx::SqlitePool,
    tag_level: Option<TagLevel>,
) -> Result<Vec<WordRecord>, sqlx::Error> {
    let query_string = match tag_level {
        Some(_) => {
            r#"
                SELECT w.id, w.expression, w.reading, w.meaning, w.tags
                FROM words w
                INNER JOIN familiarity f ON w.id = f.id
                WHERE f.user_mark = 1 AND w.tags = ?
            "#
        }
        None => {
            r#"
                SELECT w.id, w.expression, w.reading, w.meaning, w.tags
                FROM words w
                INNER JOIN familiarity f ON w.id = f.id
                WHERE f.user_mark = 1
            "#
        }
    };

    // Start building the query
    let mut query = sqlx::query(query_string);

    // Bind the tag_level value only if it exists
    if let Some(tag) = tag_level {
        query = query.bind(tag.to_string());
    }

    // Fetch all rows from the database
    let rows = query.fetch_all(pool).await?;

    // Manually map each row to a WordRecord struct
    let records = rows
        .into_iter()
        .map(|row| WordRecord {
            id: row.get("id"),
            expression: row.get("expression"),
            reading: row.get("reading"),
            meaning: row.get("meaning"),
            tags: row.get("tags"),
        })
        .collect();

    Ok(records)
}




pub async fn reset_familiarity(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM familiarity"
    )
    .execute(pool)
    .await?;

    // Optional: Reclaim disk space in SQLite.
    // For most applications this is not necessary, but if you have a very large
    // database and are concerned about file size, you can run VACUUM.
    // sqlx::query("VACUUM").execute(pool).await?;

    Ok(())
}




pub async fn update_word_field(pool: &sqlx::SqlitePool, ids: Vec<i64>, field: WordField, new_value: &str) -> Result<(), sqlx::Error> {
     
     match field {
        WordField::Id => {
            println!("Updating ID field is not allowed.");
            return Err(sqlx::Error::InvalidArgument("Cannot update ID field".into()));
        }
        _ => {}
    }
    let query = format!(r#"UPDATE words SET {} = ? WHERE id = ?"#, field.to_string());
    for id in ids {
        sqlx::query(&query)
            .bind(new_value)
            .bind(id)
            .execute(pool)
            .await?;
    }
    
    Ok(())
}

pub async fn delete_words(pool: &sqlx::SqlitePool, ids: Vec<i64>) -> Result<(), sqlx::Error> {
    for id in ids {
        sqlx::query("DELETE FROM words WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn insert_words(pool: &sqlx::SqlitePool, records: Vec<WordRecord>) -> Result<(), sqlx::Error> {
    for record in records {
        sqlx::query(
            r#"
            INSERT INTO words (expression, reading, meaning, tags)
            VALUES (?, ?, ?, ?)
            "#,)
            .bind(&record.expression)
            .bind(&record.reading)
            .bind(&record.meaning)
            .bind(&record.tags)
            .execute(pool)
            .await?;
    }
    Ok(())
}




#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePool;

    /// Helper function to set up an in-memory database for testing.
    async fn setup_test_db() -> SqlitePool {
        // Use an in-memory SQLite database for fast, isolated tests.
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory db pool.");

        // 3.1 & 3.2: Reset and create tables
        create_table(&pool)
            .await
            .expect("Failed to create tables.");

        pool
    }

    /// Helper function to create fake word data.
    fn create_fake_data() -> Vec<WordRecord> {
        vec![
            WordRecord { id: 0, expression: "一".to_string(), reading: "いち".to_string(), meaning: "one".to_string(), tags: "n5".to_string() },
            WordRecord { id: 0, expression: "二".to_string(), reading: "に".to_string(), meaning: "two".to_string(), tags: "n5".to_string() },
            WordRecord { id: 0, expression: "時間".to_string(), reading: "じかん".to_string(), meaning: "time".to_string(), tags: "n4".to_string() },
            WordRecord { id: 0, expression: "経済".to_string(), reading: "けいざい".to_string(), meaning: "economy".to_string(), tags: "n1".to_string() },
            WordRecord { id: 0, expression: "政治".to_string(), reading: "せいじ".to_string(), meaning: "politics".to_string(), tags: "n1".to_string() },
        ]
    }

    #[tokio::test]
    async fn test_database_operations_in_sequence() {
        // 3.1 & 3.2: Setup the database
        println!("STEP 1 & 2: Setting up the database and creating tables...");
        let pool = setup_test_db().await;

        // 3.3: Make up some fake data and insert it
        println!("STEP 3: Inserting fake data...");
        let fake_data = create_fake_data();
        bulk_insert_words(&pool, fake_data)
            .await
            .expect("Failed to bulk insert fake data.");

        // Check that 5 words were inserted
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM words")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 5);
        println!("-> Verified: 5 words inserted.");

        // 3.4: Test all DB functions
        println!("STEP 4: Testing all DB functions...");

        // Test `update_familiar`: Make word with id=1 familiar.
        // This will create a familiarity record for it.
        println!("  - Testing update_familiar...");
        update_familiar(&pool, 1, true).await.unwrap();
        let familiar_status: bool = sqlx::query_scalar("SELECT familiar FROM familiarity WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(familiar_status, "Word 1 should be familiar.");
        println!("  -> Verified: update_familiar works.");

        // Test `mark_word` on a word that already has a familiarity record (id=1)
        println!("  - Testing mark_word (on existing familiarity record)...");
        mark_word(&pool, 1).await.unwrap();
        let mark_status: bool = sqlx::query_scalar("SELECT user_mark FROM familiarity WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(mark_status, "Word 1 should be marked.");
        println!("  -> Verified: mark_word (update) works.");
        
        // Test `mark_word` on a word with no familiarity record (id=3)
        println!("  - Testing mark_word (on new familiarity record)...");
        mark_word(&pool, 3).await.unwrap();
        let mark_status_3: bool = sqlx::query_scalar("SELECT user_mark FROM familiarity WHERE id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(mark_status_3, "Word 3 should be marked.");
         let practice_time_3: i64 = sqlx::query_scalar("SELECT practice_time FROM familiarity WHERE id = 3")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(practice_time_3, 0, "Practice time should be 0 for new marked word.");
        println!("  -> Verified: mark_word (insert) works.");

        // Mark another word (id=4, "経済") for the next test
        mark_word(&pool, 4).await.unwrap();

        // Test `return_marked_words` with None (should return all marked words: 1, 3, 4)
        println!("  - Testing return_marked_words(None)...");
        let all_marked = return_marked_words(&pool, None).await.unwrap();
        assert_eq!(all_marked.len(), 3);
        // Check if the IDs are correct, regardless of order
        let marked_ids: Vec<i64> = all_marked.iter().map(|w| w.id).collect();
        assert!(marked_ids.contains(&1));
        assert!(marked_ids.contains(&3));
        assert!(marked_ids.contains(&4));
        println!("  -> Verified: return_marked_words(None) returns 3 words.");

        // Test `return_marked_words` with Some(TagLevel::N1) (should return word 4: "経済")
        println!("  - Testing return_marked_words(Some(N1))...");
        let n1_marked = return_marked_words(&pool, Some(TagLevel::N1)).await.unwrap();
        assert_eq!(n1_marked.len(), 1);
        assert_eq!(n1_marked[0].id, 4);
        assert_eq!(n1_marked[0].expression, "経済");
        println!("  -> Verified: return_marked_words(Some(N1)) returns 1 word.");
        
        // Test `get_unfamiliar_word_ids` to ensure it correctly identifies unfamiliar words
        println!("  - Testing get_unfamiliar_word_ids(N5)...");
        // Word 1 (id=1) is familiar. Word 2 (id=2) is unfamiliar. Words 3,4,5 are not N5.
        let unfamiliar_n5 = get_unfamiliar_word_ids(&pool, TagLevel::N5, 2, false).await.unwrap();
        assert_eq!(unfamiliar_n5.len(), 1, "Should only be one unfamiliar N5 word.");
        assert_eq!(unfamiliar_n5[0], 2);
        println!("  -> Verified: get_unfamiliar_word_ids works correctly.");

        // Test `reset_familiarity`
        println!("  - Testing reset_familiarity...");
        reset_familiarity(&pool).await.unwrap();
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM familiarity")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "Familiarity table should be empty after reset.");
        println!("  -> Verified: reset_familiarity works.");

        println!("\nALL TESTS PASSED!");
    }
}
