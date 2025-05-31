use std::clone;

use dioxus::{html::g::origin, prelude::*};

#[component]
pub fn TextInputPanel() -> Element {
    // Access the shared signal from context
    let mut shared_text = use_context::<Signal<String>>();
    let mut input_text = use_signal(|| "".to_string());

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

    rsx! {
        div {
            class: "p-2",

            button { 
                class: "btn-large",
                onclick: move |_| {
                    combo.set((combo().0 + 1, !combo().1));
                    eprintln!("combo: {:?}", combo());
                },
                "Increment Combo on both elements"
             }

            button { 
                class: "btn-large",
                onclick: move |_| {
                    combo.set((combo().0 + 1, combo().1));
                    eprintln!("combo: {:?}", combo());
                },
                "Increment Combo on one elements"
             }

            input {
                class: "border p-1 mr-2",
                value: "{input_text}",
                oninput: move |evt| input_text.set(evt.value())
            }

            button {
                class: "btn-large",
                style: "margin : 10px",
                onclick: move |_| {
                    shared_text.set(input_text());
                },
                "Submit"
            }

            label { "original number is {original_number}" }

            button { 
                class: "btn-large",
                onclick: move |_| {

                    original_number.set(original_number() + 1);
                    eprintln!("original_number: {}, new_number: {}, clone_number: {}"
                    , original_number(), new_number, cloned_number());
                    
                },
                "Increment Original"
             }

             button {
                class: "btn-large",
                onclick: move |_| {
                    new_number +=1;
                    eprintln!("original_number: {}, new_number: {}, clone_number: {}"
                    , original_number(), new_number, cloned_number());
                },
                "Increment New"
             }
             
        }
        br {  }

    }
}


