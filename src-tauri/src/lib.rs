mod cache;
mod commands;
mod config;
mod connection_pool;
mod data_processing;
mod datasource;
mod error;
pub mod logging;
mod models;
mod tag_group;

use cache::QueryCache;
use commands::*;
use std::sync::Arc;
use tauri::{Manager, RunEvent};
use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统，保持 guard 直到应用退出
    let _log_guards = match logging::init_logging() {
        Ok(guards) => Some(guards),
        Err(e) => {
            eprintln!("日志初始化失败: {}", e);
            None
        }
    };
    
    // 初始化查询缓存
    let query_cache = Arc::new(QueryCache::with_defaults());
    
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(query_cache)
        .invoke_handler(tauri::generate_handler![
            // 配置相关
            load_config,
            save_config,
            test_connection,
            // 数据查询
            get_available_tags,
            query_history,
            query_history_v2,
            export_to_csv,
            // 缓存管理
            clear_cache,
            get_cache_stats,
            // 标签分组
            search_tags,
            list_tag_groups,
            create_tag_group,
            update_tag_group,
            delete_tag_group,
        ])
        .on_window_event(|window, event| {
            // 当窗口关闭时，退出整个应用
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                info!(target: "industry_vis_lib::commands", "窗口关闭，准备退出应用");
                window.app_handle().exit(0);
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    
    // 使用 run_iteration 模式处理退出事件，确保资源正确释放
    app.run(|_app_handle, event| {
        if let RunEvent::Exit = event {
            info!(target: "industry_vis_lib::commands", "应用正在退出，清理资源...");
            // LogGuards 会在这里被 drop，确保日志刷新
        }
    });
    
    // 应用已退出，日志 guards 会在此处 drop
}
