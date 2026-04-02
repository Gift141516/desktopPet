// commands.rs
// 导入 Tauri 核心组件：Runtime（运行时环境）、Window（窗口对象）、Menu（菜单对象）
use tauri::{Runtime, Window, menu::Menu};
use tauri::menu::ContextMenu;

// -------------------------------------------------------------------------
// 1. #[tauri::command] 宏：
//    这个标记告诉 Tauri，这个 Rust 函数可以被前端 JS 通过 invoke('函数名') 直接调用。
//    如果没有这个标记，前端是无法访问这个函数的。
// -------------------------------------------------------------------------

/// 在当前鼠标位置显示主菜单
///
/// 参数说明：
/// - window: 当前触发调用的窗口实例。
/// - menu: Tauri 的状态管理器。它会自动寻找我们在 lib.rs 中通过 app.manage(m) 存入的 Menu 对象。
#[tauri::command]
pub fn show_main_menu<R: Runtime>(window: Window<R>, menu: tauri::State<'_, Menu<R>>) {
    // 使用 window.show_menu 在鼠标当前位置弹出菜单。
    // .inner() 用于从 State 容器中取出原始的 Menu 对象，.clone() 是为了满足所有权传递。
//     let _ = window.show_menu(menu.inner().clone());
    let _ = menu.popup(window);
}

// -------------------------------------------------------------------------
// 2. 未来扩展示例：如何添加一个天气查询功能
// -------------------------------------------------------------------------

/*
// 步骤 1: 在这里写一个新的函数并加上 #[tauri::command]
#[tauri::command]
pub fn get_weather(city: String) -> String {
    println!("收到前端请求，查询城市: {}", city);
    // 这里可以接入实际的天气 API 逻辑
    format!("{} 的天气：晴朗，25度", city)
}

// 步骤 2: 去 lib.rs 的 invoke_handler 中注册这个新函数：
// .invoke_handler(tauri::generate_handler![
//     set_ignore_mouse,
//     commands::show_main_menu,
//     commands::get_weather  <-- 加上这一行
// ])
*/

// 提示：
// - 函数必须是 pub (public) 的，lib.rs 才能看见它。
// - 返回值会自动序列化为 JSON，方便前端 JS 直接接收。