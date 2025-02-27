//! Database infrastructure for the podcast crawler.
//!
//! This module provides a connection pool and database context for managing
//! database connections. It supports:
//! - Connection pooling
//! - Async connections
//! - Error handling with context
//! - Connection management

use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::pooled_connection::{bb8::Pool, AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::AsyncPgConnection;

use crate::infrastructure::config::DatabaseConfig;
use crate::infrastructure::Settings;
use crate::infrastructure::{
    error::infrastructure::{InfrastructureError, InfrastructureErrorKind},
    AppError, AppResult,
};
use futures::future::BoxFuture;
use futures::FutureExt;
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection<'a> = PooledConnection<'a, AsyncPgConnection>;

/// Database context for managing database connections
#[derive(Debug, Clone)]
pub struct DatabaseContext {
    pool: DbPool,
}

impl DatabaseContext {
    /// Creates a new `DatabaseContext` with the provided configuration
    pub async fn new_with_config(config: &DatabaseConfig) -> AppResult<Self> {
        let manager = if config.no_ssl {
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(config.url.clone())
        } else {
            let mut mgr_config = ManagerConfig::default();
            mgr_config.custom_setup = Box::new(establish_connection);
            AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
                config.url.clone(),
                mgr_config,
            )
        };

        let mut builder = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_connections))
            .connection_timeout(std::time::Duration::from_secs(
                config.connect_timeout_seconds,
            ));

        builder = builder.idle_timeout(Some(std::time::Duration::from_secs(
            config.idle_timeout_seconds,
        )));

        let pool = builder.build(manager).await.map_err(|e| {
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Database,
                format!("Failed to create database pool: {}", e),
                Some(Box::new(e)),
            ))
        })?;

        Ok(Self { pool })
    }

    /// Creates a new `DatabaseContext` with default configuration
    pub async fn new() -> AppResult<Self> {
        let settings = Settings::new()?;
        Self::new_with_config(&settings.database).await
    }

    /// Gets a connection from the pool
    pub async fn get_connection(&self) -> AppResult<DbConnection<'_>> {
        self.pool.get().await.map_err(|e| {
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Database,
                format!("Failed to get database connection: {}", e),
                Some(Box::new(e)),
            ))
        })
    }

    /// Gets the underlying connection pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // Create a new connection to the database
        // Setup the TLS configuration for the connection using native certs
        // Using https://crates.io/crates/rustls-platform-verifier
        // replaces using rustls-native-certs on its own (recommended)
        let tls_config = ClientConfig::with_platform_verifier();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(tls_config);

        // get the client and connection future
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;

        AsyncPgConnection::try_from_client_and_connection(client, conn).await
    };
    fut.boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio;

    async fn setup() -> Settings {
        Settings::new().unwrap()
    }

    #[tokio::test]
    async fn test_database_context_creation() {
        let config = setup().await;
        let db_context = DatabaseContext::new_with_config(&config.database).await;
        assert!(db_context.is_ok(), "Failed to create DatabaseContext");

        if let Ok(ctx) = db_context {
            let conn_result = ctx.get_connection().await;
            assert!(
                conn_result.is_ok(),
                "Failed to acquire a connection from the pool"
            );
        }
    }

    #[tokio::test]
    async fn test_multiple_connections() {
        let config = setup().await;
        let db_context = DatabaseContext::new_with_config(&config.database)
            .await
            .expect("Failed to create DatabaseContext");

        let db_context = Arc::new(db_context);
        let mut handles = vec![];

        for _ in 0..10 {
            let ctx_clone = db_context.clone();
            let handle = tokio::spawn(async move {
                let conn_result = ctx_clone.get_connection().await;
                assert!(
                    conn_result.is_ok(),
                    "Failed to acquire connection in concurrent test"
                );
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_connection_error_handling() {
        let mut config = setup().await;
        config.database.url = "postgres://invalid:5432/nonexistent".to_string();

        let result = DatabaseContext::new_with_config(&config.database).await;
        assert!(matches!(result, Err(AppError::Infrastructure(_))));
    }
}
