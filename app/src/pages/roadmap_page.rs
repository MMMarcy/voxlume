use leptos::prelude::*;

use crate::utils::sidebar::use_hide_sidebar;

#[component]
pub fn RoadmapPage() -> impl IntoView {
    use_hide_sidebar();
    view! {
        <section class="section pt-1">
            <div class="container content">
                <h1 class="title">"Roadmap"</h1>
                <p>"Here is what we are working on:"</p>

                <div class="box">
                    <table class="table is-fullwidth is-striped is-hoverable">
                        <thead>
                            <tr>
                                <th>"Item"</th>
                                <th>"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td>"Better responsive UI"</td>
                                <td><span class="tag is-warning">"WIP"</span></td>
                            </tr>
                            <tr>
                                <td>"Integration with Hardcover for reviews/ratings"</td>
                                <td><span class="tag is-warning">"WIP"</span></td>
                            </tr>
                            <tr>
                                <td>"Linking to Audible"</td>
                                <td><span class="tag is-light">"Not started"</span></td>
                            </tr>
                            <tr>
                                <td>"Recommendations of similar books"</td>
                                <td>
                                    <span class="tag is-info">"Planned"</span>
                                    <br/>
                                    <small>"Have the embeddings, missing the logic and surfacing it to the UI"</small>
                                </td>
                            </tr>
                            <tr>
                                <td>"Add form for feedback"</td>
                                <td><span class="tag is-light">"Not started"</span></td>
                            </tr>
                            <tr>
                                <td>"Push notifications"</td>
                                <td>
                                    <span class="tag is-info">"Considering"</span>
                                    <br/>
                                    <small>"Maybe, if there is enough appetite for it"</small>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </section>
    }
}
