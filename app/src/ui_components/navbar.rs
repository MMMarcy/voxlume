//! Navbar component and utilities to toggle the `is_active` class.

use crate::ui_components::{
    searchbar::SearchBar, theme_toggler::ThemeSwitcher, user_login_component::LoginMenuItem,
    user_logout_component::LogoutMenuItem, user_register_component::RegisterMenuItem,
};
use entities_lib::entities::user::User;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_active, set_is_active) = signal(false);
    let on_click_handler = move |_| {
        set_is_active.update(|v| *v = !*v);
    };
    let svg = include_str!("../../../public/temp.svg");
    let user_signal = use_context::<ReadSignal<User>>().unwrap();
    view! {
        <nav class="navbar has-shadow" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <A class:navbar-item=true href="/">
                    <div inner_html=svg />
                </A>

                <a
                    role="button"
                    class="navbar-burger"
                    aria-label="menu"
                    aria-expanded="false"
                    data-target="top-navbar"
                    on:click=on_click_handler
                    class:is-active=is_active
                >
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>

            <div id="top-navbar" class="navbar-menu" class:is-active=is_active>
                <div class="navbar-start">
                    <a class="navbar-item">Explore</a>
                    // Navigate the subscriptions.
                    <Show when=move || { !user_signal.get().is_guest() }>
                        <div class="navbar-item has-dropdown is-hoverable">
                            <a class="navbar-link">My subscriptions</a>

                            <div class="navbar-dropdown">
                                <a class="navbar-item">By author</a>
                                <a class="navbar-item">By series</a>
                                <a class="navbar-item">By reader</a>
                                <a class="navbar-item">By search terms</a>
                            </div>
                        </div>
                    </Show>
                    <div class="navbar-item"></div>
                </div>

                <div class="navbar-item">
                    <SearchBar />
                </div>
                <div class="navbar-end">

                    <div class="navbar-item">
                        <ThemeSwitcher />
                    </div>

                    <div class="navbar-item has-dropdown is-hoverable">
                        <a class="navbar-link">
                            <span class="icon is-medium">
                                <i class="fa-solid fa-user fa-lg"></i>
                            </span>
                        </a>

                        <div class="navbar-dropdown is-right">
                            <Show when=move || user_signal.get().is_guest()>
                                <LoginMenuItem />
                                <RegisterMenuItem />
                            </Show>
                            <Show when=move || !user_signal.get().is_guest()>
                                <LogoutMenuItem />
                                <a class="navbar-item">Settings</a>
                                <a class="navbar-item">Export subscriptions</a>
                                <a class="navbar-item">Import subscriptions</a>
                            </Show>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}
