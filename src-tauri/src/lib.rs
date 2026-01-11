//! Industry Vis 后端库
//!
//! 工业数据可视化系统的 Tauri 后端。
//!
//! ## 模块结构
//! - `cache` - 查询结果缓存
//! - `commands` - Tauri IPC 命令
//! - `config` - 配置管理（支持热更新）
//! - `datasource` - 数据源访问（bb8 连接池）
//! - `error` - 错误类型
//! - `logging` - 日志系统
//! - `models` - 数据模型
//! - `processing` - 数据处理
//! - `services` - 业务逻辑层
//! - `state` - 应用状态管理

pub mod cache;
pub mod commands;
pub mod config;
pub mod datasource;
pub mod error;
pub mod logging;
pub mod models;
pub mod processing;
pub mod services;
pub mod state;

use commands::*;
use state::AppState;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tauri::{async_runtime, Manager, RunEvent, Url};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::RwLock;
use tracing::info;

fn check_dev_server_status(dev_url: &Url) -> Option<String> {
    let host = dev_url.host_str()?;
    let port = dev_url.port_or_known_default()?;
    let addrs = (host, port).to_socket_addrs().ok()?;
    let mut stream = None;

    for addr in addrs {
        if let Ok(s) = TcpStream::connect_timeout(&addr, Duration::from_millis(800)) {
            stream = Some(s);
            break;
        }
    }

    let mut stream = match stream {
        Some(s) => s,
        None => {
            return Some(format!(
                "开发端口 {} 无法访问，可能被占用或开发服务器未启动。\n请确认 Vite 已启动且端口一致。",
                port
            ))
        }
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(800)));

    let path = if dev_url.path().is_empty() { "/" } else { dev_url.path() };
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return Some(format!(
            "无法与开发端口 {} 建立有效通信，可能被占用或开发服务器未启动。",
            port
        ));
    }

    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).unwrap_or(0);
    let text = String::from_utf8_lossy(&buf[..n]);
    if !text.contains("@vite/client") {
        return Some(format!(
            "开发端口 {} 已响应但不是 Vite 开发服务器，可能被占用。\n请检查 Vite 输出确认实际端口。",
            port
        ));
    }

    None
}

/// 应用入口
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    let _log_guards = match logging::init_logging() {
        Ok(guards) => Some(guards),
        Err(e) => {
            eprintln!("日志初始化失败: {}", e);
            None
        }
    };

    // 使用 tokio 运行时初始化异步状态
    let app_state = async_runtime::block_on(async {
        match AppState::new().await {
            Ok(state) => Arc::new(RwLock::new(state)),
            Err(e) => {
                tracing::error!(target: "industry_vis::lib", "初始化应用状态失败: {}", e);
                panic!("无法初始化应用状态: {}", e);
            }
        }
    });

    // 克隆一份状态用于后台初始化连接池
    let app_state_for_pool = app_state.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 开发环境检查 devUrl 端口占用情况并提示
            if cfg!(debug_assertions) {
                if let Some(dev_url) = app.config().build.dev_url.clone() {
                    let handle = app.handle().clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1200));
                        if let Some(message) = check_dev_server_status(&dev_url) {
                            handle
                                .dialog()
                                .message(message)
                                .kind(MessageDialogKind::Warning)
                                .buttons(MessageDialogButtons::Ok)
                                .show(|_| {});
                        }
                    });
                }
            }
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // 配置相关
            load_config,
            save_config,
            test_connection,
            get_connection_status,
            // 数据查询
            get_available_tags,
            search_tags,
            query_history,
            query_history_v2,
            export_to_csv,
            // 缓存管理
            clear_cache,
            get_cache_stats,
            // 标签分组
            list_tag_groups,
            create_tag_group,
            update_tag_group,
            delete_tag_group,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                info!(target: "industry_vis::lib", "窗口关闭，准备退出应用");
                window.app_handle().exit(0);
            }
        })
        .build(tauri::generate_context!())
        .expect("构建 Tauri 应用失败");

    // 在后台尝试初始化连接池（失败不影响应用启动）
    async_runtime::spawn(async move {
        let mut state = app_state_for_pool.write().await;
        if state.is_pool_initialized() {
            return;
        }
        if let Err(e) = state.init_pool().await {
            tracing::warn!(target: "industry_vis::lib", "后台初始化连接池失败: {}", e);
        } else {
            tracing::info!(target: "industry_vis::lib", "后台初始化连接池成功");
        }
    });

    app.run(|_app_handle, event| {
        if let RunEvent::Exit = event {
            info!(target: "industry_vis::lib", "应用正在退出，清理资源...");
        }
    });
}
