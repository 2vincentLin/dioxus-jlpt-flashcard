use sqlx::{migrate::MigrateDatabase, Sqlite};
use sqlx::Row;
use std::error::Error;
use std::fs;
use csv::{ReaderBuilder};

pub const DB_URL: &str = "sqlite:data/words_database.db";


#[derive(Debug)]
#[allow(dead_code)]
pub enum WordField {
    Id,
    Expression,
    Reading,
    Meaning,
    JLPT(JLPTlv),
    PracticeTime,
    Familiar,
    UserMark,
}

#[derive(Debug, Copy, Clone)]
pub enum JLPTlv {
    N1,
    N2,
    N3,
    N4,
    N5,
}

impl JLPTlv {
    pub fn to_string(&self) -> String {
        match self {
            JLPTlv::N1 => "n1".to_string(),
            JLPTlv::N2 => "n2".to_string(),
            JLPTlv::N3 => "n3".to_string(),
            JLPTlv::N4 => "n4".to_string(),
            JLPTlv::N5 => "n5".to_string(),
        }
    }
    pub fn from_string(jlpt: &str) -> Option<JLPTlv> {
        match jlpt {
            "n1" => Some(JLPTlv::N1),
            "n2" => Some(JLPTlv::N2),
            "n3" => Some(JLPTlv::N3),
            "n4" => Some(JLPTlv::N4),
            "n5" => Some(JLPTlv::N5),
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
            WordField::JLPT(jlptlv) => { 
                jlptlv.to_string()
               }
            WordField::PracticeTime => "practice_time".to_string(),
            WordField::Familiar => "familiar".to_string(),
            WordField::UserMark => "user_mark".to_string(),
    
        }
    }
}


#[derive(Debug, Clone)]
pub struct WordRecord {
    pub id: i64,
    pub expression: String,
    pub reading: String,
    pub meaning: String,
    pub jlpt: String,
    pub practice_time: i64,
    pub familiar: bool,
    pub user_mark: bool,

}

pub fn load_csv_to_word_records(
    file_path: &str,
    records: &mut Vec<WordRecord>,
    jplt: &str,
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
            jlpt: jplt.to_string(), // manually pass jlpt level for each jlpt csv
            practice_time: 0, // DB auto assign 0 when initiate
            familiar: false, // DB  auto assign false when initiate
            user_mark: false, // DB auto assign false when initiate
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
            jlpt TEXT NOT NULL, 

            -- User Progress Fields --
            practice_time INTEGER NOT NULL DEFAULT 0,
            familiar      BOOLEAN NOT NULL DEFAULT 0,
            user_mark     BOOLEAN NOT NULL DEFAULT 0
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
            INSERT INTO words (expression, reading, meaning, jlpt)
            VALUES (?, ?, ?, ?)
            "#,)
            .bind(&record.expression)
            .bind(&record.reading)
            .bind(&record.meaning)
            .bind(&record.jlpt)
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
                jlpt: row.get::<String, _>("jlpt"),
                practice_time: row.get::<i64, _>("practice_time"),
                familiar: row.get::<bool, _>("familiar"),
                user_mark: row.get::<bool, _>("user_mark"),
            });
        }
    }
    Ok(records)
}


pub async fn return_words_by_user_progress(
    pool: &sqlx::SqlitePool, 
    jlpt: JLPTlv,
    practice_time: i64, 
    familiar: bool, 
    user_mark: bool,
    num: usize,
    random: bool,

) -> Result<Vec<WordRecord>, sqlx::Error> {

    let jlpt_str = jlpt.to_string();
    let query = if random {
        format!(
            r#"
            SELECT *
            FROM words
            WHERE jlpt = ? AND practice_time >= ? AND familiar = ? AND user_mark = ?
            ORDER BY RANDOM()
            LIMIT {}
            "#,
            num
        )
    }  else {
        format!(
            r#"
            SELECT *
            FROM words
            WHERE jlpt = ? AND practice_time >= ? AND familiar = ? AND user_mark = ?
            LIMIT {}
            "#,
            num
        )
    };

    let rows = sqlx::query(&query)
        .bind(&jlpt_str)
        .bind(practice_time)
        .bind(familiar)
        .bind(user_mark)
        .fetch_all(pool)
        .await?;

    let records = rows
        .into_iter()
        .map(|row| WordRecord {
            id: row.get::<i64, _>("id"),
            expression: row.get::<String, _>("expression"),
            reading: row.get::<String, _>("reading"),
            meaning: row.get::<String, _>("meaning"),
            jlpt: row.get::<String, _>("jlpt"),
            practice_time: row.get::<i64, _>("practice_time"),
            familiar: row.get::<bool, _>("familiar"),
            user_mark: row.get::<bool, _>("user_mark"),
        })
        .collect();
    eprintln!("return_words_by_user_progress called");
    Ok(records)
}




// do not use this, use ProgressUpdate instead
pub async fn update_user_progress(pool: &sqlx::SqlitePool, word_id: i64, familiar: bool, user_mark: bool) -> Result<(), sqlx::Error> {

    sqlx::query("UPDATE words SET practice_time = practice_time + 1, familiar = ?, user_mark = ? WHERE id = ?")
        .bind(familiar)
        .bind(user_mark)
        .bind(word_id)
        .execute(pool)
        .await?;

    Ok(())
}




pub async fn reset_all_user_progress(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    // This query resets all progress fields for all words back to their default state.
    sqlx::query(
        "UPDATE words SET practice_time = 0, familiar = 0, user_mark = 0"
    )
    .execute(pool)
    .await?;
    Ok(())
}



/// Counts the number of unique words that have been practiced (i.e., practice_time > 0).
pub async fn count_unique_practiced_words(pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(id)
        FROM words
        WHERE practice_time > 0
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}


/// counts total number of words practiced by user
pub async fn count_total_practiced_words(pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT SUM(practice_time) as total FROM words"
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// counts total number of familiar words by user
pub async fn count_total_familiar_words(pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(id)
        FROM words
        WHERE familiar = 1
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// counts total number of user marked words by user
pub async fn count_total_user_marked_words(pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(id)
        FROM words
        WHERE user_mark = 1
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Counts unfamiliar words that have been practiced (i.e., practice_time > 0 and familiar = 0).
pub async fn count_unfamiliar_practiced_words(pool: &sqlx::SqlitePool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(id)
        FROM words
        WHERE practice_time > 0 AND familiar = 0
        "#,
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
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
            INSERT INTO words (expression, reading, meaning, jlpt)
            VALUES (?, ?, ?, ?)
            "#,)
            .bind(&record.expression)
            .bind(&record.reading)
            .bind(&record.meaning)
            .bind(&record.jlpt)
            .execute(pool)
            .await?;
    }
    Ok(())
}





// This struct defines the changes we might want to make.
#[derive(Default)]
pub struct ProgressUpdate {
    increment_practice: bool,
    familiar: Option<bool>,
    user_mark: Option<bool>,
}

impl ProgressUpdate {
    // Start with a new, empty update operation.
    pub fn new() -> Self {
        Self::default()
    }

    // Chainable method to set the 'familiar' status.
    pub fn set_familiar(mut self, value: bool) -> Self {
        self.familiar = Some(value);
        self
    }

    // Chainable method to set the 'user_mark' status.
    pub fn set_user_mark(mut self, value: bool) -> Self {
        self.user_mark = Some(value);
        self
    }
    
    // Chainable method to indicate we should increment the practice time.
    pub fn increment_practice_time(mut self) -> Self {
        self.increment_practice = true;
        self
    }

    /// Executes the update operation against the database.
    pub async fn execute(self, pool: &sqlx::SqlitePool, word_id: i64) -> Result<(), sqlx::Error> {
        // 1. Check for changes at the very top. It's more efficient.
        if !self.increment_practice && self.familiar.is_none() && self.user_mark.is_none() {
            eprintln!("Update called with no changes, doing nothing.");
            return Ok(());
        }

        // Create a dynamic query builder.
        let mut builder = sqlx::QueryBuilder::new("UPDATE words SET ");
        let mut separated = builder.separated(", ");

        if self.increment_practice {
            separated.push("practice_time = practice_time + 1");
        }
        if let Some(val) = self.familiar {
            // 2. THE FIX: Push the SQL and bind the value as one logical unit.
            separated.push("familiar = ").push_bind_unseparated(val);
        }
        if let Some(val) = self.user_mark {
            // 3. THE FIX: Do the same for user_mark.
            separated.push("user_mark = ").push_bind_unseparated(val);
        }

        // Add the final WHERE clause.
        builder.push(" WHERE id = ");
        builder.push_bind(word_id);

        // Build and execute the query.
        builder.build().execute(pool).await?;

        Ok(())
    }


}


#[derive(Default)]
pub struct ProgressSelect {
    jlpt: Option<JLPTlv>,
    practice_time: Option<i64>,
    familiar: Option<bool>,
    user_mark: Option<bool>,
}

impl ProgressSelect {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select_jlpt(mut self, jlpt: JLPTlv) -> Self {
        self.jlpt = Some(jlpt);
        self
    }

    pub fn select_practice_time(mut self, time: i64) -> Self {
        self.practice_time = Some(time);
        self
    }

    pub fn select_familiar(mut self, value: bool) -> Self {
        self.familiar = Some(value);
        self
    }

    pub fn select_user_mark(mut self, value: bool) -> Self {
        self.user_mark = Some(value);
        self
    }

    pub async fn execute(self, pool: &sqlx::SqlitePool) -> Result<Vec<WordRecord>, sqlx::Error> {
        let mut query = String::from("SELECT * FROM words WHERE 1=1");
        
        if let Some(jlpt) = self.jlpt {
            query.push_str(" AND jlpt = ?");
        }
        if let Some(practic_time) = self.practice_time {
            query.push_str(" AND practice_time >= ?");
        }
        if let Some(familiar) = self.familiar {
            query.push_str(" AND familiar = ?");
        }
        if let Some(user_mark) = self.user_mark {
            query.push_str(" AND user_mark = ?");
        }

        let mut sql_query = sqlx::query(&query);
        
        if let Some(jlpt) = self.jlpt {
            sql_query = sql_query.bind(jlpt.to_string());
        }
        if let Some(practice_time) = self.practice_time {
            sql_query = sql_query.bind(practice_time);
        }
        if let Some(familiar) = self.familiar {
            sql_query = sql_query.bind(familiar);
        }
        if let Some(user_mark) = self.user_mark {
            sql_query = sql_query.bind(user_mark);
        }

        let rows = sql_query.fetch_all(pool).await?;

        // Map the rows to WordRecord
        let records: Vec<WordRecord> = rows.into_iter().map(|row| WordRecord {
            id: row.get("id"),
            expression: row.get("expression"),
            reading: row.get("reading"),
            meaning: row.get("meaning"),
            jlpt: row.get("jlpt"),
            practice_time: row.get("practice_time"),
            familiar: row.get("familiar"),
            user_mark: row.get("user_mark"),
        }).collect();

        Ok(records)
    }

    
}




#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePool;
    use sqlx::sqlite::SqlitePoolOptions;

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
            WordRecord { id: 0, expression: "一".to_string(), reading: "いち".to_string(), meaning: "one".to_string(), jlpt: "n5".to_string(), practice_time: 0, familiar: false, user_mark: false },
            WordRecord { id: 0, expression: "二".to_string(), reading: "に".to_string(), meaning: "two".to_string(), jlpt: "n5".to_string() , practice_time: 0, familiar: false, user_mark: false },
            WordRecord { id: 0, expression: "時間".to_string(), reading: "じかん".to_string(), meaning: "time".to_string(), jlpt: "n4".to_string() , practice_time: 0, familiar: false, user_mark: false },
            WordRecord { id: 0, expression: "経済".to_string(), reading: "けいざい".to_string(), meaning: "economy".to_string(), jlpt: "n1".to_string() , practice_time: 0, familiar: false, user_mark: false },
            WordRecord { id: 0, expression: "政治".to_string(), reading: "せいじ".to_string(), meaning: "politics".to_string(), jlpt: "n1".to_string() , practice_time: 0, familiar: false, user_mark: false },
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
        
        // Check progressUpdate
        ProgressUpdate::new()
            .set_familiar(true)
            .set_user_mark(true)
            .increment_practice_time()
            .execute(&pool, 2)
            .await
            .expect("Failed to update progress.");

        let word = find_word_by_ids(&pool, vec![2 as i64])
            .await
            .expect("Failed to find word by ID.");

        println!("word = {:?}", word);
        assert_eq!(word[0].practice_time, 1 as i64);
        assert_eq!(word[0].familiar, true);
        assert_eq!(word[0].user_mark, true);
        println!("-> Verified: Progress updated correctly.");
    

        // count unique practiced words
        let unique_count = count_unique_practiced_words(&pool)
            .await
            .expect("Failed to count unique practiced words.");
        assert_eq!(unique_count, 1);
        println!("-> Verified: Unique practiced words count is correct: {}", unique_count);

        // count total practiced words
        let total_practiced_count = count_total_practiced_words(&pool)
            .await
            .expect("Failed to count total practiced words.");
        assert_eq!(total_practiced_count, 1);
        println!("-> Verified: Total practiced words count is correct: {}", total_practiced_count);

        // count total familiar words
        let total_familiar_count = count_total_familiar_words(&pool)
            .await
            .expect("Failed to count total familiar words.");
        assert_eq!(total_familiar_count, 1);
        println!("-> Verified: Total familiar words count is correct: {}", total_familiar_count);

        // count total user marked words
        let total_user_marked_count = count_total_user_marked_words(&pool)
            .await
            .expect("Failed to count total user marked words.");
        assert_eq!(total_user_marked_count, 1);
        println!("-> Verified: Total user marked words count is correct: {}", total_user_marked_count);

        // count unfamiliar practiced words
        let unfamiliar_practiced_count = count_unfamiliar_practiced_words(&pool)
            .await
            .expect("Failed to count unfamiliar practiced words.");
        assert_eq!(unfamiliar_practiced_count, 0);
        println!("-> Verified: Unfamiliar practiced words count is correct: {}", unfamiliar_practiced_count);

        // check ProgressSelect
        let select_result = ProgressSelect::new()
            .select_jlpt(JLPTlv::N5)
            .select_familiar(true)
            .select_user_mark(true)
            .execute(&pool)
            .await
            .expect("Failed to execute progress select.");
        assert_eq!(select_result.len(), 1);
        assert_eq!(select_result[0].id, 2);
        assert_eq!(select_result[0].expression, "二");
        assert_eq!(select_result[0].familiar, true);
        assert_eq!(select_result[0].user_mark, true);
        println!("-> Verified: ProgressSelect returned correct results.");


        let select_result = ProgressSelect::new()
            .select_practice_time(1)
            .execute(&pool)
            .await
            .expect("Failed to execute progress select for N1.");
        assert_eq!(select_result.len(), 1);
        println!("-> Verified: ProgressSelect returned correct results for practice time.");

        // Reset progress test
        reset_all_user_progress(&pool)
            .await
            .expect("Failed to reset progress.");

        let word = find_word_by_ids(&pool, vec![2 as i64])
            .await
            .expect("Failed to find word by ID.");

        assert_eq!(word[0].practice_time, 0 as i64);
        assert_eq!(word[0].familiar, false);
        assert_eq!(word[0].user_mark, false);
        println!("-> Verified: Progress reset correctly.");




    }
}
