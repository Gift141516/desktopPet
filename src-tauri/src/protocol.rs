// src-tauri/src/protocol.rs
use std::fs::File;
use std::io::Read;
use tauri::http::{header::CONTENT_TYPE, Response};
use tauri::{Manager, Runtime};

pub fn register_stream_protocol<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.register_uri_scheme_protocol("stream", |ctx, request| {
        // 1. 通过 ctx.app_handle() 拿到句柄
        let app_handle = ctx.app_handle();

        // 2. 获取请求路径
        let path = request.uri().path().trim_start_matches('/');
        // 3. 获取缓存目录
        let cache_dir = app_handle
            .path()
            .app_cache_dir()
            .expect("failed to get cache dir");
        let file_path = cache_dir.join(path);

        // 4. 读取并返回数据
        if let Ok(mut file) = File::open(&file_path) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                return Response::builder()
                    .header(CONTENT_TYPE, "audio/wav")
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "GET, OPTIONS")
                    .header("Access-Control-Allow-Headers", "*")
                    .body(buffer)
                    .unwrap();
            }
        }
        Response::builder().status(404).body(Vec::new()).unwrap()
    })
}
