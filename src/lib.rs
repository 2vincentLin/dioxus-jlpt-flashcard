pub mod db;
pub mod flashcard;
pub mod test;

use tts::*;

use dioxus::prelude::*;
use test::TextInputPanel;
use flashcard::{GenerateCard, DisplayCard};


const HEADER_SVG: Asset = asset!("/assets/header.svg");


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
    #[route("/test")]
    TextInputPanel {},
    #[route("/output")]
    OutputPanel {},

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
                    "MyApp" // Your app's name or logo
                }

                // Navbar toggler button (for small screens)
                button {
                    class: "navbar-toggler",
                    r#type: "button",
                    // "data-bs-toggle": "collapse", // Removed: Bootstrap JS hook
                    // "data-bs-target": "#navbarNavDropdown", // Removed: Points to the ID of the collapsible content
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
                                to: Route::TextInputPanel {},
                                onclick: move |_| is_nav_open.set(false), // Close nav on link click
                                "Test Input"
                            }
                        }
                        li { class: "nav-item",
                            Link {
                                class: "nav-link",
                                to: Route::OutputPanel {},
                                onclick: move |_| is_nav_open.set(false), // Close nav on link click
                                "Output"
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







/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        // you can put two components in one component
        h1 {"This is a Japanese flash card app"}

    }
}



#[component]
fn OutputPanel() -> Element {
    let shared_text = use_context::<Signal<String>>();
    rsx! {
        div { "Output: {shared_text}" }
    }
}


// this function returns a tts voice based on lang and Geneder enum in tts crate
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
