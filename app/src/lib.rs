use leptos::logging::error;
use leptos::{logging::log, prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/voxlume.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[server]
async fn test_server_fn() -> Result<(), ServerFnError> {
    use leptos::logging::{log, warn};
    use shared::state::AppState;
    if let Some(str) = use_context::<AppState>() {
        log!("test_server_fn state: {}", str.state_str);
    } else {
        warn!("No state");
    }
    Ok(())
}

/// Renders the homepage of your application.
#[component]
fn HomePage() -> impl IntoView {
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

    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    let server_fn = move |_| {
        spawn_local(async {
            let _ = test_server_fn().await;
        });
    };
    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <br/>
        <button
            class="has-text-primary"
            on:click=server_fn>"Call server fn"</button>
        <br/>
        <button on:click=toggle_theme>"Change theme"</button>
    }
}
