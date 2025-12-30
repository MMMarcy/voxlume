pub mod pages;
pub mod ui_components;
pub mod utils;

use std::future::IntoFuture;

use crate::pages::about_page::AboutPage;
use crate::pages::audiobook_page::AudiobookDetailedView;
use crate::pages::author_page::AuthorPage;
use crate::pages::by_series_page::BySeriesPage;
use crate::pages::category_page::CategoryPage;
use crate::pages::homepage::HomePage;
use crate::pages::keyword_page::KeywordPage;
use crate::pages::login::AuthPage;
use crate::pages::logout::LogoutPage;
use crate::pages::most_recent_page::MostRecentPage;
use crate::pages::notifications::notifications_page::NotificationsPage;
use crate::pages::reader_page::ReaderPage;
use crate::pages::register::RegisterPage;
use crate::pages::roadmap_page::RoadmapPage;
use crate::pages::search_page::SearchPage;
use crate::pages::subscriptions::manage_subscription_page::ManageSubscriptionsPage;
use entities_lib::ShareableArgsValues;
use entities_lib::entities::user::User;
use leptos::hydration::HydrationScripts;
use leptos::logging::debug_warn;
use leptos::prelude::*;
use leptos_meta::{HashedStylesheet, MetaTags, Title, provide_meta_context};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::{SsrMode, StaticSegment, path};

use crate::utils::sidebar::SidebarVisible;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <link rel="icon" type="image/x-icon" href="/download.ico" />
                <script async
                    src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-5212734413081238"
                    crossorigin="anonymous"
                ></script>
                <script src="/ads_script.js"></script>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script
                    src="https://kit.fontawesome.com/9af46bbad0.js"
                    crossorigin="anonymous"
                ></script>
                <AutoReload options=options.clone() />
                <MetaTags />
                <HashedStylesheet options=options.clone() id="voxlume" />
                <meta name="google-adsense-account" content="ca-pub-5212734413081238"/>
            </head>
            <body>
                <App />
                <HydrationScripts options />
            </body>
        </html>
    }
}

#[allow(clippy::unused_async)]
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<User>, ServerFnError> {
    use shared::auth_user::AuthSession;
    use tracing::{Level, span};

    let span = span!(Level::TRACE, "get_current_user");
    let _guard = span.enter();

    if let Some(auth) = use_context::<AuthSession>() {
        Ok(auth
            .current_user
            .clone()
            .map(shared::sql_user::SqlUser::into_user))
    } else {
        Err(ServerFnError::new("No auth context found."))
    }
}

#[allow(clippy::unused_async)]
#[server(GetSharedArgs, "/api")]
pub async fn get_shared_args() -> Result<ShareableArgsValues, ServerFnError> {
    use shared::state::AppState;
    let state = AppState::get_app_state()?;
    Ok(state.shareable_args)
}

#[component]
pub fn App() -> impl IntoView {
    use crate::ui_components::navbar::Navbar;
    use crate::ui_components::footer::Footer;

    use crate::ui_components::sidebar::sidebar_impl::ExplorerSidebar; // Import the sidebar
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (get_shared_args_signal, set_shared_args_signal) =
        signal::<Option<ShareableArgsValues>>(None);
    provide_context(get_shared_args_signal);

    let get_shared_args_op =
        OnceResource::new_blocking(async { get_shared_args().await }.into_future());
    Effect::new(move || {
        if let Some(Ok(shared_args)) = get_shared_args_op.get() {
            set_shared_args_signal.set(Some(shared_args));
        }
    });

    let user_signal = RwSignal::new(Option::None::<User>);
    let (sidebar_open, set_sidebar_open) = signal(true);
    provide_context(user_signal);

    // Sidebar visibility context
    // 0 = visible, > 0 = hidden (counts number of components requesting hide)
    let sidebar_visible = RwSignal::new(0);
    provide_context(SidebarVisible(sidebar_visible));

    Effect::new(move || {
        let w = window();
        if let Ok(width_val) = w.inner_width()
            && let Some(width) = width_val.as_f64()
            && width < 1024.0
        {
            set_sidebar_open.set(false);
        }
    });

    let get_current_user_op = OnceResource::new_blocking(
        async {
            let current_user = get_current_user().await;
            debug_warn!("Fetched current user: {:?}", current_user);
            current_user
        }
        .into_future(),
    );
    Effect::new(move || {
        match get_current_user_op
            .get()
            .and_then(std::result::Result::ok)
            .flatten()
        {
            Some(user) => {
                debug_warn!("Got user {:?} from session", user);
                user_signal.set(Some(user));
            }
            None => {
                debug_warn!("Got None from get_current_user_op");
            }
        }
    });

    view! {
        <Title text="Voxlume" />
        <Router>
            <main>
                // Your Navbar can now get the user signal from context
                <Navbar />

                // Added layout structure for sidebar
                <div class="container mt-4">
                    <div class="columns is-relative">
                        // Sidebar Column (25% on desktop, hidden on mobile if desired, or stacks)
                        <Show when=move || sidebar_visible.get() == 0>
                            <div class=move || {
                                let base = if sidebar_open.get() { "column is-3-desktop is-2-widescreen" } else { "column is-narrow" };
                                format!("{base} sidebar-column")
                            }>
                                <ExplorerSidebar
                                    is_open=sidebar_open
                                    toggle=Callback::new(move |()| set_sidebar_open.update(|v| *v = !*v))
                                />
                            </div>
                        </Show>

                        // Main Content Column
                        <div class="column">
                            <Routes fallback=|| "Page not found.".into_view()>
                                <Route path=StaticSegment("") view=HomePage ssr=SsrMode::Async />
                                <Route path=StaticSegment("/about") view=AboutPage ssr=SsrMode::Async />
                                <Route path=StaticSegment("/roadmap") view=RoadmapPage ssr=SsrMode::Async />
                                <Route path=path!("/notifications") view=NotificationsPage />
                                <Route path=path!("/most-recent/:page") view=MostRecentPage />
                                <Route path=path!("/manage/subscriptions") view=ManageSubscriptionsPage />
                                <Route path=path!("/author/:author_id/:author_name/:page") view=AuthorPage ssr=SsrMode::Async />
                                <Route path=path!("/reader/:reader_id/:reader_name/:page") view=ReaderPage ssr=SsrMode::Async/>
                                <Route path=path!("/category/:category_id/:category_name/:page") view=CategoryPage ssr=SsrMode::Async/>
                                <Route path=path!("/keyword/:keyword_id/:keyword_name/:page") view=KeywordPage ssr=SsrMode::Async/>

                                <Route path=path!("/series/:series_id/:series_title/:page") view=BySeriesPage ssr=SsrMode::Async/>
                                <Route path=path!("/audiobook/:audiobook_id") view=AudiobookDetailedView ssr=SsrMode::Async/>

                                <Route path=path!("/search/:search_string") view=SearchPage ssr=SsrMode::Async/>

                                <Route path=StaticSegment("/register") view=RegisterPage />
                                <Route path=StaticSegment("/login") view=AuthPage />
                                <Route path=StaticSegment("/logout") view=LogoutPage />
                            </Routes>
                        </div>
                    </div>
                </div>
                <Footer />
            </main>
        </Router>
    }
}
