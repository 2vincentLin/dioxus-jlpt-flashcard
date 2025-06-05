pub mod db;

use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder};
mod flashcard;
mod test;
mod nav;
use flashcard::FlashCard;
use test::TextInputPanel;
use nav::DataDisplayPage;
use nav::FetchAndNavigateComponent;



#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
    #[route("/flashcard")]
    FlashCard {},
    #[route("/test")]
    TextInputPanel {},
    #[route("/output")]
    OutputPanel {},
    #[route("/fetch")]
    FetchAndNavigateComponent {},
    #[route("/data_diaplay")]
    DataDisplayPage {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const BOOTSTRAP_CSS: Asset = asset!("/assets/bootstrap.min.css");


fn main() {
    
    dioxus::launch(App);
    
}
#[component]
fn App() -> Element {
    let shared_text = use_signal(|| "".to_string());
    let mut shared_data = use_signal(|| 1);
    let mut another_shared_data = use_signal(|| 100);

    provide_context(shared_text.clone()); // now available to all children
    provide_context(shared_data.clone());
    provide_context(another_shared_data.clone());
    
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
        div {
            
            Router::<Route> {}
        }
    }
}



/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home  "
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog "
            }
            Link {
                to: Route::FlashCard {},
                "FlashCard "
            }
            Link {
                to: Route::TextInputPanel {},
                "Test "
            }
            Link {
                to: Route::OutputPanel {},
                "Output "
            }
            Link {
                to: Route::FetchAndNavigateComponent {},
                "fetch"
            }
        }

        Outlet::<Route> {}
    }
}



#[component]
fn OutputPanel() -> Element {
    let shared_text = use_context::<Signal<String>>();
    rsx! {
        div { "Output: {shared_text}" }
    }
}


#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        // you can put two components in one component
        Hero {}
        Blog { id: 1 }

    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}


