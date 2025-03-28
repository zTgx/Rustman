#![allow(non_snake_case)]

use dioxus::{html::g::class, prelude::*};
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
    params: Vec<Param>, // 存储所有参数
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

                let args = to_value(&args);
                match args {
                    Ok(args) => {
                        let response = invoke("do_request", args).await;
                        state.write().response = response.as_string().unwrap();
                    }
                    Err(e) => {
                        console::error_1(&format!("Failed to serialize args: {:#?}", e).into());
                    }
                }
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
                    RequestTabs { state: state.clone() } // Pass the `state` signal here

                    div { class: "response-view",
                        h2 { "Response" }
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
fn RequestTabs(state: Signal<AppState>) -> Element {
    let tabs = vec![
        ("Params", "params"),
        ("Headers", "headers"),
        ("Body", "body"),
        ("Auth", "auth"),
        ("Docs", "docs"),
    ];

    // Parse URL parameters when URL changes
    let on_url_change = move |new_url: String| {
        let mut state = state.write();
        state.url = new_url.clone();

        if new_url.contains('?') {
            state.params.clear();
            if let Some(query) = new_url.split('?').nth(1) {
                for pair in query.split('&') {
                    let mut kv = pair.splitn(2, '=');
                    if let (Some(name), Some(value)) = (kv.next(), kv.next()) {
                        state.params.push(Param {
                            name: name.to_string(),
                            value: urlencoding::decode(value).unwrap_or_default().to_string(),
                            type_: "string".to_string(),
                            required: false,
                            description: "".to_string(),
                        });
                    }
                }
            }
        }
    };

    // Update URL when parameters change
    let mut update_url = move || {
        let mut state = state.write();
        let base_url = state.url.split('?').next().unwrap_or_default();
        let query_string = state
            .params
            .iter()
            .filter(|p| !p.name.is_empty())
            .map(|p| {
                format!(
                    "{}={}",
                    urlencoding::encode(&p.name),
                    urlencoding::encode(&p.value)
                )
            })
            .collect::<Vec<_>>()
            .join("&");

        state.url = if query_string.is_empty() {
            base_url.to_string()
        } else {
            format!("{}?{}", base_url, query_string)
        };
    };

    let mut active_tab = use_signal(|| "params".to_string());

    rsx! {
        div { class: "request-tabs",
            div { class: "tab-buttons",
                for (label, id) in tabs {
                    button {
                        class: format!(
                            "tab-btn {}",
                            if *active_tab.read() == *id { "active" } else { "" }
                        ),
                        onclick: move |_| active_tab.set(id.to_string()),
                        id: "{id}",
                        "{label}"
                    }
                }
            }

            div { class: "tab-content",
                // Params Tab
                div { class: "tab-pane active", id: "params",
                    h3 { "Query Params" }
                    table { class: "params-table",
                        thead {
                            tr {
                                th { "Name" }
                                th { "Value" }
                                th { "Type" }
                                th { "*" }
                                th { "Description" }
                                th { "" }
                            }
                        }
                        tbody {
                            for (index, param) in state.read().params.iter().enumerate() {
                                tr { key: "{index}",
                                    td {
                                        input {
                                            type: "text",
                                            value: "{param.name}",
                                            oninput: move |e| {
                                                state.write().params[index].name = e.value();
                                                update_url();
                                            }
                                        }
                                    }
                                    td {
                                        input {
                                            type: "text",
                                            value: "{param.value}",
                                            oninput: move |e| {
                                                state.write().params[index].value = e.value();
                                                update_url();
                                            }
                                        }
                                    }
                                    td {
                                        select {
                                            value: "{param.type_}",
                                            onchange: move |e| {
                                                state.write().params[index].type_ = e.value();
                                                update_url();
                                            },
                                            option { value: "string", "string" }
                                            option { value: "array", "array" }
                                            option { value: "number", "number" }
                                            option { value: "boolean", "boolean" }
                                        }
                                    }
                                    td {
                                        input {
                                            type: "checkbox",
                                            checked: param.required,
                                            onchange: move |e| {
                                                state.write().params[index].required = e.value().parse().unwrap_or(false);
                                            }
                                        }
                                    }
                                    td {
                                        input {
                                            type: "text",
                                            value: "{param.description}",
                                            oninput: move |e| {
                                                state.write().params[index].description = e.value();
                                            }
                                        }
                                    }
                                    td {
                                        button {
                                            class: "delete-btn",
                                            onclick: move |_| {
                                                state.write().params.remove(index);
                                                update_url();
                                            },
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    button {
                        class: "add-param-btn",
                        onclick: move |_| {
                            state.write().params.push(Param::default());
                            update_url();
                        },
                        "Add param"
                    }
                }

                // Other tabs...
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

// 参数数据结构
#[derive(Clone, Default)]
struct Param {
    name: String,
    value: String,
    type_: String,
    required: bool,
    description: String,
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

// 工具函数：解析URL参数到Params结构
fn parse_url_params(url: &str) -> Vec<Param> {
    let mut params = vec![];
    if let Some(query) = url.split('?').nth(1) {
        for pair in query.split('&') {
            let mut kv = pair.split('=');
            if let (Some(name), Some(value)) = (kv.next(), kv.next()) {
                params.push(Param {
                    name: name.to_string(),
                    value: urlencoding::decode(value).unwrap_or_default().to_string(),
                    type_: "string".to_string(),
                    required: false,
                    description: "".to_string(),
                });
            }
        }
    }
    params
}

// 工具函数：从Params生成URL查询字符串
fn build_query_string(params: &[Param]) -> String {
    params
        .iter()
        .filter(|p| !p.name.is_empty())
        .map(|p| {
            format!(
                "{}={}",
                urlencoding::encode(&p.name),
                urlencoding::encode(&p.value)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
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
