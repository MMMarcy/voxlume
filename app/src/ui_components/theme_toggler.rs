//! Component that toggles between dark and light theme.

use leptos::logging::error;
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

    fn from_string(s: Option<String>) -> Theme {
        match s.as_deref() {
            Some("dark") => Theme::Dark,
            _ => Theme::Light, // Default to Light
        }
    }
}

// Helper function to get localStorage (only runs in the browser)
fn local_storage() -> Option<web_sys::Storage> {
    // TODO: Maybe we don't need these cfg statements or maybe it is possible to use cfg_if.
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

#[component]
pub fn ThemeSwitcher() -> impl IntoView {
    // Creates a reactive value to update the button
    // TODO: Maybe we don't need these cfg statements. Or it might be possible to use cfg_if.
    let initial_theme: Theme = {
        let stored_theme =
            local_storage().and_then(|storage| storage.get_item("theme").ok().flatten());
        Theme::from_string(stored_theme)
    };
    let (theme, set_theme) = signal(initial_theme);

    Effect::new(move |_| {
        // Get the current theme value from the signal
        let current_theme = theme.get();

        // Access the document (client-side only)
        let h: web_sys::Element = document().document_element().expect("boh");
        match h.set_attribute("data-theme", &current_theme.to_string()) {
            Ok(_) => (),
            Err(err) => error!("Not cool -> {:?}", err),
        }
    });

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
            <Show when=move || { theme.get() == Theme::Light } fallback=|| view! { <SunButton /> }>
                <MoonButton />
            </Show>
        </div>
    }
}
