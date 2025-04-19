use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub mod ui_components;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script src="https://kit.fontawesome.com/9af46bbad0.js" crossorigin="anonymous"></script>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
                <Stylesheet id="leptos" href="/pkg/voxlume.css"/>
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
    use entities_lib::entities::audiobook::AudioBook;
    use leptos::logging::{log, warn};
    use neo4rs::{query, Graph};
    use shared::state::AppState;
    use tokio;

    if let Some(app_state) = use_context::<AppState>() {
        let graph: Graph = app_state.graph;

        let res = tokio::spawn(async move {
            let mut acc: Vec<AudioBook> = vec![];
            let mut stream = graph
                .execute(query(
                    r#"
            MATCH (p:Audiobook)
            RETURN p
            "#,
                ))
                .await
                .unwrap();
            while let Some(row) = stream.next().await.unwrap() {
                match row.get("p") {
                    Ok(value) => acc.push(value),
                    Err(e) => log!("{}", e),
                }
            }
            acc
        })
        .await
        .unwrap();

        log!("Query result: {:?}", res.len());
    } else {
        warn!("No state");
    }
    Ok(())
}

/// Renders the homepage of your application.
#[component]
fn HomePage() -> impl IntoView {
    use crate::ui_components::navbar::Navbar;
    use crate::ui_components::theme_toggler::ThemeSwitcher;
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;
    let server_fn = move |_| {
        spawn_local(async {
            let _ = test_server_fn().await;
        });
    };
    view! {
        <Navbar />
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <br/>
        <button
            class="has-text-primary"
            on:click=server_fn>"Call server fn"</button>
        <br/>
        <div class="container">
            <ThemeSwitcher/>
        </div>
    }
}
