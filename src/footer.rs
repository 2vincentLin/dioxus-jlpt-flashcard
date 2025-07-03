use dioxus::prelude::*;

// Define the possible levels for a status message.
// Using an enum is safer than raw strings and allows for better logic.
#[derive(Clone, PartialEq, Default, Debug)]
pub enum StatusLevel {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl StatusLevel {
    /// Returns the corresponding Bootstrap text color class for the level.
    fn to_class(&self) -> &str {
        match self {
            StatusLevel::Info => "text-white",
            StatusLevel::Success => "text-success",
            StatusLevel::Warning => "text-warning",
            StatusLevel::Error => "text-danger",
        }
    }
}

// The struct for our status message. It's Clone and PartialEq to work with Dioxus state.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct StatusMessage {
    pub message: String,
    pub level: StatusLevel,
}


/// The Footer component consumes the context and displays the message.
#[component]
pub fn Footer() -> Element {
    // 2. CONSUME THE CONTEXT
    // `use_context` retrieves the signal from the context.
    // We expect it to exist, so we can unwrap it.
    let status_signal = use_context::<Signal<StatusMessage>>();

    // By reading the signal here, Dioxus knows to re-render this component
    // whenever the signal's value changes.
    let status = status_signal.read();
    let status_class = status.level.to_class();

    rsx! {
        div {
            class: "bg-dark p-2 small mt-auto",
            // Display the message from the context.
            span { class: "{status_class}", "{status.message}" }
        }
    }
}