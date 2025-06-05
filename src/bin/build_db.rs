

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Row;
use dxgui::db::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let db_url = "sqlite:data/words_database.db";
    let csv_file_path5 = "data//n5.csv"; 
    let csv_file_path4 = "data//n4.csv";
    let csv_file_path3 = "data//n3.csv";
    let csv_file_path2 = "data//n2.csv";
    let csv_file_path1 = "data//n1.csv";
    let mut records: Vec<WordRecord> = Vec::new();

    // load csv file to word records
    load_csv_to_word_records(csv_file_path5, &mut records,"n5")?;
    load_csv_to_word_records(csv_file_path4, &mut records,"n4")?;
    load_csv_to_word_records(csv_file_path3, &mut records,"n3")?;    
    load_csv_to_word_records(csv_file_path2, &mut records,"n2")?;
    load_csv_to_word_records(csv_file_path1, &mut records,"n1")?;

    // reset the database
    match reset_database(db_url).await {
        Ok(_) => println!("Database reset successfully."),
        Err(error) => panic!("Error resetting database: {}", error),
    }

    // connect to the database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    println!("Connected to database: {}", db_url);

    // create table if not exists
    match create_table(&pool).await {
        Ok(_) => println!("Table created successfully."),
        Err(error) => panic!("Error creating table: {}", error),
    }

    // insert records to the database
    match bulk_insert_words(&pool, records).await {
        Ok(_) => println!("Records inserted successfully."),
        Err(error) => panic!("Error inserting records: {}", error),
    }

    // check the number of records
    let count = sqlx::query("SELECT COUNT(*) FROM words")
        .fetch_one(&pool)
        .await?
        .get::<i64, _>(0);
    println!("Number of records in the database: {}", count);

    Ok(())
}



