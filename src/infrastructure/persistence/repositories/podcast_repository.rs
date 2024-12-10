use crate::infrastructure::error::{AppError, AppResult};
use crate::infrastructure::persistence::database::DatabaseContext;
use crate::infrastructure::persistence::models::episode::NewEpisode;
use crate::infrastructure::persistence::models::podcast::{NewPodcast, Podcast, UpdatePodcast};
use crate::schema::{episodes, podcasts};
use diesel::prelude::*;
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
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        let result = podcasts::table
            .find(id)
            .first::<Podcast>(&mut conn)
            .await
            .optional()?;
        Ok(result)
    }

    pub async fn get_by_title(&self, title: &str) -> AppResult<Option<Podcast>> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
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
                // 插入播客并获取ID
                let inserted_podcast = diesel::insert_into(podcasts::table)
                    .values(new_podcast)
                    .get_result::<Podcast>(conn)
                    .await?;

                // 为剧集添加播客ID
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

                // 批量插入剧集
                if !episodes_with_podcast_id.is_empty() {
                    diesel::insert_into(episodes::table)
                        .values(episodes_with_podcast_id.as_slice())
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

    pub async fn batch_insert_with_episodes(
        &self,
        podcasts_with_episodes: &[(NewPodcast, Vec<NewEpisode>)],
    ) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;

        conn.transaction::<_, AppError, _>(|conn| {
            async move {
                for (new_podcast, new_episodes) in podcasts_with_episodes {
                    // 插入播客并获取ID
                    let inserted_podcast = diesel::insert_into(podcasts::table)
                        .values(new_podcast)
                        .get_result::<Podcast>(conn)
                        .await?;

                    // 为剧集添加播客ID
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

                    // 批量插入剧集
                    if !episodes_with_podcast_id.is_empty() {
                        diesel::insert_into(episodes::table)
                            .values(episodes_with_podcast_id.as_slice())
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
}
