//! Navbar component and utilities to toggle the `is_active` class.

use crate::ui_components::navbar::display_mode::DisplayModeSwitcher;
use crate::ui_components::navbar::notifications_entry::NotificationsWidget;
use crate::ui_components::navbar::searchbar::SearchBar;
use crate::ui_components::navbar::theme_toggler::ThemeSwitcher;
use crate::ui_components::user_login_component::LoginMenuItem;
use crate::ui_components::user_logout_component::LogoutMenuItem;
use crate::ui_components::user_register_component::RegisterMenuItem;
use entities_lib::entities::user::User;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_active, set_is_active) = signal(false);
    let on_click_handler = move |_| {
        set_is_active.update(|v| *v = !*v);
    };
    let user_signal = use_context::<RwSignal<Option<User>>>().unwrap();

    let location = use_location();
    Effect::new(move |_| {
        let path = location.pathname.get();
        leptos::logging::debug_warn!("Navbar effect: Path changed to {}", path);
        set_is_active.set(false);
    });

    view! {
        <nav class="navbar has-shadow" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <A class:navbar-item=true href="/">
                    <div>
                        <img src="/download.png" />
                    </div>
                </A>
                <A
                    attr:class="navbar-item is-hidden-desktop"
                    href="/most-recent/1"
                >
                    New arrivals
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
                    <A attr:class="navbar-item" href="/most-recent/1">New arrivals</A>
                    <A attr:class="navbar-item" href="/about">About</A>
                    <A attr:class="navbar-item" href="/roadmap">Roadmap</A>
                    // Navigate the subscriptions.
                    <Show when=move || { user_signal.get().is_some() }>
                        <NotificationsWidget />
                    </Show>
                </div>

                <div class="navbar-item">
                    <SearchBar />
                </div>
                <div class="navbar-end">

                    <a class="navbar-item" href="https://github.com/MMMarcy/voxlume" target="_blank">
                        <span class="icon is-medium">
                            <i class="fa-brands fa-github fa-lg"></i>
                        </span>
                    </a>

                    <div class="navbar-item">
                        <DisplayModeSwitcher />
                    </div>

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
                            <Show when=move || user_signal.get().is_none()>
                                <LoginMenuItem />
                                <RegisterMenuItem />
                            </Show>
                            <Show when=move || user_signal.get().is_some()>
                                <LogoutMenuItem />
                                <a class="navbar-item">Settings</a>
                                <A href="/manage/subscriptions" class:navbar-item=true>
                                    Manage subscriptions
                                </A>
                                // <a class="navbar-item">Export subscriptions</a>
                                // <a class="navbar-item">Import subscriptions</a>
                            </Show>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}
