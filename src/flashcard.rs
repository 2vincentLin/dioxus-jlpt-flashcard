

use dioxus::{prelude::*};
use crate::db::*;
use sqlx::sqlite::SqlitePoolOptions;
use crate::Route;

// update the flashcard based on the current index and selected words
fn update_flashcard(
    j_to_e: bool,
    current_index: usize,
    select_word: Signal<Vec<WordRecord>>,
    mut question: Signal<String>,
    mut answer: Signal<String>,
    mut answer_visible: Signal<bool>,
    ) {
        answer_visible.set(false);
        if let Some(word) = select_word.get(current_index) {
            if j_to_e {
                let text = format!("{} \n({})", word.expression, word.reading);
                question.set(text.clone());
                answer.set("Click me for answer".to_string());
            } else {
                question.set(word.meaning.clone());
                answer.set("Click me for answer".to_string());

            }
        } else {
            eprintln!("No words selected");
        }
    }



#[component]
pub fn FlashCard() -> Element {
    let mut question = use_signal(|| "".to_string());
    let mut answer = use_signal(|| "".to_string());
    let mut answer_visible = use_signal(|| false);
    let mut current_index = use_signal(|| 101 as usize);
    let mut number_of_cards = use_signal(|| 15);
    let mut j_to_e= use_signal(|| true);
    let mut unfamiliar_only = use_signal(|| true);
    let mut random_shuffle = use_signal(|| true);
    let mut tag_level = use_signal(|| TagLevel::N5.to_string());

    let mut select_word_ids = use_signal(|| Vec::<i64>::new());
    let mut select_word = use_signal(|| Vec::<WordRecord>::new());
    // used this id to track the current word index
    let mut separate_ids = use_signal(|| 0 as usize);

    // this is used to update the familiar status and increment the practice count
    let mut id_familiar = use_signal(|| (-1 as i64, false));

    let mut generate_flashcard = use_signal(|| false);


    // region: return word reousrce based on selected word ids
    // it'll automatically re-fetch when the signal changes 
    // (select_word, number_of_cards, j_to_e, unfamiliar_only, random_shuffle, tag_level)
    let word_resource = use_resource(move || async move {
        
        eprintln!("use_resource for retrive record called");
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
                            select_word.set(Vec::new());
                            select_word.set(records);
                            
                            // update flashcard after fetching words
                            update_flashcard(
                                j_to_e(),
                                current_index(),
                                select_word,
                                question,
                                answer,
                                answer_visible,
                            );
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
        
    });

    // endregion



    // region: resource - update familiar and increment practice count
    let _ = use_resource(move || async move {
        if id_familiar().0 == -1 {
            eprintln!("use_resource for update familiar early return");
            return;
        }

        eprintln!("use_resource for update familiar called");
        match SqlitePoolOptions::new()
            .max_connections(5)
            .connect(DB_URL)
            .await {
                Ok(pool) => {
                    match update_familiar(&pool,id_familiar().0, id_familiar().1).await {
                        Ok(_) => {
                            eprintln!("Familiar status updated successfully");
                        },
                        Err(e) => {
                            eprintln!("Familiar status updated fail: {}", e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error connecting to database: {}", e);
                    ()
                }
            };
    });



    // endregion



    rsx! {
        div {
            
            h2 { "FlashCard Component" }
            p { "This is your custom component." }
            style { "padding: 16px;"}
        }
    
        
        div {
            class: "d-flex flex-row align-items-center gap-3",
            div {
                label { class: "from-label", "number of card: " }
            }
            
            // Dropdown
            div {
                select {
                id: "number_select",
                value: "{number_of_cards}",
                class: "from-select",
                // style: "margin: 8px;",
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
            
            div {
                label { class: "from-label", "Choose Level: " }
            }
            div {
                select {
                id: "level_select",
                class: "from-select",
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
            
            // input { r#type: "checkbox", id: "checkbox1", style: "margin: 8px;" }
            // label { r#for: "checkbox1", "Option 1"}


            // region: generate flashcard button
            button { 
                id: "generate",
                class: "btn btn-primary",
                onclick: move |_| {
                    // Logic to handle "generate flashcard" button click

                    // word_resource.restart();
                    // this is used to trigger the resource to re-fetch
                    current_index.set(0);
                    eprintln!("Generate FlashCards clicked");
                    eprintln!("number_of_cards: {}, j_to_e: {}, unfamiliar_only: {}, random_shuffle: {}, tag_level: {}",
                        number_of_cards(), j_to_e(), unfamiliar_only(), random_shuffle(), tag_level()); 
                    
                    // we use this to fetch the words from the database
                    separate_ids.set(0);
                },
                "Generate FlashCards",
             
            }
            // endregion
        }

        div {

        }
        br {}

        div {
            // Three checkboxes
            input { class: "form-check-input", style: "margin: 8px;", r#type: "checkbox", id: "checkbox3" , 
                checked: j_to_e,
                oninput: move |evt| j_to_e.set(evt.value().parse::<bool>().unwrap_or(false))}
            label { r#for: "checkbox3", "JtoE" }

            input { class: "form-check-input", style: "margin: 8px;", r#type: "checkbox", id: "checkbox4" , 
                checked: *unfamiliar_only.read(),
                oninput: move |evt| unfamiliar_only.set(evt.value().parse::<bool>().unwrap_or(false))}
            label { r#for: "checkbox4", "unfamilia only" }

            input { class: "form-check-input", style: "margin: 8px;", r#type: "checkbox", id: "checkbox5" , 
                checked: random_shuffle,
                oninput: move |evt| random_shuffle.set(evt.value().parse::<bool>().unwrap_or(false))}
            label { r#for: "checkbox5", "random shuffle card" }
        }
        br {}

        div {
            // Large label for question
            h4 { id: "question", class: "mb-3", "{question}" }
        }
        br {}

        div {
            class: "mt-4",
            // region: answer label
            // Large label for answer
            label { id: "answer", 
            class : "fs-3 fw-bold text-success", 
            onclick: move |_| {
                    // Logic to handle question click
                    if answer_visible() {
                        return;
                    }
                    eprintln!("answer clicked");
                    eprintln!("separate_ids: {}", separate_ids());
                    if let Some(word) = select_word.get(separate_ids()) {
                        eprintln!("Selected word: {:?}", word);
                        if j_to_e() {
                            answer.set(word.meaning.clone());
                        } else {
                            let text = format!("{} \n({})", word.expression, word.reading);
                            answer.set(text.clone());
                            
                        }
                        answer_visible.set(true);
                    } else {
                        eprintln!("No words selected");
                    }
                    
                },
                "{answer}" }
            // endregion
        }
        br {}

        div {
            // Two buttons
            // region: next card buttons
            button {
                id: "study",
                class: "btn btn-secondary",
                
                onclick: move |_| {
                    // Logic to handle "I need more study" button click
                    println!("Need more study clicked");
                    // Increment the current index to show the next card
                    if separate_ids() < select_word.len() - 1 {
                        separate_ids.set(separate_ids() + 1);
                        update_flashcard(
                            j_to_e(),
                            separate_ids(),
                            select_word,
                            question,
                            answer,
                            answer_visible,
                        );
                        // update the word_id to increment the practice count
                        if let Some(word) = select_word.get(separate_ids()) {
                            id_familiar.set((word.id, false));

                            eprintln!("word.id: {}", word.id);
                        } else {
                            eprintln!("No words selected");
                        }
                    } else {
                        eprintln!("No more cards to study");
                    }

                },
                "I need more study",
            }
            button {
                id: "got_it",
                class: "btn btn-primary",

                
                onclick: move |_| {
                    // Logic to handle "I got it" button click
                    println!("Got it clicked");
                    if separate_ids() < select_word.len() - 1 {
                        separate_ids.set(separate_ids() + 1);
                        eprintln!("separate ids: {}", separate_ids());
                        
                        update_flashcard(
                            j_to_e(),
                            separate_ids(),
                            select_word,
                            question,
                            answer,
                            answer_visible,
                        );
                        // update the word_id to increment the practice count and familiar status
                        if let Some(word) = select_word.get(separate_ids()) {
                            id_familiar.set((word.id, true));

                            eprintln!("word_id: {}, familiar: {}", id_familiar().0, id_familiar().1);
                        } else {
                            eprintln!("No words selected");
                        }
                    } else {
                        eprintln!("No more cards to study");
                    }
                },
                "I got it",
            }
        }


        // endregion
        
    }
}


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
    let mut index = use_signal(|| 0 as usize);
    let select_words = use_context::<Signal<Vec<WordRecord>>>();
    let total_cards = select_words.len();
    

     // --- Signals for UI State ---
    let mut show_question = use_signal(|| true);
    let mut show_reading = use_signal(|| true);
    let mut show_answer = use_signal(|| false);

    let mut question = use_signal(|| "".to_string());
    let mut reading = use_signal(|| "".to_string());
    let mut answer = use_signal(|| "".to_string());



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

    // 4. --- update familiar event handle
    let update_db_familiar = move |familar: bool| async move {
        eprintln!("update_db_familiar is called with familar: {}", familar);
        match SqlitePoolOptions::new()
            .max_connections(5)
            .connect(DB_URL)
            .await {
                Ok(pool) => {
                    match update_familiar(&pool,index() as i64, familar).await {
                        Ok(_) => {
                            eprintln!("Familiar status updated successfully");
                        },
                        Err(e) => {
                            eprintln!("Familiar status updated fail: {}", e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error connecting to database: {}", e);
                    ()
                }
            };
    };












    rsx! {
        

        div { class: "container vh-100 d-flex flex-column",
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

                div { class: "col",
                        h2 { class: "display-4",
                            if show_question() {"{question()}" } else {""}
                        }
                }

                if show_reading() {
                    div { class: "col d-flex justify-content-between align-items-center",
                        p { class: "lead my-3", "{reading()}" }
                        button { class: "btn btn-light", "🔊" }
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
                    onclick: move |_| async move {
                        update_db_familiar(false).await;
                        go_to_next_card();
                    }, 
                    "Need more practice" }
                }
                div { class: "col",
                    button { class: "btn btn-success w-100", onclick: move |_| async move {
                        update_db_familiar(true).await;
                        go_to_next_card();
                    }, 
                    "Got it!" }
                }
            }
        }
    }
}