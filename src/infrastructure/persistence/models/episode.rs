use crate::schema::episodes;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Queryable, Selectable, Serialize, Deserialize, Debug, AsChangeset, Clone, QueryableByName,
)]
#[diesel(table_name = episodes)]
pub struct Episode {
    pub episode_id: i32,
    pub podcast_id: Option<i32>,
    pub episode_image_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub pub_date: Option<DateTime<Utc>>,
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

#[derive(Insertable, Serialize, Deserialize, AsChangeset, Debug, Default, Clone)]
#[diesel(table_name = episodes)]
pub struct NewEpisode {
    pub podcast_id: Option<i32>,
    pub episode_image_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub pub_date: Option<DateTime<Utc>>,
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

#[derive(AsChangeset, Serialize, Deserialize, Debug)]
#[diesel(table_name = episodes)]
pub struct UpdateEpisode {
    pub podcast_id: Option<i32>,
    pub episode_image_url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub pub_date: Option<DateTime<Utc>>,
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
