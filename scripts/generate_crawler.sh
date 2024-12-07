#!/bin/bash

# 检查参数
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <crawler_name>"
    echo "Example: $0 apple_podcasts"
    exit 1
fi

CRAWLER_NAME=$1
CRAWLER_PATH="src/crawler/providers/$CRAWLER_NAME"

# 创建目录
mkdir -p "$CRAWLER_PATH"

# 创建 mod.rs
cat > "$CRAWLER_PATH/mod.rs" << EOL
//! $CRAWLER_NAME crawler implementation
//!
//! This module handles podcast crawling for $CRAWLER_NAME platform.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::infrastructure::error::AppResult;
use crate::crawler::traits::{PodcastCrawler, PodcastFeed};

mod client;
mod parser;
mod types;

pub use self::client::*;
pub use self::parser::*;
pub use self::types::*;

#[derive(Debug, Clone)]
pub struct ${CRAWLER_NAME^}Crawler {
    client: ${CRAWLER_NAME^}Client,
}

#[async_trait]
impl PodcastCrawler for ${CRAWLER_NAME^}Crawler {
    async fn fetch_feed(&self, url: &str) -> AppResult<PodcastFeed> {
        let raw_feed = self.client.fetch_feed(url).await?;
        let feed = self.parse_feed(&raw_feed)?;
        Ok(feed)
    }

    async fn validate_feed(&self, url: &str) -> AppResult<bool> {
        self.client.validate_feed(url).await
    }
}

impl ${CRAWLER_NAME^}Crawler {
    pub fn new() -> Self {
        Self {
            client: ${CRAWLER_NAME^}Client::new(),
        }
    }

    fn parse_feed(&self, raw_feed: &str) -> AppResult<PodcastFeed> {
        let parser = ${CRAWLER_NAME^}Parser::new();
        parser.parse(raw_feed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_crawler_creation() {
        let crawler = ${CRAWLER_NAME^}Crawler::new();
        assert!(crawler.client.is_initialized());
    }

    // TODO: Add more tests
}
EOL

# 创建 client.rs
cat > "$CRAWLER_PATH/client.rs" << EOL
//! HTTP client for $CRAWLER_NAME

use reqwest::Client;
use crate::infrastructure::error::AppResult;

#[derive(Debug, Clone)]
pub struct ${CRAWLER_NAME^}Client {
    client: Client,
    base_url: String,
}

impl ${CRAWLER_NAME^}Client {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: String::from("https://api.example.com"), // TODO: Update base URL
        }
    }

    pub async fn fetch_feed(&self, url: &str) -> AppResult<String> {
        let response = self.client
            .get(url)
            .send()
            .await?
            .text()
            .await?;
        Ok(response)
    }

    pub async fn validate_feed(&self, url: &str) -> AppResult<bool> {
        let response = self.client
            .head(url)
            .send()
            .await?;
        Ok(response.status().is_success())
    }

    pub fn is_initialized(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    // TODO: Add tests
}
EOL

# 创建 parser.rs
cat > "$CRAWLER_PATH/parser.rs" << EOL
//! Feed parser for $CRAWLER_NAME

use crate::infrastructure::error::AppResult;
use crate::crawler::traits::PodcastFeed;
use super::types::*;

#[derive(Debug)]
pub struct ${CRAWLER_NAME^}Parser;

impl ${CRAWLER_NAME^}Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, content: &str) -> AppResult<PodcastFeed> {
        // TODO: Implement feed parsing
        unimplemented!("Feed parsing not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = ${CRAWLER_NAME^}Parser::new();
        // Add assertions
    }

    // TODO: Add more tests
}
EOL

# 创建 types.rs
cat > "$CRAWLER_PATH/types.rs" << EOL
//! Type definitions for $CRAWLER_NAME

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ${CRAWLER_NAME^}Feed {
    pub title: String,
    pub description: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub link: Option<String>,
    pub image: Option<String>,
    pub categories: Vec<String>,
    pub episodes: Vec<${CRAWLER_NAME^}Episode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ${CRAWLER_NAME^}Episode {
    pub title: String,
    pub description: String,
    pub guid: String,
    pub published_at: DateTime<Utc>,
    pub duration: Option<i32>,
    pub audio_url: String,
    pub image: Option<String>,
}

impl ${CRAWLER_NAME^}Feed {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            author: None,
            language: None,
            link: None,
            image: None,
            categories: Vec::new(),
            episodes: Vec::new(),
        }
    }
}

impl Default for ${CRAWLER_NAME^}Feed {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_creation() {
        let feed = ${CRAWLER_NAME^}Feed::new();
        assert!(feed.episodes.is_empty());
    }
}
EOL

# 设置文件权限
chmod 644 "$CRAWLER_PATH"/*.rs

echo "Generated crawler module at $CRAWLER_PATH"
echo "Don't forget to:"
echo "1. Add the module to src/crawler/providers/mod.rs"
echo "2. Update the base URL in client.rs"
echo "3. Implement feed parsing in parser.rs"
echo "4. Add necessary tests"
