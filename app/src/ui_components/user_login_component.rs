use leptos::{prelude::*, view, IntoView};
use leptos_router::components::A;

#[component]
pub fn LoginMenuItem() -> impl IntoView {
    view! {
        <A href="/login" attr:class="navbar-item">
            Log in
        </A>
    }
}
