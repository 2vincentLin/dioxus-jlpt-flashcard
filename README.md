# Dioxus JLPT Flashcards

This is a personal project built to practice Rust and the Dioxus framework by creating a desktop flashcard application for studying Japanese Language Proficiency Test (JLPT) vocabulary. Original goal is just to have flashcard to memorize the words, but now expand to use local AI to explain the the word and generate example sentences. 

# Features

### 🎴 Dynamic Flashcard Decks

Stop studying words you already know. Create highly customized flashcard decks and study sessions based on a combination of filters:

- **JLPT Level:** Focus your efforts on a specific level, from N5 to N1.    
- **Word Familiarity:** Automatically create decks of words you need more practice with versus words you already know.    
- **Favorites:** Curate your own lists by starring words you find interesting or difficult.
    
### 🔀 Interactive Flashcards

Each flashcard is more than just a word; it's an interactive tool.

- **⭐️ Favorite:** Hit the star icon to save a word for later review.    
- **🗣️ Text-to-Speech:** Click the speaker button to hear the Japanese word pronounced by your system's native TTS engine.    
- **✅ Got It:** Mark a word as "familiar" to reduce its appearance frequency in practice sessions.    
- **🔄 Need More Practice:** Keep a word as "unfamiliar" to ensure it appears more often.    

### 📝 Custom Test Generation

Turn your practice into progress by creating tests on demand.

- Create tests from the words you have practiced or favorited.    
- From the word list page, click **"Generate Test"** to start a quiz.    
- The test will randomly pick answers from your deck to challenge you and reinforce your memory.
    

### 🧠 AI-Powered Deep Dive Explanations

Go beyond simple definitions with a fully interactive, AI-powered explainer.

- From your word list, click on any practiced word to get a detailed breakdown.    
- An on-device AI (**Gemma 3 4B** via Ollama) will instantly generate a clear explanation of the word's nuance and provide three new, context-rich example sentences.    
	- why Gemma 3 4B? because it's surprisingly capable and fast, if using GPU, it only take less than 7gb VRAM. Which should be enough for most discrete GPU.
- Each example sentence is automatically processed by **Lindera** to provide an accurate Romaji reading and break the sentence down into its component words.    
- The magic doesn't stop there. **Every word in the new sentences is also clickable**, allowing you to dive deeper and deeper into the vocabulary in a continuous, interactive learning loop without ever leaving the page.
# Getting Started

1. install rust in your system, download the whole project, 
2. run `cargo run --bin build_db` to build the jlpt word database, 
3. then you can build the project.
4. download and install Ollama, this is very easy in any system.
5. run `ollama run gemma3:4b` before open the app

# Tech Stack
This project is built with Rust and leverages the following core crates:

- [Dioxus](https://docs.rs/dioxus/latest/dioxus/index.html): A portable, performant, and ergonomic framework for building user interfaces in Rust.
- [SQLx](https://docs.rs/sqlx/latest/sqlx/index.html): A modern, async-ready, and type-safe SQL toolkit for Rust. Used for the local flashcard database.
- [tts-rs](https://docs.rs/tts/latest/tts/): A cross-platform text-to-speech library that provides access to native system voices.
- [lindera](https://github.com/lindera/lindera): A multilingual morphological analysis library. Used for analyze Japanese text for correct pronunciation.
- [wana_kana](https://github.com/PSeitz/wana_kana_rust): A library for converting between Japanese characters. Used for convert Katakana to romaji.
- [ollama-rs](https://github.com/pepperoni21/ollama-rs): A library used to interact with Ollama.

# Acknowledgments

- The vocabulary decks used in this application are sourced from the excellent [chyyran/jlpt-anki-decks](https://github.com/chyyran/jlpt-anki-decks) repository.

# ⚠️ Disclaimer

This application leverages a Large Language Model (LLM) to generate explanations and example sentences. While powerful, **AI can make mistakes** or produce explanations that are subtly incorrect or unnatural. Always use these AI-generated materials as a supplementary learning tool and cross-reference with trusted dictionaries or native speakers when in doubt.