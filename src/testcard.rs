use std::vec;

use dioxus::prelude::*;
use crate::db::*;
use crate::footer::{StatusMessage, StatusLevel};
use crate::Route;
use tts::*;
use crate::return_voice;
use crate::utils::speak_text;

use futures_util::StreamExt;
use rand::seq::SliceRandom;
use rand::{rng, Rng};
// Use the IndexedRandom trait for random selection
use rand::prelude::IndexedRandom;

/// This the componenet used to generate test for the selected words
#[component]
pub fn GnerateTestCard() -> Element {
    // Get the status message and level from the context
    let mut status_message = use_context::<Signal<StatusMessage>>();

    let navigator = use_navigator();
    let mut number_of_cards = use_signal(|| 15);


    let mut j_to_e= use_signal(|| true);


    // let db_pool = use_context::<sqlx::SqlitePool>();
    // let pool_ge = db_pool.clone();
    let mut select_words = use_context::<Signal<Vec<WordRecord>>>();


    rsx!(
        div {
            class: "container mt-2 p-4 border rounded shadow-sm bg-dark", // Bootstrap container with some styling
            // --- Top Controls ---
            div { class: "row my-3",
                div { class: "col-auto",
                    button { class: "btn btn-secondary", 
                    onclick: move |_| {
                        navigator.push(Route::Home {});
                    }, 
                    "Go Back" }
                }
            }
            // 2nd Row: Number of Cards and jToE Checkbox
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
                
                div { class: "col-md-6",
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
                }
            }


            // 3rd Row - Generate Button
            div { 
                //    `justify-content-end` pushes everything to the right.
                //    `align-items-center` vertically centers the message and button.
                class: "row d-flex justify-content-end align-items-center",
                

                // 3. The button block, also using `col-auto`.
                div { class: "col-auto",
                    button {
                        class: "btn btn-primary btn-lg",
                        r#type: "button",
                        onclick: move |_| {
                            

                            // randomly select words from the select_words context
                            let mut rng = rng();
                            let mut words = select_words();
                            words.shuffle(&mut rng);
                            let selected: Vec<WordRecord> = words.into_iter().take(number_of_cards()).collect();
                            select_words.set(selected);

                            eprintln!("Selected {} words for test cards", select_words.len());
                            // eprintln!("the cards picked are: {:?}", select_words.read());

                            status_message.set(StatusMessage {
                                message: "Generating test cards successfully".to_string(),
                                level: StatusLevel::Info,
                            });
                            navigator.push(Route::TestCard { j_to_e: j_to_e() });


                        },
                        "Generate Test Cards"
                    }
                }
            }
        }
    )


}


/// Define the actions the user can perform
#[derive(Debug, Clone, Copy)]
enum TestcardAction {
    ShowQuestion,
    MarkFamiliar,
    MarkUnfamiliar,
    UserMark,
    UserPickAnswer(usize),
    Pronounce,
}


/// this component is the main test card interface
/// It's most the same as flashcard, but with multiple choice answers
#[component]
pub fn TestCard(j_to_e: bool) -> Element {
    let navigator = use_navigator();
    let mut index = use_signal(|| 0 as usize); // current index in select_words
    let select_words = use_context::<Signal<Vec<WordRecord>>>();
    let total_cards = select_words.len();
    let mut status_message = use_context::<Signal<StatusMessage>>();
    
    // --- Signals for UI State ---
    let mut show_question = use_signal(|| true);
    let mut show_reading = use_signal(|| true);
    let mut show_answer = use_signal(|| false);
    let mut is_marked = use_signal(|| false);
    let mut selected_answer = use_signal(|| None as Option<usize>);
    let mut correct_answer_index = use_signal(|| 0 as usize);
    
    // --- content for UI ---
    let mut question = use_signal(|| "".to_string());
    let mut reading = use_signal(|| "".to_string());
    let mut vector_of_answers = use_signal(|| vec!["".to_string(); 4]);
    let mut score = use_signal(|| 0);

    // --- pool for db op ---
    let db_pool = use_context::<sqlx::SqlitePool>();
    let pool_action = db_pool.clone(); // pool for km_actions

    // --- voice for tts ---
    let voice = return_voice("ja", Gender::Male)?;

    // --- setup button class and icon for user_mark
    let button_class = if is_marked() {
        "btn btn-lg btn-warning" // Solid yellow if marked
        } else {
            "btn btn-lg btn-outline-warning" // Outline if not marked
        };
    let star_icon = if is_marked() { "★" } else { "☆" }; // Solid vs. Outline star

    // 1. --- Create a reusable closure to load a card ---
    // This closure takes the index of the card to load.
    let mut load_card = move |card_index: usize| {
        if let Some(word) = select_words.get(card_index) {
            eprintln!("Loading card at index {}: {:?}", card_index, word);
            reading.set(word.reading.clone());
            is_marked.set(word.user_mark);

            // randomly select an the correct answer index from 0 to 3
            let idx = rng().random_range(0..=3);
            correct_answer_index.set(idx);
            eprintln!("the correct answer index is: {}", correct_answer_index());

            // pick the rest of the answers from the select_words
            // Exclude the current word index from the selection
            let mut possible_numbers: Vec<usize> = (0..=select_words.len() - 1).collect();
            possible_numbers.retain(|&x| x != card_index); // Exclude the current word index
            
            // from the possible numbers, randomly select 3 indices
            let mut rngen = rng();
            let mut chosen_numbers: Vec<usize> = possible_numbers
                .choose_multiple(&mut rngen, 3) // Choose 3 random indices
                .cloned() // Clone the &usize references into usize values
                .collect(); // Collect the values into a new Vec

            eprintln!("Chosen numbers for other answers: {:?}", chosen_numbers);

            if j_to_e {
                question.set(word.expression.clone());

                // Fill the correct answer at the random index
                vector_of_answers.write()[correct_answer_index()] = word.meaning.clone();
                for i in 0..4 {
                    if i != correct_answer_index() {
              
                        let idx = chosen_numbers.pop().unwrap();
                        let random_word = select_words.read()[idx].clone();
                        vector_of_answers.write()[i] = random_word.meaning.clone();
                    }
                }
                
            } else {
                question.set(word.meaning.clone());

                vector_of_answers.write()[correct_answer_index()] = word.expression.clone();
                for i in 0..4 {
                    if i != correct_answer_index() {
              
                        let idx = chosen_numbers.pop().unwrap();
                        let random_word = select_words.read()[idx].clone();
                        vector_of_answers.write()[i] = random_word.expression.clone();
                    }
                }
            }
            eprintln!("vector_of_answers: {:?}", vector_of_answers());
            // eprintln!("4th answer: {}", vector_of_answers()[3]);
            // Always hide the answer when loading a new card
            show_answer.set(false);
        }
    };

    // 2. --- Effect for Initial Load ---
    // This effect runs once when the component mounts to load the first card.
    // and whenever the index changes.
    use_effect(move || {
        load_card(index());
    }); 

    // 3. --- Simplified Event Handler ---
    let mut go_to_next_card = move || {
        let current_index = index();
        let next_index = if current_index < total_cards - 1 {
            current_index + 1
        } else {
            status_message.set(StatusMessage {
                message: "End of cards, looping back to the start.".to_string(),
                level: StatusLevel::Info,
            });
            // Show final score
            status_message.set(StatusMessage {
                message: format!("Test finished! Your score: {}/{}", score(), total_cards),
                level: StatusLevel::Info,
            });
            0 // Loop back to the start
        };

        index.set(next_index); // Update the index signal
        // load_card(next_index); // remove this line to avoid reloading the card
    };

    // 4. --- Use coroutine to handle keyboard/mouse event ---
    // The coroutine will handle all keyboard/mouse events.
    // We give it a name `km_actions` to send messages to it.
    let km_actions = use_coroutine( move |mut rx: UnboundedReceiver<TestcardAction>| {
        // Clone all the state the logic will need into the coroutine
        let pool = pool_action.clone();

        // prepare the voice configuration   
        let voice_to_use = voice.clone(); // Clone the voice configuration

        async move {
            // This loop waits for messages to be sent to the coroutine
            while let Some(action) = rx.next().await {
                let Some(word_id) = select_words.get(index()).map(|w| w.id) else {
                    eprintln!("Could not get word at current index.");
                    continue;
                };
                match action {
                    TestcardAction::ShowQuestion => {
                        eprintln!("Showing question for word {}", word_id);
                        show_question.set(!show_question());
                    }
                    TestcardAction::MarkUnfamiliar => {
                        if !show_answer() {
                            eprintln!("Cannot mark as 'Needs Practice' without showing the answer first");
                            status_message.set(StatusMessage {
                                message: "Please show the answer before marking as 'Needs Practice'.".to_string(),
                                level: StatusLevel::Warning,
                            });
                            continue; // Skip if answer is not shown
                        }
                        eprintln!("Marking word {} as 'Needs Practice'", word_id);
                        match ProgressUpdate::new()
                            .increment_practice_time()
                            .set_familiar(false)
                            .execute(&pool, word_id)
                            .await
                        {
                            Ok(_) => eprintln!("id: {:?} updated unfamiliar successfully", word_id),
                            Err(e) => eprintln!("Background update failed: {}", e),
                        }
                         go_to_next_card();
                    }
                    TestcardAction::MarkFamiliar => {
                        if !show_answer() {
                            eprintln!("Cannot mark as 'Got It!' without showing the answer first");
                            status_message.set(StatusMessage {
                                message: "Please show the answer before marking as 'Got It!'.".to_string(),
                                level: StatusLevel::Warning,
                            });
                            continue; // Skip if answer is not shown
                        }
                     
                        match ProgressUpdate::new()
                            .increment_practice_time()
                            .set_familiar(true)
                            .execute(&pool, word_id)
                            .await
                        {
                            Ok(_) => eprintln!("id: {:?} updated familiar successfully", word_id),
                            Err(e) => eprintln!("Background update failed: {}", e),
                        }
                         go_to_next_card();
                    }
                    TestcardAction::UserMark => {
                        eprintln!("User mark for word {}", word_id);
                        is_marked.set(!is_marked());
                        match ProgressUpdate::new()
                            .set_user_mark(is_marked())
                            .execute(&pool, word_id)
                            .await
                        {
                            Ok(_) => eprintln!("id: {:?} mark {:?} successfully", word_id, is_marked()),
                            Err(e) => eprintln!("Background update failed: {}", e),
                        }
                    }
                    TestcardAction::UserPickAnswer(selected) => {
                        if show_answer() {
                            eprintln!("Answer already displayed for word {}", word_id);
                            continue; // Skip if answer is already shown
                        }
                        eprintln!("Displaying answer for word {}", word_id);
                        selected_answer.set(Some(selected));
                        show_answer.set(true);

                        // Check if the selected answer is correct
                        if selected == correct_answer_index() {
                            score.set(score() + 1);
                        }
                    }   
                    TestcardAction::Pronounce => {
                        eprintln!("Pronouncing word {}", word_id);

                        let text_to_speak = reading(); // Clone the text to speak
                        let voice_to_use = voice_to_use.clone(); // Clone the voice configuration
                        
                        // --- SPAWN THE THREAD ---
                        // This moves the entire block of work to a background thread.
                        speak_text(text_to_speak, voice_to_use, 5, Some(1.1), Some(0.9));
                    }
                }


            }
        
        }
    });


    // 5. --- For each card, determine the class ---
    let get_card_class = |i: usize| {
        if let Some(selected) = selected_answer() {
            if show_answer() {
                if i == correct_answer_index() {
                    "card mb-2 bg-success text-light"
                } else if i == selected {
                    "card mb-2 bg-danger text-light"
                } else {
                    "card clickable mb-2 bg-dark text-light"
                }
            } else {
                "card clickable mb-2 bg-dark text-light"
            }
        } else {
            "card clickable mb-2 bg-dark text-light"
        }
    };

    rsx!(

        div { 
            class: "container h-75 d-flex flex-column",
            // Add the onkeydown listener to this wrapping div

            // --- Make the div focusable ---
            tabindex: "0", // <-- Make it focusable!
            onmounted: move |evt| {
                // Focus the div when mounted to capture key events
                let element = evt.data();
                spawn(async move {
                    let _ = element.set_focus(true).await;
                });
            },
            onkeydown: move |event: KeyboardEvent| {
                match event.key() {
                    // Check for 'n' or 'N'
                    Key::Character(s) if s.eq_ignore_ascii_case("q") => {
                        eprintln!("q key pressed, showing/hiding question");
                        km_actions.send(TestcardAction::ShowQuestion);
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("n") => {
                        eprintln!("n key pressed, marking as unfamiliar");
                        km_actions.send(TestcardAction::MarkUnfamiliar);
                    },
                    // Add a key for the second button, e.g., 'G' for "Got it!"
                    Key::Character(s) if s.eq_ignore_ascii_case("g") => {
                        eprintln!("g key pressed, marking as familiar");
                        km_actions.send(TestcardAction::MarkFamiliar);
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("m") => {
                        eprintln!("m key pressed, toggling user mark");
                        km_actions.send(TestcardAction::UserMark);
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("1") => {
                        eprintln!("1 key pressed, displaying answer");
                        km_actions.send(TestcardAction::UserPickAnswer(0));
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("2") => {
                        eprintln!("2 key pressed, displaying answer");
                        km_actions.send(TestcardAction::UserPickAnswer(1));
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("3") => {
                        eprintln!("3 key pressed, displaying answer");
                        km_actions.send(TestcardAction::UserPickAnswer(2));
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("4") => {
                        eprintln!("4 key pressed, displaying answer");
                        km_actions.send(TestcardAction::UserPickAnswer(3));
                    },
                    Key::Character(s) if s.eq_ignore_ascii_case("p") => {
                        eprintln!("p key pressed, pronouncing word");
                        km_actions.send(TestcardAction::Pronounce);
                    },
                    // Ignore any other key presses
                    _ => {}
                }
            },

            // --- Top Controls ---
            div { class: "row my-3",
                div { class: "col-auto",
                    button { class: "btn btn-secondary", 
                    onclick: move |_| {
                        navigator.push(Route::GnerateTestCard {});
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
                        onclick: move |_| km_actions.send(TestcardAction::UserMark),                                                    
                        {star_icon}
                    }
                }

                if show_reading() {
                    div { class: "col d-flex justify-content-between align-items-center",
                        p { class: "lead my-3", "{reading()}" }
                        button { class: "btn btn-light",
                            onclick: move |_| km_actions.send(TestcardAction::Pronounce),
                            "🔊"}
                        }
                    }
                
                // --- Answers Section ---
                div { class: "col",
             
                        div { class: "d-flex flex-column gap-3",

                            for i in 0..4 {                                
                                div {
                                    class: get_card_class(i),
                                    style: "cursor: pointer;",
                                    onclick: move |_| {
                                        km_actions.send(TestcardAction::UserPickAnswer(i as usize));
                                    },
                                    div {
                                        class: "card-body d-flex align-items-center",
                                        span {
                                            class: "badge bg-secondary me-3",
                                            style: "width: 2rem;",
                                            u {"{i+1}"}
                                        }
                                        span { "{vector_of_answers()[i]}" }
                                    }
                                }
                            }

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
                    onclick: move |_| km_actions.send(TestcardAction::MarkUnfamiliar),
                    u {"N"}, "eed more practice" }
                }
                div { class: "col",
                    button { class: "btn btn-success w-100", 
                    onclick: move |_| km_actions.send(TestcardAction::MarkFamiliar),                 
                    u {"G"}, "ot it!" }
                }
            }
        }
    )
}