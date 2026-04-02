// menu.rs
// 导入 Tauri 菜单相关的核心组件
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    Runtime,
    Emitter
};

/// 创建原生上下文菜单
/// 这里的 <R: Runtime> 是 Tauri 的泛型写法，确保该函数可以兼容所有平台（Windows/macOS/Linux）
pub fn create_menu<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    // 1. 创建单个菜单项
    // 参数说明：(handle, ID, 显示文本, 是否启用, 快捷键)
    // ID ("weather") 非常重要，它决定了用户点击时我们能收到什么标识
    let weather = MenuItem::with_id(app, "weather", "今天天气怎么样？", true, None::<&str>)?;
    let chat = MenuItem::with_id(app, "chat", "陪我聊聊天", true, None::<&str>)?;

    // 2. 创建一个分割线（让菜单看起来更整齐）
    let sep = PredefinedMenuItem::separator(app)?;

    // 3. 创建退出菜单
    let quit = MenuItem::with_id(app, "quit", "退出宠物", true, None::<&str>)?;

    // 4. 将所有创建好的菜单项组合成一个完整的菜单并返回
    // 注意：这里的数组顺序就是菜单显示的上下顺序
    Menu::with_items(app, &[&weather, &chat, &sep, &quit])
}

/// 处理菜单点击事件
/// 当用户点击菜单中的某一项时，lib.rs 会调用这个函数进行分发
pub fn handle_menu_event<R: Runtime>(app: &tauri::AppHandle<R>, event: tauri::menu::MenuEvent) {
    // 根据事件的 ID（即上面 MenuItem::with_id 定义的 ID）进行匹配
    match event.id.as_ref() {
        "weather" => {
            // 向前端发送全局事件 'menu-action'，负载数据为 'weather'
            // 前端 JS 可以通过 listen('menu-action', (e) => { ... }) 监听到这个信号
            app.emit("menu-action", "weather").unwrap();
        }
        "chat" => {
            // 通知前端开启聊天对话框或执行聊天逻辑
            app.emit("menu-action", "chat").unwrap();
        }
        "quit" => {
            // 直接退出程序进程
            std::process::exit(0);
        }
        _ => {
            // 匹配未定义的其他 ID，通常留空即可
        }
    }
}