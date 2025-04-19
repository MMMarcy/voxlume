//! Navbar component and utilities to toggle the `is_active` class.

use leptos::{html::HtmlElement, logging, prelude::*};

#[component]
pub fn Navbar() -> impl IntoView {
    let (is_active, set_is_active) = signal(false);
    let on_click_handler = move |_| {
        set_is_active.update(|v| *v = !*v);
    };
    let svg = include_str!("../../../public/test.svg");
    view! {
    <nav class="navbar" role="navigation" aria-label="main navigation">
        <div class="navbar-brand">
            <a class="navbar-item" href="https://bulma.io" inner_html=svg>

            </a>

            <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false" data-target="top-navbar" on:click=on_click_handler class:is-active=is_active>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            </a>
        </div>

        <div id="top-navbar" class="navbar-menu" class:is-active=is_active>
            <div class="navbar-start">
            <a class="navbar-item">
                Home
            </a>

            <a class="navbar-item">
                Documentation
            </a>

            <div class="navbar-item has-dropdown is-hoverable">
                <a class="navbar-link">
                More
                </a>

                <div class="navbar-dropdown">
                <a class="navbar-item">
                    About
                </a>
                <a class="navbar-item is-selected">
                    Jobs
                </a>
                <a class="navbar-item">
                    Contact
                </a>
                <hr class="navbar-divider"/>
                <a class="navbar-item">
                    Report an issue
                </a>
                </div>
            </div>
            </div>

            <div class="navbar-end">
            <div class="navbar-item">
                <div class="buttons">
                <a class="button is-primary">
                    <strong>Sign up</strong>
                </a>
                <a class="button is-light">
                    Log in
                </a>
                </div>
            </div>
            </div>
        </div>
    </nav>
    }
}
