// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn do_request(method: String, url: String) -> Result<String, String> {
    println!("method: {method}");

    let client = reqwest::Client::new();

    let response = match method.as_str() {
        "GET" => client.get(&url).send().await,
        "POST" => client.post(&url).send().await,
        _ => return Err("Unsupported method".into()),
    };

    match response {
        Ok(res) => res.text().await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![do_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
