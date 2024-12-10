use std::io::BufRead;

use crate::crawler::traits::FeedParser;
use crate::infrastructure::error::{
    parse::{ParseError, ParseErrorKind},
    AppError, AppResult,
};
use crate::infrastructure::persistence::models::{episode::NewEpisode, podcast::NewPodcast};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use tracing::{debug, warn};

/// Debugging macro for parser events.
///
/// Outputs a debug message with the following format:
/// `[DEBUG][PARSER] [event_type][Tag Name: tag_name][Text: text][Path: path][State: state]`
///
/// # Parameters
///
/// * `event_type`: The type of parser event (e.g. "tag", "content", "event")
/// * `text`: The text content of the current tag (if any)
/// * `state`: The current state of the parser
macro_rules! debug_info {
    ($event_type:literal, $state:expr) => {
        debug_info!($event_type, "None", $state);
    };
    ($event_type:literal, $text:expr, $state:expr) => {
        debug!(
            "[PARSER] [{}][Tag Name: {}][Text: {}][Path: {}][State: {:?}][Depth: {}]",
            $event_type,
            $state.current_tag,
            $text,
            $state.context.current_path(),
            $state.current_state,
            $state.context.current_depth()
        );
    };
}

/// RSS 解析上下文，用于错误处理和状态跟踪
#[derive(Debug, Default)]
pub struct ParseContext {
    /// XML 元素路径，用于错误定位
    element_path: Vec<String>,
    /// 当前处理的行号
    line_number: Option<u32>,
    /// 当前处理的列号
    column_number: Option<u32>,
    /// 原始内容片段
    raw_content: Option<String>,
    /// URL
    url: String,
}

impl ParseContext {
    fn new(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }

    fn push_element(&mut self, name: String) {
        self.element_path.push(name);
    }

    fn pop_element(&mut self) {
        self.element_path.pop();
    }

    fn current_path(&self) -> String {
        self.element_path.join("/")
    }

    fn current_depth(&self) -> usize {
        self.element_path.len()
    }
}

/// Parsing states
#[derive(Debug, Default, PartialEq)]
enum ParsingState {
    #[default]
    Initial,
    InChannel,
    InItem,
    InChannelLink,
    InChannelItunesOwner,
    InChannelItunesOwnerName,
    InChannelItunesOwnerEmail,
}

/// Parser state during RSS processing
#[derive(Debug, Default)]
struct RssParserState {
    current_state: ParsingState,
    current_tag: String,
    podcast: Option<NewPodcast>,
    current_episode: Option<NewEpisode>,
    episodes: Vec<NewEpisode>,
    context: ParseContext,
}

impl RssParserState {
    fn new(url: String) -> Self {
        Self {
            context: ParseContext::new(url),
            ..Default::default()
        }
    }

    fn validate_podcast(&self, podcast: &NewPodcast) -> Result<(), AppError> {
        if podcast.title.is_empty() {
            return Err(ParseError::new(
                ParseErrorKind::MissingField,
                "Missing podcast title",
                &self.context.url,
                None,
            )
            .into());
        }
        Ok(())
    }

    fn validate_episode(&self, episode: &NewEpisode) -> Result<(), AppError> {
        if episode.title.is_empty() {
            return Err(ParseError::new(
                ParseErrorKind::MissingField,
                "Missing episode title",
                &self.context.url,
                None,
            )
            .into());
        }
        Ok(())
    }
}

/// RSS feed parser
#[derive(Clone, Debug, Default)]
pub struct RssFeedParser {
    /// 最大 episode 数量限制
    max_episodes: Option<usize>,
    /// 严格模式：在严格模式下，任何解析错误都会导致整个解析失败
    strict_mode: bool,
    /// 解析器配置
    config: ParserConfig,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// 是否清理 HTML 内容
    clean_html: bool,
    /// 是否验证 URLs
    validate_urls: bool,
    /// 是否允许空的必需字段
    allow_empty_required: bool,
    /// 重试次数
    retry_count: u32,
    /// 严格模式
    strict_mode: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            clean_html: true,
            validate_urls: true,
            allow_empty_required: false,
            retry_count: 3,
            strict_mode: true,
        }
    }
}

impl RssFeedParser {
    pub fn new() -> Self {
        Self {
            max_episodes: None,
            strict_mode: false,
            config: ParserConfig::default(),
        }
    }

    pub fn with_config(
        max_episodes: Option<usize>,
        strict_mode: bool,
        config: ParserConfig,
    ) -> Self {
        Self {
            max_episodes,
            strict_mode,
            config,
        }
    }

    async fn parse_internal<R: BufRead>(
        &self,
        content: R,
        url: &str,
    ) -> AppResult<(NewPodcast, Vec<NewEpisode>)> {
        let mut reader = Reader::from_reader(content);
        reader.trim_text(true);
        reader.expand_empty_elements(true); // 展开空标签

        let mut state = RssParserState::new(url.to_string());
        state.podcast = Some(NewPodcast {
            rss_feed_url: Some(url.to_string()),
            ..Default::default()
        });

        debug!("Starting RSS parsing for URL: {}", url);
        let mut buf = Vec::new();
        let mut depth = 0;
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    depth += 1;
                    let name = e.name();
                    let tag_name = String::from_utf8_lossy(name.as_ref()).into_owned();
                    state.current_tag = tag_name.clone();
                    state.context.push_element(tag_name.clone());

                    // debug_event_info!("Start", &tag_name, &state);
                    debug_info!("START EVENT", &state);

                    self.handle_start_event(&mut state, &e, tag_name)?;
                }
                Ok(Event::End(e)) => {
                    let name = e.name();
                    let tag_name = String::from_utf8_lossy(name.as_ref()).into_owned();
                    debug_info!("END EVENT", &state);
                    // debug_tag_info!("End", &tag_name, &depth);

                    self.handle_end_element(&mut state, &e)?;
                    depth -= 1;
                }
                Ok(Event::Empty(e)) => {
                    let name = e.name();
                    let tag_name = String::from_utf8_lossy(name.as_ref()).into_owned();
                    // debug_tag_info!("Empty", &tag_name, &depth);
                    debug_info!("EMPTY EVENT", &state);

                    if tag_name == "enclosure" {
                        self.handle_enclosure(&mut state, &e)?;
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().map_err(|e| {
                        AppError::from(ParseError::new(
                            ParseErrorKind::InvalidXml,
                            "Failed to unescape text",
                            url,
                            Some(Box::new(e)),
                        ))
                    })?;

                    // debug_content_info!("Text", &text, &depth);

                    self.handle_text_event(&e, &mut state)?;
                }
                Ok(Event::CData(e)) => {
                    let text = String::from_utf8_lossy(&e.into_inner()).into_owned();
                    // debug_content_info!("CDATA", &text, &depth);
                    debug_info!("CDATA EVENT", &text, &state);

                    // Convert to BytesText for consistent handling
                    let text_event = BytesText::new(&text);
                    self.handle_text_event(&text_event, &mut state)?;
                }
                Ok(Event::Eof) => {
                    debug!("Reached end of RSS feed");
                    break;
                }
                Err(e) => {
                    return Err(AppError::from(ParseError::new(
                        ParseErrorKind::InvalidXml,
                        format!("Error at position {}: {:?}", reader.buffer_position(), e),
                        url,
                        Some(Box::new(e)),
                    )))
                }
                _ => buf.clear(), // 忽略其他事件
            }
            buf.clear();
        }

        // 验证结果
        let podcast = state.podcast.as_ref().ok_or_else(|| {
            AppError::from(ParseError::new(
                ParseErrorKind::Other,
                "Missing podcast metadata",
                state.context.url.clone(),
                None,
            ))
        })?;

        state.validate_podcast(podcast).map_err(AppError::from)?;
        let podcast = state.podcast.unwrap();

        debug!("Successfully parsed RSS feed:");
        debug!("- Podcast: {:#?}", podcast);
        debug!("- Episodes: {:#?}", state.episodes);

        Ok((podcast, state.episodes))
    }

    fn handle_start_event(
        &self,
        mut state: &mut RssParserState,
        e: &BytesStart,
        tag_name: String,
    ) -> Result<(), AppError> {
        match tag_name.as_str() {
            "channel" => {
                state.current_state = ParsingState::InChannel;
                state.podcast = Some(NewPodcast {
                    rss_feed_url: Some(state.context.url.clone()),
                    ..Default::default()
                });
            }
            "item" => {
                state.current_state = ParsingState::InItem;
                state.current_episode = Some(NewEpisode::default());
            }
            "itunes:owner" => {
                state.current_state = ParsingState::InChannelItunesOwner;
            }
            "itunes:name" if state.current_state == ParsingState::InChannelItunesOwner => {
                state.current_state = ParsingState::InChannelItunesOwnerName;
            }
            "itunes:email" if state.current_state == ParsingState::InChannelItunesOwner => {
                state.current_state = ParsingState::InChannelItunesOwnerEmail;
            }
            "itunes:image" | "itunes:category" => {
                self.handle_image_and_category(&mut state, &e)?;
            }
            "link" => {
                if state.current_state == ParsingState::InChannel {
                    state.current_state = ParsingState::InChannelLink;
                    self.handle_channel_link(&mut state, &e)?;
                }
            }
            "enclosure" => {
                if state.current_state == ParsingState::InItem {
                    self.handle_enclosure(&mut state, &e)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_enclosure(
        &self,
        state: &mut RssParserState,
        event: &BytesStart,
    ) -> Result<(), AppError> {
        // debug_event_info!("Handling enclosure", &state.current_tag, &state);
        debug_info!("ENCLOSURE EVENT", &state);

        let episode = state.current_episode.as_mut().ok_or_else(|| {
            AppError::from(ParseError::new(
                ParseErrorKind::Other,
                "Enclosure tag found outside of episode context",
                &state.context.url,
                None,
            ))
        })?;

        let mut found_url = false;
        for attr in event.attributes() {
            let attr = attr.map_err(|e| {
                AppError::from(ParseError::new(
                    ParseErrorKind::InvalidXml,
                    "Failed to parse enclosure attribute",
                    &state.context.url,
                    Some(Box::new(e)),
                ))
            })?;

            let original = String::from_utf8_lossy(attr.value.as_ref()).into_owned();
            let value = handle_xml_error(attr.unescape_value(), Some(original))?.into_owned();
            match attr.key.as_ref() {
                b"url" => {
                    // 先解码 XML 实体
                    let decoded_url = value.replace("&amp;", "&");
                    let normalized_url = if decoded_url.starts_with("http") {
                        // 保持原始 URL 不变，因为喜马拉雅的 URL 包含特殊的查询参数
                        decoded_url
                    } else {
                        decoded_url.replace("//", "/")
                    };
                    if self.config.validate_urls {
                        validate_url(&normalized_url).map_err(|_| {
                            AppError::from(ParseError::new(
                                ParseErrorKind::InvalidFormat,
                                format!("Invalid enclosure URL: {}", normalized_url),
                                &state.context.url,
                                None,
                            ))
                        })?;
                    }
                    debug!("Found enclosure URL: {}", normalized_url);
                    episode.enclosure_url = Some(normalized_url);
                    found_url = true;
                }
                b"type" => {
                    debug!("Found enclosure type: {}", value);
                    episode.enclosure_type = Some(value);
                }
                b"length" => {
                    if let Ok(length) = value.parse() {
                        debug!("Found enclosure length: {}", length);
                        episode.enclosure_length = Some(length);
                    } else {
                        debug!("Failed to parse enclosure length: {}", value);
                        if self.config.strict_mode {
                            return Err(AppError::from(ParseError::new(
                                ParseErrorKind::InvalidFormat,
                                format!("Invalid enclosure length: {}", value),
                                &state.context.url,
                                None,
                            )));
                        }
                    }
                }
                _ => {
                    debug!("Ignoring unknown enclosure attribute: {:?}", attr.key);
                }
            }
        }

        if !found_url && self.config.strict_mode {
            return Err(AppError::from(ParseError::new(
                ParseErrorKind::MissingField,
                "Enclosure tag missing required URL attribute",
                &state.context.url,
                None,
            )));
        }

        Ok(())
    }

    fn handle_channel_link(
        &self,
        state: &mut RssParserState,
        event: &BytesStart,
    ) -> Result<(), AppError> {
        if let Some(podcast) = &mut state.podcast {
            // 处理 link 标签的属性
            for attr in event.attributes() {
                let attr = attr.map_err(|e| {
                    AppError::from(ParseError::new(
                        ParseErrorKind::InvalidXml,
                        "Failed to parse link attribute",
                        &state.context.url,
                        Some(Box::new(e)),
                    ))
                })?;

                let original = String::from_utf8_lossy(attr.value.as_ref()).into_owned();
                let value = handle_xml_error(attr.unescape_value(), Some(original))?.into_owned();
                match attr.key.as_ref() {
                    b"href" => {
                        let url = value;
                        if self.config.validate_urls {
                            validate_url(&url).map_err(|_| {
                                AppError::from(ParseError::new(
                                    ParseErrorKind::InvalidFormat,
                                    "Invalid link URL",
                                    &state.context.url,
                                    None,
                                ))
                            })?;
                        }
                        podcast.link = Some(url);
                        return Ok(());
                    }
                    _ => {
                        debug!("Ignoring unknown link attribute: {:?}", attr.key);
                    }
                }
            }

            // 如果没有 href 属性，等待处理标签内容
            state.current_state = ParsingState::InChannelLink;
        }
        Ok(())
    }

    fn handle_end_element(
        &self,
        state: &mut RssParserState,
        event: &BytesEnd,
    ) -> Result<(), AppError> {
        let name = event.name();
        let tag_name = String::from_utf8_lossy(name.as_ref()).into_owned();
        state.context.pop_element();

        match (tag_name.as_str(), &state.current_state) {
            ("channel", ParsingState::InChannel) => {
                state.current_state = ParsingState::Initial;
            }
            ("item", ParsingState::InItem) => {
                self.handle_item_end(state)?;
                state.current_state = ParsingState::InChannel;
            }
            ("link", ParsingState::InChannelLink) => {
                state.current_state = ParsingState::InChannel;
            }
            ("itunes:owner", ParsingState::InChannelItunesOwner) => {
                state.current_state = ParsingState::InChannel;
            }
            (
                "itunes:name" | "itunes:email",
                ParsingState::InChannelItunesOwnerEmail | ParsingState::InChannelItunesOwnerName,
            ) => {
                state.current_state = ParsingState::InChannelItunesOwner;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_item_end(&self, state: &mut RssParserState) -> Result<(), AppError> {
        if let Some(episode) = state.current_episode.take() {
            debug!("Finishing episode: {:?}", episode);
            state.validate_episode(&episode)?;
            state.episodes.push(episode);
        }
        Ok(())
    }

    fn handle_text_event(
        &self,
        event: &BytesText,
        state: &mut RssParserState,
    ) -> Result<(), AppError> {
        let text = event.unescape().map_err(|e| {
            AppError::from(ParseError::new(
                ParseErrorKind::InvalidXml,
                "Failed to unescape text",
                &state.context.url,
                Some(Box::new(e)),
            ))
        })?;

        let text = if self.config.clean_html {
            clean_html(&text)
        } else {
            text.into_owned()
        };

        if text.trim().is_empty() && !self.config.allow_empty_required {
            return Ok(());
        }
        debug_info!("TEXT EVENT", &text, &state);

        match (state.current_tag.as_str(), &state.current_state) {
            // Podcast 字段
            ("title", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.title = text.trim().to_string();
                }
            }
            ("description", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.description = Some(text.trim().to_string());
                }
            }
            ("language", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.language = Some(text.trim().to_string());
                }
            }
            ("copyright", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.copyright = Some(text.trim().to_string());
                }
            }
            ("itunes:author", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.author = Some(text.trim().to_string());
                }
            }
            ("itunes:name", ParsingState::InChannelItunesOwnerName) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.owner_name = Some(text.trim().to_string());
                }
            }
            ("itunes:email", ParsingState::InChannelItunesOwnerEmail) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.owner_email = Some(text.trim().to_string());
                }
            }
            ("itunes:category", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast
                        .category
                        .get_or_insert_with(Vec::new)
                        .push(Some(text.trim().to_string()));
                }
            }
            ("itunes:keywords", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast
                        .keywords
                        .get_or_insert_with(Vec::new)
                        .push(Some(text.trim().to_string()));
                }
            }
            ("itunes:explicit", ParsingState::InChannelLink) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.explicit = parse_bool(&text.trim());
                }
            }
            ("itunes:summary", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.summary = Some(text.trim().to_string());
                }
            }
            ("itunes:subtitle", ParsingState::InChannel) => {
                if let Some(podcast) = &mut state.podcast {
                    podcast.subtitle = Some(text.trim().to_string());
                }
            }

            // Episode 字段
            ("title", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.title = text.trim().to_string();
                }
            }
            ("description", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.description = Some(text.trim().to_string());
                }
            }
            ("link", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    let url = text.trim();
                    if self.config.validate_urls {
                        validate_url(url).map_err(|_| {
                            AppError::from(ParseError::new(
                                ParseErrorKind::InvalidFormat,
                                "Invalid episode link URL",
                                &state.context.url,
                                None,
                            ))
                        })?;
                    }
                    episode.link = Some(url.to_string());
                }
            }
            ("pubDate", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.pub_date = parse_date(text.trim());
                }
            }
            ("guid", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.guid = Some(text.trim().to_string());
                }
            }
            ("itunes:duration", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.duration = Some(text.trim().to_string());
                }
            }

            ("itunes:author", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.author = Some(text.trim().to_string());
                }
            }
            ("itunes:subtitle", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.subtitle = Some(text.trim().to_string());
                }
            }
            ("itunes:summary", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.summary = Some(text.trim().to_string());
                }
            }
            ("itunes:explicit", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    episode.explicit = parse_bool(text.trim());
                }
            }
            ("itunes:duration", ParsingState::InItem) => {
                if let Some(episode) = &mut state.current_episode {
                    // TODO: Parse duration
                }
            }
            ("link", ParsingState::InChannelLink) => {
                if let Some(podcast) = &mut state.podcast {
                    let url = text.trim();
                    if self.config.validate_urls {
                        validate_url(url).map_err(|_| {
                            AppError::from(ParseError::new(
                                ParseErrorKind::InvalidFormat,
                                "Invalid link URL",
                                &state.context.url,
                                None,
                            ))
                        })?;
                    }
                    podcast.link = Some(url.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_image_and_category(
        &self,
        state: &mut RssParserState,
        e: &BytesStart,
    ) -> Result<(), AppError> {
        // 处理 link 标签的属性
        for attr in e.attributes() {
            let attr = attr.map_err(|e| {
                AppError::from(ParseError::new(
                    ParseErrorKind::InvalidXml,
                    "Failed to parse link attribute",
                    &state.context.url,
                    Some(Box::new(e)),
                ))
            })?;

            let original = String::from_utf8_lossy(attr.value.as_ref()).into_owned();
            let value = handle_xml_error(attr.unescape_value(), Some(original))?.into_owned();
            match attr.key.as_ref() {
                b"href" => {
                    let url = value;
                    if self.config.validate_urls {
                        validate_url(&url).map_err(|_| {
                            AppError::from(ParseError::new(
                                ParseErrorKind::InvalidFormat,
                                "Invalid link URL",
                                &state.context.url,
                                None,
                            ))
                        })?;
                    }
                    debug_info!("IMAGE URL", &url, &state);
                    match &state.current_state {
                        ParsingState::InChannel => {
                            let podcast = state.podcast.as_mut().ok_or_else(|| {
                                AppError::from(ParseError::new(
                                    ParseErrorKind::Other,
                                    "image_url tag found outside of podcast context",
                                    &state.context.url,
                                    None,
                                ))
                            })?;
                            podcast.image_url = Some(url);
                        }
                        ParsingState::InItem => {
                            let episode = state.current_episode.as_mut().ok_or_else(|| {
                                AppError::from(ParseError::new(
                                    ParseErrorKind::Other,
                                    "image_url tag found outside of episode context",
                                    &state.context.url,
                                    None,
                                ))
                            })?;
                            episode.episode_image_url = Some(url);
                        }
                        _ => {}
                    }
                    return Ok(());
                }
                b"text" => {
                    let podcast = state.podcast.as_mut().ok_or_else(|| {
                        AppError::from(ParseError::new(
                            ParseErrorKind::Other,
                            "image_url tag found outside of podcast context",
                            &state.context.url,
                            None,
                        ))
                    })?;
                    podcast
                        .category
                        .get_or_insert_with(Vec::new)
                        .push(Some(value.trim().to_string()));
                }
                _ => {
                    debug!("Ignoring unknown link attribute: {:?}", attr.key);
                }
            }
        }

        // 如果没有 href 属性，等待处理标签内容
        state.current_state = ParsingState::InChannelLink;

        Ok(())
    }
}

#[async_trait]
impl FeedParser<(NewPodcast, Vec<NewEpisode>)> for RssFeedParser {
    async fn parse(&self, content: &[u8], url: &str) -> AppResult<(NewPodcast, Vec<NewEpisode>)> {
        let cursor = std::io::Cursor::new(content);
        self.parse_internal(cursor, url).await
    }
}

/// Parse boolean value from string
pub fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

/// Clean HTML content
pub fn clean_html(content: &str) -> String {
    use ammonia::clean;

    // 使用 ammonia 清理 HTML，只保留安全的标签和属性
    clean(content)
}

/// Validate URL format
pub fn validate_url(url: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use url::Url;

    if let Ok(url) = Url::parse(url) {
        if url.scheme() == "http" || url.scheme() == "https" {
            return Ok(());
        }
    }
    Err(Box::new(url::ParseError::EmptyHost))
}

/// Parse date string to DateTime<Utc>
pub fn parse_date(date_str: &str) -> Option<DateTime<Utc>> {
    use chrono::prelude::*;

    let parse_with_format = |format: &str| {
        NaiveDateTime::parse_from_str(date_str, format)
            .map(|dt| Utc.from_utc_datetime(&dt))
            .ok()
    };

    // RFC 2822
    if let Ok(date) = DateTime::parse_from_rfc2822(date_str) {
        return Some(date.into());
    }

    // RFC 3339 / ISO 8601
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        return Some(date.into());
    }

    // 自定义格式
    let formats = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d"];

    for format in formats {
        if let Some(date) = parse_with_format(format) {
            return Some(date);
        }
    }

    warn!("Failed to parse date: {}", date_str);
    None
}

fn handle_xml_error<T>(
    result: Result<T, quick_xml::Error>,
    fallback: Option<String>,
) -> Result<T, AppError>
where
    T: From<String>,
{
    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            debug!("XML parsing error: {}", e);
            match e {
                quick_xml::Error::EscapeError(e1) => {
                    // 对于转义错误，使用传入的原始值
                    if let Some(original) = fallback {
                        return Ok(T::from(original));
                    }
                    Err(AppError::from(ParseError::new(
                        ParseErrorKind::InvalidXml,
                        "Failed to get original value",
                        "",
                        Some(Box::new(e1)),
                    )))
                }
                _ => Err(AppError::from(ParseError::new(
                    ParseErrorKind::InvalidXml,
                    format!("XML parsing error: {}", e),
                    "",
                    Some(Box::new(e)),
                ))),
            }
        }
    }
}
