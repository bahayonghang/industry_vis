//! 数据源模块
//!
//! 提供数据库访问抽象和连接池管理。

mod pool;
mod sqlserver;
mod traits;

pub use pool::{ConnectionManager, ConnectionPool, PoolConfig};
pub use sqlserver::SqlServerSource;
pub use traits::{DataSource, SourceMetadata, TableInfo};
