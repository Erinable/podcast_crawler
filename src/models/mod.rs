// src/models.rs
use crate::database::DbConn;
use crate::schema::podcast_rank::rank;
use crate::schema::{episodes, podcast_rank, podcasts};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Queryable, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Episode {
    pub episode_id: i32,
    pub podcast_id: Option<i32>,
    pub episode_image_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub pub_date: Option<NaiveDateTime>, // Use chrono::NaiveDateTime for Timestamptz
    pub guid: Option<String>,
    pub enclosure_url: Option<String>,
    pub enclosure_type: Option<String>,
    pub enclosure_length: Option<i64>,
    pub explicit: Option<bool>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    pub category: Option<Vec<Option<String>>>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[table_name = "episodes"]
pub struct NewEpisode {
    pub podcast_id: Option<i32>,
    pub episode_image_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub pub_date: Option<NaiveDateTime>, // Use chrono::NaiveDateTime for Timestamptz
    pub guid: Option<String>,
    pub enclosure_url: Option<String>,
    pub enclosure_type: Option<String>,
    pub enclosure_length: Option<i64>,
    pub explicit: Option<bool>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    pub category: Option<Vec<Option<String>>>,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[table_name = "episodes"]
pub struct UpdateEpisode {
    pub podcast_id: Option<i32>,
    pub episode_image_url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub pub_date: Option<NaiveDateTime>, // Use chrono::NaiveDateTime for Timestamptz
    pub guid: Option<String>,
    pub enclosure_url: Option<String>,
    pub enclosure_type: Option<String>,
    pub enclosure_length: Option<i64>,
    pub explicit: Option<bool>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    pub category: Option<Vec<Option<String>>>,
}

// Create a new episode
pub fn create_episode(conn: &mut DbConn, new_episode: NewEpisode) -> QueryResult<Episode> {
    diesel::insert_into(episodes::table)
        .values(&new_episode)
        .get_result(conn)
}

// Get a single episode by id
pub fn get_episode_by_id(conn: &mut DbConn, episode_id: i32) -> QueryResult<Episode> {
    episodes::table.find(episode_id).first(conn)
}

// Update an episode
pub fn update_episode(
    conn: &mut DbConn,
    episode_id: i32,
    updated_episode: UpdateEpisode,
) -> QueryResult<Episode> {
    diesel::update(episodes::table.filter(episodes::episode_id.eq(episode_id)))
        .set(&updated_episode)
        .get_result(conn)
}

// Delete an episode
pub fn delete_episode(conn: &mut DbConn, episode_id: i32) -> QueryResult<usize> {
    diesel::delete(episodes::table.filter(episodes::episode_id.eq(episode_id))).execute(conn)
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Podcast {
    pub podcast_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub last_build_date: Option<NaiveDateTime>, // Use chrono::NaiveDateTime for Timestamptz
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub image_url: Option<String>,
    pub rss_feed_url: Option<String>,
    pub category: Option<Vec<Option<String>>>,
    pub author: Option<String>,
    pub owner_name: Option<String>,
    pub owner_email: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    pub explicit: Option<bool>,
    pub summary: Option<String>,
    pub subtitle: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[table_name = "podcasts"]
pub struct NewPodcast {
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub last_build_date: Option<NaiveDateTime>, // Use chrono::NaiveDateTime for Timestamptz
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub image_url: Option<String>,
    pub rss_feed_url: Option<String>,
    pub category: Option<Vec<Option<String>>>,
    pub author: Option<String>,
    pub owner_name: Option<String>,
    pub owner_email: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    pub explicit: Option<bool>,
    pub summary: Option<String>,
    pub subtitle: Option<String>,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[table_name = "podcasts"]
pub struct UpdatePodcast {
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub last_build_date: Option<NaiveDateTime>, // Use chrono::NaiveDateTime for Timestamptz
    pub language: Option<String>,
    pub copyright: Option<String>,
    pub image_url: Option<String>,
    pub rss_feed_url: Option<String>,
    pub category: Option<Vec<Option<String>>>,
    pub author: Option<String>,
    pub owner_name: Option<String>,
    pub owner_email: Option<String>,
    pub keywords: Option<Vec<Option<String>>>,
    pub explicit: Option<bool>,
    pub summary: Option<String>,
    pub subtitle: Option<String>,
}

// Create a new episode
pub fn create_podcast(conn: &mut DbConn, new_podcast: NewPodcast) -> QueryResult<Podcast> {
    diesel::insert_into(podcasts::table)
        .values(&new_podcast)
        .get_result(conn)
}

/// PodcastRank 数据结构
#[derive(Queryable, Serialize, Deserialize, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Link {
    pub name: String,
    pub url: String,
}

fn load_podcasts(conn: &mut DbConn) -> QueryResult<Vec<PodcastRank>> {
    podcast_rank::table.order(rank).load(conn)
}

pub fn print_podcast_details(conn: &mut DbConn) -> QueryResult<Vec<String>> {
    let podcasts = load_podcasts(conn)?;

    let rss_list = podcasts
        .into_iter()
        .filter_map(|podcast| match podcast.links {
            Some(Value::Array(links)) => Some(links),
            _ => None,
        })
        .flatten()
        .filter_map(|link| {
            if link.get("name") == Some(&Value::String("rss".to_string())) {
                link.get("url")
                    .and_then(Value::as_str)
                    .filter(|url| !url.is_empty()) // 确保 URL 非空
                    .map(String::from)
            } else {
                None
            }
        })
        .collect();

    Ok(rss_list)
}

