use dioxus::prelude::*;

#[component]
pub fn Header(cx: Scope) -> Element {
    cx.render(rsx! {
        header { class: "header",
            h1 { "[tauriasync file]" }
        }
    })
}