mod commands;
mod config;
mod data_processing;
mod datasource;
mod error;
pub mod logging;
mod models;
mod tag_group;

use commands::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统
    if let Err(e) = logging::init_logging() {
        eprintln!("日志初始化失败: {}", e);
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // 配置相关
            load_config,
            save_config,
            test_connection,
            // 数据查询
            get_available_tags,
            query_history,
            export_to_csv,
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
                window.app_handle().exit(0);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
