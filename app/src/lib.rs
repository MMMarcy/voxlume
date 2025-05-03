use entities_lib::entities::user::User;
use leptos::logging::debug_warn;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use pages::logout::LogoutPage;

pub mod pages;
pub mod ui_components;

use crate::pages::login::LoginPage;
use crate::pages::register::RegisterPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script
                    src="https://kit.fontawesome.com/9af46bbad0.js"
                    crossorigin="anonymous"
                ></script>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
                <Stylesheet id="leptos" href="/pkg/voxlume.css" />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    use shared::auth_user::AuthSession;
    use tracing::{debug, error, span, Level};

    let span = span!(Level::TRACE, "get_current_user");
    let _guard = span.enter();

    if let Some(auth) = use_context::<AuthSession>() {
        debug!("Authsession available");
        let maybe_current_user = Ok(auth.current_user.clone().map(|v| v.into_user()));
        debug!("Maybe current_user: {:?}", maybe_current_user);
        return maybe_current_user;
    } else {
        error!("Authsession not available");
        return Err(ServerFnError::new("No auth context found."));
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    use crate::ui_components::navbar::Navbar;

    let default_user: User = Default::default();
    let (get_user_signal, set_user_signal) = signal(default_user);
    provide_context(get_user_signal);
    provide_context(set_user_signal);
    let get_current_user_op = OnceResource::new(get_current_user());
    Effect::new(move || match get_current_user_op.get() {
        Some(Ok(Some(user))) => {
            debug_warn!("Got user {:?} from session", user);
            set_user_signal(user);
        }
        Some(Ok(None)) => {
            debug_warn!("No user found for this session");
        }
        Some(Err(_)) => debug_warn!("Problems with user authentication"),
        None => {
            debug_warn!("Got None from get_current_user_op");
        }
    });

    view! {
        // sets the document title
        <Title text="Welcome to Leptos" />

        // content for this welcome page
        <Router>
            <main>
                <Navbar />
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=StaticSegment("/register") view=RegisterPage />
                    <Route path=StaticSegment("/login") view=LoginPage />
                    <Route path=StaticSegment("/logout") view=LogoutPage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the homepage of your application.
#[component]
fn HomePage() -> impl IntoView {
    let user_signal = use_context::<ReadSignal<User>>().unwrap();
    view! {
        <h1>Welcome to Leptos user {move || user_signal.get().username}!</h1>
        <br />
    }
}
