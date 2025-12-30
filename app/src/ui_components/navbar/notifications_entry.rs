use crate::pages::notifications::notifications_page::has_unseen_notifications_server;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn NotificationsWidget() -> impl IntoView {
    let has_unseen_notifications = Resource::new(
        || (),
        |()| async move { has_unseen_notifications_server().await },
    );

    view! {
        <Suspense>
        <A
            class:navbar-item=true
            class:has-notification-dot=move || {
                has_unseen_notifications
                    .get()
                    .is_some_and(|res| res.unwrap_or(false))
            }
            href="/notifications"
        >
            "New audiobooks for you"
            {move || {
                has_unseen_notifications
                    .get()
                    .is_some_and(|res| res.unwrap_or(false))
                    .then(|| view! { <span class="notification-dot"></span> })
            }}
        </A>
        </Suspense>
    }
}
