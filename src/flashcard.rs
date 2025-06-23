

use dioxus::prelude::*;
use crate::db::*;
use crate::Route;
use tts::*;
use crate::return_voice;


#[component]
pub fn GenerateCard() -> Element {

    let mut number_of_cards = use_signal(|| 15);
    let mut jlpt_lv = use_signal(|| JLPTlv::N5.to_string());
    let mut j_to_e= use_signal(|| true);
    let mut unfamiliar_only = use_signal(|| true);
    let mut random_shuffle = use_signal(|| true);
    let mut user_mark = use_signal(|| false);
    let mut info_message = use_signal(|| String::new());
    let navigator = use_navigator();

    let db_pool = use_context::<sqlx::SqlitePool>();
    let pool_ge = db_pool.clone();




    rsx!(
        div {
            class: "container mt-2 p-4 border rounded shadow-sm bg-dark", // Bootstrap container with some styling
            // Top Row: Number of Cards and jlpt Level
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
                    label { class: "form-label", r#for: "JLPTlvSelect", "jlpt Level:" }
                    select {
                        class: "form-select",
                        id: "JLPTlvSelect",
                        value: "{jlpt_lv}",
                        oninput: move |evt| {
                            // Update the level based on dropdown selection
                            jlpt_lv.set(evt.value()); },
                        option { value: JLPTlv::N1.to_string(), "n1" }
                        option { value: JLPTlv::N2.to_string(), "n2" }
                        option { value: JLPTlv::N3.to_string(), "n3" }
                        option { value: JLPTlv::N4.to_string(), "n4" }
                        option { value: JLPTlv::N5.to_string(), "n5" }
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
            div { 
                // 1. Use flexbox to align items.
                //    `justify-content-end` pushes everything to the right.
                //    `align-items-center` vertically centers the message and button.
                class: "row d-flex justify-content-end align-items-center",
                
                // 2. The new message block. It's inside the flex row.
                div { class: "col-auto", // `col-auto` makes it only as wide as its content
                    if !info_message().is_empty() {
                        // Display the message with some margin on the right (me-3)
                        p { class: "alert alert-warning text-center mb-0 me-3", "{info_message}" }
                    }
                }
                
                // 3. The button block, also using `col-auto`.
                div { class: "col-auto",
                    button {
                        class: "btn btn-primary btn-lg",
                        r#type: "button",
                        onclick: move |_| {
                            // Clear any previous message
                            info_message.set(String::new());

                            let pool = pool_ge.clone();
                            // We need to clone the signal to move it into the async block

                            async move {
                                eprintln!("Generate Cards Clicked! ..."); // Your logging

                                let mut select_words = use_context::<Signal<Vec<WordRecord>>>();
                                let jlpt = JLPTlv::from_string(&jlpt_lv()).unwrap();
                                let num = number_of_cards();
                                let random = random_shuffle();

                                match return_words_by_user_progress(&pool, jlpt, 0, !unfamiliar_only(), user_mark(), num, random).await {
                                    Ok(records) => {
                                        if records.is_empty() {
                                            // Set the message and DO NOT navigate
                                            info_message.set("No cards found for these settings.".to_string());
                                        } else {
                                            // Load records and navigate
                                            select_words.set(records);
                                            navigator.push(Route::DisplayCard { j_to_e: j_to_e() });
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("Error fetching word IDs: {}", e);
                                        info_message.set("A database error occurred.".to_string());
                                    }
                                }
                            }
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
    let mut is_marked = use_signal(|| false);


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
    

    // --- setup button class and icon for user_mark
    let button_class = if is_marked() {
        "btn btn-lg btn-warning" // Solid yellow if marked
        } else {
            "btn btn-lg btn-outline-warning" // Outline if not marked
        };
    let star_icon = if is_marked() { "★" } else { "☆" }; // Solid vs. Outline star


    // 1. --- Refactored Logic: Create a reusable closure to load a card ---
    // This closure takes the index of the card to load.
    let mut load_card = move |card_index: usize| {
        if let Some(word) = select_words.get(card_index) {
            eprintln!("Loading card at index {}: {:?}", card_index, word);
            reading.set(word.reading.clone());
            is_marked.set(word.user_mark);
            if j_to_e {
                question.set(word.expression.clone());
                answer.set(word.meaning.clone());
            } else {
                question.set(word.meaning.clone());
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
                div { class: "col d-flex justify-content-between align-items-center",
            
                    p { class: "lead my-3",
                        if show_question() { "{question()}" } else { "" }
                    }

                    button { 
                        class: button_class, 

                        onclick: move |_| {
                            let pool = pool_um.clone();
                            let word_id = select_words.get(index()).unwrap().id;
                            eprintln!("word_id: {}", word_id);
                            
                            async move {

                                match ProgressUpdate::new()
                                    .set_user_mark(!is_marked())
                                    .execute(&pool, word_id)
                                    .await {
                                        Ok(_) => {
                                            eprintln!("id {:?} marked {:?} successfully", word_id, !is_marked());
                                        }
                                        Err(e) => {
                                            eprintln!("Background update failed: {}", e);
                                        }, 
                                    }
                                is_marked.set(!is_marked());
                            }
                            
                        },
                        {star_icon}
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
                            match ProgressUpdate::new()
                                .increment_practice_time()
                                .set_familiar(false)
                                .execute(&pool, word_id)
                                .await {
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
                            match ProgressUpdate::new()
                                .increment_practice_time()
                                .set_familiar(true)
                                .execute(&pool, word_id)
                                .await {
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