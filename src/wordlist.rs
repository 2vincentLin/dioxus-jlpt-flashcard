use dioxus::prelude::*;
use sqlx::SqlitePool;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use std::error::Error;
use crate::db::*;
use crate::Route;

// for 2nd route enum, we cannot derive Routable, it'll conflict with the first one.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WordListType {
    Practiced,
    Familiar,
    Unfamiliar,
    Marked,
}

// We need to manually implement Display trait for Routable trait to work.
// This is used to build the URL string.
impl Display for WordListType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            WordListType::Practiced => "practiced",
            WordListType::Familiar => "familiar",
            WordListType::Unfamiliar => "unfamiliar",
            WordListType::Marked => "marked",
        };
        write!(f, "{}", s)
    }
}
// A simple error type for our FromStr implementation
#[derive(Debug)]
pub struct ParseWordListTypeError;


// 1. Implement Display for the error type.
//    This tells Rust how to format the error as a user-facing string.
impl Display for ParseWordListTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "Invalid WordListType in URL segment")
    }
}

// 2. (Recommended) Implement the standard Error trait for your error type.
//    This signals that it's a proper error type.
impl Error for ParseWordListTypeError {}


// we also need to implement FromStr trait.
impl FromStr for WordListType {
    type Err = ParseWordListTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "practiced" => Ok(WordListType::Practiced),
            "familiar" => Ok(WordListType::Familiar),
            "unfamiliar" => Ok(WordListType::Unfamiliar),
            "marked" => Ok(WordListType::Marked),
            _ => Err(ParseWordListTypeError),
        }
    }
}

#[component]
pub fn WordListPage(list_type: WordListType) -> Element {
    
    let navigator = use_navigator();

    let db_pool = use_context::<sqlx::SqlitePool>();
    // Here we can use the word_list_type to determine what to display.
    let word_list = use_resource(move || {
        let pool = db_pool.clone();
        async move {
            match list_type {
                WordListType::Practiced => {
                    ProgressSelect::new()
                        .select_practice_time(1)
                        .execute(&pool)
                        .await
                        .map_err(|e| e.to_string())
                },
                WordListType::Familiar => {
                    ProgressSelect::new()
                        .select_familiar(true)
                        .execute(&pool)
                        .await
                        .map_err(|e| e.to_string())
                },
                WordListType::Unfamiliar => {
                    ProgressSelect::new()
                        .select_familiar(false)
                        .execute(&pool)
                        .await
                        .map_err(|e| e.to_string())
                },
                WordListType::Marked => {
                    ProgressSelect::new()
                        .select_user_mark(true)
                        .execute(&pool)
                        .await
                        .map_err(|e| e.to_string())
                },
            }
        }
    });

    // Determine the title based on the list_type
    let title = format!("{} words", list_type);

    rsx! {
        div { class: "container p-4",
            div { class: "col-auto",
                button { class: "btn btn-secondary", 
                onclick: move |_| {
                    navigator.push(Route::Home {  });
                }, 
                "Go Back" }
            }
            h1 { "{title}" }
            
            // Render based on the state of our resource
            
            match &*word_list.read_unchecked() {
                Some(Ok(words)) => rsx! {
                    table { class: "table table-dark table-striped",
                        thead {
                            tr {
                                th { "Expression" }
                                th { "Reading" }
                                th { "Meaning" }
                            }
                        }
                        tbody {
                            for word in words {
                                tr {
                                    td { "{word.expression}" }
                                    td { "{word.reading}" }
                                    td { "{word.meaning}" }
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-danger", "Error loading words: {e}" }
                },
                None => rsx! {
                    p { "Loading word list..." }
                }
            }
        }
    }
}