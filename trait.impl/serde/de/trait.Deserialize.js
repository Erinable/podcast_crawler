(function() {
    var implementors = Object.fromEntries([["podcast_crawler",[["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/crawler_refactor/task_management_system/struct.ResultData.html\" title=\"struct podcast_crawler::crawler_refactor::task_management_system::ResultData\">ResultData</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/config/crawler/struct.CrawlerConfig.html\" title=\"struct podcast_crawler::infrastructure::config::crawler::CrawlerConfig\">CrawlerConfig</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/config/database/struct.DatabaseConfig.html\" title=\"struct podcast_crawler::infrastructure::config::database::DatabaseConfig\">DatabaseConfig</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/config/logging/struct.LoggingConfig.html\" title=\"struct podcast_crawler::infrastructure::config::logging::LoggingConfig\">LoggingConfig</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/config/server/struct.ServerConfig.html\" title=\"struct podcast_crawler::infrastructure::config::server::ServerConfig\">ServerConfig</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/config/struct.Settings.html\" title=\"struct podcast_crawler::infrastructure::config::Settings\">Settings</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/episode/struct.Episode.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::episode::Episode\">Episode</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/episode/struct.NewEpisode.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::episode::NewEpisode\">NewEpisode</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/episode/struct.UpdateEpisode.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::episode::UpdateEpisode\">UpdateEpisode</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast/struct.NewPodcast.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast::NewPodcast\">NewPodcast</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast/struct.Podcast.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast::Podcast\">Podcast</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast/struct.UpdatePodcast.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast::UpdatePodcast\">UpdatePodcast</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast_rank_model/struct.Link.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast_rank_model::Link\">Link</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast_rank_model/struct.NewPodcastRank.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast_rank_model::NewPodcastRank\">NewPodcastRank</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast_rank_model/struct.PodcastRank.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast_rank_model::PodcastRank\">PodcastRank</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/infrastructure/persistence/models/podcast_rank_model/struct.UpdatePodcastRank.html\" title=\"struct podcast_crawler::infrastructure::persistence::models::podcast_rank_model::UpdatePodcastRank\">UpdatePodcastRank</a>"],["impl&lt;'de&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.217/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"podcast_crawler/metrics/struct.AddTaskRequest.html\" title=\"struct podcast_crawler::metrics::AddTaskRequest\">AddTaskRequest</a>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[6682]}