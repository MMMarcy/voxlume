//! Component that toggles between dark and light theme.

use entities_lib::entities::theme::Theme;
use leptos::leptos_dom::debug_error;
use leptos::prelude::*;
use web_sys::{self, MouseEvent};

use crate::utils::local_storage::local_storage;

// Define an enum for themes for better type safety (optional but recommended)

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
    let initial_theme = LocalResource::new(move || async {
        let stored_theme =
            local_storage().and_then(|storage| storage.get_item("theme").ok().flatten());
        Theme::from_string(stored_theme)
    });
    let (theme, set_theme) = signal(Theme::default());

    Effect::new(move |_| {
        if let Some(theme_loaded) = initial_theme.get() {
            set_theme.set(theme_loaded);
        }
    });
    Effect::new(move |_| {
        // Get the current theme value from the signal
        let maybe_current_theme = theme.get();
        let document: web_sys::Element = document().document_element().unwrap();
        // Access the document (client-side only)
        match document.set_attribute("data-theme", &maybe_current_theme.to_string()) {
            Ok(()) => (),
            Err(err) => debug_error!("Not cool -> {:?}", err),
        }
    });

    let toggle_theme = move |ev: MouseEvent| {
        ev.prevent_default();
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
