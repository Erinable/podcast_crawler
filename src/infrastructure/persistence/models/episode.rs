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
    pub duration: Option<String>,
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
    pub duration: Option<String>,
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
    pub duration: Option<String>,
}

impl From<&NewEpisode> for UpdateEpisode {
    fn from(episode: &NewEpisode) -> Self {
        Self {
            podcast_id: episode.podcast_id,
            episode_image_url: episode.episode_image_url.clone(),
            title: Some(episode.title.clone()),
            description: episode.description.clone(),
            link: episode.link.clone(),
            pub_date: episode.pub_date,
            guid: episode.guid.clone(),
            enclosure_url: episode.enclosure_url.clone(),
            enclosure_type: episode.enclosure_type.clone(),
            enclosure_length: episode.enclosure_length,
            explicit: episode.explicit,
            subtitle: episode.subtitle.clone(),
            author: episode.author.clone(),
            summary: episode.summary.clone(),
            keywords: episode.keywords.clone(),
            category: episode.category.clone(),
            duration: episode.duration.clone(),
        }
    }
}
