use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum IsModalActive {
    Active,
    Hidden,
}

impl From<bool> for IsModalActive {
    fn from(value: bool) -> Self {
        match value {
            true => IsModalActive::Active,
            false => IsModalActive::Hidden,
        }
    }
}

impl From<IsModalActive> for bool {
    fn from(value: IsModalActive) -> Self {
        match value {
            IsModalActive::Active => true,
            _ => false,
        }
    }
}

#[component]
fn SyntaxExplainerModal() -> impl IntoView {
    let modal_signal: RwSignal<IsModalActive> =
        use_context().expect("The signal should be mounted");
    let read_signal_temporary: Signal<bool> = Signal::derive(move || modal_signal.get().into());
    let handle_exit_button = move |_| modal_signal.update(|v| *v = IsModalActive::Hidden);
    view! {
        <div class="modal" class:is-active=read_signal_temporary>
            <div class="modal-background"></div>
            <div class="modal-card">
                <header class="modal-card-head">
                    <p class="modal-card-title">Modal title</p>
                    <button class="delete" aria-label="close"
                        on:click=handle_exit_button
                    />
                </header>
                <section class="modal-card-body">
                    <div class="content">
                        <h1>Hello World</h1>
                        <p>
                            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla accumsan, metus ultrices eleifend gravida, nulla nunc varius lectus, nec rutrum justo nibh eu lectus. Ut vulputate semper dui. Fusce erat odio, sollicitudin vel erat vel, interdum mattis neque.
                        </p>
                        <h2>Second level</h2>
                        <p>
                            Curabitur accumsan turpis pharetra <strong>augue tincidunt</strong>
                            blandit. Quisque condimentum maximus mi, sit amet commodo arcu rutrum id. Proin pretium urna vel cursus venenatis. Suspendisse potenti. Etiam mattis sem rhoncus lacus dapibus facilisis. Donec at dignissim dui. Ut et neque nisl.
                        </p>
                        <ul>
                            <li>In fermentum leo eu lectus mollis, quis dictum mi aliquet.</li>
                            <li>Morbi eu nulla lobortis, lobortis est in, fringilla felis.</li>
                            <li>
                                Aliquam nec felis in sapien venenatis viverra fermentum nec lectus.
                            </li>
                            <li>Ut non enim metus.</li>
                        </ul>
                        <h3>Third level</h3>
                        <p>
                            Quisque ante lacus, malesuada ac auctor vitae, congue
                            <a href="#">non ante</a>
                            . Phasellus lacus ex, semper ac tortor nec, fringilla condimentum orci. Fusce eu rutrum tellus.
                        </p>
                        <ol>
                            <li>Donec blandit a lorem id convallis.</li>
                            <li>Cras gravida arcu at diam gravida gravida.</li>
                            <li>Integer in volutpat libero.</li>
                            <li>Donec a diam tellus.</li>
                            <li>Aenean nec tortor orci.</li>
                            <li>Quisque aliquam cursus urna, non bibendum massa viverra eget.</li>
                            <li>Vivamus maximus ultricies pulvinar.</li>
                        </ol>
                        <blockquote>
                            Ut venenatis, nisl scelerisque sollicitudin fermentum, quam libero hendrerit ipsum, ut blandit est tellus sit amet turpis.
                        </blockquote>
                        <p>
                            Quisque at semper enim, eu hendrerit odio. Etiam auctor nisl et
                            <em>justo sodales</em>
                            elementum. Maecenas ultrices lacus quis neque consectetur, et lobortis nisi molestie.
                        </p>
                        <p>
                            Sed sagittis enim ac tortor maximus rutrum. Nulla facilisi. Donec mattis vulputate risus in luctus. Maecenas vestibulum interdum commodo.
                        </p>
                        <p>
                            Suspendisse egestas sapien non felis placerat elementum. Morbi tortor nisl, suscipit sed mi sit amet, mollis malesuada nulla. Nulla facilisi. Nullam ac erat ante.
                        </p>
                        <h4>Fourth level</h4>
                        <p>
                            Nulla efficitur eleifend nisi, sit amet bibendum sapien fringilla ac. Mauris euismod metus a tellus laoreet, at elementum ex efficitur.
                        </p>
                        <p>
                            Maecenas eleifend sollicitudin dui, faucibus sollicitudin augue cursus non. Ut finibus eleifend arcu ut vehicula. Mauris eu est maximus est porta condimentum in eu justo. Nulla id iaculis sapien.
                        </p>
                        <p>
                            Phasellus porttitor enim id metus volutpat ultricies. Ut nisi nunc, blandit sed dapibus at, vestibulum in felis. Etiam iaculis lorem ac nibh bibendum rhoncus. Nam interdum efficitur ligula sit amet ullamcorper. Etiam tristique, leo vitae porta faucibus, mi lacus laoreet metus, at cursus leo est vel tellus. Sed ac posuere est. Nunc ultricies nunc neque, vitae ultricies ex sodales quis. Aliquam eu nibh in libero accumsan pulvinar. Nullam nec nisl placerat, pretium metus vel, euismod ipsum. Proin tempor cursus nisl vel condimentum. Nam pharetra varius metus non pellentesque.
                        </p>
                        <h5>Fifth level</h5>
                        <p>
                            Aliquam sagittis rhoncus vulputate. Cras non luctus sem, sed tincidunt ligula. Vestibulum at nunc elit. Praesent aliquet ligula mi, in luctus elit volutpat porta. Phasellus molestie diam vel nisi sodales, a eleifend augue laoreet. Sed nec eleifend justo. Nam et sollicitudin odio.
                        </p>
                        <h6>Sixth level</h6>
                        <p>
                            Cras in nibh lacinia, venenatis nisi et, auctor urna. Donec pulvinar lacus sed diam dignissim, ut eleifend eros accumsan. Phasellus non tortor eros. Ut sed rutrum lacus. Etiam purus nunc, scelerisque quis enim vitae, malesuada ultrices turpis. Nunc vitae maximus purus, nec consectetur dui. Suspendisse euismod, elit vel rutrum commodo, ipsum tortor maximus dui, sed varius sapien odio vitae est. Etiam at cursus metus.
                        </p>

                    </div>
                </section>
                <footer class="modal-card-foot">
                    <div class="buttons">
                        <button class="button" on:click=handle_exit_button>Cancel</button>
                    </div>
                </footer>
            </div>
        </div>
    }
}

#[component]
pub fn SearchBar() -> impl IntoView {
    let modal_active_signal = RwSignal::new(IsModalActive::Hidden);
    provide_context(modal_active_signal);
    let activate_modal = move |_| {
        modal_active_signal.set(IsModalActive::Active);
    };
    view! {
        <div class="field has-addons">
            <div class="control has-icons-left is-expanded">
                <input class="input is-medium" type="text" placeholder="Search here" />
                <span class="icon is-left">
                    <i class="fas fa-magnifying-glass"></i>
                </span>
            </div>
            <div class="control">
                <button class="button is-primary is-medium" on:click=activate_modal>Syntax</button>
            </div>
        </div>
        <SyntaxExplainerModal />
    }
}
