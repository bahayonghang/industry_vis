//! Schema Profile 实现模块
//!
//! 包含各厂商的 Profile 实现和 Profile 注册表。

mod default;
mod registry;

pub use default::DefaultProfile;
pub use registry::ProfileRegistry;
