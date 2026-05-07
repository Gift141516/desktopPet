// 1. 声明模块
mod commands; // 对应你的 commands 文件夹
mod menu;
mod protocol;

use tauri::Manager;

// 基础命令保留
#[tauri::command]
fn set_ignore_mouse(window: tauri::Window, ignore: bool) {
    let _ = window.set_ignore_cursor_events(ignore);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 【关键改动】：先创建一个 builder 实例
    let mut builder = tauri::Builder::default();

    // 【核心修复】：在这里调用 protocol 模块注册流协议
    // 因为你的 protocol::register_stream_protocol 现在接收并返回 builder
    builder = protocol::register_stream_protocol(builder);

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            set_ignore_mouse,
            // 注意这里的路径：commands::模块名::函数名
            commands::tts::show_main_menu,
            commands::tts::generate_tts,
            commands::window::toggle_side_status // 你新写的贴边指令
        ])
        .setup(|app| {
            // --- 菜单初始化 ---
            let m = menu::create_menu(app.handle())?;
            app.manage(m);
            app.on_menu_event(menu::handle_menu_event);

            // --- 窗口基础配置 ---
            if let Some(window) = app.get_webview_window("main") {
                window.set_always_on_top(true).unwrap();
                window.set_resizable(false).unwrap();
                window.set_decorations(false).unwrap();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
