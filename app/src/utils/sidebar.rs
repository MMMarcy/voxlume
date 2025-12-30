use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct SidebarVisible(pub RwSignal<i32>);

pub fn use_hide_sidebar() {
    if let Some(SidebarVisible(sidebar_counter)) = use_context::<SidebarVisible>() {
        // Increment on mount (synchronously to avoid flash)
        sidebar_counter.update(|c| *c += 1);

        // Decrement on cleanup
        on_cleanup(move || {
            sidebar_counter.update(|c| *c -= 1);
        });
    }
}
