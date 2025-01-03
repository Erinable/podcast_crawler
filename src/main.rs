use std::sync::Arc;

use tracing::info;

use podcast_crawler::crawler_refactor::rss_crawler::RssCrawler;
use podcast_crawler::{
    infrastructure::{initialize, AppResult, AppState},
    metrics, try_with_log,
};

use rand::seq::SliceRandom;
use rand::thread_rng;

async fn init_app() -> AppResult<Arc<AppState>> {
    metrics::init_metrics();
    let state = Arc::new(initialize().await?);
    try_with_log!(state.health_check().await, "Health check completed");

    let mut crawler = RssCrawler::new(state.clone(), 5, 50).await;
    crawler.start().await;
    metrics::set_crawler(crawler).await;
    info!("App initialized successfully");
    Ok(state)
}

async fn run_test_tasks(state: Arc<AppState>) -> AppResult<()> {
    let n = 1;
    let urls = state.repositories.podcast_rank.get_rss_urls().await?;
    let random_samples: Vec<_> = if n != 0 {
        let mut rng = thread_rng();
        urls.choose_multiple(&mut rng, n).cloned().collect()
    } else {
        urls.clone()
    };
    let mut crawler_guard = metrics::CRAWLER.lock().await;
    for url in random_samples {
        if let Some(crawler) = crawler_guard.as_mut() {
            if let Err(e) = crawler.add_task(&url).await {
                eprintln!("Failed to add task for {}: {}", url, e);
            }
        }
    }
    info!("Test tasks submitted successfully");
    Ok(())
}

async fn start_http_server(state: Arc<AppState>) -> AppResult<actix_web::dev::Server> {
    let metrics_server = metrics::start_metrics_server();
    info!("HTTP server started successfully");
    Ok(metrics_server)
}

async fn handle_shutdown(metrics_server: actix_web::dev::Server) -> AppResult<()> {
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();

    let server_handle = tokio::spawn(async move {
        tokio::select! {
            result = metrics_server => {
                if let Err(e) = result {
                    eprintln!("Metrics server error: {}", e);
                }
            }
            _ = shutdown_receiver => {
                info!("Received shutdown signal, stopping metrics server");
            }
        }
    });

    let ctrl_c_handle = tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C signal");
        info!("Ctrl+C received, initiating shutdown");
        let _ = shutdown_sender.send(());
    });

    let (server_result, ctrl_c_result) = tokio::join!(server_handle, ctrl_c_handle);

    if let Err(e) = server_result {
        eprintln!("Metrics server task panicked: {}", e);
    }
    if let Err(e) = ctrl_c_result {
        eprintln!("Ctrl+C handler panicked: {}", e);
    }

    info!("Shutting down application...");
    Ok(())
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let state = init_app().await?;
    run_test_tasks(state.clone()).await?;
    let metrics_server = start_http_server(state).await?;
    handle_shutdown(metrics_server).await?;
    Ok(())
}
