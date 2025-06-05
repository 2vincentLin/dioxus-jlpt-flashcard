use dioxus::prelude::*;

// Import Route from your routes module (adjust the path as needed)
use crate::Route;


// Component that navigates
#[component]
pub fn FetchAndNavigateComponent() -> Element {
    let navigator = use_navigator();
    let mut shared_data = use_context::<Signal<i32>>();
    let mut another_shared_data = use_context::<Signal<i32>>();
    let value = shared_data();
    use_effect(move || {
        shared_data.set(value + 1);
});
    

    rsx! {
        button {
            onclick: move |_| async move {
                // let data = fetch_my_data_from_db().await; // You might fetch an ID here
                // For example, if you fetched an item with id = 5

                navigator.push(Route::DataDisplayPage {} );
            },
            "Fetch and Go to Display"
        }
    }
}

// Component that receives data via route param and might fetch more
#[component]
pub fn DataDisplayPage() -> Element {
    let shared_data = use_context::<Signal<i32>>();
    let another_shared_data = use_context::<Signal<i32>>();
    let navigator = use_navigator();

    let value = shared_data();
    let another_value = another_shared_data();

    rsx! {
        // Display data for id
        "shared_data: {value}, another_shared_data: {another_value} "
        // ... display data_from_db ...
        button {
            onclick: move |_| async move {
                // let data = fetch_my_data_from_db().await; // You might fetch an ID here
                // For example, if you fetched an item with id = 5

                navigator.push(Route::FetchAndNavigateComponent {  } );
            },
            "Go back"
        }
    }
}