use crate::pages::login::AuthPage;
use entities_lib::entities::user::User;
use leptos::prelude::*;

#[server(RegisterUser, "/api")]
pub async fn register_user(
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

    debug!(
        "Fn get_current_user: {:?}",
        auth.current_user
            .clone()
            .map(shared::sql_user::SqlUser::into_user)
    );

    let local_user = SqlUser::create_local_user(username, &password, &argon2);

    info!("Fn get_current_user: {:?}", &local_user);
    match local_user.register_user(&db_pool).await {
        Ok(registered_user) => {
            return {
                auth.login_user(registered_user.id);
                Ok(registered_user.into_user())
            };
        }
        Err(err) => {
            error!("Error: {:?}", err);
        }
    }
    Ok(User::default())
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <AuthPage />
    }
}
