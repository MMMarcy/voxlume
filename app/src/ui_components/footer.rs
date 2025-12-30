use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="footer">
            <div class="content has-text-centered">
                <p>
                    <strong>Voxlume</strong> by <a href="https://voxlu.me/">Voxlume Team</a>.
                </p>
            </div>
        </footer>
    }
}
