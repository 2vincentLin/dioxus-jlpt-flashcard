

use dioxus::{prelude::*};
use crate::db::*;
use sqlx::sqlite::SqlitePoolOptions;

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

