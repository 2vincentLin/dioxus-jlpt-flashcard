// use lindera::LinderaResult;
use lindera::dictionary::load_dictionary_from_kind;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;

use wana_kana::ConvertJapanese;

use serde::Deserialize;

/// after llm output, use extract_json_from_llm_output to extract the json and put it to this struct
#[derive(Deserialize, Debug)]
pub struct Story {
    pub story: String,
    pub english: String,
}

/// Returns a correct romaji reading for any Japanese text including kanji.
/// it use lindera and IPADIC dic to analyze the Japanese text and return pronounciate in katakana
/// then use wana_kana to convert it to romaji
pub fn romaji_pronounciation(text: &String) -> Result<String, Box<dyn std::error::Error>> {
    let dictionary = load_dictionary_from_kind(lindera::dictionary::DictionaryKind::IPADIC)?;
    let segmenter = Segmenter::new(Mode::Normal, dictionary, None);
    let tokenizer = Tokenizer::new(segmenter);

    let mut romaji_words: Vec<String> = Vec::new();
    let mut tokens = tokenizer.tokenize(&text)?;

    // the data we want is in the details field, index 8, 
    // but to access it, we have to call token.details() 1st, then it'll populate the details field
    for token in tokens.iter_mut() {
        // This is the key: call .details() to populate the field.
        // We don't need to use the return value of this function call.
        token.details();

        // Now that the field is populated, we can safely access it.
        // This `if let` will always succeed after the call above.
        if let Some(details) = &token.details {
            // Check if the pronunciation field exists (IPADIC has 9+ fields)
            if details.len() > 8 {
                let pronunciation_katakana = &details[8];

                // Convert this specific token's pronunciation to Romaji using wana_kana's trait for String.
                let romaji = pronunciation_katakana.to_romaji();
                
                // Add the resulting Romaji to our vector
                romaji_words.push(romaji);
            }
        }
    }

    let final_romaji_spaced = romaji_words.join(" ");

    eprintln!("\nFinal Spaced Romaji:\n{}", final_romaji_spaced);

    Ok(final_romaji_spaced)

}

/// Extract JSON from the messy llm output,
/// the output is expected to contains a JSON with two keys,
/// story and english, 
/// story: a Japanese short story
/// english: the english translation of the story.
/// if it cannot find JSON, return None
pub fn extract_json_from_llm_output(text: &str) -> Option<String> {
    let mut first_brace_index: Option<usize> = None;
    let mut brace_counter = 0;

    // Find the starting position of the JSON object
    if let Some(start) = text.find('{') {
        first_brace_index = Some(start);
        brace_counter = 1;
    } else {
        // No opening brace found, so no JSON object
        return None;
    }

    let start_index = first_brace_index.unwrap();
    // Start searching for the closing brace from the character after the first brace
    for (i, char) in text[start_index + 1..].char_indices() {
        match char {
            '{' => brace_counter += 1,
            '}' => brace_counter -= 1,
            _ => (),
        }

        // When the counter is zero, we've found the matching closing brace
        if brace_counter == 0 {
            // The slice should be from the start brace to the current (closing) brace
            let end_index = start_index + 1 + i;
            return Some(text[start_index..=end_index].to_string());
        }
    }

    // If we finish the loop and the counter isn't zero, the JSON is malformed
    None
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_romaji_pronounciation() {

        let result = romaji_pronounciation(&"私は学生です。".to_string()).unwrap();
        let expected = "watashi wa gakusei desu .".to_string();
        assert_eq!(result, expected);

    }

    #[test]
    fn test_extract_json_from_llm_output_and_parse() {
        let messy_output = r#"
            <|user|>
            Can you add more details to the story to make it more interesting for a beginner?

            <|assistant<|im_mum|>
            Sure, here's an enhanced version of the story:

            {
            "story": "毎日、私は図書館で赤いりんごを食べます。図書館は赤いです。時々、僕は図書館の中で歩くのが好きです。特別な日は、赤いりんごが図書館の中にあり",
            "english": "Every day, I eat an apple in the library. The library is red. Sometimes, I like to walk around in the library. Today is a special day. There is a red apple in the library."
            }, {
            "#;

        let json_str = extract_json_from_llm_output(messy_output).unwrap();
        let parse_story: Story = serde_json::from_str(&json_str).unwrap();

        let expected_story = "毎日、私は図書館で赤いりんごを食べます。図書館は赤いです。時々、僕は図書館の中で歩くのが好きです。特別な日は、赤いりんごが図書館の中にあり".to_string();

        assert_eq!(parse_story.story, expected_story);


        // if let Some(json_str) = extract_json_from_llm_output(messy_output) {
        //     println!("--- Clean JSON Extracted ---");
        //     println!("{}", json_str);

        //     // You can now parse it with serde_json
        //     // let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        //     // assert_eq!(parsed["english"], "Every day, I eat an apple in the library. The library is red. Sometimes, I like to walk around in the library. Today is a special day. There is a red apple in the library.");

        //     let parsed_story: Story = serde_json::from_str(&json_str).unwrap();

        //     // 3. Access the data through the struct's fields.
        //     println!("--- Parsed with a Struct ---");
        //     println!("Japanese Story: {}", parsed_story.story);
        //     println!("English Translation: {}", parsed_story.english);
            

        // } else {
        //     println!("No valid JSON object found.");
        // }



    }


}