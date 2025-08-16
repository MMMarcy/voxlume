use entities_lib::entities::user::User;
use leptos::logging::error;
use leptos::{html::Input, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

#[server(LoginUser, "/api")]
pub async fn login_user(
    username: Option<String>,
    password: Option<String>,
) -> Result<User, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::state::AppState;
    use tracing::{debug, error, info};

    use shared::sql_user::SqlUser;

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

#[component]
fn LoginUsernameField(node_ref: NodeRef<Input>) -> impl IntoView {
    view! {
        <div class="field mt-2">
            <label class="label">Username</label>
            <div class="control has-icons-left has-icons-right">
                <input
                    node_ref=node_ref
                    class="input"
                    type="text"
                    placeholder="Username here"
                    value=""
                />
                <span class="icon is-small is-left">
                    <i class="fas fa-user"></i>
                </span>
            </div>
        </div>
    }
}

#[component]
fn PasswordField(node_ref: NodeRef<Input>) -> impl IntoView {
    view! {
        <div class="field mt-2">
            <label class="label">Password</label>
            <div class="control has-icons-left has-icons-right">
                <input
                    node_ref=node_ref
                    class="input"
                    type="password"
                    placeholder="Password"
                    value=""
                />
                <span class="icon is-small is-left">
                    <i class="fas fa-key"></i>
                </span>
            </div>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let set_user_signal =
        use_context::<WriteSignal<User>>().expect("The user write signal should be mounted");
    let username_ref: NodeRef<Input> = NodeRef::new();
    let password_ref: NodeRef<Input> = NodeRef::new();
    let on_submit = move |_| {
        spawn_local(async move {
            let navigation = use_navigate();
            match login_user(
                Some(
                    username_ref
                        .get_untracked()
                        .expect("Input should be mounted")
                        .value(),
                ),
                Some(
                    password_ref
                        .get_untracked()
                        .expect("Password input should be mounted")
                        .value(),
                ),
            )
            .await
            {
                Ok(registered_user) => {
                    let () = &set_user_signal.set(registered_user);
                    navigation("/", NavigateOptions::default());
                }
                // TODO: Make this error surface also to the UI.
                Err(err) => error!("{}", err),
            }
        });
    };
    view! {
        <section class="section is-medium">

            <div class="box">
                <LoginUsernameField node_ref=username_ref />

                <PasswordField node_ref=password_ref />

                <div class="field is-grouped mt-5">
                    <div class="control">
                        <button class="button is-link" on:click=on_submit>
                            Submit
                        </button>
                    </div>
                    <div class="control">
                        <button class="button is-link is-light">Cancel</button>
                    </div>
                </div>

            </div>
        </section>
    }
}
