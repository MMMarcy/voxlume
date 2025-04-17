//! Component that toggles between dark and light theme.

use leptos::logging::{error, log};
use leptos::prelude::*;
use web_sys;

#[component]
pub fn ThemeSwitcher() -> impl IntoView {
    // Creates a reactive value to update the button
    // 1. Create a signal to hold the theme state
    let (theme, set_theme) = signal("light".to_string());

    // 3. Create an effect to update the <html> tag's attribute
    Effect::new(move |_| {
        // Get the current theme value from the signal
        let current_theme = theme.get();

        // Access the document (client-side only)
        let h: web_sys::Element = document().document_element().expect("boh");
        match h.set_attribute("data-theme", &current_theme) {
            Ok(_) => log!("Cool!"),
            Err(err) => error!("Not cool -> {:?}", err),
        }
    });

    // 2. Button to toggle the theme
    let toggle_theme = move |_| {
        set_theme.update(|current| {
            *current = if current == "light" {
                "dark".to_string()
            } else {
                "light".to_string()
            };
        });
    };

    view! {
        <Show
            when=
        <div class="bd-cycle-moon" on:click=toggle_theme>
          <span class="icon">
            <i class="fas fa-lg fa-moon"></i>
          </span>
        </div>
    }
}
