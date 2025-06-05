
# Signal and use_

## 1. signal test

1. if you have a signal variable, then all component use it will update if the value updates.
2. if you copy signal variable to another non-signal variable, and you changed the signal variable, non-signal variable will be updated automatically, but slower.
3. if you change the non-signal variable, original signal variable won't be affected.
4. if you clone it by calling clone(), then both will stay in sync immediately.
5. use_resource will always run when you load the page at 1st time. stop is using some variables.
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

## 2. batch database read but without automatically reload by signal

1. all the async work must be put in the use_resource.
2. use_resource automatically trigger if any signal value it depends changes.
3. so if you want to use the resource but don't want the resource reload, don't touch those signal you put it the user_resource. Except String type (2025/05/31)
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


## 3. use_effect

1. you use it for "side effects"
2. one use case if you need some signal changes based on another signal, put them in the use_effect

```rust
let _ = use_effect(move || {
    if combo().0 > 3 {
        eprintln!("combo: {:?}", combo());
        side_effect.set(combo().0);
        eprintln!("side_effect: {:?}", side_effect());
        
    };
});


```



## 4. use_state

1. creata a local reactive state variable


## 5. use_context, provide_context


# How to use Bootstrap with dioxus desktop (no tailwind, no CDN)

## 1. download bootstrap CSS
- visit [Bootstrap](https://getbootstrap.com/docs/5.3/getting-started/download/), and download ``bootstrap.min.css```` or others.
- put the css file in ``assets/`` folder.

## 2. Setup CSS injection in Dioxus
- use ``include_str!`` to indicate the css file path
- inject them in the root (App->head)
```rust
fn App() -> Element {
    // Include the Bootstrap and global stylesheets
    let bootstrap = include_str!("../assets/bootstrap.min.css");
    let global= include_str!("../assets/main.css");
    rsx! {

        head {
            style { dangerous_inner_html: bootstrap }
            style { dangerous_inner_html: global }
        }
       
        // keep these for web app
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: BOOTSTRAP_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}
```

## 3. Use Bootstrap normally
```rust
pub fn Testboot() -> Element {
    rsx! {
        div { class: "container mt-5 bg-dark text-light",
            h1 { class: "text-info", "Hello Bootstrap Dark Theme!" }
            p { class: "lead", "This is styled with Bootstrap's dark theme classes." }
            button { class: "btn btn-primary", "Click Me!" }
        }
        
    }
}

```

## 4. Use extension 
vscode has some bootstrap extension that offer autocomplet, but they only work in html file, not in rust file. So the best option is to use the autocomplete in some html file, then copy the style to rust.

## useful link 
1. [Spacing](https://getbootstrap.com/docs/5.3/utilities/spacing/) shorthand responsive margin, padding, and gap utility
2. [Sizing](https://getbootstrap.com/docs/5.3/utilities/sizing/) Easily make an element as wide or as tall
3. [Flex](https://getbootstrap.com/docs/5.3/utilities/flex/): quickly manage layout
4. [Grid](https://mdbootstrap.com/docs/standard/layout/grid/) grid example
5. [Cheat sheet](https://getbootstrap.com/docs/5.3/examples/cheatsheet/)