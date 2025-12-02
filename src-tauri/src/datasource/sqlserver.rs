use async_trait::async_trait;
use tiberius::{Client, Config, AuthMethod, Query};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use tracing::{debug, error, info};

use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};
use crate::models::HistoryRecord;
use super::traits::{DataSource, SourceMetadata, TableInfo};

/// SQL Server 数据源实现
#[allow(dead_code)]
pub struct SqlServerSource {
    config: DatabaseConfig,
    metadata: SourceMetadata,
}

impl SqlServerSource {
    pub fn new(config: DatabaseConfig) -> Self {
        let metadata = SourceMetadata {
            name: format!("{}:{}", config.server, config.port),
            database: config.database.clone(),
        };
        Self { config, metadata }
    }

    async fn connect(&self) -> AppResult<Client<tokio_util::compat::Compat<TcpStream>>> {
        let mut tiberius_config = Config::new();
        tiberius_config.host(&self.config.server);
        tiberius_config.port(self.config.port);
        tiberius_config.database(&self.config.database);
        tiberius_config.authentication(AuthMethod::sql_server(
            &self.config.username,
            &self.config.password,
        ));
        tiberius_config.trust_cert();

        let tcp = TcpStream::connect(tiberius_config.get_addr())
            .await
            .map_err(|e| AppError::Connection(format!("TCP connection failed: {}", e)))?;

        tcp.set_nodelay(true)
            .map_err(|e| AppError::Connection(format!("Failed to set TCP_NODELAY: {}", e)))?;

        let client = Client::connect(tiberius_config, tcp.compat_write())
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                // 提供更友好的中文错误提示
                if err_str.contains("4060") {
                    AppError::Connection(format!(
                        "数据库 '{}' 不存在或无访问权限。请检查数据库名称是否正确。原始错误: {}",
                        self.config.database, err_str
                    ))
                } else if err_str.contains("18456") {
                    AppError::Connection(format!(
                        "用户名或密码错误。原始错误: {}", err_str
                    ))
                } else if err_str.contains("Login failed") {
                    AppError::Connection(format!(
                        "登录失败，请检查用户名和密码。原始错误: {}", err_str
                    ))
                } else {
                    AppError::Connection(format!("SQL Server 连接失败: {}", err_str))
                }
            })?;

        Ok(client)
    }
}

#[async_trait]
impl DataSource for SqlServerSource {
    async fn test_connection(&self) -> AppResult<()> {
        let mut client = self.connect().await?;
        
        // Execute a simple query to verify connection
        let query = Query::new("SELECT 1");
        query.execute(&mut client)
            .await
            .map_err(|e| AppError::Connection(format!("Test query failed: {}", e)))?;
        
        Ok(())
    }

    fn metadata(&self) -> &SourceMetadata {
        &self.metadata
    }

    async fn list_tables(&self) -> AppResult<Vec<TableInfo>> {
        let mut client = self.connect().await?;
        
        let query = Query::new(
            "SELECT TABLE_SCHEMA, TABLE_NAME FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_TYPE = 'BASE TABLE'"
        );
        
        let stream = query.query(&mut client)
            .await
            .map_err(|e| AppError::Query(format!("Failed to list tables: {}", e)))?;
        
        let rows = stream.into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch table results: {}", e)))?;
        
        let tables = rows.iter().map(|row| {
            TableInfo {
                schema: row.get::<&str, _>(0).unwrap_or("dbo").to_string(),
                name: row.get::<&str, _>(1).unwrap_or("").to_string(),
            }
        }).collect();
        
        Ok(tables)
    }

    async fn get_available_tags(&self, table: &str) -> AppResult<Vec<String>> {
        let mut client = self.connect().await?;
        
        let sql = format!(
            "SELECT DISTINCT TagName FROM [{}] ORDER BY TagName",
            table.replace(']', "]]")
        );
        
        let query = Query::new(&sql);
        let stream = query.query(&mut client)
            .await
            .map_err(|e| AppError::Query(format!("Failed to get tags: {}", e)))?;
        
        let rows = stream.into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch tag results: {}", e)))?;
        
        let tags = rows.iter()
            .filter_map(|row| row.get::<&str, _>(0).map(|s| s.trim().to_string()))
            .collect();
        
        Ok(tags)
    }

    async fn search_tags(&self, keyword: &str, limit: usize) -> AppResult<Vec<String>> {
        let mut client = self.connect().await?;
        
        // 从 TagDataBase 表模糊搜索 TagName
        // 使用参数化查询避免 SQL 注入和特殊字符问题
        let search_pattern = format!("%{}%", keyword);
        
        let sql = format!(
            r#"SELECT DISTINCT TOP {} TagName 
               FROM [TagDataBase] 
               WHERE TagName LIKE @P1
               ORDER BY TagName"#,
            limit
        );
        
        debug!(target: "industry_vis_lib::datasource", 
            database = %self.config.database,
            keyword = %keyword,
            pattern = %search_pattern,
            "执行标签搜索 SQL: {}", sql
        );
        
        let mut query = Query::new(&sql);
        query.bind(&search_pattern);
        
        let stream = query.query(&mut client)
            .await
            .map_err(|e| {
                error!(target: "industry_vis_lib::datasource",
                    database = %self.config.database,
                    keyword = %keyword,
                    error = %e,
                    "标签搜索失败"
                );
                AppError::Query(format!(
                    "搜索标签失败 (数据库: {}, 关键词: {}): {}", 
                    self.config.database, keyword, e
                ))
            })?;
        
        let rows = stream.into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("获取搜索结果失败: {}", e)))?;
        
        let tags: Vec<String> = rows.iter()
            .filter_map(|row| row.get::<&str, _>(0).map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect();
        
        info!(target: "industry_vis_lib::datasource",
            database = %self.config.database,
            keyword = %keyword,
            count = tags.len(),
            "标签搜索完成"
        );
        
        Ok(tags)
    }

    async fn query_history(
        &self,
        table: &str,
        start_time: &str,
        end_time: &str,
        tags: Option<&[String]>,
    ) -> AppResult<Vec<HistoryRecord>> {
        let mut client = self.connect().await?;
        
        let tag_count = tags.map(|t| t.len()).unwrap_or(0);
        let tag_filter = match tags {
            Some(t) if !t.is_empty() => {
                let tag_list = t.iter()
                    .map(|s| format!("'{}'", s.replace('\'', "''")))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("AND TagName IN ({})", tag_list)
            }
            _ => String::new(),
        };
        
        let sql = format!(
            r#"
            SELECT DateTime, TagName, TagVal, TagQuality 
            FROM [{}]
            WHERE DateTime >= '{}' AND DateTime <= '{}'
            {}
            ORDER BY DateTime, TagName
            "#,
            table.replace(']', "]]"),
            start_time.replace('\'', "''"),
            end_time.replace('\'', "''"),
            tag_filter
        );
        
        debug!(target: "industry_vis_lib::datasource",
            database = %self.config.database,
            table = %table,
            start_time = %start_time,
            end_time = %end_time,
            tag_count = tag_count,
            "执行历史查询 SQL: {}", sql.replace('\n', " ").replace("  ", " ")
        );
        
        let query = Query::new(&sql);
        let stream = query.query(&mut client)
            .await
            .map_err(|e| {
                error!(target: "industry_vis_lib::datasource",
                    database = %self.config.database,
                    error = %e,
                    "历史查询失败"
                );
                AppError::Query(format!("Failed to query history: {}", e))
            })?;
        
        let rows = stream.into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch history results: {}", e)))?;
        
        let records: Vec<HistoryRecord> = rows.iter().map(|row| {
            let dt: Option<chrono::NaiveDateTime> = row.get(0);
            let date_time = dt
                .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3f").to_string())
                .unwrap_or_default();
            
            HistoryRecord {
                date_time,
                tag_name: row.get::<&str, _>(1).unwrap_or("").trim().to_string(),
                tag_val: row.get::<f32, _>(2).unwrap_or(0.0) as f64,
                tag_quality: row.get::<&str, _>(3).unwrap_or("").trim().to_string(),
            }
        }).collect();
        
        info!(target: "industry_vis_lib::datasource",
            database = %self.config.database,
            table = %table,
            records = records.len(),
            "历史查询完成"
        );
        
        Ok(records)
    }
}
