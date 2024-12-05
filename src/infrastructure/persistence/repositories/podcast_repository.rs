use crate::infrastructure::error::AppResult;
use crate::infrastructure::persistence::database::DatabaseContext;
use crate::infrastructure::persistence::models::podcast::{NewPodcast, Podcast, UpdatePodcast};
use crate::schema::podcasts;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

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
}
