use entities_lib::entities::subscription::Subscription;
use entities_lib::{SubscriptionExists, SubscriptionType, User};
/// Buttons that let you add/delete subscriptions.
use leptos::{leptos_dom::debug_error, logging::debug_warn, prelude::*, task::spawn_local};

#[server(DoesSubscriptionExists, "/api")]
async fn subscription_exists_server_fn(
    maybe_subscription: Option<Subscription>,
) -> Result<SubscriptionExists, ServerFnError> {
    use shared::db_ops::parade::subscription_ops::subscription_exists;
    use shared::state::AppState;
    use tracing::{debug, info};

    if maybe_subscription.is_none() {
        return Ok(SubscriptionExists::Unknown);
    }
    let subscription = maybe_subscription.unwrap();

    debug!("Before getting app state");
    let state = AppState::get_app_state()?;
    debug!("Gotten app state");
    let res = subscription_exists(&state.database_connection_pool, subscription)
        .await
        .map(|v| {
            if v {
                return SubscriptionExists::Yes;
            }
            SubscriptionExists::No
        })
        .map_err(|e| ServerFnError::new(e.to_string()));
    info!("{:?}", res);
    res
}

#[server(SubscribeUnsubscribe, "/api")]
async fn subscribe_unsubscribe_server_fn(
    subscription_exists_value: SubscriptionExists,
    maybe_subscription: Option<Subscription>,
) -> Result<SubscriptionExists, ServerFnError> {
    use shared::db_ops::parade::subscription_ops::{add_subscription, delete_subscription};
    use shared::state::AppState;
    use tracing::debug;

    if maybe_subscription.is_none() {
        return Ok(SubscriptionExists::Unknown);
    }
    if subscription_exists_value == SubscriptionExists::Unknown {
        return Err(ServerFnError::new(
            "Can't register/delete a subscription with unknown value",
        ));
    }
    let subscription = maybe_subscription.unwrap();

    debug!("Before getting app state");
    let state = AppState::get_app_state()?;
    debug!("Gotten app state");
    if subscription_exists_value == SubscriptionExists::Yes {
        // Delete subscription.
        delete_subscription(&state.database_connection_pool, subscription)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        return Ok(SubscriptionExists::No);
    }
    add_subscription(&state.database_connection_pool, subscription)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(SubscriptionExists::Yes)
}

#[component]
pub fn SubscriptionPanel(sub_type: Option<Signal<SubscriptionType>>) -> impl IntoView {
    let user_signal = use_context::<RwSignal<Option<User>>>().unwrap();
    let should_show = Signal::derive(move || user_signal().is_some() && sub_type.is_some());
    let maybe_subscription = Signal::derive(move || {
        if should_show() {
            let sub = Subscription {
                user_id: user_signal().unwrap().id,
                subscription_type: sub_type.unwrap().get(),
            };
            return Some(sub);
        }
        None
    });
    let subscription_exists_signal = RwSignal::new(SubscriptionExists::Unknown);
    let button_text = Signal::derive(move || match subscription_exists_signal() {
        SubscriptionExists::Unknown => "Loading...",
        SubscriptionExists::Yes => "Unsubscribe",
        SubscriptionExists::No => "Subscribe",
    });

    let check_subscription_exists_resource = Resource::new(maybe_subscription, move |maybe_sub| {
        subscription_exists_server_fn(maybe_sub)
    });

    Effect::new(move || {
        let res = check_subscription_exists_resource.get();
        match res {
            Some(Ok(value)) => subscription_exists_signal.set(value),
            Some(Err(e)) => debug_error!("An error occurred {e}"),
            None => debug_warn!("Got None when checking if subscription exists."),
        }
    });

    let on_click_handler = move |_| {
        spawn_local(async move {
            match subscribe_unsubscribe_server_fn(
                subscription_exists_signal.get_untracked(),
                maybe_subscription.get_untracked(),
            )
            .await
            {
                Ok(v) => subscription_exists_signal.set(v),
                Err(e) => debug_error!("{e}"),
            }
        });
    };

    view! {
        <Show when=should_show>
            <div>
                <button class="button" on:click=on_click_handler>
                    {move || button_text()}
                </button>
            </div>
        </Show>
    }
}
