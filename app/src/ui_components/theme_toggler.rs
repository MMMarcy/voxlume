//! Component that toggles between dark and light theme.

use leptos::logging::{error, log};
use leptos::prelude::*;
use web_sys;

// Define an enum for themes for better type safety (optional but recommended)
#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    fn to_string(&self) -> String {
        match self {
            Theme::Light => "light".to_string(),
            Theme::Dark => "dark".to_string(),
        }
    }

    #[cfg(feature = "hydrate")]
    fn from_string(s: Option<String>) -> Theme {
        match s.as_deref() {
            Some("dark") => Theme::Dark,
            _ => Theme::Light, // Default to Light
        }
    }
}

#[component]
fn MoonButton() -> impl IntoView {
    view! {
        <span class="icon">
            <i class="fas fa-lg fa-moon"></i>
        </span>
    }
}

#[component]
fn SunButton() -> impl IntoView {
    view! {
        <span class="icon">
            <i class="fas fa-lg fa-sun"></i>
        </span>
    }
}

// Helper function to get localStorage (only runs in the browser)
fn local_storage() -> Option<web_sys::Storage> {
    #[cfg(feature = "hydrate")]
    {
        window().local_storage().unwrap()
    }
    #[cfg(feature = "ssr")]
    {
        None // Return None on the server
    }
}

#[component]
pub fn ThemeSwitcher() -> impl IntoView {
    // Creates a reactive value to update the button
    // 1. Create a signal to hold the theme state
    let initial_theme = {
        #[cfg(feature = "hydrate")] // Only run on the client
        {
            let stored_theme =
                local_storage().and_then(|storage| storage.get_item("theme").ok().flatten());
            Theme::from_string(stored_theme)
        }
        #[cfg(feature = "ssr")] // Default on the server
        {
            Theme::Light
        }
    };
    let (theme, set_theme) = signal(initial_theme);

    // 3. Create an effect to update the <html> tag's attribute
    Effect::new(move |_| {
        // Get the current theme value from the signal
        let current_theme = theme.get();
        log!("{:?}", current_theme);

        // Access the document (client-side only)
        let h: web_sys::Element = document().document_element().expect("boh");
        match h.set_attribute("data-theme", &current_theme.to_string()) {
            Ok(_) => log!("Cool!"),
            Err(err) => error!("Not cool -> {:?}", err),
        }
    });

    // 2. Button to toggle the theme
    let toggle_theme = move |_| {
        set_theme.update(|current| {
            *current = match current {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
            };
            local_storage().and_then(|storage| {
                storage.set_item("theme", &current.to_string()).ok() // t
            });
        });
    };

    view! {
        <div on:click=toggle_theme>
            <Show
                when=move || {theme.get() == Theme::Light}
                fallback=|| view! {<SunButton/>}
            >
                <MoonButton/>
            </Show>
        </div>
    }
}
