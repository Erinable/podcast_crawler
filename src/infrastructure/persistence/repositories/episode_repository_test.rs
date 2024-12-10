use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use futures::TryFutureExt;

use crate::infrastructure::error::AppResult;
use crate::infrastructure::persistence::database::DbPool;
use crate::infrastructure::persistence::models::episode::{Episode, NewEpisode};
use crate::infrastructure::persistence::repositories::episode_repository::EpisodeRepository;
use crate::schema::episodes;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::database::establish_connection;
    use diesel::result::Error;
    use std::env;

    async fn setup_test_db() -> DbPool {
        // Set up test database connection
        if env::var("DATABASE_URL").is_err() {
            env::set_var("DATABASE_URL", "postgres://localhost/podcast_crawler_test");
        }
        establish_connection().await
    }

    async fn cleanup_episodes(pool: &DbPool) -> AppResult<()> {
        let mut conn = pool.get().await?;
        diesel::delete(episodes::table)
            .execute(&mut conn)
            .await?;
        Ok(())
    }

    fn create_test_episode() -> NewEpisode {
        NewEpisode {
            podcast_id: Some(1),
            episode_image_url: Some("https://example.com/image.jpg".to_string()),
            title: "Test Episode".to_string(),
            description: Some("Test Description".to_string()),
            link: Some("https://example.com/episode".to_string()),
            pub_date: Some(Utc::now()),
            guid: Some("test-guid".to_string()),
            enclosure_url: Some("https://example.com/audio.mp3".to_string()),
            enclosure_type: Some("audio/mpeg".to_string()),
            enclosure_length: Some(1000),
            explicit: Some(false),
            subtitle: Some("Test Subtitle".to_string()),
            author: Some("Test Author".to_string()),
            summary: Some("Test Summary".to_string()),
            keywords: Some(vec![Some("test".to_string()), Some("podcast".to_string())]),
            category: Some(vec![Some("Technology".to_string())]),
        }
    }

    #[tokio::test]
    async fn test_insert_and_get_episode() -> AppResult<()> {
        let pool = setup_test_db().await;
        let repo = EpisodeRepository::new(pool.clone());

        // Clean up before test
        cleanup_episodes(&pool).await?;

        // Test insertion
        let new_episode = create_test_episode();
        repo.insert(&new_episode).await?;

        // Test get_all
        let episodes = repo.get_all().await?;
        assert_eq!(episodes.len(), 1);
        let episode = &episodes[0];
        assert_eq!(episode.title, new_episode.title);
        assert_eq!(episode.podcast_id, new_episode.podcast_id);
        assert_eq!(episode.description, new_episode.description);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_by_id() -> AppResult<()> {
        let pool = setup_test_db().await;
        let repo = EpisodeRepository::new(pool.clone());

        // Clean up before test
        cleanup_episodes(&pool).await?;

        // Insert test episode
        let new_episode = create_test_episode();
        repo.insert(&new_episode).await?;

        // Get the inserted episode's ID
        let episodes = repo.get_all().await?;
        let episode_id = episodes[0].episode_id;

        // Test get_by_id
        let found_episode = repo.get_by_id(episode_id).await?;
        assert!(found_episode.is_some());
        let found_episode = found_episode.unwrap();
        assert_eq!(found_episode.title, new_episode.title);
        assert_eq!(found_episode.podcast_id, new_episode.podcast_id);

        // Test get_by_id with non-existent ID
        let not_found = repo.get_by_id(-1).await?;
        assert!(not_found.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_insert() -> AppResult<()> {
        let pool = setup_test_db().await;
        let repo = EpisodeRepository::new(pool.clone());

        // Clean up before test
        cleanup_episodes(&pool).await?;

        // Create multiple test episodes
        let mut episodes = Vec::new();
        for i in 1..=3 {
            let mut episode = create_test_episode();
            episode.title = format!("Test Episode {}", i);
            episodes.push(episode);
        }

        // Test batch insertion
        repo.batch_insert(&episodes).await?;

        // Verify all episodes were inserted
        let found_episodes = repo.get_all().await?;
        assert_eq!(found_episodes.len(), 3);
        for (i, episode) in found_episodes.iter().enumerate() {
            assert_eq!(episode.title, format!("Test Episode {}", i + 1));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_episode() -> AppResult<()> {
        let pool = setup_test_db().await;
        let repo = EpisodeRepository::new(pool.clone());

        // Clean up before test
        cleanup_episodes(&pool).await?;

        // Insert test episode
        let new_episode = create_test_episode();
        repo.insert(&new_episode).await?;

        // Get the inserted episode's ID
        let episodes = repo.get_all().await?;
        let episode_id = episodes[0].episode_id;

        // Create update data
        let update_episode = NewEpisode {
            title: "Updated Title".to_string(),
            description: Some("Updated Description".to_string()),
            ..new_episode
        };

        // Test update
        repo.update(episode_id, &update_episode).await?;

        // Verify update
        let updated = repo.get_by_id(episode_id).await?.unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, Some("Updated Description".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_episode() -> AppResult<()> {
        let pool = setup_test_db().await;
        let repo = EpisodeRepository::new(pool.clone());

        // Clean up before test
        cleanup_episodes(&pool).await?;

        // Insert test episode
        let new_episode = create_test_episode();
        repo.insert(&new_episode).await?;

        // Get the inserted episode's ID
        let episodes = repo.get_all().await?;
        let episode_id = episodes[0].episode_id;

        // Test delete
        let deleted = repo.delete(episode_id).await?;
        assert!(deleted);

        // Verify deletion
        let episodes = repo.get_all().await?;
        assert_eq!(episodes.len(), 0);

        // Test delete non-existent episode
        let deleted = repo.delete(-1).await?;
        assert!(!deleted);

        Ok(())
    }
}
