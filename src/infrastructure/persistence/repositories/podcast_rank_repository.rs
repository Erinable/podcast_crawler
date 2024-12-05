use crate::infrastructure::error::AppResult;
use crate::infrastructure::persistence::database::DatabaseContext;
use crate::infrastructure::persistence::models::podcast_rank_model::{
    NewPodcastRank, PodcastRank, UpdatePodcastRank,
};

use crate::schema::podcast_rank;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

pub struct PodcastRankRepository {
    base: Arc<DatabaseContext>,
}

impl PodcastRankRepository {
    pub fn new(base: Arc<DatabaseContext>) -> Self {
        Self { base }
    }

    pub async fn get_by_id(&self, id: String) -> AppResult<Option<PodcastRank>> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        let result = podcast_rank::table
            .find(id)
            .first::<PodcastRank>(&mut conn)
            .await
            .optional()?;
        Ok(result)
    }

    pub async fn get_all(&self) -> AppResult<Vec<PodcastRank>> {
        let mut conn = self.base.get_connection().await?;
        let result = podcast_rank::table.load::<PodcastRank>(&mut conn).await?;
        Ok(result)
    }

    pub async fn insert(&self, new_podcast: &NewPodcastRank) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;
        diesel::insert_into(podcast_rank::table)
            .values(new_podcast)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn batch_insert(&self, new_podcasts: &[NewPodcastRank]) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;
        diesel::insert_into(podcast_rank::table)
            .values(new_podcasts)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn update_by_id(
        &self,
        id: String,
        update_podcast: &UpdatePodcastRank,
    ) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?;
        diesel::update(podcast_rank::table.find(id))
            .set(update_podcast)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(&self, id: String) -> AppResult<bool> {
        let mut conn = self.base.get_connection().await?;
        let rows_affected = diesel::delete(podcast_rank::table.find(id))
            .execute(&mut conn)
            .await?;
        Ok(rows_affected > 0)
    }

    pub async fn print_podcast_details(&self) -> AppResult<Vec<String>> {
        use crate::schema::podcast_rank::dsl::{
            avg_duration, avg_play_count, id, name, primary_genre_name, rank,
        };

        let mut conn = self.base.get_connection().await?;

        // 加载数据
        let ranks = podcast_rank::table
            .select((
                id,
                name,
                rank,
                primary_genre_name,
                avg_duration,
                avg_play_count,
            ))
            .load::<(
                String,
                Option<String>,
                Option<i32>,
                Option<String>,
                Option<i32>,
                Option<i32>,
            )>(&mut conn)
            .await?;

        // 处理并格式化数据
        let details: Vec<String> = ranks.into_iter().map(Self::format_podcast_detail).collect();

        Ok(details)
    }

    fn format_podcast_detail(
        record: (
            String,
            Option<String>,
            Option<i32>,
            Option<String>,
            Option<i32>,
            Option<i32>,
        ),
    ) -> String {
        let (id, name, rank, genre, duration, plays) = record;

        format!(
            "Podcast {} ({}): Rank {}, Genre {}, Avg Duration {} mins, Avg Plays {}",
            name.unwrap_or_else(|| "Unknown".to_string()),
            id,
            rank.map_or("N/A".to_string(), |r| r.to_string()),
            genre.unwrap_or_else(|| "Unknown".to_string()),
            duration.map_or("N/A".to_string(), |d| (d / 60).to_string()),
            plays.map_or("N/A".to_string(), |p| p.to_string())
        )
    }
}

#[cfg(test)]
mod tests {}
