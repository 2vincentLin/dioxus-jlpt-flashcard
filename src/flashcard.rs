

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
    // word_id is used to pass it to the database to update the practice count
    let mut word_id = use_signal(|| -1 as i64);
    // familiar is ued to pass it to the database to update the familiar status
    let mut familiar = use_signal(|| false);
    // word_id_f is used to pass it to the database to update the familiar status
    let mut id_familiar = use_signal(|| (-1 as i64, false));


    // region: return word reousrce based on selected word ids
    // it'll automatically re-fetch when the signal changes 
    // (select_word, number_of_cards, j_to_e, unfamiliar_only, random_shuffle, tag_level)
    let word_resource = use_resource(move || async move {
        if current_index() == 101 {
            eprintln!("use_resource for word resource early return");
            return;
        }
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

    // region: resource - increase practice count
    // this will be called when word_id changes
    let _ = use_resource(move || async move {
        if word_id() == -1 {
            eprintln!("use_resource for increase practice count early return");
            return;
        }
        eprintln!("use_resource for increase practice count called");
        match SqlitePoolOptions::new()
            .max_connections(5)
            .connect(DB_URL)
            .await {
                Ok(pool) => {
                    
                    match increment_practice_time(&pool, word_id()).await {
                        Ok(_) => {
                            eprintln!("Practice count updated successfully");
                        },
                        Err(e) => {
                            eprintln!("Practice count updated fail: {}", e);
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

    // region: resource - update familiar
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
            
            // Dropdown
            select {
                id: "number_select",
                value: "{number_of_cards}",
                style: "margin: 8px;",
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
            label { "number of card" }
            
            input { r#type: "checkbox", id: "checkbox1", style: "margin: 8px;" }
            label { r#for: "checkbox1", "Option 1"}

            select {
                id: "level_select",
                value: "{tag_level}",
                style: "margin: 8px;",
                oninput: move |evt| {
                    // Update the level based on dropdown selection
                    tag_level.set(evt.value()); },
                option { value: TagLevel::N1.to_string(), "N1" }
                option { value: TagLevel::N2.to_string(), "N2" }
                option { value: TagLevel::N3.to_string(), "N3" }
                option { value: TagLevel::N4.to_string(), "N4" }
                option { value: TagLevel::N5.to_string(), "N5" }

            }

            // region: generate flashcard button
            button { 
                id: "generate",
                style: "margin: 8px;",
                onclick: move |_| {
                    // Logic to handle "Start Study" button click
                     // Clear the selected words

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
        br {}

        div {
            // Three checkboxes
            input { style: "margin: 8px;", r#type: "checkbox", id: "checkbox3" , 
                checked: j_to_e,
                oninput: move |evt| j_to_e.set(evt.value().parse::<bool>().unwrap_or(false))}
            label { r#for: "checkbox3", "JtoE" }

            input { style: "margin: 8px;", r#type: "checkbox", id: "checkbox4" , 
                checked: *unfamiliar_only.read(),
                oninput: move |evt| unfamiliar_only.set(evt.value().parse::<bool>().unwrap_or(false))}
            label { r#for: "checkbox4", "unfamilia only" }

            input { style: "margin: 8px;", r#type: "checkbox", id: "checkbox5" , 
                checked: random_shuffle,
                oninput: move |evt| random_shuffle.set(evt.value().parse::<bool>().unwrap_or(false))}
            label { r#for: "checkbox5", "random shuffle card" }
        }
        br {}

        div {
            // Large label for question
            label { id: "question", "{question}" }
        }
        br {}

        div {

            // region: answer label
            // Large label for answer
            label { id: "answer", 
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
                            answer.set(text.clone());;
                            
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
                class: "btn-large",
                style: "background-color: pink; margin: 16px; size: 50px;",
                
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
                            familiar.set(false);
                            word_id.set(word.id);
                            eprintln!("word_id: {}", word_id());
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
                class: "btn-large",
                style: "background-color: green; margin: 16px; size: 50px;",

                
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
}
