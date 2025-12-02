use async_trait::async_trait;
use tiberius::{Client, Config, AuthMethod, Query};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

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
            .map_err(|e| AppError::Connection(format!("SQL Server connection failed: {}", e)))?;

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
        
        // 从 TagDatabase 表模糊搜索 TagName
        let sql = format!(
            r#"
            SELECT DISTINCT TOP {} TagName 
            FROM [TagDatabase] 
            WHERE TagName LIKE '%{}%'
            ORDER BY TagName
            "#,
            limit,
            keyword.replace('\'', "''").replace('%', "[%]").replace('_', "[_]")
        );
        
        let query = Query::new(&sql);
        let stream = query.query(&mut client)
            .await
            .map_err(|e| AppError::Query(format!("搜索标签失败: {}", e)))?;
        
        let rows = stream.into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("获取搜索结果失败: {}", e)))?;
        
        let tags = rows.iter()
            .filter_map(|row| row.get::<&str, _>(0).map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect();
        
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
        
        let query = Query::new(&sql);
        let stream = query.query(&mut client)
            .await
            .map_err(|e| AppError::Query(format!("Failed to query history: {}", e)))?;
        
        let rows = stream.into_first_result()
            .await
            .map_err(|e| AppError::Query(format!("Failed to fetch history results: {}", e)))?;
        
        let records = rows.iter().map(|row| {
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
        
        Ok(records)
    }
}
