use dioxus::prelude::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use std::error::Error;
use crate::db::*;
use crate::Route;
use crate::footer::{StatusMessage, StatusLevel};

/// Represents the type of word list to display.
/// This enum is used to determine which set of words to fetch from the database.
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

/// A component that displays a list of words based on the specified `WordListType`.
#[component]
pub fn WordListPage(list_type: WordListType) -> Element {
    
    let navigator = use_navigator();
    let mut select_words = use_context::<Signal<Vec<WordRecord>>>();
    // Add a state to control button enabled/disabled
    let mut generate_enabled = use_signal(|| false);
    let db_pool = use_context::<sqlx::SqlitePool>();

    let mut status_message = use_context::<Signal<StatusMessage>>();



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
                        .select_practice_time(1)
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

    use_effect(move || {
        // This effect runs when the word_list resource changes
        status_message.set(StatusMessage {
            message: format!("App is ready."),
            level: StatusLevel::Success,
        });
        if let Some(Ok(words)) = &*word_list.read_unchecked() {
            if words.len() > 0 {
                eprintln!("Word list loaded to select_words signal");
                select_words.set(words.clone()); // Store the words in the context
                generate_enabled.set(true); // Enable the button when words are loaded
            }

        }
    });
    

    rsx! {
        div { class: "container p-4 d-flex flex-column h-75",
            div { class: "d-flex justify-content-between align-items-center mb-3",
                button { class: "btn btn-secondary",
                    onclick: move |_| {
                        navigator.push(Route::Home {  });
                    },
                    "Go Back"
                }
                button { class: "btn btn-primary",
                    disabled: !generate_enabled(), // Disable until ready
                    onclick: move |_| {
                        if generate_enabled() {
                            navigator.push(Route::GnerateTestCard {});
                        }
                    },
                "Generate Test"
                }
            }
            h1 { "{title}" }
            
            // Render based on the state of our resource            
            match &*word_list.read_unchecked() {
                Some(Ok(words)) =>{ 
                    // select_words.set(words.clone()); // Store the words in the context
                    rsx! {
                    
                    div { class: "flex-grow-1 overflow-auto",
                        table { class: "table table-dark table-striped table-hover", // Added table-hover for better UX
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