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
    InPodcast,
    InEpisode,
    Finished,
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

    fn validate_podcast(&self, podcast: &NewPodcast) -> AppResult<()> {
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

    fn validate_episode(&self, episode: &NewEpisode) -> AppResult<()> {
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
    /// 严格模式
    strict_mode: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            clean_html: true,
            validate_urls: true,
            allow_empty_required: false,
            strict_mode: true,
        }
    }
}

impl RssFeedParser {
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    async fn parse_internal<R: BufRead>(
        &self,
        content: R,
        url: &str,
    ) -> AppResult<(NewPodcast, Vec<NewEpisode>)> {
        let mut reader = Reader::from_reader(content);
        // reader.trim_text(true);
        reader.expand_empty_elements(true); // 展开空标签

        let mut state = RssParserState::new(url.to_string());
        state.podcast = Some(NewPodcast {
            rss_feed_url: Some(url.to_string()),
            ..Default::default()
        });

        debug!("Starting RSS parsing for URL: {}", url);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let (tag_name, attributes) = self.extract_tag_info(&e)?;
                    state.current_tag = tag_name.clone();
                    state.context.push_element(tag_name.clone());
                    debug_info!("START EVENT", &state);
                    self.handle_start_event(&mut state, tag_name, attributes)?;
                }
                Ok(Event::End(e)) => {
                    debug_info!("END EVENT", &state);
                    self.handle_end_event(&mut state, &e)?;
                }
                Ok(Event::Empty(e)) => {
                    let (tag_name, attributes) = self.extract_tag_info(&e)?;
                    debug_info!("EMPTY EVENT", &state);
                    if tag_name == "enclosure" {
                        self.handle_enclosure(&mut state, attributes)?;
                    }
                }
                Ok(Event::Text(e)) => {
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
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidXml,
                        format!("Error at position {}: {:?}", reader.buffer_position(), e),
                        url,
                        Some(Box::new(e)),
                    )
                    .into())
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
                state.context.url.as_str(),
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
        state: &mut RssParserState,
        tag_name: String,
        attributes: Vec<(String, String)>,
    ) -> AppResult<()> {
        match tag_name.as_str() {
            "channel" => {
                state.current_state = ParsingState::InPodcast;
                state.podcast = Some(NewPodcast {
                    rss_feed_url: Some(state.context.url.clone()),
                    ..Default::default()
                });
            }
            "item" => {
                state.current_state = ParsingState::InEpisode;
                state.current_episode = Some(NewEpisode::default());
            }
            _ => {
                self.handle_start_event_internal(state, attributes)?;
            }
        }
        Ok(())
    }

    fn handle_start_event_internal(
        &self,
        state: &mut RssParserState,
        attributes: Vec<(String, String)>,
    ) -> AppResult<()> {
        match state.current_state {
            ParsingState::InPodcast => self.handle_podcast_start(state, attributes)?,
            ParsingState::InEpisode => self.handle_episode_start(state, attributes)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_text_event(&self, event: &BytesText, state: &mut RssParserState) -> AppResult<()> {
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

        match state.current_state {
            // Podcast 字段
            ParsingState::InPodcast => {
                self.handle_podcast_text(state, &text)?;
            }
            // Episode 字段
            ParsingState::InEpisode => {
                self.handle_episode_text(state, &text)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_end_event(&self, state: &mut RssParserState, event: &BytesEnd) -> AppResult<()> {
        let name = event.name();
        let tag_name = String::from_utf8_lossy(name.as_ref()).into_owned();
        state.context.pop_element();

        match (tag_name.as_str(), &state.current_state) {
            ("channel", ParsingState::InPodcast) => {
                state.current_state = ParsingState::Finished;
            }
            ("item", ParsingState::InEpisode) => {
                self.handle_item_end(state)?;
                state.current_state = ParsingState::InPodcast;
            }
            _ => {}
        }

        Ok(())
    }

    /// Extract tag name and attributes from a BytesStart event
    ///
    /// # Returns
    /// A tuple containing:
    /// - Tag name as a String
    /// - Vector of (attribute name, attribute value) pairs
    fn extract_tag_info(
        &self,
        event: &BytesStart,
    ) -> Result<(String, Vec<(String, String)>), AppError> {
        // Extract tag name
        let tag_name = String::from_utf8_lossy(event.name().as_ref()).to_string();

        // Extract attributes
        let attributes = event
            .attributes()
            .map(|attr_result| {
                // Convert each attribute to a (name, value) pair
                attr_result
                    .map(|attr| {
                        (
                            String::from_utf8_lossy(attr.key.as_ref()).to_string(),
                            String::from_utf8_lossy(attr.value.as_ref()).to_string(),
                        )
                    })
                    .map_err(|e| {
                        AppError::from(ParseError::new(
                            ParseErrorKind::InvalidXml,
                            format!("Failed to parse attribute for tag {}: {}", tag_name, e),
                            "",
                            Some(Box::new(e)),
                        ))
                    })
            })
            .collect::<Result<Vec<(String, String)>, AppError>>()?;

        Ok((tag_name, attributes))
    }

    fn handle_podcast_text(&self, state: &mut RssParserState, text: &str) -> AppResult<()> {
        let (tag_name, podcast_mut, feed_url) = get_context_as_mut(state)?;
        let podcast = podcast_mut
            .downcast_mut::<NewPodcast>()
            .ok_or_else(|| make_invalid_url_error(feed_url, "Podcast not found", None))?;
        match tag_name {
            "title" => update_field(&mut podcast.title, text),
            "description" => update_field_option(&mut podcast.description, text),
            "language" => update_field_option(&mut podcast.language, text),
            "copyright" => update_field_option(&mut podcast.copyright, text),
            "itunes:author" => update_field_option(&mut podcast.author, text),
            "itunes:name" => update_field_option(&mut podcast.owner_name, text),
            "itunes:email" => update_field_option(&mut podcast.owner_email, text),
            "itunes:category" => add_to_vec_option(&mut podcast.category, text),
            "itunes:keywords" => add_to_vec_option(&mut podcast.keywords, text),
            "itunes:explicit" => podcast.explicit = parse_bool(text),
            "itunes:summary" => update_field_option(&mut podcast.summary, text),
            "itunes:subtitle" => update_field_option(&mut podcast.subtitle, text),
            "link" => {
                self.check_url(text, feed_url)?;
                update_field_option(&mut podcast.link, text);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_episode_text(&self, state: &mut RssParserState, text: &str) -> AppResult<()> {
        let (tag_name, episode_mut, feed_url) = get_context_as_mut(state)?;
        let episode = episode_mut
            .downcast_mut::<NewEpisode>()
            .ok_or_else(|| make_invalid_url_error(feed_url, "Episode not found", None))?;
        match tag_name {
            "title" => update_field(&mut episode.title, text),
            "description" => update_field_option(&mut episode.description, text),
            "pubDate" => episode.pub_date = parse_date(text),
            "guid" => update_field_option(&mut episode.guid, text),
            "itunes:duration" => update_field_option(&mut episode.duration, text),
            "itunes:author" => update_field_option(&mut episode.author, text),
            "itunes:subtitle" => update_field_option(&mut episode.subtitle, text),
            "itunes:summary" => update_field_option(&mut episode.summary, text),
            "itunes:explicit" => episode.explicit = parse_bool(text),
            "link" => {
                self.check_url(text, feed_url)?;
                update_field_option(&mut episode.link, text);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_enclosure(
        &self,
        state: &mut RssParserState,
        attributes: Vec<(String, String)>,
    ) -> AppResult<()> {
        // debug_event_info!("Handling enclosure", &state.current_tag, &state);
        debug_info!("ENCLOSURE EVENT", &state);

        let episode = state.current_episode.as_mut().ok_or_else(|| {
            make_invalid_scope_error(
                state.context.url.as_str(),
                "Enclosure tag found outside of episode context",
            )
        })?;

        let mut found_url = ",url not found";
        let mut error_msg = String::new();
        for (key, value) in attributes {
            match key.as_str() {
                "url" => {
                    // 先解码 XML 实体
                    let decoded_url = value.replace("&amp;", "&");
                    let normalized_url = if decoded_url.starts_with("http") {
                        // 保持原始 URL 不变，因为喜马拉雅的 URL 包含特殊的查询参数
                        decoded_url
                    } else {
                        decoded_url.replace("//", "/")
                    };
                    self.check_url(&normalized_url, &state.context.url)?;
                    debug!("Found enclosure URL: {}", normalized_url);
                    update_field_option(&mut episode.enclosure_url, &normalized_url);
                    found_url = "";
                }
                "type" => {
                    debug!("Found enclosure type: {}", value);
                    update_field_option(&mut episode.enclosure_type, &value);
                }
                "length" => {
                    if let Ok(length) = value.parse() {
                        debug!("Found enclosure length: {}", length);
                        episode.enclosure_length = Some(length);
                    } else {
                        debug!("Failed to parse enclosure length: {}", value);
                        if self.config.strict_mode {
                            error_msg = format!("Invalid enclosure length: {}", value);
                        }
                    }
                }
                _ => {
                    debug!("Ignoring unknown enclosure attribute: {:?}", key);
                }
            }
        }
        error_msg.push_str(found_url);
        if !error_msg.is_empty() && self.config.strict_mode {
            return Err(AppError::from(ParseError::new(
                ParseErrorKind::MissingField,
                error_msg,
                &state.context.url,
                None,
            )));
        }

        Ok(())
    }

    fn handle_item_end(&self, state: &mut RssParserState) -> AppResult<()> {
        if let Some(episode) = state.current_episode.take() {
            debug!("Finishing episode: {:?}", episode);
            state.validate_episode(&episode)?;
            state.episodes.push(episode);
        }
        Ok(())
    }

    fn check_url(&self, text: &str, feed_url: &str) -> AppResult<()> {
        if self.config.validate_urls {
            validate_url(text).map_err(|e| {
                make_invalid_url_error(feed_url, &format!("Invalid link URL: {}", text), Some(e))
            })?;
        };
        Ok(())
    }

    fn handle_podcast_start(
        &self,
        state: &mut RssParserState,
        attributes: Vec<(String, String)>,
    ) -> AppResult<()> {
        let (tag_name, podcast_mut, feed_url) = get_context_as_mut(state)?;
        let podcast = podcast_mut
            .downcast_mut::<NewPodcast>()
            .ok_or_else(|| make_invalid_url_error(feed_url, "Podcast not found", None))?;
        match tag_name {
            "link" => {
                if let Some(url) = get_attribute_value(&attributes, "href") {
                    self.check_url(&url, feed_url)?;
                    update_field_option(&mut podcast.link, &url);
                }
            }
            "itunes:image" => {
                if let Some(url) = get_attribute_value(&attributes, "href") {
                    self.check_url(&url, feed_url)?;
                    update_field_option(&mut podcast.image_url, &url);
                }
            }
            "itunes:category" => {
                if let Some(text) = get_attribute_value(&attributes, "text") {
                    add_to_vec_option(&mut podcast.category, &text);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_episode_start(
        &self,
        state: &mut RssParserState,

        attributes: Vec<(String, String)>,
    ) -> AppResult<()> {
        let (tag_name, episode_mut, feed_url) = get_context_as_mut(state)?;
        let episode = episode_mut
            .downcast_mut::<NewEpisode>()
            .ok_or_else(|| make_invalid_url_error(feed_url, "Episode not found", None))?;
        match tag_name {
            "enclosure" => self.handle_enclosure(state, attributes)?,
            "itunes:image" => {
                if let Some(url) = get_attribute_value(&attributes, "href") {
                    self.check_url(&url, feed_url)?;
                    update_field_option(&mut episode.episode_image_url, &url);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn get_context_as_mut(
    state: &mut RssParserState,
) -> AppResult<(&str, &mut dyn std::any::Any, &str)> {
    let tag_name = state.current_tag.as_str(); // Get current tag name as a str slice
    let url = &state.context.url; // Borrow URL

    // Dynamically determine context type based on parsing state
    match state.current_state {
        ParsingState::InPodcast => state
            .podcast
            .as_mut()
            .map(|ctx| (tag_name, ctx as &mut dyn std::any::Any, url.as_str()))
            .ok_or_else(|| {
                make_invalid_scope_error(
                    url,
                    &format!("tag {} found outside of podcast context", tag_name),
                )
            }),
        ParsingState::InEpisode => state
            .current_episode
            .as_mut()
            .map(|ctx| (tag_name, ctx as &mut dyn std::any::Any, url.as_str()))
            .ok_or_else(|| {
                make_invalid_scope_error(
                    url,
                    &format!("tag {} found outside of episode context", tag_name),
                )
            }),
        _ => Err(make_invalid_scope_error(
            url,
            &format!("tag {} found outside of a valid context", tag_name),
        )),
    }
}

#[async_trait]
impl FeedParser<(NewPodcast, Vec<NewEpisode>)> for RssFeedParser {
    async fn parse(&self, content: &[u8], url: &str) -> AppResult<(NewPodcast, Vec<NewEpisode>)> {
        let cursor = std::io::Cursor::new(content);
        self.parse_internal(cursor, url).await
    }
}

fn make_invalid_url_error(
    url: &str,
    error_message: &str,
    error: Option<Box<dyn std::error::Error + Send + Sync>>,
) -> AppError {
    ParseError::new(ParseErrorKind::InvalidFormat, error_message, url, error).into()
}

fn make_invalid_scope_error(url: &str, error_message: &str) -> AppError {
    ParseError::new(ParseErrorKind::Other, error_message, url, None).into()
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

pub fn get_attribute_value(attrs: &[(String, String)], name: &str) -> Option<String> {
    attrs
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone())
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

fn update_field(field: &mut String, text: &str) {
    field.clear();
    field.push_str(text);
}

fn update_field_option(field: &mut Option<String>, text: &str) {
    *field = Some(text.to_string());
}

fn add_to_vec_option(field: &mut Option<Vec<Option<String>>>, text: &str) {
    field
        .get_or_insert_with(Vec::new)
        .push(Some(text.to_string()));
}
