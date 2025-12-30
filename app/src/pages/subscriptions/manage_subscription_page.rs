use entities_lib::entities::subscription::Subscription;
use leptos::prelude::*;
use leptos_meta::Title;

#[server(GetSubscriptions, "/api")]
async fn get_subscriptions() -> Result<Vec<Subscription>, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::subscription_ops::list_subscriptions;
    use shared::state::AppState;
    use tracing::info;

    info!("Before getting app state");
    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    info!("Gotten app state");
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;
    list_subscriptions(&pgpool, user.id)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))
}

#[server(DeleteSubscriptions, "/api")]
async fn remove_subscription(sub: Subscription) -> Result<(), ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::subscription_ops::delete_subscription;
    use shared::state::AppState;
    use tracing::{error, info};

    info!("Before getting app state");
    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    info!("Gotten app state");
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;
    if user.id != sub.user_id {
        error!(
            "User {} tried to delete a subscription that doesn't belong to it.",
            user.id
        );
        return Err(ServerFnError::new(
            "You tried to delete a subscription that doesn't belong to you",
        ));
    }
    return delete_subscription(&pgpool, sub)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")));
}

#[component]
pub fn ManageSubscriptionsPage() -> impl IntoView {
    crate::utils::sidebar::use_hide_sidebar();

    // Action to wrap the server function call
    let delete_action = Action::new(move |sub: &Subscription| {
        let sub_clone = sub.clone();
        async move { remove_subscription(sub_clone).await }
    });

    // A resource that refetches whenever the delete action finishes
    let subscriptions_resource = Resource::new(
        move || delete_action.version().get(), // Depend on the action's version
        move |_| get_subscriptions(),
    );

    let remove_sub = move |sub: Subscription| {
        delete_action.dispatch(sub);
    };

    view! {
        <Title text="Manage subscriptions" />
        <div class="container">
            <table class="table is-hoverable is-fullwidth">
                <thead>
                    <tr>
                        <th>"Type"</th>
                        <th>"Name"</th>
                        <th>"Action"</th>
                    </tr>
                </thead>
                <tbody>
                    <Suspense fallback=move || {
                        view! {
                            <tr>
                                <td colspan="3">"Loading..."</td>
                            </tr>
                        }
                    }>
                        {move || {
                            subscriptions_resource
                                .get()
                                .map(|res| {
                                    match res {
                                        Err(e) => {
                                            view! {
                                                <tr>
                                                    <td colspan="3">{format!("Error: {e}")}</td>
                                                </tr>
                                            }
                                                .into_any()
                                        }
                                        Ok(subs) => {
                                            if subs.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td colspan="3">"No subscriptions found."</td>
                                                    </tr>
                                                }
                                                    .into_any()
                                            } else {
                                                subs.into_iter()
                                                    .map(move |sub| {
                                                        let sub_for_handler = sub.clone();
                                                        view! {
                                                            <tr>
                                                                <td>{sub.render_type()}</td>
                                                                <td>{sub.render_name()}</td>
                                                                <td>
                                                                    <button on:click=move |_| remove_sub(
                                                                        sub_for_handler.clone(),
                                                                    )>"Remove"</button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                                    .into_any()
                                            }
                                        }
                                    }
                                })
                        }}
                    </Suspense>
                </tbody>
            </table>
        </div>
    }
}
