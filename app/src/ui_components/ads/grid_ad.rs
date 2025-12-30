use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Bind to the helper function we added in index.html
    #[wasm_bindgen(js_namespace = window, js_name = triggerAdSense, catch)]
    fn trigger_adsense() -> Result<(), JsValue>;
}

#[component]
pub fn GridAd(ad_slot: &'static str) -> impl IntoView {
    let container_ref: NodeRef<Div> = NodeRef::new();

    Effect::new(move |_| {
        match container_ref.get() {
            Some(container) if container.child_element_count() == 0 => {
                if let Some(document) = window().document()
                    && let Ok(ins) = document.create_element("ins")
                {
                    ins.set_class_name("adsbygoogle");

                    // 1. Apply size strictly to the style
                    let style = "display:block;";
                    // let style = format!("display:inline-block; width:{width}; height:{height};",);
                    let _ = ins.set_attribute("style", style);

                    // 2. Set REQUIRED AdSense attributes
                    let _ = ins.set_attribute("data-ad-client", "ca-pub-5212734413081238");
                    let _ = ins.set_attribute("data-ad-slot", ad_slot);

                    // 3. REMOVE these lines if using fixed width/height:
                    let _ = ins.set_attribute("data-ad-format", "auto");
                    let _ = ins.set_attribute("data-full-width-responsive", "true");

                    // Append to container
                    let _ = container.append_child(&ins);

                    // Trigger AdSense
                    // We wrap this in a slight timeout to ensure the DOM has reflowed
                    // and the container has actual dimensions before AdSense scans it.
                    set_timeout(
                        move || match trigger_adsense() {
                            Ok(()) => leptos::logging::log!("Rust: Triggered AdSense"),
                            Err(e) => leptos::logging::error!("Rust: AdSense error: {:?}", e),
                        },
                        // 100ms delay to allow DOM painting
                        std::time::Duration::from_millis(100),
                    );
                }
            }
            _ => (),
        }
    });

    view! {
        <div
            class="ad-container"
            node_ref=container_ref
            // Ensure the parent also holds the space to prevent CLS (Layout Shift)
            // style=format!("min-height: {}; width:{}; height:{};", height, width, height)
        >
        </div>
    }
}
