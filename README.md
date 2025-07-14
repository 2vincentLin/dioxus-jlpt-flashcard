# Dioxus JLPT Flashcards

This is a personal project built to practice Rust and the Dioxus framework by creating a desktop flashcard application for studying Japanese Language Proficiency Test (JLPT) vocabulary.

# Features
- Generate Flashcard Decks: Filter and create study sessions based on:
    - JLPT Level (N5 to N1)
    - Word Familiarity (words you know vs. words you need to practice)
    - Favorites (words you have personally starred)
- Interactive Flashcards:
    - ⭐️ Favorite: Hit the star icon to save a word for later review.
    - 🗣️ Text-to-Speech: Hit the speaker button to hear the Japanese word pronounced by the system's native TTS engine.
    - ✅ Got It: Mark a word as "familiar" to reduce its appearance frequency.
    - 🔄 Need More Practice: Keep a word as "unfamiliar" to ensure it appears more often.
- Generate Test Desk:
    - You can now create tests from the words you have practiced or favorited.
    - From the home page, click a summary to view your word list.
    - From the word list page, click "Generate Test" to start a quiz.
    - The test will randomly pick answers from your deck to help you review.
 
# Getting Started

1. install rust in your system, download the whole project, 
2. run `cargo run --bin build_db` to build the jlpt word database, 
3. then you can build the project.

# Tech Stack
This project is built with Rust and leverages the following core crates:

- [Dioxus](https://docs.rs/dioxus/latest/dioxus/index.html): A portable, performant, and ergonomic framework for building user interfaces in Rust.
- [SQLx](https://docs.rs/sqlx/latest/sqlx/index.html): A modern, async-ready, and type-safe SQL toolkit for Rust. Used for the local flashcard database.
- [tts-rs](https://docs.rs/tts/latest/tts/): A cross-platform text-to-speech library that provides access to native system voices.
- [lindera](https://github.com/lindera/lindera): A multilingual morphological analysis library. Used for analyze Japanese text for correct pronounciation.
- [wana_kana](https://github.com/PSeitz/wana_kana_rust): A library for converting between Japanese characters. Used for convert Katakana to romaji.

# Acknowledgments

- The vocabulary decks used in this application are sourced from the excellent [chyyran/jlpt-anki-decks](https://github.com/chyyran/jlpt-anki-decks) repository.