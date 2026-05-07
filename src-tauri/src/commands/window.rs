// src-tauri/src/commands/window.rs
use tauri::{PhysicalPosition, Runtime, WebviewWindow};

#[tauri::command]
pub async fn toggle_side_status<R: Runtime>(window: WebviewWindow<R>, is_hide: bool) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_size = monitor.size();
        let window_size = window.outer_size().unwrap();

        let x = if is_hide {
            // 隐藏：只露 20 像素
            screen_size.width as i32 - 20
        } else {
            // 展开：完全显示
            screen_size.width as i32 - window_size.width as i32
        };

        let y = (screen_size.height as i32 - window_size.height as i32) / 2;
        window.set_position(PhysicalPosition::new(x, y)).unwrap();
    }
}
