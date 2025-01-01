use crate::infrastructure::error::{AppError, AppResult};
use crate::infrastructure::persistence::database::DatabaseContext;
use crate::infrastructure::persistence::models::episode::NewEpisode;
use crate::infrastructure::persistence::models::podcast::{NewPodcast, Podcast, UpdatePodcast};
use crate::infrastructure::persistence::models::UpdateEpisode;
use crate::schema::{episodes, podcasts};
use diesel::prelude::*;
use diesel::upsert::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::sync::Arc;

#[derive(Debug)]
pub struct PodcastRepository {
    base: Arc<DatabaseContext>,
}

impl PodcastRepository {
    pub fn new(pool: Arc<DatabaseContext>) -> Self {
        Self { base: pool }
    }

    pub async fn get_by_id(&self, id: i32) -> AppResult<Option<Podcast>> {
        let mut conn = self.base.get_connection().await?;
        let result = podcasts::table
            .find(id)
            .first::<Podcast>(&mut conn)
            .await
            .optional()?;
        Ok(result)
    }

    pub async fn get_by_title(&self, title: &str) -> AppResult<Option<Podcast>> {
        let mut conn = self.base.get_connection().await?;
        let result = podcasts::table
            .filter(podcasts::title.eq(title))
            .first::<Podcast>(&mut conn)
            .await
            .optional()?;
        Ok(result)
    }

    pub async fn get_all(&self) -> AppResult<Vec<Podcast>> {
        let mut conn = self.base.get_connection().await?;
        let result = podcasts::table.load::<Podcast>(&mut conn).await?;
        Ok(result)
    }

    pub async fn insert(&self, new_podcast: &NewPodcast) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;
        diesel::insert_into(podcasts::table)
            .values(new_podcast)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn batch_insert(&self, new_podcasts: &[NewPodcast]) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;
        diesel::insert_into(podcasts::table)
            .values(new_podcasts)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_by_id(&self, id: i32, update_podcast: &UpdatePodcast) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;
        diesel::update(podcasts::table.find(id))
            .set(update_podcast)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(&self, id: i32) -> AppResult<bool> {
        let mut conn = self.base.get_connection().await?;
        let rows_affected = diesel::delete(podcasts::table.find(id))
            .execute(&mut conn)
            .await?;
        Ok(rows_affected > 0)
    }

    pub async fn insert_with_episodes(
        &self,
        new_podcast: &NewPodcast,
        new_episodes: &[NewEpisode],
    ) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;

        conn.transaction::<_, AppError, _>(|conn| {
            async move {
                let update_p: UpdatePodcast = new_podcast.into();
                let inserted_podcast = diesel::insert_into(podcasts::table)
                    .values(new_podcast)
                    .on_conflict(podcasts::title)
                    .do_update()
                    .set(&update_p)
                    .get_result::<Podcast>(conn)
                    .await?;

                let episodes_with_podcast_id: Vec<NewEpisode> = new_episodes
                    .iter()
                    .map(|episode| NewEpisode {
                        podcast_id: Some(inserted_podcast.podcast_id),
                        episode_image_url: episode.episode_image_url.clone(),
                        title: episode.title.clone(),
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
                    })
                    .collect();

                if !episodes_with_podcast_id.is_empty() {
                    for episode in &episodes_with_podcast_id {
                        let update: UpdateEpisode = episode.into();
                        diesel::insert_into(episodes::table)
                            .values(episode)
                            .on_conflict(episodes::title)
                            .do_update()
                            .set(update)
                            .execute(conn)
                            .await?;
                    }
                }

                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(())
    }

    pub async fn batch_insert_with_episodes(
        &self,
        podcasts_with_episodes: &[(NewPodcast, Vec<NewEpisode>)],
    ) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;

        conn.transaction::<_, AppError, _>(|conn| {
            async move {
                for (new_podcast, new_episodes) in podcasts_with_episodes {
                    let update_p: UpdatePodcast = new_podcast.into();
                    let inserted_podcast = diesel::insert_into(podcasts::table)
                        .values(new_podcast)
                        .on_conflict(podcasts::rss_feed_url)
                        .do_update()
                        .set(&update_p)
                        .get_result::<Podcast>(conn)
                        .await?;

                    let episodes_with_podcast_id: Vec<NewEpisode> = new_episodes
                        .iter()
                        .map(|episode| NewEpisode {
                            podcast_id: Some(inserted_podcast.podcast_id),
                            episode_image_url: episode.episode_image_url.clone(),
                            title: episode.title.clone(),
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
                        })
                        .collect();

                    if !episodes_with_podcast_id.is_empty() {
                        for episode in &episodes_with_podcast_id {
                            let update: UpdateEpisode = episode.into();
                            diesel::insert_into(episodes::table)
                                .values(episode)
                                .on_conflict(episodes::guid)
                                .do_update()
                                .set(update)
                                .execute(conn)
                                .await?;
                        }
                    }
                }
                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(())
    }

    pub async fn batch_upsert(&self, podcasts: &[NewPodcast]) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;

        conn.transaction::<_, AppError, _>(|conn| {
            async move {
                for podcast in podcasts {
                    let update: UpdatePodcast = podcast.into();
                    diesel::insert_into(podcasts::table)
                        .values(podcast)
                        .on_conflict(podcasts::rss_feed_url)
                        .do_update()
                        .set(&update)
                        .execute(conn)
                        .await?;
                }
                Ok(())
            }
            .scope_boxed()
        })
        .await?;

        Ok(())
    }
}
