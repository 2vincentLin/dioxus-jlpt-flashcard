// get rid of console, but also remove any println!
// #![windows_subsystem = "windows"]

// This will only apply the `windows_subsystem` attribute when compiling
// in release mode (i.e., with `cargo build --release`).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use dioxus::prelude::*;
use dioxus::desktop::{Config, WindowBuilder, LogicalSize};
use sqlx::sqlite::SqlitePoolOptions;
use std::time::Duration;

use dxgui::Route;
use dxgui::db::WordRecord;
use dxgui::db::DB_URL;



const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
// const HEADER_SVG: Asset = asset!("/assets/header.svg");
// const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const BOOTSTRAP_CSS: Asset = asset!("/assets/bootstrap.min.css");



fn main() {

    // initiate Window builder
    let window_builder = WindowBuilder::new()
        .with_title("My language partner")
        .with_inner_size(LogicalSize::new(600.0, 800.0)); // Set initial width and height


    // Create the Config with the custom window builder
    let cfg = Config::new()
        .with_window(window_builder)
        .with_menu(None); // this diable the menu

    // Launch app from cfg
    LaunchBuilder::desktop().with_cfg(cfg).launch(App);

    
    
}

#[component]
fn App() -> Element {
    let shared_text = use_signal(|| "".to_string());
    // this will be used in flashcard to hold the retrieved data
    let select_words = use_signal(|| Vec::<WordRecord>::new());

    provide_context(shared_text.clone()); // now available to all children
    provide_context(select_words.clone());

    // Include the Bootstrap and global stylesheets
    let bootstrap = include_str!("../assets/bootstrap.min.css");
    let global= include_str!("../assets/main.css");
    // let header_svg = include_str!("../assets/header.svg");


    // initiate db pool for all children component
    let db_pool = use_resource(move || async move {
        eprintln!("use_resource for db pool called");
        SqlitePoolOptions::new()
            .max_connections(5)
            // Proactively close connections that have been idle for 10 minutes.
            // This is safer than letting them die from a server-side timeout.
            .idle_timeout(Duration::from_secs(600)) 
            // Optionally, force connections to be recycled every 30 minutes.
            .max_lifetime(Duration::from_secs(1800))
            .connect(DB_URL)
            .await
    });

    rsx! {
        head {
            style { dangerous_inner_html: bootstrap }
            style { dangerous_inner_html: global }
        }

        // keep these for web app
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: BOOTSTRAP_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        // Conditionally render based on the db_pool resource state
        match &*db_pool.read_unchecked() {
            Some(Ok(pool)) => {
                // --- SUCCESS STATE ---
                // The pool is ready. Provide it to the context and render the main app.
                // By placing provide_context here, we guarantee the pool exists for all
                // children of Router.
                provide_context(pool.clone());
                println!("Database pool ready.");

                rsx! {
                    div { class: "d-flex flex-column vh-100",
                        // The Router is now only rendered when the pool is available
                        Router::<Route> {}
                    }
                }
            }
            Some(Err(e)) => {
                // --- ERROR STATE ---
                // Render an error message if the connection failed
                rsx! {
                    div { class: "vh-100 d-flex justify-content-center align-items-center",
                        div { class: "alert alert-danger", role: "alert",
                            h4 { class: "alert-heading", "Database Connection Error" }
                            p { "Could not connect to the database. Please check your configuration." }
                            hr {}
                            p { class: "mb-0", "Error: {e}" }
                        }
                    }
                }
            }
            None => {
                // --- LOADING STATE ---
                // Render a loading indicator while the future is running
                rsx! {
                    div { class: "vh-100 d-flex justify-content-center align-items-center",
                        div { class: "spinner-border text-primary", role: "status",
                            span { class: "visually-hidden", "Loading..." }
                        }
                        p { class: "ms-3", "Connecting to database..." }
                    }
                }
            }
        }
    }
}









