
use rand::seq::IndexedRandom;
use wana_kana::ConvertJapanese;
use lindera::{dictionary::load_dictionary_from_kind};
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;
use dioxus::prelude::*;
use crate::{utils::{get_pos_color_class, PartOfSpeech}};
use std::sync::Arc;
use std::error::Error;
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use crate::footer::{StatusMessage, StatusLevel};
use tts::*;
use crate::return_voice;

// use futures_util::StreamExt;

/// this struct represents a word token with its properties
#[derive(Clone, Debug, PartialEq)]
pub struct WordToken {
    pub surface: String,      // The word as it appears in the text (e.g., "食べ")
    pub part_of_speech: String, // e.g., "動詞" (Verb)
    pub base_form: String,      // The dictionary form (e.g., "食べる")
    pub romaji: String,         // The romaji reading (e.g., "tabe")
}

/// This struct is used to deserialize the JSON response from the LLM
#[derive(serde::Deserialize)]
struct StoryResponse {
    story: String,
    english: String,
}

/// --- Parsing function is the same ---
fn parse_llm_story_json(raw_text: &str) -> Result<(String, String), Box<dyn Error>> {
    // (Your robust parsing logic from the previous step goes here)
    let start_byte = raw_text.find('{').ok_or("Could not find opening '{'")?;
    let end_byte = raw_text.rfind('}').ok_or("Could not find closing '}'")? + 1;
    let mut json_slice = &raw_text[start_byte..end_byte];

    // Remove control characters (except \n, \r, \t if you want to keep them)
    let cleaned: String = json_slice.chars()
        .filter(|c| !c.is_control())
        .collect();
    let resp: StoryResponse = serde_json::from_str(&cleaned)?;
    Ok((resp.story, resp.english))
}

/// this function generates a story using a list of words.
async fn get_story(
    ollama_client: Arc<Ollama>, 
    all_words: &[String]
)  -> Result<(String, String), Box<dyn Error>> {
    // Ensure we have at least 10 words to choose from
    if all_words.len() < 10 {
        return Err("Not enough words to generate a story.".to_string().into());
    }

    let model = "gemma3:4b".to_string();

    // Randomly select 10 words
    let mut thrng = rand::rng();
    let selected_words: Vec<String> = all_words
        .choose_multiple(&mut thrng, 10)
        .cloned()
        .collect();
    
    // Join the words into a comma-separated string for the prompt
    let word_list = selected_words.join("、 ");

    // Construct the final, robust prompt
    let prompt = format!(
        r#"You are a creative writer for a Japanese language learner.

        Follow these instructions precisely:
        1. Write a short story in Japanese, approximately 100-150 words long.
        2. The story MUST include the following Japanese words: {word_list}
        3. The story must ONLY contain Japanese characters (Kanji, Hiragana, Katakana). Do NOT include furigana, Romaji, or any parenthetical notes in the story.
        4. Provide a separate, full English translation of the story.
        5. Structure your entire response as a single, valid JSON object following the template below. Do not add any text before or after the JSON object.

        JSON Template:
        {{
          "story": "...",
          "english": "..."
        }}"#
    );

    eprintln!("Requesting word list for story are '{}'\n", word_list); 

    let request = GenerationRequest::new(model, prompt);
    let response = ollama_client.generate(request).await?;
    eprintln!("Response from Ollama: {}\n", response.response);
    parse_llm_story_json(&response.response)
        
}




/// This function takes the raw story text and processes it into our structured data.
pub fn process_story_text(text: &str) -> Result<Vec<WordToken>, Box<dyn std::error::Error>> {
    let dictionary = load_dictionary_from_kind(lindera::dictionary::DictionaryKind::IPADIC)?;
    let segmenter = Segmenter::new(Mode::Normal, dictionary, None);
    let tokenizer = Tokenizer::new(segmenter);

    let mut word_tokens = Vec::new();


    let mut tokens = tokenizer.tokenize(&text)?;


    for token in tokens.iter_mut() {
        // This is the key: call .details() to populate the field.
        // We don't need to use the return value of this function call.
        token.details();

        if let Some(details) = &token.details {
            if details.len() > 8 {
                word_tokens.push(WordToken {
                    surface: token.text.to_string(),
                    part_of_speech: details[0].to_string(), // e.g., "名詞"
                    base_form: details[6].to_string(),      // e.g., "猫"
                    // Lindera provides Katakana reading, so we convert it to Romaji
                    romaji: details[8].to_romaji(),
                });
            }
        }
    }

    Ok(word_tokens)
}


#[component]
pub fn StoryGenerator() -> Element {
    let mut status_message = use_context::<Signal<StatusMessage>>();

    let ollama_client = use_context::<Arc<Ollama>>();

    let words_to_use = use_context::<Signal<Vec<String>>>();
    
    // State to hold the generated story and its translation
    let mut story_data = use_signal(|| None as Option<(String, String)>);


    rsx! {
        div { class: "container py-4",
            h2 { "Story Generator" }
            p { class: "text-secondary", "Generate a new story using your practiced words."}

            // --- The "Smart" Button ---
            div { class: "mb-4",
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {

                        

                        let words = words_to_use.clone();
                        let client = ollama_client.clone();

                        status_message.set(StatusMessage {
                            message: format!("Waiting for LLM, please be patient..."),
                            level: StatusLevel::Info,
                        });
                    
                        async move {
                            eprintln!("Generating story with words: {:?}", words());
                            if words.is_empty() {
                                status_message.set(StatusMessage {
                                    message: "No words available to generate a story.".to_string(),
                                    level: StatusLevel::Error,
                                });
                                return;
                            }
                            match get_story(client, &words()).await {
                                Ok((story, english)) => {
                                    eprintln!("Story generated successfully.");
                                    story_data.set(Some((story, english))); // Store the story and translation
                                    status_message.set(StatusMessage {
                                        message: format!("Story generated successfully."),
                                        level: StatusLevel::Success,
                                    });
                                }
                                Err(e) => {
                                    eprintln!("Error generating story: {}", e);
                                    status_message.set(StatusMessage {
                                        message: format!("Error generating story: {}", e),
                                        level: StatusLevel::Error,
                                    });
                                    
                                }
                            };
                        }
                    },
                    "Generate Story from Recent Words"
                }
                // You could add a dropdown here for other options
            }
        }
    
    
    
        // --- Conditional Rendering ---
        // If a story has been generated, render the InteractiveStory component.
        // Otherwise, show a placeholder.
        if let Some((story, english)) = story_data() {
            
            InteractiveStory {
                story_text: story,
                english_translation: english,
            }
        
        } else {
        
            div { class: "text-center text-muted p-5 border rounded",
                "Click the button to generate a story."
            }
        
        }
        
    
    }   
}

#[component]
pub fn InteractiveStory(story_text: String, english_translation: String) -> Element { 

    // will be used for tts 
    let story = story_text.clone();
    let voice_to_use = return_voice("ja", Gender::Male)?;


    let processed_tokens = process_story_text(&story_text).ok();

    
    rsx! {
        div { class: "story-container p-3 border rounded",

            p {
                class: "text-secondary fst-italic", // Muted and italicized for context
                "{english_translation}"
            }

            hr {} // A horizontal rule to separate the translation from the story

            div { class: "col-2 d-flex align-items-center justify-content-end",
                button { class: "btn btn-light btn-sm",
                    onclick: move |_| {
                        eprintln!("Pronouncing word {}", story.clone());
                        let text_to_speak = story.clone();
                        let voice_to_use = voice_to_use.clone();
                        std::thread::spawn(move || {
                            match Tts::default() {
                                Ok(mut tts) => {
                                    if tts.set_voice(&voice_to_use).is_err() {
                                        eprintln!("[Thread] Error: Failed to set voice.");
                                    }
                                    let _ = tts.speak(text_to_speak, false);
                                    std::thread::sleep(std::time::Duration::from_secs(60));
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

            if let Some(tokens) = processed_tokens.as_ref() {
                for token in tokens {
                    div { class: "word-unit",
                        div {
                            class: "tooltip-container",
                            span { class: "main-word {get_pos_color_class(&PartOfSpeech::from(&*token.part_of_speech.as_str()))}", "{token.surface}" }
                            span {
                                class: "tooltip-text",
                                "Type: {token.part_of_speech}\nBase: {token.base_form}"
                            }
                        }
                        span { class: "romaji-reading", "{token.romaji}" }
                    }
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_story() {
        let story = r#"今日、雨が降っていました。空は灰色で、窓の外はとても寂しそうでした。
            私は家で、古い本 (ほん - hon) を読みました。物語は興味深いもので、時間を忘れました。私はコーヒーを
            飲み、少しだけコンピュータで作業しました。その後、友達と電話 (でんわ - denwa) で話しました。
            話は楽しいもので、私はとても嬉しい気持ちになりました。  雨上がりの景色は、とても綺麗でした。"#;

            let word_tokens = process_story_text(story).unwrap();
            println!("surface: {:?}", word_tokens[0]);
            for token in word_tokens {
                println!("Surface: {}, POS: {}, Base Form: {}, Romaji: {}",
                         token.surface, token.part_of_speech, token.base_form, token.romaji);
            }
    }
}