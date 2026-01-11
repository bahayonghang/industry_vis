//! 数据源模块
//!
//! 提供数据库访问抽象和连接池管理。

mod pool;
mod profiles;
mod schema_profile;
mod sqlserver;
mod traits;

pub use pool::{ConnectionManager, ConnectionPool, PoolConfig, PoolState};
pub use profiles::{DefaultProfile, ProfileRegistry};
pub use schema_profile::SchemaProfile;
pub use sqlserver::SqlServerSource;
pub use traits::{DataSource, SourceMetadata, TableInfo};
