use entities_lib::entities::user::User;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

#[server(LogoutUser, "/api")]
#[allow(clippy::unused_async)]
pub async fn logout_user() -> Result<(), ServerFnError> {
    use shared::auth_user::AuthSession;
    use tracing::{debug, error, info, warn};

    debug!("Inside logout user");

    if let Some(auth) = use_context::<AuthSession>() {
        debug!("auth_session available");
        if let Some(user) = &auth.current_user {
            debug!("User can be logged out");
            auth.logout_user();
            info!("User {} logged out", &user.username);
        } else {
            warn!("User not available to be logged out");
        }
    } else {
        error!("Couldn't get the the auth from the context");
        return Err(ServerFnError::new("Couldn't get the auth context"));
    }
    Ok(())
}

#[component]
pub fn LogoutPage() -> impl IntoView {
    crate::utils::sidebar::use_hide_sidebar();
    let navigation = use_navigate();
    let set_user_signal = use_context::<RwSignal<Option<User>>>().unwrap();
    let res = OnceResource::new(logout_user());
    Effect::new(move || match res.get() {
        Some(Ok(())) => {
            log!("Successful logout");
            set_user_signal.set(None);
            navigation("/", NavigateOptions::default());
        }
        Some(Err(_)) => log!("Error in logging user out"),
        None => log!("Couldn't get shit fro the logout function"),
    });
    view! { <p>Log out</p> }
}
