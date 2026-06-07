// src-tauri/src/commands/window.rs
use tauri::{PhysicalPosition, Runtime, WebviewWindow, AppHandle};
use std::time::Duration;
use tokio::time::sleep;

#[tauri::command]
pub async fn toggle_side_status<R: Runtime>(window: WebviewWindow<R>, is_hide: bool) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_size = monitor.size();
        let window_size = window.outer_size().unwrap();

        let start_x = window.outer_position().unwrap().x;
        let target_x = if is_hide {
            // 隐藏：整个窗口移出屏幕，只露出 30 像素（给标签用）
            screen_size.width as i32 - 30
        } else {
            // 展开：完全显示
            screen_size.width as i32 - window_size.width as i32
        };

        let y = (screen_size.height as i32 - window_size.height as i32) / 2;

        // 快速平滑滑动：分 15 帧，每帧 10ms（约 150ms 总时长）
        let frames = 15;
        let duration_per_frame = 10;

        for i in 1..=frames {
            // 使用 ease-out 缓动函数
            let progress = i as f64 / frames as f64;
            let eased_progress = 1.0 - (1.0 - progress).powi(3);

            let current_x = start_x + ((target_x - start_x) as f64 * eased_progress) as i32;
            let _ = window.set_position(PhysicalPosition::new(current_x, y));

            sleep(Duration::from_millis(duration_per_frame)).await;
        }

        // 确保最终到达目标位置
        let _ = window.set_position(PhysicalPosition::new(target_x, y));
    }
}

#[tauri::command]
pub fn exit_app<R: Runtime>(app: AppHandle<R>) {
    app.exit(0);
}
