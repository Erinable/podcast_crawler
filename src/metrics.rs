use actix_web::web::Json;
use actix_web::{web, HttpResponse, Responder};
use prometheus::{
    register_histogram_vec, register_int_counter, register_int_gauge, register_int_gauge_vec,
    Encoder, HistogramVec, IntCounter, IntGauge, IntGaugeVec, TextEncoder,
};
use serde::Deserialize;
use serde_json::{json, to_value, Value};
use std::sync::Arc;
use std::sync::Once;
use tokio::sync::Mutex;

use crate::crawler_refactor::rss_crawler::RssCrawler;
use crate::infrastructure::AppState;

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

#[derive(Deserialize)]
struct SearchPodcastsQuery {
    q: String,
}

#[derive(Deserialize)]
struct GetPodcastsQuery {
    include_episodes: Option<bool>,
}

async fn search_podcasts_handler(
    query: web::Query<SearchPodcastsQuery>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    match state.repositories.podcast.search_by_title(&query.q).await {
        Ok(podcasts) => HttpResponse::Ok().json(podcasts),
        Err(_) => HttpResponse::InternalServerError().body("Failed to search podcasts"),
    }
}

async fn get_podcasts_handler(
    query: web::Query<GetPodcastsQuery>,
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let include_episodes = query.include_episodes.unwrap_or(false);
    match state.repositories.podcast.get_all(1, 10).await {
        Ok((podcasts, _total)) => {
            if include_episodes {
                let mut podcasts_with_episodes = Vec::new();
                for podcast in podcasts {
                    if let Ok(Some((podcast, episodes))) = state
                        .repositories
                        .podcast
                        .get_podcast_with_episodes_by_id(podcast.podcast_id)
                        .await
                    {
                        podcasts_with_episodes.push((podcast, episodes));
                    }
                }
                HttpResponse::Ok().json(podcasts_with_episodes)
            } else {
                HttpResponse::Ok().json(podcasts)
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch podcasts"),
    }
}

async fn get_podcasts_paginated_handler(
    state: web::Data<Arc<AppState>>,
    path: web::Path<(i64, i64)>,
) -> impl Responder {
    let (page, per_page) = path.into_inner();
    match state.repositories.podcast.get_all(page, per_page).await {
        Ok((podcasts, total)) => HttpResponse::Ok().json((podcasts, total)),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize)]
struct PodcastPathParams {
    id: i32,
    page: i64,
    per_page: i64,
}

async fn get_podcast_handler(
    path: web::Path<PodcastPathParams>,
    state: web::Data<Arc<AppState>>,
) -> impl Responder {
    let params = path.into_inner();
    match state
        .repositories
        .podcast
        .get_podcast_with_paginated_episodes(params.id, params.page, params.per_page)
        .await
    {
        Ok(Some((podcast, episodes, total_episodes))) => {
            let mut podcast_json = to_value(podcast).unwrap();
            if let Value::Object(obj) = &mut podcast_json {
                obj.insert("episodes".to_string(), json!(episodes));
                obj.insert("total_episodes".to_string(), json!(total_episodes));
                obj.insert("current_page".to_string(), json!(params.page));
                obj.insert("per_page".to_string(), json!(params.per_page));
            }
            HttpResponse::Ok().json(podcast_json)
        }
        Ok(None) => HttpResponse::NotFound().body("Podcast not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch podcast"),
    }
}

async fn get_podcast_by_title_handler(
    path: web::Path<String>,
    state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let title = path.into_inner();
    match state.repositories.podcast.get_by_title(&title).await {
        Ok(Some(podcast)) => HttpResponse::Ok().json(podcast),
        Ok(None) => HttpResponse::NotFound().body("Podcast not found"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch podcast"),
    }
}

pub fn start_metrics_server(state: Arc<AppState>) -> actix_web::dev::Server {
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_cors::Cors::permissive())
            .app_data(web::Data::new(state.clone()))
            .route("/metrics", web::get().to(metrics_handler))
            .route("/add_task", web::post().to(add_task_handler))
            .route("/podcasts/search", web::get().to(search_podcasts_handler))
            .route("/podcasts", web::get().to(get_podcasts_handler))
            .route(
                "/podcasts/page/{page}/{per_page}",
                web::get().to(get_podcasts_paginated_handler),
            )
            .route(
                "/podcasts/by-title/{title}",
                web::get().to(get_podcast_by_title_handler),
            )
            .route(
                "/podcasts/{id}/episodes/{page}/{per_page}",
                web::get().to(get_podcast_handler),
            )
    })
    .bind("127.0.0.1:8080")
    .expect("Failed to bind metrics server")
    .run()
}
