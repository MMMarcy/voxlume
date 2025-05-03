use entities_lib::entities::user::User;
use leptos::logging::error;
use leptos::{html::Input, prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;

#[server(LoginUser, "/api")]
pub async fn login_user(
    username: Option<String>,
    password: Option<String>,
) -> Result<User, ServerFnError> {
    use argon2::Argon2;
    use shared::auth_user::AuthSession;
    use tracing::{debug, error, info};

    use shared::auth_user::SqlUser;
    use sqlx::PgPool;

    if username.is_none() {
        return Err(ServerFnError::new("Username can't be null"));
    }

    if password.is_none() {
        return Err(ServerFnError::new("Password can't be null"));
    }
    info!("Checked inputs. They are ok");

    let maybe_auth = use_context::<AuthSession>();
    let maybe_db = use_context::<PgPool>();
    let maybe_argon2 = use_context::<Argon2<'_>>();

    if maybe_auth.is_none() {
        return Err(ServerFnError::new("Couldn't retrieve auth context"));
    }
    info!("Auth context is available");

    if maybe_db.is_none() {
        return Err(ServerFnError::new("Couldn't retreive psql connection"));
    }
    info!("Db connection context is available");

    if maybe_argon2.is_none() {
        return Err(ServerFnError::new("Couldn't retrieve argon2 params."));
    }
    info!("Argon2 params are available");

    let auth = maybe_auth.unwrap();
    let db_pool = maybe_db.unwrap();
    let argon2 = maybe_argon2.unwrap();

    info!(
        "Fn get_current_user: {:?}",
        auth.current_user.clone().map(|u| u.into_user())
    );

    // Unwrapping username and password are safe ops because we checked them above.
    let maybe_logged_in_user =
        SqlUser::login_user(username.unwrap(), password.unwrap(), &db_pool, argon2).await;
    match maybe_logged_in_user {
        Ok(sql_user) => {
            auth.login_user(sql_user.id);
            debug!("User {} successfully login", &sql_user.username);
            return Ok(sql_user.into_user());
        }
        Err(err) => {
            error!("Error logging user in: {:?}", err);
            return Err(ServerFnError::new("Couldn't log in users"));
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
                    let _ = &set_user_signal.set(registered_user);
                    navigation("/", Default::default());
                }
                // TODO: Make this error surface also to the UI.
                Err(err) => error!("{}", err),
            };
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
