//! Navbar component and utilities to toggle the `is_active` class.

use crate::ui_components::theme_toggler::ThemeSwitcher;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_active, set_is_active) = signal(false);
    let on_click_handler = move |_| {
        set_is_active.update(|v| *v = !*v);
    };
    let svg = include_str!("../../../public/temp.svg");
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
                    // Navigate the subscriptions.
                    <div class="navbar-item has-dropdown is-hoverable">
                        <a class="navbar-link">My subscriptions</a>

                        <div class="navbar-dropdown">
                            <a class="navbar-item">By author</a>
                            <a class="navbar-item is-selected">By series</a>
                            <a class="navbar-item">By reader</a>
                            <a class="navbar-item">By search terms</a>
                        </div>
                    </div>
                    <div class="navbar-item">
                    </div>
                </div>

                <div class="navbar-item">
                    <div class="field has-addons">
                        <div class="control has-icons-left">
                            <input class="input is-medium" type="email" placeholder="Search here" />
                            <span class="icon is-left">
                                <i class="fas fa-magnifying-glass"></i>
                            </span>
                        </div>
                        <div class="control">
                            <button class="button is-warning is-medium">
                                Syntax
                            </button>
                        </div>
                    </div>
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
                            <a class="navbar-item">Settings</a>
                            <a class="navbar-item">Export subscriptions</a>
                            <a class="navbar-item">Import subscriptions</a>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}
