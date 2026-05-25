use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StudioPing {
    status: &'static str,
    message: &'static str,
}

#[tauri::command]
fn ping_studio() -> StudioPing {
    StudioPing {
        status: "ok",
        message: "v6 studio shell ready",
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping_studio])
        .run(tauri::generate_context!())
        .expect("failed to run v6 Tauri Research Studio");
}
