use crate::infrastructure::error::AppResult;
use crate::infrastructure::persistence::database::DatabaseContext;
use crate::infrastructure::persistence::models::episode::{Episode, NewEpisode, UpdateEpisode};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

use crate::schema::episodes;

#[derive(Debug)]
pub struct EpisodeRepository {
    base: Arc<DatabaseContext>,
}

impl EpisodeRepository {
    pub fn new(pool: Arc<DatabaseContext>) -> Self {
        Self { base: pool }
    }

    pub async fn get_by_id(&self, id: i32) -> AppResult<Option<Episode>> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        let result = episodes::table
            .find(id)
            .first::<Episode>(&mut conn)
            .await
            .optional()?;
        Ok(result)
    }

    // 获取所有的 Episode 记录
    pub async fn get_all(&self) -> AppResult<Vec<Episode>> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        let results = episodes::table.load::<Episode>(&mut conn).await?; // 加载所有记录
        Ok(results)
    }

    // 插入新的 Episode 记录
    pub async fn insert(&self, new_episode: &NewEpisode) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        diesel::insert_into(episodes::table) // 插入数据到 episodes 表
            .values(new_episode)
            .execute(&mut conn)
            .await?; // 执行插入操作
        Ok(())
    }

    // 批量插入 Episode 记录
    pub async fn batch_insert(&self, new_episodes: &[NewEpisode]) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        diesel::insert_into(episodes::table) // 插入多个数据到 episodes 表
            .values(new_episodes)
            .execute(&mut conn)
            .await?; // 执行插入操作
        Ok(())
    }

    // 更新指定 ID 的 Episode 记录
    pub async fn update(&self, id: i32, update_episode: &UpdateEpisode) -> AppResult<()> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        diesel::update(episodes::table.find(id)) // 找到指定 ID 的记录
            .set(update_episode) // 设置新的值
            .execute(&mut conn)
            .await?; // 执行更新操作
        Ok(())
    }

    // 删除指定 ID 的 Episode 记录
    pub async fn delete(&self, id: i32) -> AppResult<bool> {
        let mut conn = self.base.get_connection().await?; // 获取数据库连接
        let rows_affected = diesel::delete(episodes::table.find(id)) // 找到并删除指定 ID 的记录
            .execute(&mut conn)
            .await?; // 执行删除操作
        Ok(rows_affected > 0) // 返回是否成功删除
    }
}
