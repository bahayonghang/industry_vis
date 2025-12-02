mod commands;
mod config;
mod datasource;
mod error;
mod models;
mod tag_group;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
