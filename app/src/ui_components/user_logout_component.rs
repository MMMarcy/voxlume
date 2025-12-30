use leptos::prelude::*;
use leptos::{IntoView, view};
use leptos_router::components::A;

#[component]
pub fn LogoutMenuItem() -> impl IntoView {
    view! {
        <A href="/logout" attr:class="navbar-item">
            Logout
        </A>
    }
}
