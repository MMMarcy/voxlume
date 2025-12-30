use entities_lib::entities::user::User;
use leptos::html::Input;
use leptos::logging::error;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_location, use_navigate};
// Import the register server function.
// Assuming 'register' is a sibling module in 'pages'.
use super::register::register_user;

#[server(LoginUser, "/api")]
pub async fn login_user(
    username: Option<String>,
    password: Option<String>,
) -> Result<User, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::sql_user::SqlUser;
    use shared::state::AppState;
    use tracing::{debug, error, info};

    let username = {
        let username_err: ServerFnError = ServerFnError::Args("Username must be defined".into());
        username.ok_or(username_err)?
    };
    let password = {
        let password_err: ServerFnError = ServerFnError::Args("Password must be defined".into());
        password.ok_or(password_err)?
    };
    debug!("Checked inputs. They are ok");

    let maybe_auth = use_context::<AuthSession>();
    if maybe_auth.is_none() {
        return Err(ServerFnError::new("Couldn't retrieve auth context"));
    }
    debug!("Auth context is available");

    let auth = maybe_auth.unwrap();
    let db_pool = AppState::get_database_connection_pool()?;
    let argon2 = AppState::get_password_handler()?;

    info!(
        "Is current user available: {:?}",
        auth.current_user.is_some()
    );

    let maybe_logged_in_user = SqlUser::login_user(username, password, &db_pool, &argon2).await;
    match maybe_logged_in_user {
        Ok(sql_user) => {
            auth.login_user(sql_user.id);
            debug!("User {} successfully login", &sql_user.username);
            Ok(sql_user.into_user())
        }
        Err(err) => {
            error!("Error logging user in: {:?}", err);
            Err(ServerFnError::new("Couldn't log in users"))
        }
    }
}

#[allow(clippy::too_many_lines)]
#[component]
pub fn AuthPage() -> impl IntoView {
    let set_user_signal =
        use_context::<RwSignal<Option<User>>>().expect("The user write signal should be mounted");

    crate::utils::sidebar::use_hide_sidebar();

    let location = use_location();

    // Determine initial state based on URL path
    let (is_login, set_is_login) = signal(true);

    Effect::new(move |_| {
        let path = location.pathname.get();
        if path.contains("register") || path.contains("sign-up") {
            set_is_login.set(false);
        } else {
            set_is_login.set(true);
        }
    });

    let username_ref: NodeRef<Input> = NodeRef::new();
    let password_ref: NodeRef<Input> = NodeRef::new();
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let (is_loading, set_is_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_message.set(None);

        spawn_local(async move {
            let username_val = username_ref.get().map(|i| i.value());
            let password_val = password_ref.get().map(|i| i.value());

            let result = if is_login.get() {
                login_user(username_val, password_val).await
            } else {
                register_user(username_val, password_val).await
            };

            set_is_loading.set(false);

            match result {
                Ok(registered_user) => {
                    set_user_signal.set(Some(registered_user));
                    use_navigate()("/", NavigateOptions::default());
                }
                Err(err) => {
                    // Extract a user-friendly message if possible
                    let msg = err.to_string().replace("Server Fn Error: ", "");
                    set_error_message.set(Some(msg));
                    error!("{}", err);
                }
            }
        });
    };

    view! {
        <section class="hero is-fullheight-with-navbar">
            <div class="hero-body">
                <div class="container">
                    <div class="columns is-centered">
                        <div class="column is-5-tablet is-4-desktop is-3-widescreen">

                            <div class="box auth-box" >
                                <div class="has-text-centered mb-5">
                                    <span class="icon is-large has-text-link">
                                        <i class="fas fa-3x fa-user-circle"></i>
                                    </span>
                                    <h3 class="title is-4 mt-2">
                                        {move || if is_login.get() { "Welcome Back" } else { "Create Account" }}
                                    </h3>
                                </div>

                                <div class="tabs is-centered is-toggle is-fullwidth is-small mb-5">
                                    <ul>
                                        <li class:is-active=is_login>
                                            <a on:click=move |_| set_is_login.set(true)>
                                                <span class="icon is-small"><i class="fas fa-sign-in-alt"></i></span>
                                                <span>Login</span>
                                            </a>
                                        </li>
                                        <li class:is-active=move || !is_login.get()>
                                            <a on:click=move |_| set_is_login.set(false)>
                                                <span class="icon is-small"><i class="fas fa-user-plus"></i></span>
                                                <span>Register</span>
                                            </a>
                                        </li>
                                    </ul>
                                </div>

                                <Show when=move || error_message.get().is_some()>
                                    <div class="notification is-danger is-light is-size-7">
                                        <button class="delete" on:click=move |_| set_error_message.set(None)></button>
                                        {error_message.get()}
                                    </div>
                                </Show>

                                <form on:submit=on_submit>
                                    <div class="field">
                                        <label class="label">Username</label>
                                        <div class="control has-icons-left">
                                            <input
                                                node_ref=username_ref
                                                class="input"
                                                type="text"
                                                placeholder="e.g. alexsmith"
                                                required
                                            />
                                            <span class="icon is-small is-left">
                                                <i class="fas fa-user"></i>
                                            </span>
                                        </div>
                                    </div>

                                    <div class="field">
                                        <label class="label">Password</label>
                                        <div class="control has-icons-left">
                                            <input
                                                node_ref=password_ref
                                                class="input"
                                                type="password"
                                                placeholder="********"
                                                required
                                            />
                                            <span class="icon is-small is-left">
                                                <i class="fas fa-lock"></i>
                                            </span>
                                        </div>
                                    </div>

                                    <div class="field mt-5">
                                        <button
                                            class="button is-link is-fullwidth"
                                            class:is-loading=is_loading
                                            type="submit"
                                        >
                                            {move || if is_login.get() { "Log in" } else { "Sign up" }}
                                        </button>
                                    </div>
                                </form>

                                <div class="has-text-centered mt-4 is-size-7">
                                    <Show when=move || is_login.get() fallback=move || view! {
                                        <span>"Already have an account? " <a on:click=move |_| set_is_login.set(true)>"Log in"</a></span>
                                    }>
                                        <span>"Don't have an account? " <a on:click=move |_| set_is_login.set(false)>"Sign up"</a></span>
                                    </Show>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
