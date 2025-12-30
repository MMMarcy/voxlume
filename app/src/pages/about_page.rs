use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    crate::utils::sidebar::use_hide_sidebar();
    view! {
        <section class="section pt-1">
            <div class="container content">
                <h1 class="title">"About Voxlume"</h1>
                <p>
                    "Voxlume was created to fill a gap in the audiobook world."
                </p>
                <div class="block mb-6">
                    <h3 class="subtitle">"Why not Audible?"</h3>
                    <p>
                        "Audible is a fantastic platform, but it is heavily optimized for sales. "
                        "This often means that notifications about new books from your favorite "
                        "authors or series feel kneecapped. You might miss out on a release simply "
                        "because the algorithm decided to push a bestseller instead."
                    </p>
                </div>
                <div class="block mb-6">
                    <h3 class="subtitle">"Why not AudiobookBay?"</h3>
                    <p>
                        "AudiobookBay is a massive resource, but let's be honest: it's largely a dump. "
                        "It lacks reliable notifications, and the search feature is rudimentary at best, "
                        "making it hard to find exactly what you're looking for or discover new gems."
                    </p>
                </div>
                <div class="block">
                    <h3 class="subtitle">"Our Promise"</h3>
                    <p>
                        "We are committed to a user-friendly experience."
                        "We will "<strong>"never use any invasive ads"</strong>
                        ". We run the bare minimum of advertisements necessary to keep the site"
                        "running and potentially buy a round of beers for the team. Our goal is to "
                        "help you follow your favorite stories without the noise."
                    </p>
                </div>
            </div>
        </section>
    }
}
