
use dioxus::prelude::*;
use sqlx::sqlite::SqlitePool;
use crate::db::*;


#[component]
pub fn TextInputPanel() -> Element {

    let db_pool = use_context::<SqlitePool>();
    let pool = db_pool.clone();
    let ids: Vec<i64> = vec![1, 2, 3];

    spawn(async move {
        match find_word_by_ids(&pool, ids).await {
            Ok(_) => eprintln!("Background read successful!"),
            Err(e) => eprintln!("Background update failed: {}", e),
            }
        });

    rsx! {
        div {
             
        }
        
        div {
            class: "m-4 d-flex flex-row gap-5",
            div { 
                button { class: "btn btn-primary" ,"click me" }
                
            }
            div {
                button { "click me again" }
                
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


