
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
}

impl FamiliarityField {
    pub fn to_string(&self) -> String {
        match self {
            FamiliarityField::Id => "id".to_string(),
            FamiliarityField::PracticeTime => "practice_time".to_string(),
            FamiliarityField::Familiar => "familiar".to_string(),
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
            familiar BOOLEAN NOT NULL
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
