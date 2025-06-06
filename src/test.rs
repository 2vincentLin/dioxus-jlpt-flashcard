
use dioxus::prelude::*;
use sqlx::{pool, sqlite::SqlitePool};
use crate::db::*;

// when we take pool, since it's not signal, best use async fn for db
async fn create_words_future(pool: SqlitePool, ids: Vec<i64>) {
    eprintln!("Background task started: Fetching words...");
    match find_word_by_ids(&pool, ids).await {
        Ok(words) => {
            eprintln!("Background read successful!");
            for word in words {
                eprintln!("Word: {:?}", word);
            }
        }
        Err(e) => eprintln!("Background update failed: {}", e),
    }
}

#[component]
pub fn TextInputPanel() -> Element {

    let db_pool = use_context::<SqlitePool>();
    let pool = db_pool.clone();
    let ids: Vec<i64> = vec![1, 2, 3];


    // clone the pool for each case
    let pool_for_1 = db_pool.clone();
    let pool_for_2 = db_pool.clone();



    // prefer not to use closure since we don't need to capture environment variable
    let create_words_future_1 = move |pool| async move {
        // let pool = db_pool.clone();

        let ids: Vec<i64> = vec![1, 2, 3];
        eprintln!("Background task started: Fetching words...");
        match find_word_by_ids(&pool, ids).await {
            Ok(words) => {
                eprintln!("Background read successful!");
                for word in words {
                    eprintln!("Word: {:?}", word);
                }
            }
            Err(e) => eprintln!("Background update failed: {}", e),
        }
    };


    rsx! {
        div {
             
        }
        
        div {
            class: "m-4 d-flex flex-row gap-5",
            div { 
                button { class: "btn btn-primary", 
                onclick: move |_| {

                let ids_to_fetch = vec![1, 2, 3];

                spawn(create_words_future(pool_for_1.clone(), ids_to_fetch));
                }
                ,"click me" }
                
            }
            div {
                button { 
                    onclick: move |_| {
                        spawn(create_words_future_1(pool_for_2.clone()));

                    }
                    ,"click me again",
             }
                
            }
            div {
                button { "click me again and again" }
                
            }
        }
        
    
    
    div { class: "ui-playground-header",
        h1 { "UI Playground" }
        p { "Explore the UI components of the application." }
    }
    div { class: "ui-playground-content",
        div { class: "ui-component", id: "button-component",
            h2 { "Button Component" }
            button { class: "btn btn-primary", "Primary Button" }
            button { class: "btn btn-secondary", "Secondary Button" }
        }
        div { class: "ui-component", id: "input-component",
            h2 { "Input Component" }
            input { r#type: "text", placeholder: "Enter text here" }
            input { r#type: "password", placeholder: "Enter password" }
        }
        div { class: "ui-component", id: "card-component",
            h2 { "Card Component" }
            div { class: "card",
                h3 { "Card Title" }
                p { "This is a simple card component." }
            }
        }
    }

    }
}


