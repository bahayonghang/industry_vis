//! 缓存预热模块
//!
//! 提供缓存预热功能，支持手动触发和应用启动时自动预热。

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::cache::{CacheKey, QueryCache};
use crate::error::AppResult;
use crate::models::{DataProcessingConfig, HistoryRecord};

/// 预热任务定义
#[derive(Debug, Clone)]
pub struct WarmupTask {
    /// 表名
    pub table: String,
    /// 开始时间
    pub start_time: String,
    /// 结束时间
    pub end_time: String,
    /// 标签列表
    pub tags: Option<Vec<String>>,
    /// 数据处理配置（用于生成正确的缓存键）
    pub processing_config: Option<DataProcessingConfig>,
    /// 任务描述（用于日志）
    pub description: String,
}

impl WarmupTask {
    /// 创建新的预热任务
    ///
    /// 默认使用 `DataProcessingConfig::default()` 作为处理配置，
    /// 确保预热的缓存键与实际查询时使用默认配置的缓存键匹配。
    pub fn new(
        table: impl Into<String>,
        start_time: impl Into<String>,
        end_time: impl Into<String>,
        tags: Option<Vec<String>>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            table: table.into(),
            start_time: start_time.into(),
            end_time: end_time.into(),
            tags,
            processing_config: Some(DataProcessingConfig::default()),
            description: description.into(),
        }
    }

    /// 创建带自定义处理配置的预热任务
    pub fn with_processing_config(
        table: impl Into<String>,
        start_time: impl Into<String>,
        end_time: impl Into<String>,
        tags: Option<Vec<String>>,
        processing_config: Option<DataProcessingConfig>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            table: table.into(),
            start_time: start_time.into(),
            end_time: end_time.into(),
            tags,
            processing_config,
            description: description.into(),
        }
    }

    /// 生成缓存键
    ///
    /// 使用与实际查询相同的缓存键生成逻辑，确保预热结果可被命中。
    pub fn to_cache_key(&self) -> CacheKey {
        CacheKey::new(
            &self.table,
            &self.start_time,
            &self.end_time,
            self.tags.as_deref(),
            self.processing_config.as_ref(),
        )
    }
}

/// 预热进度信息
#[derive(Debug, Clone)]
pub struct WarmupProgress {
    /// 总任务数
    pub total: usize,
    /// 已完成任务数
    pub completed: usize,
    /// 当前任务描述
    pub current_task: Option<String>,
    /// 是否已完成
    pub is_done: bool,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failure_count: usize,
}

impl WarmupProgress {
    fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            current_task: None,
            is_done: false,
            success_count: 0,
            failure_count: 0,
        }
    }

    fn update(&mut self, task: &str, success: bool) {
        self.completed += 1;
        self.current_task = Some(task.to_string());
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
        if self.completed >= self.total {
            self.is_done = true;
            self.current_task = None;
        }
    }

    /// 获取完成百分比
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }
}

/// 预热执行器
///
/// 负责执行预热任务并报告进度
pub struct CacheWarmer {
    cache: Arc<QueryCache>,
    progress_tx: Option<mpsc::Sender<WarmupProgress>>,
}

impl CacheWarmer {
    /// 创建新的预热执行器
    pub fn new(cache: Arc<QueryCache>) -> Self {
        Self {
            cache,
            progress_tx: None,
        }
    }

    /// 设置进度回调通道
    pub fn with_progress_channel(mut self, tx: mpsc::Sender<WarmupProgress>) -> Self {
        self.progress_tx = Some(tx);
        self
    }

    /// 执行预热任务
    ///
    /// `data_fetcher` 是一个异步函数，用于获取数据
    pub async fn warmup<F, Fut>(
        &self,
        tasks: Vec<WarmupTask>,
        data_fetcher: F,
    ) -> AppResult<WarmupProgress>
    where
        F: Fn(WarmupTask) -> Fut,
        Fut: std::future::Future<Output = AppResult<Vec<HistoryRecord>>>,
    {
        let total = tasks.len();
        let mut progress = WarmupProgress::new(total);

        info!(target: "industry_vis::cache::warmup",
            "开始缓存预热，共 {} 个任务", total);

        for task in tasks {
            let cache_key = task.to_cache_key();
            let description = task.description.clone();

            // 检查是否已缓存
            if self.cache.get(&cache_key).await.is_some() {
                debug!(target: "industry_vis::cache::warmup",
                    "任务已缓存，跳过: {}", description);
                progress.update(&description, true);
                self.send_progress(&progress).await;
                continue;
            }

            // 执行数据获取
            match data_fetcher(task).await {
                Ok(records) => {
                    self.cache.put(cache_key, records).await;
                    debug!(target: "industry_vis::cache::warmup",
                        "预热成功: {}", description);
                    progress.update(&description, true);
                }
                Err(e) => {
                    warn!(target: "industry_vis::cache::warmup",
                        "预热失败: {} - {}", description, e);
                    progress.update(&description, false);
                }
            }

            self.send_progress(&progress).await;
        }

        info!(target: "industry_vis::cache::warmup",
            "缓存预热完成: 成功 {}, 失败 {}",
            progress.success_count, progress.failure_count);

        Ok(progress)
    }

    async fn send_progress(&self, progress: &WarmupProgress) {
        if let Some(tx) = &self.progress_tx {
            let _ = tx.send(progress.clone()).await;
        }
    }
}

/// 预热策略 trait
pub trait WarmupStrategy: Send + Sync {
    /// 生成预热任务列表
    fn generate_tasks(&self) -> Vec<WarmupTask>;

    /// 策略名称
    fn name(&self) -> &str;
}

/// 最近时间范围预热策略
///
/// 预热最近 N 天的数据
pub struct RecentTimeRangeStrategy {
    /// 表名
    pub table: String,
    /// 标签列表
    pub tags: Vec<String>,
    /// 预热天数
    pub days: u32,
}

impl RecentTimeRangeStrategy {
    pub fn new(table: impl Into<String>, tags: Vec<String>, days: u32) -> Self {
        Self {
            table: table.into(),
            tags,
            days,
        }
    }
}

impl WarmupStrategy for RecentTimeRangeStrategy {
    fn generate_tasks(&self) -> Vec<WarmupTask> {
        use chrono::{Duration, Local};

        let now = Local::now();
        let mut tasks = Vec::new();

        // 为每一天生成一个预热任务
        for day_offset in 0..self.days {
            let end = now - Duration::days(day_offset as i64);
            let start = end - Duration::days(1);

            let start_str = start.format("%Y-%m-%dT00:00:00").to_string();
            let end_str = end.format("%Y-%m-%dT00:00:00").to_string();

            tasks.push(WarmupTask::new(
                &self.table,
                &start_str,
                &end_str,
                Some(self.tags.clone()),
                format!(
                    "最近{}天数据 ({})",
                    day_offset + 1,
                    start.format("%Y-%m-%d")
                ),
            ));
        }

        tasks
    }

    fn name(&self) -> &str {
        "RecentTimeRange"
    }
}

/// 固定时间点预热策略
///
/// 预热指定的时间范围
pub struct FixedTimeRangeStrategy {
    tasks: Vec<WarmupTask>,
}

impl FixedTimeRangeStrategy {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn add_task(mut self, task: WarmupTask) -> Self {
        self.tasks.push(task);
        self
    }
}

impl Default for FixedTimeRangeStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl WarmupStrategy for FixedTimeRangeStrategy {
    fn generate_tasks(&self) -> Vec<WarmupTask> {
        self.tasks.clone()
    }

    fn name(&self) -> &str {
        "FixedTimeRange"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_task_creation() {
        let task = WarmupTask::new(
            "历史表",
            "2024-01-01T00:00:00",
            "2024-01-02T00:00:00",
            Some(vec!["Tag1".to_string()]),
            "测试任务",
        );

        assert_eq!(task.table, "历史表");
        assert_eq!(task.description, "测试任务");
    }

    #[test]
    fn test_warmup_progress() {
        let mut progress = WarmupProgress::new(10);
        assert_eq!(progress.percentage(), 0.0);

        progress.update("task1", true);
        assert_eq!(progress.completed, 1);
        assert_eq!(progress.success_count, 1);
        assert_eq!(progress.percentage(), 10.0);

        progress.update("task2", false);
        assert_eq!(progress.completed, 2);
        assert_eq!(progress.failure_count, 1);
    }

    #[test]
    fn test_recent_time_range_strategy() {
        let strategy =
            RecentTimeRangeStrategy::new("历史表", vec!["Tag1".to_string(), "Tag2".to_string()], 3);

        let tasks = strategy.generate_tasks();
        assert_eq!(tasks.len(), 3);
        assert_eq!(strategy.name(), "RecentTimeRange");
    }

    #[test]
    fn test_fixed_time_range_strategy() {
        let strategy = FixedTimeRangeStrategy::new()
            .add_task(WarmupTask::new(
                "历史表",
                "2024-01-01T00:00:00",
                "2024-01-02T00:00:00",
                None,
                "固定任务1",
            ))
            .add_task(WarmupTask::new(
                "历史表",
                "2024-01-02T00:00:00",
                "2024-01-03T00:00:00",
                None,
                "固定任务2",
            ));

        let tasks = strategy.generate_tasks();
        assert_eq!(tasks.len(), 2);
        assert_eq!(strategy.name(), "FixedTimeRange");
    }
}
