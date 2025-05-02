use leptos::{prelude::*, view, IntoView};
use leptos_router::components::A;

#[component]
pub fn RegisterMenuItem() -> impl IntoView {
    view! {
        <A href="/register" attr:class="navbar-item">
            Sign in
        </A>
    }
}
