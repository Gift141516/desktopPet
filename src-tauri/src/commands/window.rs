// src-tauri/src/commands/window.rs
use tauri::{PhysicalPosition, Runtime, WebviewWindow, AppHandle};
use std::time::Duration;
use tokio::time::sleep;

#[tauri::command]
pub async fn toggle_side_status<R: Runtime>(
    window: WebviewWindow<R>,
    is_hide: bool,
    edge: Option<String>, // "left" | "right" | "top" | "bottom"
) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_size = monitor.size();
        let window_size = window.outer_size().unwrap();
        let current_pos = window.outer_position().unwrap();

        let edge = edge.as_deref().unwrap_or("right"); // 默认右边
        let tab_size = 30; // 标签露出的尺寸

        let (start_x, start_y) = (current_pos.x, current_pos.y);
        let (target_x, target_y) = if is_hide {
            // 隐藏：根据边缘方向移出屏幕
            match edge {
                "left" => (-(window_size.width as i32) + tab_size, current_pos.y),
                "right" => (screen_size.width as i32 - tab_size, current_pos.y),
                "top" => (current_pos.x, -(window_size.height as i32) + tab_size),
                "bottom" => (current_pos.x, screen_size.height as i32 - tab_size),
                _ => (screen_size.width as i32 - tab_size, current_pos.y), // 默认右边
            }
        } else {
            // 展开：根据边缘方向完全显示
            match edge {
                "left" => (0, current_pos.y),
                "right" => (screen_size.width as i32 - window_size.width as i32, current_pos.y),
                "top" => (current_pos.x, 0),
                "bottom" => (current_pos.x, screen_size.height as i32 - window_size.height as i32),
                _ => (screen_size.width as i32 - window_size.width as i32, current_pos.y),
            }
        };

        // 快速平滑滑动：分 15 帧，每帧 10ms（约 150ms 总时长）
        let frames = 15;
        let duration_per_frame = 10;

        for i in 1..=frames {
            // 使用 ease-out 缓动函数
            let progress = i as f64 / frames as f64;
            let eased_progress = 1.0 - (1.0 - progress).powi(3);

            let current_x = start_x + ((target_x - start_x) as f64 * eased_progress) as i32;
            let current_y = start_y + ((target_y - start_y) as f64 * eased_progress) as i32;
            let _ = window.set_position(PhysicalPosition::new(current_x, current_y));

            sleep(Duration::from_millis(duration_per_frame)).await;
        }

        // 确保最终到达目标位置
        let _ = window.set_position(PhysicalPosition::new(target_x, target_y));
    }
}

#[tauri::command]
pub fn exit_app<R: Runtime>(app: AppHandle<R>) {
    app.exit(0);
}
