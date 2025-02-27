use crate::schema::podcasts;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = podcasts)]
pub struct Podcast {
    pub podcast_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub last_build_date: Option<DateTime<Utc>>,
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

#[derive(Insertable, Debug, Default, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = podcasts)]
pub struct NewPodcast {
    pub title: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub last_build_date: Option<DateTime<Utc>>,
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

#[derive(AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = podcasts)]
pub struct UpdatePodcast {
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub last_build_date: Option<DateTime<Utc>>,
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

impl From<&NewPodcast> for UpdatePodcast {
    fn from(podcast: &NewPodcast) -> Self {
        Self {
            title: Some(podcast.title.clone()),
            description: podcast.description.clone(),
            link: podcast.link.clone(),
            last_build_date: podcast.last_build_date,
            language: podcast.language.clone(),
            copyright: podcast.copyright.clone(),
            image_url: podcast.image_url.clone(),
            rss_feed_url: podcast.rss_feed_url.clone(),
            category: podcast.category.clone(),
            author: podcast.author.clone(),
            owner_name: podcast.owner_name.clone(),
            owner_email: podcast.owner_email.clone(),
            keywords: podcast.keywords.clone(),
            explicit: podcast.explicit,
            summary: podcast.summary.clone(),
            subtitle: podcast.subtitle.clone(),
        }
    }
}
