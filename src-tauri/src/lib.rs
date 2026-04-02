// 1. 声明模块：编译器会去寻找同名的 menu.rs 和 commands.rs 文件
mod menu;
mod commands;

use tauri::Manager;

// 定义基础命令：控制窗口鼠标穿透
// 提示：如果这个命令以后变多，也可以挪到 commands.rs 里
#[tauri::command]
fn set_ignore_mouse(window: tauri::Window, ignore: bool) {
    let _ = window.set_ignore_cursor_events(ignore);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // -------------------------------------------------------------------------
        // 2. 注册命令：所有在 commands.rs 中标记了 #[tauri::command] 且想让 JS 调用的函数，
        // 都必须在这里通过 commands::函数名 的形式进行注册。
        // -------------------------------------------------------------------------
        .invoke_handler(tauri::generate_handler![
            set_ignore_mouse,
            commands::show_main_menu // 注册弹出菜单的指令
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // --- 窗口基础配置 ---
            window.set_always_on_top(true).unwrap(); // 始终置顶
            window.set_resizable(false).unwrap();    // 禁用缩放（解决 Windows 阴影残留关键）
            window.set_decorations(false).unwrap();  // 去掉系统边框

            // -------------------------------------------------------------------------
            // 3. 菜单初始化逻辑：使用 menu.rs 模块
            // -------------------------------------------------------------------------
            // 调用 menu.rs 里的 create_menu 函数创建菜单对象
            let m = menu::create_menu(app.handle())?;

            // 将菜单对象存入 Tauri 的状态管理器 (State)，
            // 这样 commands::show_main_menu 才能通过参数拿到它
            app.manage(m);

            // 监听菜单点击事件，并分发给 menu.rs 里的处理函数
            app.on_menu_event(move |app_handle, event| {
                menu::handle_menu_event(app_handle, event);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}