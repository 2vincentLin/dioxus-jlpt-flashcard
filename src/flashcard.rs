

use dioxus::{prelude::*};
use crate::db::*;
use sqlx::sqlite::SqlitePoolOptions;
use crate::Route;
use tts::*;
use crate::return_voice;


#[component]
pub fn GenerateCard() -> Element {

    let mut number_of_cards = use_signal(|| 15);
    let mut tag_level = use_signal(|| TagLevel::N5.to_string());
    let mut j_to_e= use_signal(|| true);
    let mut unfamiliar_only = use_signal(|| true);
    let mut random_shuffle = use_signal(|| true);
    let mut user_mark = use_signal(|| false);
    let navigator = use_navigator();





    rsx!(
        div {
            class: "container mt-2 p-4 border rounded shadow-sm bg-dark", // Bootstrap container with some styling
            // Top Row: Number of Cards and Tag Level
            div { class: "row mb-3 g-3 align-items-end", // g-3 for gutters between columns
                div { class: "col-md-6", // Takes half width on medium screens and up
                    label { class: "form-label", r#for: "numCardsSelect", "Number of Cards:" }
                    select {
                        class: "form-select",
                        id: "numCardsSelect",
                        value: "{number_of_cards}",
                        oninput: move |evt| {
                            // Update the number of cards based on dropdown selection
                            if let Ok(value) = evt.value().parse::<usize>() {
                                number_of_cards.set(value);
                            } else {
                                eprintln!("Invalid input for number of cards");
                            }
                        }, 
                        option { value: "10", "10" }
                        option { value: "15", "15" }
                        option { value: "20", "20" }
                        option { value: "25", "25" }
                        option { value: "30", "30" }
                        }
                    }
                
                div { class: "col-md-6", // Takes half width on medium screens and up
                    label { class: "form-label", r#for: "tagLevelSelect", "Tag Level:" }
                    select {
                        class: "form-select",
                        id: "tagLevelSelect",
                        value: "{tag_level}",
                        oninput: move |evt| {
                            // Update the level based on dropdown selection
                            tag_level.set(evt.value()); },
                        option { value: TagLevel::N1.to_string(), "n1" }
                        option { value: TagLevel::N2.to_string(), "n2" }
                        option { value: TagLevel::N3.to_string(), "n3" }
                        option { value: TagLevel::N4.to_string(), "n4" }
                        option { value: TagLevel::N5.to_string(), "n5" }
                    }
                }
            }

            // Second Row: Checkboxes
            // We can use a more complex grid here or just a row of columns
            div { class: "row mb-3 g-3",
                // Grouping checkboxes for better responsiveness if needed
                div { class: "col-md-6",
                     div { class: "mb-2", // Margin bottom for spacing between checkbox groups
                        div { class: "form-check",
                            input {
                                class: "form-check-input",
                                r#type: "checkbox",
                                id: "jToECheck",
                                checked: j_to_e(),
                                oninput: move |evt| j_to_e.set(evt.checked()),
                            }
                            label { class: "form-check-label", r#for: "jToECheck", "J to E" }
                        }
                        div { class: "form-check",
                            input {
                                class: "form-check-input",
                                r#type: "checkbox",
                                id: "unfamiliarCheck",
                                checked: unfamiliar_only(),
                                oninput: move |evt| unfamiliar_only.set(evt.checked()),
                            }
                            label { class: "form-check-label", r#for: "unfamiliarCheck", "Unfamiliar Only" }
                        }
                    }
                }
                div { class: "col-md-6",
                    div { class: "mb-2",
                        div { class: "form-check",
                            input {
                                class: "form-check-input",
                                r#type: "checkbox",
                                id: "shuffleCheck",
                                checked: random_shuffle(),
                                oninput: move |evt| random_shuffle.set(evt.checked()),
                            }
                            label { class: "form-check-label", r#for: "shuffleCheck", "Random Shuffle Card" }
                        }
                        div { class: "form-check",
                            input {
                                class: "form-check-input",
                                r#type: "checkbox",
                                id: "userMarkCheck",
                                checked: user_mark(),
                                oninput: move |evt| user_mark.set(evt.checked()),
                            }
                            label { class: "form-check-label", r#for: "userMarkCheck", "User Mark" }
                        }
                    }
                }
            }

            // Button Row (or part of the second row)
            div { class: "row",
                div { class: "col-12 text-end", // Align button to the right
                    button {
                        class: "btn btn-primary btn-lg", // Bootstrap primary button, large
                        r#type: "button", // Important to prevent form submission if wrapped in <form>
                        onclick: move |_| async move {
                            // Handle card generation logic here
                            // You can access the signal values:
                            // num_cards(), tag_level(), j_to_e(), etc.
                            eprintln!(
                                "Generate Cards Clicked! Settings: Cards={}, Level='{}', JtoE={}, Unfamiliar={}, Shuffle={}, UserMark={}",
                                number_of_cards(),
                                tag_level(),
                                j_to_e(),
                                unfamiliar_only(),
                                random_shuffle(),
                                user_mark()
                            );
                            // Potentially call another function or update another signal

                            // retrieve data
                            let mut select_words = use_context::<Signal<Vec<WordRecord>>>();

                            match SqlitePoolOptions::new()
                                .max_connections(5)
                                .connect(DB_URL)
                                .await {
                                    Ok(pool) => {
                                        let tag = TagLevel::from_string(&tag_level()).unwrap();
                                        let num = number_of_cards();
                                        let random = random_shuffle();
                                        match get_unfamiliar_words(&pool, tag, num, random).await {
                                            Ok(records) => {
                                                // load the records
                                                select_words.set(records);
                                            },
                                            Err(e) => {
                                                eprintln!("Error fetching word IDs: {}", e);
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error connecting to database: {}", e);
                                        ()
                                    }
                                };
                            
                            navigator.push(Route::DisplayCard { j_to_e: j_to_e() } );



                        },
                        "Generate Card"
                    }
                }
            }
        }
    )

}

#[component]
pub fn DisplayCard(j_to_e: bool) -> Element {
    let navigator = use_navigator();
    let mut index = use_signal(|| 0 as usize); // current index in select_words
    let select_words = use_context::<Signal<Vec<WordRecord>>>();
    let total_cards = select_words.len();
    

     // --- Signals for UI State ---
    let mut show_question = use_signal(|| true);
    let mut show_reading = use_signal(|| true);
    let mut show_answer = use_signal(|| false);

    // --- content for UI ---
    let mut question = use_signal(|| "".to_string());
    let mut reading = use_signal(|| "".to_string());
    let mut answer = use_signal(|| "".to_string());

    // --- pool for db op ---
    let db_pool = use_context::<sqlx::SqlitePool>();
    let pool_un = db_pool.clone(); // pool for unfamiliar op
    let pool_fa = db_pool.clone(); // pool for familiar op
    let pool_um = db_pool.clone(); // pool for user mark op


    // --- voice for tts ---
    let voice = return_voice("ja", Gender::Male)?;
    


    // 1. --- Refactored Logic: Create a reusable closure to load a card ---
    // This closure takes the index of the card to load.
    let mut load_card = move |card_index: usize| {
        if let Some(word) = select_words.get(card_index) {
            eprintln!("Loading card at index {}: {:?}", card_index, word);
            if j_to_e {
                question.set(word.expression.clone());
                reading.set(word.reading.clone());
                answer.set(word.meaning.clone());
            } else {
                question.set(word.meaning.clone());
                reading.set(word.reading.clone());
                answer.set(word.expression.clone());
            }
            // Always hide the answer when loading a new card
            show_answer.set(false);
        }
    };

    // 2. --- Effect for Initial Load ---
    // `use_effect` runs after the component renders.
    // By calling our logic here, we load the very first card.
    use_effect(move || {
        load_card(0); // Load the card at index 0
    });


    // 3. --- Simplified Event Handler ---
    let mut go_to_next_card = move || {
        let current_index = index();
        let next_index = if current_index < total_cards - 1 {
            current_index + 1
        } else {
            0 // Loop back to the start
        };

        index.set(next_index); // Update the index signal
        load_card(next_index); // Load the new card using the reusable logic
    };

  


    rsx! {
        

        div { class: "container h-100 d-flex flex-column",
            // --- Top Controls ---
            div { class: "row my-3",
                div { class: "col-auto",
                    button { class: "btn btn-secondary", 
                    onclick: move |_| {
                        navigator.push(Route::GenerateCard {});
                    }, 
                    "Go Back" }
                }
                div { class: "col d-flex justify-content-start align-items-center gap-3",
                    div { class: "form-check form-switch",
                        input { class: "form-check-input", r#type: "checkbox", role: "switch", id: "toggleQuestion", checked: "{show_question}",
                            oninput: move |evt| show_question.set(evt.checked())
                        }
                        label { class: "form-check-label", r#for: "toggleQuestion", "Show Question" }
                    }
                    div { class: "form-check form-switch",
                        input { class: "form-check-input", r#type: "checkbox", role: "switch", id: "toggleReading", checked: "{show_reading}",
                            oninput: move |evt| show_reading.set(evt.checked())
                        }
                        label { class: "form-check-label", r#for: "toggleReading", "Show Reading" }
                    }
                }
            }

            // --- Main Content ---
            div { class: "row flex-grow-1 d-flex flex-column justify-content-center",
                // 1. Wrap the heading and button in a flex container
                div { class: "col d-flex justify-content-between align-items-center",
            
                    // 2. The original h2 tag
                    // I've added mb-0 to remove its default bottom margin for better alignment
                    p { class: "lead my-3",
                        if show_question() { "{question()}" } else { "" }
                    }

                    // 3. The new star button, with spacing on its left (ms-3)
                    button { 
                        class: "btn btn-lg btn-outline-warning ms-3", 
                        // todo: display star based on user_mark field
                        // Using a Unicode star character for the icon
                        // onclick: move |_| {
                        // let pool = pool_um.clone();
                        // let word_id = select_words.get(index()).unwrap().id;
                        // eprintln!("word_id: {}", word_id);
                        
                        // spawn (async move {
                        //     match mark_word(&pool, word_id).await {
                        //         Ok(_) => {
                        //             eprintln!("id {:?} for marked successfully", word_id);
                        //         }
                        //         Err(e) => {
                        //             eprintln!("Background update failed: {}", e);
                        //         },
                        //     }

                        //     });
                        // },
                        "★" 
                    }
                }

                if show_reading() {
                    div { class: "col d-flex justify-content-between align-items-center",
                        p { class: "lead my-3", "{reading()}" }
                        button { class: "btn btn-light",
                            onclick: move |_| {
                            // --- PREPARE DATA ---
                            // Clone the data that the new thread will need before we create it.
                            let text_to_speak = reading();
                            let voice_to_use = voice.clone(); // Clone the voice configuration

                            // --- SPAWN THE THREAD ---
                            // This moves the entire block of work to a background thread.
                            std::thread::spawn(move || {
                                // This code now runs in the background.
                                match Tts::default() {
                                    Ok(mut tts) => {
                                        // It's good practice to log from the thread to see it's working
                                        eprintln!("[Thread] TTS initialized, setting voice...");

                                        if tts.set_voice(&voice_to_use).is_err() {
                                            eprintln!("[Thread] Error: Failed to set voice.");
                                        }

                                        // Start the non-blocking speech
                                        let _ = tts.speak(text_to_speak, false);

                                        // *** THE KEY ***
                                        // Sleep on the BACKGROUND thread. This keeps the `tts` object
                                        // alive so it can finish speaking, but it does NOT block the UI.
                                        // Adjust the duration if your text is longer.
                                        std::thread::sleep(std::time::Duration::from_secs(5));

                                        eprintln!("[Thread] Sleep finished, thread is ending.");
                                    },
                                    Err(e) => {
                                        eprintln!("[Thread] Error: {}", e);
                                    }
                                }
                            });
                        }
                            , "🔊"}
                        }
                    }
                

                div { class: "col",
                    onclick: move |_| show_answer.set(true),
                    if show_answer() {
                        h3 { class: "display-5 text-success", "{answer()}" }
                    } else {
                        div { class: "alert alert-info", "Click to show answer" }
                    }
                }
            }

            // --- Progress Bar ---
            div { class: "row my-3 align-items-center",
                div { class: "col",
                    div { class: "progress",
                        div {
                            class: "progress-bar",
                            role: "progressbar",
                            style: "width: {((index() + 1) as f32 / total_cards as f32) * 100.0}%",
                            aria_valuenow: "{index() + 1}",
                            aria_valuemin: "0",
                            aria_valuemax: "{total_cards}"
                        }
                    }
                }
                div { class: "col-auto",
                    span { "{index() + 1} / {total_cards}" }
                    
                }
            }


            // --- Bottom Controls ---
            div { class: "row my-3",
                div { class: "col",
                    button { class: "btn btn-warning w-100", 
                    onclick: move |_| {
                        let pool = pool_un.clone();
                        let word_id = select_words.get(index()).unwrap().id;
                        eprintln!("word_id: {}", word_id);
                        
                        // in normal rust, async move won't be executed unless you call await, or use spawn in Dioxus
                        spawn (async move {
                            match update_familiar(&pool, word_id, false).await {
                                Ok(_) => {
                                    eprintln!("id {:?} for unfamiliar updated successfully", word_id);
                                }
                                Err(e) => {
                                    eprintln!("Background update failed: {}", e);
                                },
                            }

                            go_to_next_card();
                        });
                    }, 
                    "Need more practice" }
                }
                div { class: "col",
                    button { class: "btn btn-success w-100", 
                    onclick: move |_| {
                        let pool = pool_fa.clone();
                        let word_id = select_words.get(index()).unwrap().id;
                        eprintln!("word_id: {}", word_id);
                        
                        // but in Dioxus 6.0, they add new function to auto spawn
                        async move {
                            match update_familiar(&pool, word_id, true).await {
                                Ok(_) => {
                                    eprintln!("id {:?} for familiar updated successfully", word_id);
                                }
                                Err(e) => {
                                    eprintln!("Background update failed: {}", e);
                                },
                            }

                            go_to_next_card();
                        }
                    },
                    "Got it!" }
                }
            }
        }
    }
}