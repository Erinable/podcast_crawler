use chrono::Datelike;
use podcast_crawler::crawler::rss::{
    clean_html, parse_bool, parse_date, validate_url, RssFeedParser,
};

use podcast_crawler::crawler::traits::FeedParser;

#[tokio::test]
async fn test_parse_rss() {
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>Test Podcast</title>
                <description>Test Description</description>
                <link>https://example.com</link>
                <item>
                    <title>Test Episode</title>
                    <description>Episode Description</description>
                    <pubDate>Wed, 04 Dec 2024 10:06:00 GMT</pubDate>
                    <enclosure length="58495109" type="audio/x-m4a" url="https://jt.ximalaya.com/GKwRIRwLJTZJAVQGqQM6aIx4.m4a?channel=rss&amp;album_id=20527677&amp;track_id=780798209&amp;uid=139127380&amp;jt=https://aod.cos.tx.xmcdn.com/storages/96a7-audiofreehighqps/89/D2/GKwRIRwLJTZJAVQGqQM6aIx4.m4a" />
                </item>
            </channel>
        </rss>"#;

    let parser = RssFeedParser::new();
    let (podcast, episodes) = parser
        .parse(rss.as_bytes(), "https://example.com/feed.xml")
        .await
        .unwrap();
    print!("{:#?}", podcast);
    print!("{:#?}", episodes);

    assert_eq!(podcast.title, "Test Podcast".to_string());
    assert_eq!(podcast.description, Some("Test Description".to_string()));
    assert_eq!(podcast.link, Some("https://example.com".to_string()));

    let episode = &episodes[0];
    assert_eq!(episode.title, "Test Episode".to_string());
    assert_eq!(episode.description, Some("Episode Description".to_string()));
    assert_eq!(
            episode.enclosure_url,
            Some("https://jt.ximalaya.com/GKwRIRwLJTZJAVQGqQM6aIx4.m4a?channel=rss&album_id=20527677&track_id=780798209&uid=139127380&jt=https://aod.cos.tx.xmcdn.com/storages/96a7-audiofreehighqps/89/D2/GKwRIRwLJTZJAVQGqQM6aIx4.m4a".to_string())
        );
    assert_eq!(episode.enclosure_type, Some("audio/x-m4a".to_string()));
    assert_eq!(episode.enclosure_length, Some(58495109));
    assert_eq!(episode.link, Some("https://example.com".to_string()));
    assert_eq!(episode.explicit, Some(false));
}

#[tokio::test]
async fn test_parse_ximalaya_rss() {
    let parser = RssFeedParser::new();
    let xml_content = std::fs::read_to_string("tests/data/ximalaya.xml").unwrap();
    let url = "https://www.ximalaya.com/album/20527677.xml";

    let result = parser.parse(xml_content.as_bytes(), url).await;
    assert!(result.is_ok(), "Failed to parse Ximalaya RSS feed");

    let (podcast, episodes) = result.unwrap();

    // Verify podcast metadata
    assert_eq!(podcast.title, "能量棒".to_string());
    assert_eq!(
        podcast.link,
        Some("https://www.ximalaya.com/album/20527677".to_string())
    );
    assert_eq!(podcast.language, Some("zh-cn".to_string()));
    assert_eq!(podcast.author, Some("雨荷能量棒".to_string()));
    assert_eq!(podcast.owner_name, Some("雨荷能量棒".to_string()));
    assert_eq!(
        podcast.owner_email,
        Some("xzsydney@hotmail.com".to_string())
    );
    assert_eq!(podcast.image_url, Some("https://fdfs.xmcdn.com/storages/0a92-audiofreehighqps/4E/FC/GMCoOSQG2hUGAAKA8AGWNFf8.jpeg".to_string()));
    assert_eq!(podcast.explicit, Some(false));

    // Verify episode data
    assert_eq!(episodes.len(), 1);
    let episode = &episodes[0];
    assert_eq!(
        episode.title,
        "68. 你的职业面具是什么颜色？上班穿西装还是防弹背心？".to_string()
    );
    assert!(episode.description.as_ref().unwrap().contains("上班累吗？"));
    assert_eq!(
            episode.enclosure_url,
            Some("https://jt.ximalaya.com//GKwRIRwLJTZJAVQGqQM6aIx4.m4a?channel=rss&album_id=20527677&track_id=780798209&uid=139127380&jt=https://aod.cos.tx.xmcdn.com/storages/96a7-audiofreehighqps/89/D2/GKwRIRwLJTZJAVQGqQM6aIx4.m4a".to_string())
        );
    assert_eq!(episode.enclosure_type, Some("audio/x-m4a".to_string()));
    assert_eq!(episode.enclosure_length, Some(58495109));
    assert_eq!(
        episode.link,
        Some("https://www.ximalaya.com/sound/780798209".to_string())
    );
    assert_eq!(episode.explicit, Some(false));
    assert_eq!(episode.episode_image_url, Some("https://fdfs.xmcdn.com/storages/49d2-audiofreehighqps/2B/DD/GKwRIJEG2hRIAAEQKQGWM_Kd.jpeg".to_string()));

    // Verify the publication date
    if let Some(pub_date) = episode.pub_date {
        assert_eq!(pub_date.to_rfc2822(), "Wed, 04 Dec 2024 10:06:00 +0000");
    } else {
        panic!("Publication date is missing");
    }
}

#[tokio::test]
async fn test_parse_rss_with_cdata() {
    let rss_content = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>Test Podcast</title>
                <description><![CDATA[This is a <b>description</b> with CDATA]]></description>
                <link>https://example.com</link>
                <item>
                    <title>Test Episode</title>
                    <description><![CDATA[Episode <strong>description</strong> with CDATA]]></description>
                    <enclosure url="http://example.com/audio.mp3" type="audio/mpeg" length="1234"/>
                </item>
            </channel>
        </rss>"#;

    let parser = RssFeedParser::new();
    let (podcast, episodes) = parser
        .parse(rss_content.as_bytes(), "http://example.com/feed.xml")
        .await
        .unwrap();

    // Check podcast fields
    assert_eq!(podcast.title, "Test Podcast".to_string());
    assert_eq!(
        podcast.description,
        Some("This is a <b>description</b> with CDATA".to_string())
    );

    // Check episode fields
    let episode = &episodes[0];
    assert_eq!(episode.title, "Test Episode".to_string());
    assert_eq!(
        episode.description,
        Some("Episode <strong>description</strong> with CDATA".to_string())
    );
    assert_eq!(
        episode.enclosure_url,
        Some("http://example.com/audio.mp3".to_string())
    );
    assert_eq!(episode.enclosure_type, Some("audio/mpeg".to_string()));
    assert_eq!(episode.enclosure_length, Some(1234));
}

#[test]
fn test_parse_bool() {
    assert_eq!(parse_bool("true"), Some(true));
    assert_eq!(parse_bool("yes"), Some(true));
    assert_eq!(parse_bool("1"), Some(true));
    assert_eq!(parse_bool("false"), Some(false));
    assert_eq!(parse_bool("no"), Some(false));
    assert_eq!(parse_bool("0"), Some(false));
    assert_eq!(parse_bool("invalid"), None);
}

#[test]
fn test_parse_date() {
    // RFC 2822
    let date = parse_date("Wed, 04 Dec 2024 10:06:00 GMT").unwrap();
    assert_eq!(date.year(), 2024);
    assert_eq!(date.month(), 12);
    assert_eq!(date.day(), 4);

    // ISO 8601
    let date = parse_date("2024-12-04T10:06:00Z").unwrap();
    assert_eq!(date.year(), 2024);
    assert_eq!(date.month(), 12);
    assert_eq!(date.day(), 4);

    // 自定义格式
    let date = parse_date("2024-12-04 10:06:00").unwrap();
    assert_eq!(date.year(), 2024);
    assert_eq!(date.month(), 12);
    assert_eq!(date.day(), 4);

    // 无效格式
    assert!(parse_date("invalid date").is_none());
}

#[test]
fn test_clean_html() {
    let html =
        r#"<p>Hello <script>alert('xss')</script><a href="http://example.com">world</a>!</p>"#;
    let cleaned = clean_html(html);
    assert!(!cleaned.contains("script"));
    assert!(cleaned.contains("Hello"));
    assert!(cleaned.contains("world"));
    assert!(cleaned.contains("href"));
}

#[test]
fn test_validate_url() {
    assert!(validate_url("https://example.com").is_ok());
    assert!(validate_url("http://example.com/feed.xml").is_ok());
    assert!(validate_url("not a url").is_err());
    println!("{:?}", url::Url::parse("a:////invalid"));
    assert!(validate_url("a:////invalid").is_err());
}
