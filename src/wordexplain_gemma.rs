use futures_util::StreamExt;
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;

use dioxus::prelude::*;
use crate::utils::{word_process};
use crate::return_voice;
use crate::footer::{StatusMessage, StatusLevel};
use tts::*;




// The structs to hold our parsed data (same as before)
#[derive(Deserialize, Debug)]
struct WordExplanation {
    explain: String,
    example1: Example,
    example2: Example,
    example3: Example,
}

#[derive(Deserialize, Debug)]
struct Example {
    sentence: String,
    translation: String,
}

// A new struct to hold all the data for one processed sentence
#[derive(Clone, PartialEq)]
struct ProcessedSentence {
    original: String,
    translation: String,
    romaji: String,
    words: Vec<String>,
}


/// --- Parsing function is the same ---
fn parse_llm_json(raw_text: &str) -> Result<WordExplanation, Box<dyn Error>> {
    // (Your robust parsing logic from the previous step goes here)
    let start_byte = raw_text.find('{').ok_or("Could not find opening '{'")?;
    let end_byte = raw_text.rfind('}').ok_or("Could not find closing '}'")? + 1;
    let json_slice = &raw_text[start_byte..end_byte];
    Ok(serde_json::from_str(json_slice)?)
}

/// Asynchronous function to get word explanation from the LLM
/// This function uses the Ollama client to send a request and receive a response from Gemma:4b model.
async fn get_word_explanation(
    ollama_client: Arc<Ollama>,
    word_to_explain: &str,
) -> Result<WordExplanation, Box<dyn Error>> {
    let model = "gemma3:4b".to_string();
    
    
    let prompt = format!(
        r#"Explain the Japanese word '{}'. Provide the explanation and 3 example sentences in Japanese. The 'translation' for each example should be in English. Structure the output as a JSON object with the following keys: {{ 
            "explain": "...", 
            "example1": {{"sentence": "...", "translation": "..."}},
            "example2": {{"sentence": "...", "translation": "..."}},
            "example3": {{"sentence": "...", "translation": "..."}}
        }}"#,
        word_to_explain
    );

    eprint!("Requesting explanation for '{}'\n", word_to_explain); 

    let request = GenerationRequest::new(model, prompt);
    let response = ollama_client.generate(request).await?;
    
    // Parse the response and return the structured data
    parse_llm_json(&response.response)
}






#[component]
pub fn WordExplainer(word_to_explain: String) -> Element {
    // Get the shared Ollama client from the context.
    let ollama_client = use_context::<Arc<Ollama>>();
    
    // signal to hold the explaination text generated by the LLM
    let mut explanation_text = use_signal(|| String::new());
    
    // signal to hold processed sentences
    let mut processed_sentences = use_signal(|| Vec::<ProcessedSentence>::new());


    let mut status_message = use_context::<Signal<StatusMessage>>();



    // Create a coroutine to handle the async task
    let coroutine = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        // Clone the client and state setter to move into the async block
        let client = ollama_client.clone();

        status_message.set(StatusMessage {
            message: format!("Waiting for LLM, please be patient..."),
            level: StatusLevel::Info,
        });
        
        async move {
            while let Some(word) = rx.next().await {
                status_message.set(StatusMessage {
                    message: format!("Waiting for LLM, please be patient..."),
                    level: StatusLevel::Info,
                });
                // Call the async function to get the explanation
                if let Ok(explanation) = get_word_explanation(client.clone(), &word).await {
                    
                    status_message.set(StatusMessage {
                        message: format!("Explanation for '{}' retrieved successfully.", word),
                        level: StatusLevel::Success,
                    });
                    
                    // Update the main explanation text
                    explanation_text.set(explanation.explain.clone());
                    
                    // Process all three example sentences and update our vector
                    let mut sentences = Vec::new();
                    let examples = vec![explanation.example1, explanation.example2, explanation.example3];
                    for ex in examples {
                        if let Ok((words, romaji)) = word_process(&ex.sentence) {
                            sentences.push(ProcessedSentence {
                                original: ex.sentence,
                                translation: ex.translation,
                                romaji,
                                words,
                            });
                        }
                    }
                    processed_sentences.set(sentences);
                } else {
                    status_message.set(StatusMessage {
                        message: format!("Failed to retrieve explanation for '{}'.", word),
                        level: StatusLevel::Error,
                    });
                }
            }
        
        }});

    use_effect({
        let coroutine = coroutine.clone();
        move || {
            // coroutine.send("食べちゃいました".to_string());
            coroutine.send(word_to_explain.to_string());
        }
    });
    
           


    rsx! {
        div { class: "card mb-4",
            div { class: "card-body bg-dark text-light",
                h3 { class: "card-title", "Explanation" }
                // Render the main explanation text
                p { class: "card-text", "{explanation_text}" }
            }
        }

        // Loop through our processed sentences and render an InteractiveSentence for each one
        for sentence_data in processed_sentences() {
            div { class: "card mb-2  bg-dark text-light",
                InteractiveSentence {
                    sentence_data: sentence_data.clone(),
                    on_word_click: coroutine.clone()
                }
            }
        }
    }


}



#[component]
fn InteractiveSentence(
    sentence_data: ProcessedSentence,
    // We pass the coroutine down from the parent so this component can send messages to it
    on_word_click: Coroutine<String>, 
) -> Element {
    // --- voice for tts ---
    let voice_to_use = return_voice("ja", Gender::Male)?;

    rsx! {
        

        div { class: "card-body bg-dark text-light",
            div { class: "row mb-2",
                div { class: "col-10 d-flex flex-column justify-content-center",
                    div { class: "row mb-1",
                        div { class: "col-3 text-secondary small", "Sentence:" }
                        div { class: "col-9", "{sentence_data.original}" }
                    }
                    div { class: "row mb-1",
                        div { class: "col-3 text-secondary small", "Romaji:" }
                        div { class: "col-9 text-secondary small", "{sentence_data.romaji}" }
                    }
                    div { class: "row mb-1",
                        div { class: "col-3 text-secondary small", "English:" }
                        div { class: "col-9 text-secondary small", "{sentence_data.translation}" }
                    }
                    div { class: "row mb-1",
                        div { class: "col-3 text-secondary small", "Origins:" }
                        div { class: "col-9 japanese-sentence-interactive",
                            for word in &sentence_data.words {
                                button {
                                    class: "btn btn-link p-0 text-decoration-none text-info", // Use 'text-info' for clickable links in dark mode
                                    style: "font-size: 1.25rem; margin-right: 2px;",
                                    // When a word is clicked, send it to the parent's coroutine
                                    onclick: {
                                        
                                        let word = word.clone();
                                        let on_word_click = on_word_click.clone();
                                        move |_| on_word_click.send(word.clone())
                                    },
                                    "{word}"
                                }
                            }
                        }
                    }
                }
                div { class: "col-2 d-flex align-items-center justify-content-end",
                    button { class: "btn btn-light btn-sm",
                        onclick: move |_| {
                            eprintln!("Pronouncing word {}", sentence_data.original.clone());
                            let text_to_speak = sentence_data.original.clone();
                            let voice_to_use = voice_to_use.clone();
                            std::thread::spawn(move || {
                                match Tts::default() {
                                    Ok(mut tts) => {
                                        if tts.set_voice(&voice_to_use).is_err() {
                                            eprintln!("[Thread] Error: Failed to set voice.");
                                        }
                                        let _ = tts.speak(text_to_speak, false);
                                        std::thread::sleep(std::time::Duration::from_secs(10));
                                    },
                                    Err(e) => {
                                        eprintln!("[Thread] Error: {}", e);
                                    }
                                }
                            });
                        },
                        "🔊"
                    }
                }
            }
            // ... (your interactive word buttons below, if needed)
        }
    }
}







#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llm_json() {
        let raw_response = r#"```json
            {
                "explain": "「食べちゃいました」is a very common and informal Japanese phrase used to describe eating something accidentally or without intending to. It’s a softened, apologetic way of saying 'I ate it' or 'I unintentionally ate it'. It implies a slight embarrassment or surprise, and often carries a playful or slightly self-deprecating tone. It’s more casual than saying 食べました (tabe mashita), which is the polite form.  The ‘ちゃ’ (cha) is a reduplicated particle that adds to the feeling of something happening unexpectedly and unintentionally.",
                "example1": {
                    "sentence": "おやつを食べてしまってごめんね。",
                    "translation": "I accidentally ate the snack, sorry."
                },
                "example2": {
                    "sentence": "まだご飯食べてないのに、お寿司を食べてしまってしまった。",
                    "translation": "I accidentally ate sushi even though I hadn't eaten yet."
                },
                "example3": {
                    "sentence": "ケーキを全部食べちゃいました！",
                    "translation": "I finished/ate the entire cake!"
                }
            }
            ```"#;
        let explanation = parse_llm_json(raw_response).unwrap();
        assert_eq!(explanation.example1.sentence, "おやつを食べてしまってごめんね。");
    }
}