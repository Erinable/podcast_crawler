use podcast_crawler::crawler::{rss::RssFeedParser, HttpCrawler};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn integration_test_crawler_with_rss_parser() {
    // Start the mock server
    let mock_server = MockServer::start().await;

    // Debug print for mock server URI
    println!("Mock Server URI: {}", mock_server.uri());

    // Mount XML resources from the tests/data directory
    let xml_feeds = ["complex_feed.xml", "xiaoyuzhou.xml"];
    for (i, &file) in xml_feeds.iter().enumerate() {
        println!("Mounting mock for file: {}, path: /feed{}", file, i + 1);
        let body_content = match file {
            "complex_feed.xml" => include_str!("../tests/data/complex_feed.xml"),
            "xiaoyuzhou.xml" => include_str!("../tests/data/xiaoyuzhou.xml"),
            _ => panic!("Unknown feed file"),
        };

        println!("Mock Response Body Length: {}", body_content.len());

        let mock_response = ResponseTemplate::new(200)
            .set_body_string(body_content)
            .append_header("Content-Type", "application/xml")
            .append_header("Accept", "application/xml")
            .append_header("Connection", "keep-alive")
            .append_header("Keep-Alive", "timeout=10")
            .append_header("X-Wiremock-Test", "true")
            .append_header("Server", "WireMock")
            .append_header("Date", "Wed, 21 Jan 2026 11:32:10 GMT");
        // .append_header("Content-Length", body_content.len().to_string());

        Mock::given(method("GET"))
            .and(path(format!("/feed{}", i + 1)))
            .respond_with(mock_response)
            .mount(&mock_server)
            .await;
    }

    // Create URLs for the mock server
    let urls = xml_feeds
        .iter()
        .enumerate()
        .map(|(i, _)| format!("{}/feed{}", mock_server.uri(), i + 1))
        .collect::<Vec<_>>();

    println!("Generated URLs: {:?}", urls);

    // Initialize the RSS parser and HTTP crawler
    let parser = RssFeedParser::new();
    let crawler = HttpCrawler::new(parser, 2);

    // Perform the crawl
    let results = match crawler.crawl_batch(urls).await {
        Ok(res) => res,
        Err(err) => {
            println!("Crawl batch error: {:?}", err);
            panic!("Crawl batch failed");
        }
    };
    println!("Results: {:?}", results);

    // Debug print for each result
    for (i, result) in results.iter().enumerate() {
        println!(
            "Result {}: success = {}, error = {:?}",
            i, result.success, result.error_message
        );
    }

    // Validate the results
    assert_eq!(
        results.len(),
        xml_feeds.len(),
        "Should have results for each feed"
    );
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.success,
            "Feed{} should be parsed successfully",
            i + 1
        );
        assert!(
            result.parsed_data.is_some(),
            "Feed{} should have parsed data",
            i + 1
        );
        let parsed_data = result.parsed_data.as_ref().unwrap();
        println!("Parsed Podcast Title: {}", parsed_data.0.title);

        // Check the title of the parsed podcast
        let expected_title = match i {
            0 => "Tech Talks Daily Podcast",
            1 => "其他垃圾",
            _ => "",
        };
        assert_eq!(
            parsed_data.0.title,
            expected_title,
            "Feed{} title should match expected title",
            i + 1
        );

        // Validate the enclosure information for each episode
        let episode = &parsed_data.1[0]; // Assuming we're checking the first episode

        // Add assertions for enclosure fields
        let expected_enclosure_url = match i {
            0 => "https://media.example.com/episodes/future-ai-2024.mp3",
            1 => "https://dts-api.xiaoyuzhoufm.com/track/640599e78966402d7e9c6dbb/67371bf343dc3a4387e3094e/media.xyzcdn.net/ltdtG7FLOUAvMDMn5d9wouIDgvFR.m4a",
            _ => "",
        };
        assert_eq!(
            episode
                .enclosure_url
                .as_ref()
                .expect("enclosure_url should be Some"),
            expected_enclosure_url,
            "Feed{} enclosure URL should match expected URL",
            i + 1
        );

        let expected_enclosure_type = match i {
            0 => "audio/mpeg",
            1 => "audio/mp4",
            _ => "",
        };
        assert_eq!(
            episode
                .enclosure_type
                .as_ref()
                .expect("enclosure_type should be Some"),
            expected_enclosure_type,
            "Feed{} enclosure type should match expected type",
            i + 1
        );

        let expected_enclosure_length = match i {
            0 => 58725344, // Replace with actual length
            1 => 73940105,
            _ => 0,
        };
        assert_eq!(
            episode
                .enclosure_length
                .expect("enclosure_length should be Some"),
            expected_enclosure_length,
            "Feed{} enclosure length should match expected length",
            i + 1
        );
    }
}
