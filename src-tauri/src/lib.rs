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
use std::sync::Arc;
use tauri::{async_runtime, Manager, RunEvent};
use tokio::sync::RwLock;
use tracing::info;

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
            Ok(mut state) => {
                // 尝试初始化连接池（失败不影响应用启动）
                if let Err(e) = state.init_pool().await {
                    tracing::warn!(target: "industry_vis::lib", "初始化连接池失败: {}", e);
                }
                Arc::new(RwLock::new(state))
            }
            Err(e) => {
                tracing::error!(target: "industry_vis::lib", "初始化应用状态失败: {}", e);
                panic!("无法初始化应用状态: {}", e);
            }
        }
    });

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // 配置相关
            load_config,
            save_config,
            test_connection,
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

    app.run(|_app_handle, event| {
        if let RunEvent::Exit = event {
            info!(target: "industry_vis::lib", "应用正在退出，清理资源...");
        }
    });
}
