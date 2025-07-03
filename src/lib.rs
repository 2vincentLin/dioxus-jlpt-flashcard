pub mod db;
pub mod flashcard;
pub mod wordlist;
pub mod footer;

use tts::*;

use dioxus::prelude::*;
use flashcard::{GenerateCard, DisplayCard};
use wordlist::{WordListPage, WordListType};
use db::*;
use sqlx::SqlitePool;


#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/flaschard")]
    GenerateCard {},
    #[route("/diaplay/:j_to_e")]
    DisplayCard { j_to_e: bool},
    #[route("/word-list/:list_type")]
    WordListPage { list_type: WordListType },
    #[route("/setting")]
    Setting {},

}

#[component]
pub fn Navbar() -> Element { // Encapsulating navbar in its own component is good practice
    // Signal to manage the collapsed state of the navbar
    let mut is_nav_open = use_signal(|| false);

    rsx! {
        nav {
            // Bootstrap navbar classes:
            // - navbar: Base class
            // - navbar-expand-lg: Collapse on screens smaller than large (lg)
            // - navbar-dark: For use with dark background colors (sets text to light)
            // - bg-dark: Sets a dark background color
            // - mb-3: Adds some margin to the bottom for spacing
            class: "navbar navbar-expand-lg navbar-dark bg-dark mb-3",

            div { class: "container-fluid", // Recommended for full-width navbars
                // Brand link (optional, often links to home)
                Link {
                    class: "navbar-brand", // Bootstrap class for branding
                    to: Route::Home {},
                    "Japanese Flash Card" // Your app's name or logo
                }

                // Navbar toggler button (for small screens)
                button {
                    class: "navbar-toggler",
                    r#type: "button",
                    // "data-bs-toggle": "collapse", 
                    // "data-bs-target": "#navbarNavDropdown",
                    "aria-controls": "navbarNavDropdown", // Still good for accessibility
                    "aria-expanded": "{is_nav_open}", // Reflects state for accessibility
                    "aria-label": "Toggle navigation",
                    onclick: move |_| is_nav_open.toggle(), // Toggle the Dioxus signal
                    span { class: "navbar-toggler-icon" }
                }

                // Collapsible navbar content
                // Conditionally add the 'show' class based on the is_nav_open signal
                div {
                    class: if is_nav_open() {
                        "collapse navbar-collapse show"
                    } else {
                        "collapse navbar-collapse"
                    },
                    id: "navbarNavDropdown", 

                    // Unordered list for navigation items
                    ul { class: "navbar-nav me-auto mb-2 mb-lg-0", // me-auto pushes other items to the right
                        // List item for each link
                        li { class: "nav-item",
                            Link {
                                class: "nav-link", // Bootstrap class for nav links
                                // You can add 'active' class conditionally if the route matches
                                to: Route::Home {},
                                onclick: move |_| is_nav_open.set(false), // Close nav on link click
                                "Home"
                            }
                        }
                        
                        li { class: "nav-item",
                            Link {
                                class: "nav-link",
                                to: Route::GenerateCard {},
                                onclick: move |_| is_nav_open.set(false), // Close nav on link click
                                "Flash Card"
                            }
                        }
                        li { class: "nav-item",
                            Link {
                                class: "nav-link",
                                to: Route::Setting {},
                                onclick: move |_| is_nav_open.set(false), // Close nav on link click
                                "Setting"
                            }
                        }
                        
                        // You can add more nav items (e.g., dropdowns) here if needed
                    }
                    // You could add other elements here, like a search form or user profile link,
                    // aligned to the right, e.g., using <ul class="navbar-nav">
                }
            }
        }
        // The Outlet should be outside the navbar, typically in your main layout
        Outlet::<Route> {} // Assuming this is handled by the component that uses Navbar
    }
}







/// Home page will display the user's progress summary.
#[component]
fn Home() -> Element {

    // Signals to hold the counts of various word categories
    let mut words_practiced = use_signal(|| 0);
    let mut total_practiced = use_signal(|| 0);
    let mut familiar_words = use_signal(|| 0);
    let mut unfamiliar_practiced = use_signal(|| 0);
    let mut marked_words = use_signal(|| 0);

    let db_pool = use_context::<SqlitePool>();

    // Use another resource to fetch data, depending on db_pool
    let _ = use_resource(move || {
        let db_pool = db_pool.clone();
        async move {
            // Wait for the pool to be ready
            let pool = db_pool.clone();
            
            let words_practiced1 = match count_unique_practiced_words(&pool).await {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Error fetching words practiced: {}", e);
                    0 // Default to 0 if there's an error
                }
            };
            words_practiced.set(words_practiced1);

            let total_practiced1 = match count_total_practiced_words(&pool).await {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Error fetching total practiced words: {}", e);
                    0 // Default to 0 if there's an error
                }
            };
            total_practiced.set(total_practiced1);

            let familiar_words1 = match count_total_familiar_words(&pool).await {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Error fetching familiar words: {}", e);
                    0 // Default to 0 if there's an error
                }
            };
            familiar_words.set(familiar_words1);

            let unfamiliar_practiced1 = match count_unfamiliar_practiced_words(&pool).await {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Error fetching unfamiliar practiced words: {}", e);
                    0 // Default to 0 if there's an error
                }
            };
            unfamiliar_practiced.set(unfamiliar_practiced1);

            let marked_words1 = match count_total_user_marked_words(&pool).await {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Error fetching marked words: {}", e);
                    0 // Default to 0 if there's an error
                }
            };
            marked_words.set(marked_words1);
        }
       
    });

     

    rsx! {
        // Use a container with padding for nice spacing around the content.
        div { class: "container p-4",
            h1 { class: "mb-4 text-center text-light", "JLPT Flashcard Dashboard" }

            // Your classes here on the card are perfect.
            div { class: "card shadow-sm bg-dark text-light",
                
                // The card header will correctly be dark.
                div { class: "card-header",
                    h5 { class: "my-1", "Your Progress Summary" }
                }
                
                ul { class: "list-group list-group-flush",


                    li { class: "list-group-item d-flex justify-content-between align-items-center bg-transparent text-light",
                        "Total Practice Times"
                        span { class: "badge bg-info text-dark rounded-pill fs-6", "{total_practiced()}" }
                    }

                    Link {
                        // The `to` prop takes the Route variant we want to navigate to
                        to: Route::WordListPage { list_type: WordListType::Practiced },
                        class: "list-group-item list-group-item-action d-flex justify-content-between align-items-center bg-transparent text-light",
                        "Words Practiced"
                        span { class: "badge bg-info text-dark rounded-pill fs-6", "{words_practiced()}" }
                    }
                    
                    Link {
                        // The `to` prop takes the Route variant we want to navigate to
                        to: Route::WordListPage { list_type: WordListType::Familiar },
                        class: "list-group-item list-group-item-action d-flex justify-content-between align-items-center bg-transparent text-light",
                        "Familiar Words"
                        span { class: "badge bg-info text-dark rounded-pill fs-6", "{familiar_words()}" }
                    }

                    Link {
                        to: Route::WordListPage { list_type: WordListType::Unfamiliar },
                        class: "list-group-item list-group-item-action d-flex justify-content-between align-items-center bg-transparent text-light",
                        "Need more practice Words"
                        span { class: "badge bg-warning text-dark rounded-pill fs-6", "{unfamiliar_practiced()}" }
                    }

  
                    Link {
                        to: Route::WordListPage { list_type: WordListType::Marked },
                        class: "list-group-item list-group-item-action d-flex justify-content-between align-items-center bg-transparent text-light",
                        "Marked for Review"
                        span { class: "badge bg-warning text-dark rounded-pill fs-6", "{marked_words()}" }
                    }
                }
            }
}
    }
}


#[component]
fn Setting() -> Element {
    // --- pool for db op ---
    let db_pool = use_context::<sqlx::SqlitePool>();
    let mut show_confirm_dialog = use_signal(|| false);


    rsx!(
        div { class: "container h-100 d-flex flex-column",
            div { class: "d-flex justify-content-between align-items-center my-3",
                h3 { class: "mb-0", "Reset your practice record" }

                // button to shows the confirmation dialog.
                button {
                    class: "btn btn-danger", // Changed to red for a destructive action
                    onclick: move |_| {
                        show_confirm_dialog.set(true);
                    },
                    "Reset DB"
                }
            }
        }

        
        //  This part will only appear when `show_confirm_dialog` is true.
        if show_confirm_dialog() {
            // Modal backdrop
            div {
                class: "modal fade show d-block",
                style: "background-color: rgba(0, 0, 0, 0.5);",

                // Modal dialog
                div { class: "modal-dialog modal-dialog-centered",
                    div { class: "modal-content",
                        // Modal header
                        div { class: "modal-header",
                            h5 { class: "modal-title", "Confirm Action" }
                        }
                        // Modal body
                        div { class: "modal-body",
                            p { "Are you sure you want to reset all practice data? This action cannot be undone." }
                        }
                        // Modal footer with action buttons
                        div { class: "modal-footer",
                            // The "Cancel" button simply hides the dialog
                            button {
                                class: "btn btn-secondary",
                                onclick: move |_| show_confirm_dialog.set(false),
                                "Cancel"
                            }
                            // The "Confirm" button closes the dialog AND runs the reset logic
                            button {
                                class: "btn btn-danger",
                                onclick: move |_| {
                                    // First, hide the dialog
                                    show_confirm_dialog.set(false);

                                    // Then, run your original database reset logic
                                    let pool = db_pool.clone();
                                    spawn(async move {
                                        match reset_all_user_progress(&pool).await {
                                            Ok(_) => eprintln!("database reset successfully"),
                                            Err(e) => eprintln!("database reset failed: {}", e),
                                        }
                                    });
                                },
                                "Yes, Reset"
                            }
                        }
                    }
                }
            }
        }
    )
}




/// this function returns a tts voice based on lang and Geneder enum in tts crate
pub fn return_voice(lang: &str, gender: Gender) -> Result<Voice, Error> {
    let tts = Tts::default()?;

    for voice in tts.voices()? {
        
        if voice.language().starts_with(lang) {

            if gender == voice.gender().unwrap(){
                eprintln!("Voice: {:?}", voice);
                // tts.set_voice(&voice)?;
                return Ok(voice);
            }
        }
    }
    Err(tts::Error::NoneError)
}


// possible pop up setup
// https://dioxuslabs.com/learn/0.6/reference/event_handlers#handler-props

// #[derive(Props, Clone, PartialEq)]
// pub struct ConfirmModalProps {
//     // Signal to control the visibility of the modal.
//     // The parent component will set this to true to show the modal.
//     // The modal itself will set it to false when an action is taken.
//     pub show_dialog: Signal<bool>,
//     pub title: String,
//     pub message: String,
//     // Optional custom text for the confirm button
//     #[props(default = "Confirm".to_string())]
//     pub confirm_button_text: String,
//     // Optional custom text for the cancel button
//     #[props(default = "Cancel".to_string())]
//     pub cancel_button_text: String,
//     // Callback for when the confirm button is clicked
//     pub on_confirm: EventHandler<()>,
//     // Optional callback for when the cancel button is clicked or modal is dismissed
//     #[props(optional)]
//     pub on_cancel: Option<EventHandler<()>>,
// }


// #[component]
// pub fn ConfirmModal(props: ConfirmModalProps) -> Element {
//     if !props.show_dialog() {
//         return None; // Don't render anything if not visible
//     }

//     rsx! {
//         // Modal backdrop
//         div {
//             class: "modal fade show d-block", // "show" and "d-block" make it visible
//             style: "background-color: rgba(0, 0, 0, 0.5);", // Semi-transparent backdrop
//             tabindex: "-1", // Allows closing with Escape key (Bootstrap behavior)
//             role: "dialog",
//             "aria-labelledby": "confirmModalLabel",
//             "aria-modal": "true",

//             // Modal dialog
//             div { class: "modal-dialog modal-dialog-centered", // Vertically centered
//                 div { class: "modal-content",
//                     // Modal header
//                     div { class: "modal-header",
//                         h5 { class: "modal-title", id: "confirmModalLabel", "{props.title}" }
//                         // Optional: Add a close button (X) in the header
//                         // button {
//                         //     r#type: "button",
//                         //     class: "btn-close",
//                         //     "data-bs-dismiss": "modal", // Bootstrap's way to close
//                         //     "aria-label": "Close",
//                         //     onclick: move |_| {
//                         //         props.show_dialog.set(false);
//                         //         if let Some(cb) = &props.on_cancel {
//                         //             cb.call(());
//                         //         }
//                         //     }
//                         // }
//                     }
//                     // Modal body
//                     div { class: "modal-body",
//                         p { "{props.message}" }
//                     }
//                     // Modal footer with action buttons
//                     div { class: "modal-footer",
//                         button {
//                             class: "btn btn-secondary",
//                             r#type: "button",
//                             onclick: move |_| {
//                                 props.show_dialog.set(false);
//                                 if let Some(cb) = &props.on_cancel {
//                                     cb.call(());
//                                 }
//                             },
//                             "{props.cancel_button_text}"
//                         }
//                         button {
//                             class: "btn btn-danger", // Or btn-primary, depending on context
//                             r#type: "button",
//                             onclick: move |_| {
//                                 props.show_dialog.set(false);
//                                 props.on_confirm.call(());
//                             },
//                             "{props.confirm_button_text}"
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }



// ConfirmModal {
//     show_dialog: show_confirm_dialog,
//     title: "Confirm Action".to_string(),
//     message: "Are you sure you want to reset all practice data? This action cannot be undone.".to_string(),
//     confirm_button_text: "Yes, Reset".to_string(),
//     on_confirm: move |_| {
//         // The logic that was previously in the "Yes, Reset" button's onclick
//         // show_confirm_dialog.set(false); // Modal handles this now
//         let pool = db_pool.clone();
//         spawn(async move {
//             match reset_all_user_progress(&pool).await {
//                 Ok(_) => eprintln!("database reset successfully"),
//                 Err(e) => eprintln!("database reset failed: {}", e),
//             }
//         });
//     },
//     on_cancel: move |_| {
//         // Optional: add any specific logic for cancellation
//         // show_confirm_dialog.set(false); // Modal handles this now
//         eprintln!("Reset operation cancelled.");
//     }
// }