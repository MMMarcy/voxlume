use std::str::FromStr;

use entities_lib::entities::audiobook_display::AudiobookDisplayMode;
use leptos::leptos_dom::debug_log;
use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::utils::local_storage::local_storage;

const LOCAL_STORAGE_KEY: &str = "display_mode";

// Define an enum for themes for better type safety (optional but recommended)

#[component]
fn TableLikeButton() -> impl IntoView {
    view! {
        <span class="icon">
            <i class="fas fa-lg fa-table"></i>
        </span>
    }
}

#[component]
fn ListLikeButton() -> impl IntoView {
    view! {
        <span class="icon">
            <i class="fas fa-lg fa-list"></i>
        </span>
    }
}

#[component]
fn GridLikeButton() -> impl IntoView {
    view! {
        <span class="icon">
            <i class="fas fa-lg fa-grip"></i>
        </span>
    }
}

#[component]
pub fn DisplayModeSwitcher() -> impl IntoView {
    // Creates a reactive value to update the button
    let initial_display_mode = LocalResource::new(move || async {
        let maybe_stored_display_mode =
            local_storage().and_then(|storage| storage.get_item(LOCAL_STORAGE_KEY).ok().flatten());
        debug_log!("maybe stored display_mode {:?}", maybe_stored_display_mode);
        if let Some(stored_display_mode) = maybe_stored_display_mode {
            AudiobookDisplayMode::from_str(&stored_display_mode)
                .unwrap_or(AudiobookDisplayMode::default())
        } else {
            AudiobookDisplayMode::default()
        }
    });
    let (display_mode_read_signal, display_mode_write_signal) =
        signal(AudiobookDisplayMode::default());
    provide_context(display_mode_read_signal);

    Effect::new(move |_| {
        if let Some(loaded_mode) = initial_display_mode.get() {
            display_mode_write_signal.set(loaded_mode);
        }
    });

    let cycle_display_mode = move |ev: MouseEvent| {
        ev.prevent_default();
        display_mode_write_signal.update(|current| {
            *current = match current {
                AudiobookDisplayMode::TableLike => AudiobookDisplayMode::ListLike,
                AudiobookDisplayMode::ListLike => AudiobookDisplayMode::GridLike,
                AudiobookDisplayMode::GridLike => AudiobookDisplayMode::TableLike,
            };
            local_storage().and_then(|storage| {
                storage
                    .set_item(LOCAL_STORAGE_KEY, &current.to_string())
                    .ok()
            });
        });
    };

    view! {
        <div on:click=cycle_display_mode>
            // {
            // move || match display_mode_rw_signal.get() {
            // AudiobookDisplayMode::TableLike => view! {<TableLikeButton />}.into_any(),
            // AudiobookDisplayMode::ListLike => view! {<ListLikeButton />}.into_any(),
            // AudiobookDisplayMode::GridLike => view! {<GridLikeButton />}.into_any(),
            // }
            // }
            <Show when=move || { display_mode_read_signal.get() == AudiobookDisplayMode::ListLike }>
                <ListLikeButton />
            </Show>
            <Show when=move || {
                display_mode_read_signal.get() == AudiobookDisplayMode::TableLike
            }>
                <TableLikeButton />
            </Show>
            <Show when=move || { display_mode_read_signal.get() == AudiobookDisplayMode::GridLike }>
                <GridLikeButton />
            </Show>
        </div>
    }
}
