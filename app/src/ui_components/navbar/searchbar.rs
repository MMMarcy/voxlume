use leptos::prelude::*;
use leptos::{html, logging};
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
use web_sys::SubmitEvent;

fn handle_submit(e: &SubmitEvent, input_ref: NodeRef<html::Input>) {
    e.prevent_default();

    let navigation = use_navigate();
    let text_input_value = input_ref.get().unwrap().value();
    logging::debug_warn!("Content of text input: {}", &text_input_value);
    navigation(
        format!("/search/{text_input_value}").as_str(),
        NavigateOptions::default(),
    );
}

#[component]
pub fn SearchBar() -> impl IntoView {
    let input_ref: NodeRef<html::Input> = NodeRef::new();
    let on_submit = move |e: SubmitEvent| handle_submit(&e, input_ref);

    view! {
        <form class="field has-addons" on:submit=on_submit>
            <div class="control has-icons-left is-expanded">
                <input
                    class="input is-medium is-rounded"
                    type="text"
                    placeholder="Search library..."
                    node_ref=input_ref
                />
                <span class="icon is-left">
                    <i class="fas fa-magnifying-glass"></i>
                </span>
            </div>
            <div class="control">
                <button class="button is-link is-medium is-rounded" type="submit">
                    "Search"
                </button>
            </div>
        </form>
    }
}
