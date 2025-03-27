#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde_wasm_bindgen::to_value;
// use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
// use std::{collections::HashMap, rc::Rc};
use web_sys::console;

static CSS: Asset = asset!("/assets/styles.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone, PartialEq)]
enum SidebarItem {
    ApiCollections,
    // History,
    // Environments,
    // Settings,
    Invite,
}

impl Default for SidebarItem {
    fn default() -> Self {
        SidebarItem::ApiCollections
    }
}

#[derive(Default)]
struct AppState {
    method: String,
    url: String,
    request_body: String,
    response: String,
    selected_sidebar: SidebarItem,
}

pub fn App() -> Element {
    let mut state = use_signal(|| AppState {
        method: "GET".to_string(),
        url: "https://jsonplaceholder.typicode.com/posts/1".to_string(),
        selected_sidebar: SidebarItem::ApiCollections,
        ..Default::default()
    });

    let send_request = move || {
        console::log_1(&format!("Sending request").into());

        spawn({
            let method = state.read().method.clone();
            let url = state.read().url.clone();
            
            async move {
                let args = serde_json::json!({
                    "method": method,
                    "url": url,
                });
    
                console::log_1(&format!("Sending request with args: {:#?}", args).into());
    
                let response = invoke("do_request", to_value(&args).unwrap()).await;
                state.write().response = response.as_string().unwrap();
            }
        });
    };

    rsx! {
        document::Stylesheet { href: CSS }
    
        div { class: "app-container",
            Sidebar {
                selected_item: state.read().selected_sidebar.clone(),
                on_select: move |item| state.write().selected_sidebar = item
            }
    
            div { class: "main-panel",
                div { class: "request-header",
                    select {
                        value: "{state.read().method}",
                        onchange: move |e| state.write().method = e.value().clone(),
                        option { value: "GET", "GET" }
                        option { value: "POST", "POST" }
                    }
                    input {
                        class: "url-input",
                        r#type: "text",
                        value: "{state.read().url}",
                        oninput: move |e| state.write().url = e.value().clone(),
                    }
                    button {
                        class: "send-button",
                        onclick: move |_| {
                            console::log_1(&format!(">>> Sending request").into());

                            let _ = send_request();
                        },
                        "Send"
                    }

                    
                }
    
                div { class: "tabbed-content",
                    RequestTabs {}
    
                    div { class: "response-view",
                        h2 { "Response Preview" }
                        div { class: "response-container",
                            pre {
                                class: "response-content",
                                if state.read().response.is_empty() {
                                    "No response data yet"
                                } else {
                                    "{state.read().response}"
                                }
                            }
                        }
                    }
                }
    
                div { class: "status-bar",
                    span { "Rustman v0.0.1" }
                }
            }
        }
    }
}

#[component]
fn Sidebar(selected_item: SidebarItem, on_select: EventHandler<SidebarItem>) -> Element {
    let items = vec![
        ("APIs", "📚", SidebarItem::ApiCollections),
        // ("History", "🕒", SidebarItem::History),
        // ("Environments", "⚙️", SidebarItem::Environments),
        // ("Settings", "🔧", SidebarItem::Settings),
        ("Invite", "👥", SidebarItem::Invite),
    ];

    rsx! {
        aside { class: "sidebar",
            ul { class: "sidebar-menu",
                for (name, icon, item) in items {
                    ul {
                        class: if selected_item == item { "sidebar-item active" } else { "sidebar-item" },
                        onclick: move |_| on_select.call(item.clone()),
                        span { class: "sidebar-icon", "{icon} " }
                        span { class: "sidebar-text", "{name}" }
                    }
                }
            }
        }
    }
}

#[component]
fn RequestTabs() -> Element {
    let tabs = vec![
        ("Params", "params"),
        ("Headers", "headers"),
        ("Body", "body"),
        ("Auth", "auth"),
        ("Docs", "docs"),
    ];

    rsx! {
        div { class: "request-tabs",
            div { class: "tab-buttons",
                for (label, id) in tabs {
                    button {
                        class: "tab-btn",
                        id: "{id}",
                        "{label}"
                    }
                }
            }
            div { class: "tab-content",
                div { class: "tab-pane", id: "params",
                    textarea { placeholder: "Params (JSON)" }
                }
                div { class: "tab-pane", id: "headers",
                    textarea { placeholder: "Headers (JSON)" }
                }
                div { class: "tab-pane", id: "body",
                    textarea { placeholder: "Body (JSON)" }
                }
                div { class: "tab-pane", id: "auth",
                    textarea { placeholder: "Auth (JSON)" }
                }
                div { class: "tab-pane", id: "docs",
                    textarea { placeholder: "Docs" }
                }
            }
        }
    }
}

#[component]
fn ResponseMeta(status_code: u16, time: u128) -> Element {
    rsx! {
        div { class: "response-meta",
            div { class: "status-badge {status_code_class(status_code)}",
                "{status_code}"
            }
            div { class: "response-time",
                "Time: {time}ms"
            }
        }
    }
}

fn status_code_class(code: u16) -> &'static str {
    match code / 100 {
        2 => "success",
        3 => "redirect",
        4 => "client-error",
        5 => "server-error",
        _ => "unknown",
    }
}

// pub fn App() -> Element {

//     rsx!(
//         main { h1 { "Rustman" } }
//     )

// let mut name = use_signal(|| String::new());
// let mut greet_msg = use_signal(|| String::new());

// let greet = move |_: FormEvent| async move {
//     if name.read().is_empty() {
//         return;
//     }

//     let name = name.read();
//     let args = serde_wasm_bindgen::to_value(&GreetArgs { name: &*name }).unwrap();
//     // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//     let new_msg = invoke("greet", args).await.as_string().unwrap();
//     greet_msg.set(new_msg);
// };

// rsx! {
//     link { rel: "stylesheet", href: "styles.css" }
//     main {
//         class: "container",
//         h1 { "Welcome to Tauri + Dioxus" }

//         div {
//             class: "row",
//             a {
//                 href: "https://tauri.app",
//                 target: "_blank",
//                 img {
//                     src: "/tauri.svg",
//                     class: "logo tauri",
//                      alt: "Tauri logo"
//                 }
//             }
//             a {
//                 href: "https://dioxuslabs.com/",
//                 target: "_blank",
//                 img {
//                     src: "/dioxus.png",
//                     class: "logo dioxus",
//                     alt: "Dioxus logo"
//                 }
//             }
//         }
//         p { "Click on the Tauri and Dioxus logos to learn more." }

//         form {
//             class: "row",
//             onsubmit: greet,
//             input {
//                 id: "greet-input",
//                 placeholder: "Enter a name...",
//                 value: "{name}",
//                 oninput: move |event| name.set(event.value())
//             }
//             button { r#type: "submit", "Greet" }
//         }
//         p { "{greet_msg}" }
//     }
// }
// }
