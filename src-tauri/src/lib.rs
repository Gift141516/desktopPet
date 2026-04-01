use tauri::Manager;

// 定义一个命令供前端调用，控制鼠标是否穿透窗口
#[tauri::command]
fn set_ignore_mouse(window: tauri::Window, ignore: bool) {
    let _ = window.set_ignore_cursor_events(ignore);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // 注册命令，让 JS 可以调用 Rust 方法
        .invoke_handler(tauri::generate_handler![set_ignore_mouse])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // 初始设置：置顶
            window.set_always_on_top(true).unwrap();
//
//             // 初始状态：不穿透（方便调试），后续由 Vue 动态控制
//             let _ = window.set_ignore_cursor_events(false);
//         window.set_resizable(false).unwrap();
//
//         window.set_always_on_top(true).unwrap();
// 强制设为不可缩放（Windows 会自动去掉厚重的系统阴影）
    window.set_resizable(false).unwrap();

    // 确保 decorations 为 false
    // 这通常已经在 tauri.conf.json 配置文件中设置过了
    window.set_decorations(false).unwrap();

    // 初始状态：不穿透，始终置顶
    window.set_always_on_top(true).unwrap();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}