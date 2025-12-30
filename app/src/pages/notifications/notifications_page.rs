use crate::utils::dates::print_date;
use entities_lib::{AudiobookWithData, UserNotification};
use leptos::leptos_dom::debug_error;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

/// Checks if the current user has any unseen notifications.
#[server(HasUnseenNotifications, "/api")]
pub async fn has_unseen_notifications_server() -> Result<bool, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::notifications::has_unseen_notifications;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;

    has_unseen_notifications(&pgpool, user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Fetches notifications and joins them with Audiobook data.
#[server(GetNotifications, "/api")]
async fn get_notifications() -> Result<Vec<(UserNotification, AudiobookWithData)>, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::audiobook_ops::get_audiobooks_cached;
    use shared::db_ops::parade::notifications::list_user_notifications;
    use shared::state::AppState;
    use std::collections::HashMap;
    use tracing::info;

    let state = AppState::get_app_state()?;
    let cache = state.audiobooks_cache;
    let pgpool = state.database_connection_pool;
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;

    info!("Fetching notifications for user {}", user.id);
    let notifications = list_user_notifications(&pgpool, user.id)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    if notifications.is_empty() {
        return Ok(Vec::new());
    }

    // Extract IDs to batch fetch audiobooks
    let audiobook_ids: Vec<i64> = notifications.iter().map(|n| n.audiobook_id).collect();

    let audiobooks = get_audiobooks_cached(
        &pgpool,
        &cache,
        entities_lib::GetAudioBookRequestType::ByIdList(audiobook_ids),
        20,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    info!("Gotten audiobooks");

    let books_map: HashMap<i64, AudiobookWithData> =
        audiobooks.into_iter().map(|b| (b.0.id, b)).collect();

    let combined: Vec<_> = notifications
        .into_iter()
        .filter_map(|n| books_map.get(&n.audiobook_id).map(|b| (n, b.clone())))
        .collect();

    Ok(combined)
}

/// Marks a notification as read. Enforces ownership security.
#[server(MarkNotificationAsSeen, "/api")]
async fn mark_notification_as_read(notification: UserNotification) -> Result<(), ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::notifications::mark_notification_as_seen;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;

    if user.id != notification.user_id {
        return Err(ServerFnError::new(
            "Unauthorized: You can only mark your own notifications as read.",
        ));
    }

    mark_notification_as_seen(&pgpool, notification)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(DeleteNotification, "/api")]
async fn delete_notification(notification: UserNotification) -> Result<(), ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::notifications::delete_notification;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;

    if user.id != notification.user_id {
        return Err(ServerFnError::new(
            "Unauthorized: You can only mark your own notifications as read.",
        ));
    }

    delete_notification(&pgpool, notification)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Marks all notifications as read for the current user.
#[server(MarkAllNotificationsAsSeen, "/api")]
async fn mark_all_notifications_as_seen() -> Result<(), ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::notifications::mark_all_notifications_as_seen;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;

    mark_all_notifications_as_seen(&pgpool, user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Deletes all notifications for the current user.
#[server(DeleteAllNotifications, "/api")]
async fn delete_all_notifications() -> Result<(), ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::notifications::delete_all_user_notifications;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user = auth_session
        .current_user
        .ok_or(ServerFnError::new("Couldn't find current user."))?;

    delete_all_user_notifications(&pgpool, user.id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

/// Renders a single notification row.
/// Handles interaction logic (hover/click) to mark as read.
#[component]
fn NotificationRow(
    item: (UserNotification, AudiobookWithData),
    mark_as_read_action: Action<UserNotification, Result<(), ServerFnError>>,
    delete_notification_action: Action<UserNotification, Result<(), ServerFnError>>,
) -> impl IntoView {
    let (notification, (audiobook, authors, _categories, _keywords, _readers, _maybe_series)) =
        item;

    // Clone for the event closure
    let notification_for_closure = notification.clone();
    let notification_for_delete = notification.clone();
    let (is_hovered, set_is_hovered) = signal(false);

    let mark_as_read_closure = move |_| {
        if !notification_for_closure.has_been_seen {
            // Dispatch the server action
            mark_as_read_action.dispatch(notification_for_closure.clone());
        }
    };

    let on_delete_click = move |ev: leptos::ev::MouseEvent| {
        ev.stop_propagation(); // Prevent triggering row click (mark as read)
        delete_notification_action.dispatch(notification_for_delete.clone());
    };

    // Row styling based on read status
    let row_class = move || {
        if notification.has_been_seen {
            "has-text-grey"
        } else {
            "has-background-white-ter has-text-weight-medium has-text-grey-dark"
        }
    };

    // Title styling to ensure contrast in both states
    let title_class = move || {
        if notification.has_been_seen {
            "" // Default / Inherit
        } else {
            "has-text-grey-darker" // Strong dark text for unread
        }
    };

    view! {
        <tr
            class=row_class
            // UX: Mark as read on hover or click for convenience
            on:mouseenter={
                let value = mark_as_read_closure.clone();
                move |ev| {
                    set_is_hovered.set(true);
                    value(ev);
                }
            }
            on:mouseleave=move |_| set_is_hovered.set(false)
            on:click=mark_as_read_closure
            style="cursor: pointer; transition: background-color 0.2s;"
        >
            <td class="is-vcentered has-text-centered" style="width: 3em;">
                {move || {
                    if notification.has_been_seen {
                         view! { <span class="icon is-small has-text-grey-light"><i class="far fa-circle"></i></span> }.into_any()
                    } else {
                         view! { <span class="icon is-small has-text-info"><i class="fas fa-circle"></i></span> }.into_any()
                    }
                }}
            </td>
            <td class="is-vcentered">
                <A href=format!("/audiobook/{}", audiobook.id) attr:class=title_class>
                    {audiobook.title.clone()}
                </A>
            </td>
            <td class=move || format!("{} is-vcentered", title_class())>
                {print_date(notification.created_at)}
            </td>
            <td class=move || format!("{} is-vcentered", title_class())>
                {authors.first().map(|a| a.name.clone()).unwrap_or_default()}
            </td>
            <td class=move || format!("{} is-vcentered", title_class()) inner_html=notification.format_reasons()></td>

            // Action column: Delete button appears on hover
            <td class="is-vcentered has-text-right">
                <div class:is-invisible=move || !is_hovered.get()>
                    <button
                        class="button is-small is-danger is-light is-rounded"
                        title="Delete notification"
                        on:click=on_delete_click
                    >
                        <span class="icon is-small"><i class="fas fa-trash"></i></span>
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[allow(clippy::too_many_lines)]
#[component]
pub fn NotificationsPage() -> impl IntoView {
    crate::utils::sidebar::use_hide_sidebar();

    // We use a Signal to store notifications so we can mutate the list locally
    // (e.g., update "has_been_seen") without refetching the entire list from the server.
    let notifications_signal: RwSignal<Vec<(UserNotification, AudiobookWithData)>> =
        RwSignal::new(Vec::new());

    let get_notifications_resource = OnceResource::new(get_notifications());

    let mark_as_read_action = Action::new(|notification: &UserNotification| {
        let notification_clone = notification.clone();
        async move { mark_notification_as_read(notification_clone).await }
    });
    let delete_notification_action = Action::new(|notification: &UserNotification| {
        let notification_clone = notification.clone();
        async move { delete_notification(notification_clone).await }
    });
    let mark_all_as_seen_action = Action::new(|(): &()| mark_all_notifications_as_seen());
    let delete_all_action = Action::new(|()| delete_all_notifications());

    let _ = Effect::new(move |_| {
        delete_notification_action.version().track();
        if let Some(Ok(())) = delete_notification_action.value().get()
            && let Some(notification) = delete_notification_action.input().get()
        {
            // TODO: This doesn't delete right away from the UI. Investigate why.
            notifications_signal.update(|notifications| {
                notifications.retain(|v| v.0 != notification);
            });
        }
    });

    // Effect: Update local state when "Mark as Read" action succeeds
    let _ = Effect::new(move |_| {
        mark_as_read_action.version().track();

        if let Some(Ok(())) = mark_as_read_action.value().get()
            && let Some(seen_notification) = mark_as_read_action.input().get()
        {
            notifications_signal.update(|notifications| {
                if let Some((notification_to_update, _)) = notifications
                    .iter_mut()
                    .find(|(n, _)| *n == seen_notification)
                {
                    notification_to_update.has_been_seen = true;
                }
            });
        }
    });

    // Effect: Populate signal when initial resource loads
    let _ = Effect::new(move || {
        if let Some(user_notifications_res) = get_notifications_resource.get() {
            match user_notifications_res {
                Ok(value) => notifications_signal.set(value),
                Err(error) => debug_error!("Error loading notifications: {}", error.to_string()),
            }
        }
    });

    // Effect: Update local state when "Mark All as Seen" action succeeds
    let _ = Effect::new(move |_| {
        mark_all_as_seen_action.version().track();

        if let Some(Ok(())) = mark_all_as_seen_action.value().get() {
            notifications_signal.update(|notifications| {
                for (n, _) in notifications.iter_mut() {
                    n.has_been_seen = true;
                }
            });
        }
    });

    // Effect: Update local state when "Delete All" action succeeds
    let _ = Effect::new(move |_| {
        delete_all_action.version().track();

        if let Some(Ok(())) = delete_all_action.value().get() {
            notifications_signal.set(Vec::new());
        }
    });

    let on_delete_all_click = move |_| {
        let confirmed = window()
            .confirm_with_message("Are you sure you want to delete all notifications?")
            .unwrap_or(false);

        if confirmed {
            delete_all_action.dispatch(());
        }
    };

    view! {
        <Title text="Notifications" />
        <section class="section">
            <div class="container">
                <div class="columns is-centered">
                    <div class="column is-10-widescreen is-12-desktop">
                        <div class="box">
                            // Header Section
                            <div class="level is-mobile mb-4">
                                <div class="level-left">
                                    <div class="level-item">
                                        <h1 class="title is-4 has-text-grey-dark">
                                            <span class="icon is-medium mr-2 has-text-link">
                                                <i class="fas fa-bell"></i>
                                            </span>
                                            "Notifications"
                                        </h1>
                                    </div>
                                </div>
                                <div class="level-right">
                                    <div class="level-item">
                                        <div class="buttons has-addons">
                                            <button
                                                class="button is-small is-info is-light"
                                                on:click=move |_| {mark_all_as_seen_action.dispatch(());}
                                                disabled=move || notifications_signal.with(std::vec::Vec::is_empty)
                                                title="Mark all as read"
                                            >
                                                <span class="icon is-small"><i class="fas fa-check-double"></i></span>
                                                <span class="is-hidden-mobile">"Mark all read"</span>
                                            </button>
                                            <button
                                                class="button is-small is-danger is-light"
                                                on:click=on_delete_all_click
                                                disabled=move || notifications_signal.with(std::vec::Vec::is_empty)
                                                title="Delete all"
                                            >
                                                <span class="icon is-small"><i class="fas fa-trash-alt"></i></span>
                                                <span class="is-hidden-mobile">"Delete all"</span>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <hr class="mt-0" />

                            // Notifications List
                            <Show
                                when=move || !notifications_signal.with(std::vec::Vec::is_empty)
                                fallback=|| view! {
                                    <div class="has-text-centered py-6">
                                        <span class="icon is-large has-text-grey-lighter mb-3">
                                            <i class="far fa-bell-slash fa-3x"></i>
                                        </span>
                                        <p class="is-size-5 has-text-grey">"No new notifications"</p>
                                        <p class="is-size-7 has-text-grey-light">"We'll let you know when something happens."</p>
                                    </div>
                                }
                            >
                                <div class="table-container">
                                    <table class="table is-fullwidth is-hoverable">
                                        <thead>
                                            <tr>
                                                <th class="has-text-grey-light" style="width: 3em;"></th>
                                                <th class="has-text-grey">"Audiobook"</th>
                                                <th class="has-text-grey">"Date"</th>
                                                <th class="has-text-grey">"Author"</th>
                                                <th class="has-text-grey">"Reason"</th>
                                                <th style="width: 4em;"></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || notifications_signal.get()
                                                key=|item| (item.0.user_id, item.0.audiobook_id, item.0.has_been_seen)
                                                let:item
                                            >
                                                <NotificationRow
                                                    item=item
                                                    mark_as_read_action=mark_as_read_action
                                                    delete_notification_action=delete_notification_action
                                                />
                                            </For>
                                        </tbody>
                                    </table>
                                </div>
                            </Show>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
