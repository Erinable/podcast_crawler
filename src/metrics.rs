use actix_web::web::Json;
use actix_web::{web, HttpResponse, Responder};
use prometheus::{
    register_histogram_vec, register_int_counter, register_int_gauge, register_int_gauge_vec,
    Encoder, HistogramVec, IntCounter, IntGauge, IntGaugeVec, TextEncoder,
};
use serde::Deserialize;
use std::sync::Once;
use tokio::sync::Mutex;

use crate::crawler_refactor::rss_crawler::RssCrawler;

#[derive(Deserialize)]
struct AddTaskRequest {
    rss_url: String,
}

lazy_static::lazy_static! {
    pub static ref CRAWLER: Mutex<Option<RssCrawler>> = Mutex::new(None);
}

pub async fn set_crawler(crawler: RssCrawler) {
    let mut guard = CRAWLER.lock().await;
    *guard = Some(crawler);
}

async fn add_task_handler(req: Json<AddTaskRequest>) -> impl Responder {
    let rss_url = &req.rss_url;
    let mut crawler_guard = CRAWLER.lock().await;
    if let Some(crawler) = crawler_guard.as_mut() {
        match crawler.add_task(rss_url).await {
            Ok(_) => HttpResponse::Ok().body("Task added successfully"),
            Err(e) => {
                HttpResponse::InternalServerError().body(format!("Failed to add task: {}", e))
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Crawler not initialized")
    }
}

static INIT: Once = Once::new();

lazy_static::lazy_static! {
    pub static ref ACTIVE_WORKERS: IntGauge = register_int_gauge!(
        "active_workers",
        "Number of active workers"
    ).unwrap();

    pub static ref PROCESSED_TASKS: IntCounter = register_int_counter!(
        "processed_tasks",
        "Total number of processed tasks"
    ).unwrap();

    pub static ref FAILED_TASKS: IntCounter = register_int_counter!(
        "failed_tasks",
        "Total number of failed tasks"
    ).unwrap();

    pub static ref TASK_RETRIES: IntCounter = register_int_counter!(
        "task_retries",
        "Total number of task retries"
    ).unwrap();

    pub static ref TASK_STATUS: IntGaugeVec = register_int_gauge_vec!(
        "task_status",
        "Current status of tasks",
        &["stage", "status"]
    ).unwrap();

    pub static ref TASK_STAGE_DURATION: HistogramVec = register_histogram_vec!(
        "task_stage_duration_seconds",
        "Time taken for each task stage",
        &["stage"],
        {
            let buckets_str = std::env::var("TASK_STAGE_DURATION_BUCKETS").unwrap_or("0.1,0.5,1.0,2.0,5.0,10.0".to_string());
            let buckets: Vec<f64> = buckets_str
                .split(',')
                .map(|s| s.parse().expect("Failed to parse bucket value"))
                .collect();
            buckets
        }
    ).unwrap();

    pub static ref SUBMITTED_TASKS: IntCounter = register_int_counter!(
        "submitted_tasks",
        "Total number of submitted tasks"
    ).unwrap();
}

pub fn init_metrics() {
    INIT.call_once(|| {
        // Initialize metrics
        ACTIVE_WORKERS.set(0);
        PROCESSED_TASKS.reset();
        FAILED_TASKS.reset();
        TASK_RETRIES.reset();
        TASK_STATUS.reset();
        TASK_STAGE_DURATION.reset();
        SUBMITTED_TASKS.reset();
        // Initialize all possible status counts to 0
        let stages = vec!["distribution", "fetching", "parsing", "inserting"];
        let statuses = vec!["pending", "in_progress", "completed", "failed"];

        for stage in stages {
            for status in &statuses {
                crate::metrics::TASK_STATUS
                    .with_label_values(&[stage, status])
                    .set(0);
            }
        }
    });
}

pub async fn metrics_handler() -> impl Responder {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}

pub fn start_metrics_server() -> actix_web::dev::Server {
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .route("/metrics", web::get().to(metrics_handler))
            .route("/add_task", web::post().to(add_task_handler))
    })
    .bind("127.0.0.1:8080")
    .expect("Failed to bind metrics server")
    .run()
}
