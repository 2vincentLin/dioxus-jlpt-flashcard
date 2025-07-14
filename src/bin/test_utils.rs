use dxgui::utils;

fn main() {
    let text = r#"
        私の名前は太郎です。毎日、図書館でリンゴを買います。図書館は赤いところがあります。赤い壁にはたくさんの本があります。私は時々、図書館の中で歩くのが好きです。"#.to_string();

    let result = utils::romaji_pronounciation(&text).unwrap();
    println!("result: {}", result);

    let messy_output = r#"
        <|user|>
        Can you add more details to the story to make it more interesting for a beginner?

        <|assistant<|im_mum|>
        Sure, here's an enhanced version of the story:

        {
        "story": "毎日、私は図書館で赤いりんごを食べます。図書館は赤いです。時々、僕は図書館の中で歩くのが好きです。特別な日は、赤いりんごが図書館の中にあり",
        "english": "Every day, I eat an apple in the library. The library is red. Sometimes, I like to walk around in the library. Today is a special day. There is a red apple in the library."
        }
        "#;

    if let Some(json_str) = utils::extract_json_from_llm_output(messy_output) {
        println!("--- Clean JSON Extracted ---");
        println!("{}", json_str);

        // You can now parse it with serde_json
        // let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        // assert_eq!(parsed["english"], "Every day, I eat an apple in the library. The library is red. Sometimes, I like to walk around in the library. Today is a special day. There is a red apple in the library.");

        let parsed_story: utils::Story = serde_json::from_str(&json_str).unwrap();

        // 3. Access the data through the struct's fields.
        println!("--- Parsed with a Struct ---");
        println!("Japanese Story: {}", parsed_story.story);
        println!("English Translation: {}", parsed_story.english);

    } else {
        println!("No valid JSON object found.");
    }
}
