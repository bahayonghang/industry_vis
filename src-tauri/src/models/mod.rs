//! 数据模型模块
//!
//! 包含所有纯数据结构定义，不包含业务逻辑。

mod history;
mod processing;
mod query;
mod tag_group;

pub use history::HistoryRecord;
pub use processing::{DataProcessingConfig, OutlierRemovalConfig, ResampleConfig, SmoothingConfig};
pub use query::{ChartSeriesData, ConnectionTestResult, QueryParams, QueryResult, QueryResultV2};
pub use tag_group::{ChartConfig, TagGroup, TagGroupConfig};
