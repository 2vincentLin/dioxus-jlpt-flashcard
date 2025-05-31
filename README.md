# Development

Your new bare-bones project includes minimal organization with a single `main.rs` file and a few assets.

```
project/
├─ assets/ # Any assets that are used by the app should be placed here
├─ src/
│  ├─ main.rs # main.rs is the entry point to your application and currently contains all components for the app
├─ Cargo.toml # The Cargo.toml file defines the dependencies and feature flags for your project
```

### Tailwind
1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```bash
npx tailwindcss -i ./tailwind.css -o ./assets/tailwind.css --watch
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:

```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```


### signal test

1. if you have a signal variable, then all component use it will update if the value updates.
2. if you copy singal variable to another non-signal variable, and you changed the signal variable, non-signal varialbe will be updated automatically, but slower.
3. if you change the non-signal variable, original signal variable won't be affected.
4. if you clone it by calling clone(), then both will stay in sync immediately.
5. use_resource will alway run when you load the page at 1st time. stop is using some variables.
6. if you have a resource depends on multiple signal simultaneously, using tuple to wrap them 

```rust
// copy the original single value to another variable, they will be linked but update slower
// update the new variable won't update the original variable
let mut original_number = use_signal(|| 1 as usize);
let mut new_number = original_number();
let mut cloned_number = original_number.clone();
let mut combo = use_signal(|| (-1 as i64, false));

let _ = use_resource(move || async move {
    if combo().0 < 0 {
        eprintln!("early return from use_resource with combo: {:?}", combo());
        return;
    }
    eprintln!("use_resource called with combo: {:?}", combo());
});


```

### batch database read but without automatically reload by signal

1. all the async work must be put in the use_resource.
2. use_resource automatcially trigger if any signal value it depends changes.
3. so if you want to use the resource but don't want the resource reload, don't toucch those singal you put it the user_resource. Except String type (2025/05/31)
4. 

```rust
// it'll automatically re-fetch when the signal changes 
// (select_word, number_of_cards, j_to_e, unfamiliar_only, random_shuffle, tag_level, current_index)
let mut word_resource = use_resource(move || async move {
    eprintln!("use_resource called");
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

```
