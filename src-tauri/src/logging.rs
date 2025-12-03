//! 日志模块
//!
//! 功能：
//! - 前台操作日志 (app.log)
//! - SQL 查询日志 (sql.log)
//! - 按日期轮转，保留14天

use std::path::PathBuf;
use std::fs;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

/// 获取日志目录路径
/// 优先使用 exe 同目录的 logs，否则使用 AppData
pub fn get_log_dir() -> PathBuf {
    // 优先使用便携模式（exe 同目录）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_logs = exe_dir.join("logs");
            // 尝试创建目录，如果成功则使用便携模式
            if fs::create_dir_all(&portable_logs).is_ok() {
                return portable_logs;
            }
        }
    }

    // 回退到 AppData 目录
    if let Some(data_dir) = dirs::data_dir() {
        let app_logs = data_dir.join("IndustryVis").join("logs");
        let _ = fs::create_dir_all(&app_logs);
        return app_logs;
    }

    // 最后回退到当前目录
    PathBuf::from("logs")
}

/// 清理超过指定天数的日志文件
fn cleanup_old_logs(log_dir: &PathBuf, prefix: &str, max_days: u32) {
    let now = chrono::Local::now();
    
    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // 检查是否是目标日志文件
                if filename.starts_with(prefix) && filename.ends_with(".log") {
                    // 获取文件修改时间
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            let modified_time: chrono::DateTime<chrono::Local> = modified.into();
                            let age = now.signed_duration_since(modified_time);
                            
                            // 删除超过 max_days 天的日志
                            if age.num_days() > max_days as i64 {
                                let _ = fs::remove_file(&path);
                                println!("[LOG] 清理过期日志: {}", filename);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 日志系统 Guard，持有此结构直到应用退出以确保日志正确刷新
pub struct LogGuards {
    _app_guard: WorkerGuard,
    _sql_guard: WorkerGuard,
}

/// 初始化日志系统
/// 返回 LogGuards，必须持有直到应用退出以确保日志正确刷新
pub fn init_logging() -> Result<LogGuards, Box<dyn std::error::Error>> {
    let log_dir = get_log_dir();
    
    // 清理超过14天的日志
    cleanup_old_logs(&log_dir, "app", 14);
    cleanup_old_logs(&log_dir, "sql", 14);
    
    // 创建前台操作日志 appender（按天轮转）
    let app_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("app")
        .filename_suffix("log")
        .max_log_files(14)
        .build(&log_dir)?;
    
    // 创建 SQL 日志 appender（按天轮转）
    let sql_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("sql")
        .filename_suffix("log")
        .max_log_files(14)
        .build(&log_dir)?;
    
    // 使用 non_blocking 包装，返回 guard 以确保退出时刷新
    let (app_writer, app_guard) = tracing_appender::non_blocking(app_appender);
    let (sql_writer, sql_guard) = tracing_appender::non_blocking(sql_appender);
    
    // 前台日志层
    let app_layer = fmt::layer()
        .with_writer(app_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_filter(EnvFilter::new("industry_vis_lib::commands=info,industry_vis_lib::config=info"));
    
    // SQL 日志层
    let sql_layer = fmt::layer()
        .with_writer(sql_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_filter(EnvFilter::new("industry_vis_lib::datasource=debug"));
    
    // 控制台输出层（开发时使用）
    let console_layer = fmt::layer()
        .with_target(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(EnvFilter::from_default_env().add_directive("industry_vis_lib=debug".parse()?));
    
    // 组合所有日志层
    tracing_subscriber::registry()
        .with(app_layer)
        .with(sql_layer)
        .with(console_layer)
        .init();
    
    tracing::info!(target: "industry_vis_lib::commands", "日志系统初始化完成，日志目录: {}", log_dir.display());
    
    Ok(LogGuards {
        _app_guard: app_guard,
        _sql_guard: sql_guard,
    })
}

/// 记录前台操作
#[macro_export]
macro_rules! log_app {
    ($($arg:tt)*) => {
        tracing::info!(target: "industry_vis_lib::commands", $($arg)*)
    };
}

/// 记录 SQL 查询
#[macro_export]
macro_rules! log_sql {
    ($($arg:tt)*) => {
        tracing::debug!(target: "industry_vis_lib::datasource", $($arg)*)
    };
}

/// 记录 SQL 错误
#[macro_export]
macro_rules! log_sql_error {
    ($($arg:tt)*) => {
        tracing::error!(target: "industry_vis_lib::datasource", $($arg)*)
    };
}
