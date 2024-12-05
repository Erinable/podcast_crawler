use diesel_async::pooled_connection::{bb8::Pool, AsyncDieselConnectionManager};
use diesel_async::AsyncPgConnection;
use dotenv::dotenv;
use std::env;

use diesel_async::pooled_connection::bb8::PooledConnection;
use crate::infrastructure::{AppError, AppResult,AppResultExt};

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncPgConnection>;

#[derive(Clone)]
pub struct DatabaseContext {
    pool: DbPool,
}

impl DatabaseContext {
    /// 创建新的 `DatabaseContext`
    pub async fn new() -> AppResult<Self> {
        // 加载环境变量
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        // 创建连接池管理器
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

        // 配置连接池，设置最大连接数
        let pool = Pool::builder()
            .max_size(16) // 设置最大连接数为 16，可以根据你的需求调整
            .build(manager)
            .await
            .map_err(|e| AppError::database(format!("Failed to create DB pool: {}", e)))?;

        Ok(Self { pool })
    }

    /// 获取数据库连接
    pub async fn get_connection(&self) -> AppResult<DbConnection<'_>> {
        let conn = self.pool.get().await.error_info("Failed to get database connection")?;
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;
    use tokio;

    /// 测试数据库上下文的创建
    #[tokio::test]
    async fn test_database_context_creation() {
        // 创建数据库上下文
        let db_context = DatabaseContext::new().await;
        assert!(db_context.is_ok(), "Failed to create DatabaseContext");

        if let Ok(ctx) = db_context {
            // 获取一个连接
            let conn_result = ctx.get_connection().await;
            assert!(conn_result.is_ok(), "Failed to acquire a connection from the pool");
        }
    }

    /// 测试多个连接获取
    #[tokio::test]
    async fn test_multiple_connections() {
        let db_context = DatabaseContext::new().await.expect("Failed to create DatabaseContext");
        let db_context = Arc::new(db_context);
        let mut handles = vec![];

        // 模拟多个异步任务获取连接
        for _ in 0..10 {
            let ctx_clone = db_context.clone();
            let handle = tokio::spawn(async move {
                let conn_result = ctx_clone.get_connection().await;
                assert!(conn_result.is_ok(), "Failed to acquire connection in concurrent test");
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }
    }
}
