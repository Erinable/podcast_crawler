use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl};
use chrono::NaiveDateTime;
use crate::schema::podcast_rank;
use diesel::pg::PgConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// PodcastRank 数据结构
#[derive(Queryable, Selectable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = podcast_rank)]
pub struct PodcastRank {
    pub id: String,
    pub rank: Option<i32>,
    pub name: Option<String>,
    pub logo_url: Option<String>,
    pub primary_genre_name: Option<String>,
    pub authors_text: Option<String>,
    pub track_count: Option<i32>,
    pub last_release_date: Option<NaiveDateTime>,
    pub last_release_date_day_count: Option<f64>,
    pub first_episode_post_time: Option<NaiveDateTime>,
    pub active_rate: Option<f64>,
    pub avg_duration: Option<i32>,
    pub avg_play_count: Option<i32>,
    pub avg_update_freq: Option<i32>,
    pub avg_comment_count: Option<i32>,
    pub avg_interact_indicator: Option<f64>,
    pub avg_open_rate: Option<f64>,
    pub links: Option<Value>, // JSONB -> Option<Value>
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = podcast_rank)]
pub struct NewPodcastRank {
    pub id: String,
    pub rank: Option<i32>,
    pub name: Option<String>,
    pub logo_url: Option<String>,
    pub primary_genre_name: Option<String>,
    pub authors_text: Option<String>,
    pub track_count: Option<i32>,
    pub last_release_date: Option<NaiveDateTime>,
    pub last_release_date_day_count: Option<f64>,
    pub first_episode_post_time: Option<NaiveDateTime>,
    pub active_rate: Option<f64>,
    pub avg_duration: Option<i32>,
    pub avg_play_count: Option<i32>,
    pub avg_update_freq: Option<i32>,
    pub avg_comment_count: Option<i32>,
    pub avg_interact_indicator: Option<f64>,
    pub avg_open_rate: Option<f64>,
    pub links: Option<Value>,
}

#[derive(AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = podcast_rank)]
pub struct UpdatePodcastRank {
    pub rank: Option<i32>,
    pub name: Option<String>,
    pub logo_url: Option<String>,
    pub primary_genre_name: Option<String>,
    pub authors_text: Option<String>,
    pub track_count: Option<i32>,
    pub last_release_date: Option<NaiveDateTime>,
    pub last_release_date_day_count: Option<f64>,
    pub first_episode_post_time: Option<NaiveDateTime>,
    pub active_rate: Option<f64>,
    pub avg_duration: Option<i32>,
    pub avg_play_count: Option<i32>,
    pub avg_update_freq: Option<i32>,
    pub avg_comment_count: Option<i32>,
    pub avg_interact_indicator: Option<f64>,
    pub avg_open_rate: Option<f64>,
    pub links: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Link {
    pub url: Option<String>,
    pub name: String,
}

impl PodcastRank {
    pub fn load(conn: &mut PgConnection) -> QueryResult<Vec<PodcastRank>> {
        podcast_rank::table.order(podcast_rank::rank).load::<PodcastRank>(conn)
    }
}