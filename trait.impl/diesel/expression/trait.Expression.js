(function() {
    var implementors = Object.fromEntries([["podcast_crawler",[["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.comment_count.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::comment_count\">comment_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.duration.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::duration\">duration</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.id.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::id\">id</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.last_release_date_day_count.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::last_release_date_day_count\">last_release_date_day_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.link.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::link\">link</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.logo_url.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::logo_url\">logo_url</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.open_rate.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::open_rate\">open_rate</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.play_count.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::play_count\">play_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.podcast_id.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::podcast_id\">podcast_id</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.podcast_name.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::podcast_name\">podcast_name</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.post_time.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::post_time\">post_time</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.primary_genre_name.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::primary_genre_name\">primary_genre_name</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.star.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::star\">star</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.subscription.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::subscription\">subscription</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.title.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::title\">title</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episode_rank/columns/struct.total_episodes_count.html\" title=\"struct podcast_crawler::schema::episode_rank::columns::total_episodes_count\">total_episodes_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.author.html\" title=\"struct podcast_crawler::schema::episodes::columns::author\">author</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.category.html\" title=\"struct podcast_crawler::schema::episodes::columns::category\">category</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.description.html\" title=\"struct podcast_crawler::schema::episodes::columns::description\">description</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.duration.html\" title=\"struct podcast_crawler::schema::episodes::columns::duration\">duration</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.enclosure_length.html\" title=\"struct podcast_crawler::schema::episodes::columns::enclosure_length\">enclosure_length</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.enclosure_type.html\" title=\"struct podcast_crawler::schema::episodes::columns::enclosure_type\">enclosure_type</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.enclosure_url.html\" title=\"struct podcast_crawler::schema::episodes::columns::enclosure_url\">enclosure_url</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.episode_id.html\" title=\"struct podcast_crawler::schema::episodes::columns::episode_id\">episode_id</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.episode_image_url.html\" title=\"struct podcast_crawler::schema::episodes::columns::episode_image_url\">episode_image_url</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.explicit.html\" title=\"struct podcast_crawler::schema::episodes::columns::explicit\">explicit</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.guid.html\" title=\"struct podcast_crawler::schema::episodes::columns::guid\">guid</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.keywords.html\" title=\"struct podcast_crawler::schema::episodes::columns::keywords\">keywords</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.link.html\" title=\"struct podcast_crawler::schema::episodes::columns::link\">link</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.podcast_id.html\" title=\"struct podcast_crawler::schema::episodes::columns::podcast_id\">podcast_id</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.pub_date.html\" title=\"struct podcast_crawler::schema::episodes::columns::pub_date\">pub_date</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.star.html\" title=\"struct podcast_crawler::schema::episodes::columns::star\">star</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.subtitle.html\" title=\"struct podcast_crawler::schema::episodes::columns::subtitle\">subtitle</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.summary.html\" title=\"struct podcast_crawler::schema::episodes::columns::summary\">summary</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/episodes/columns/struct.title.html\" title=\"struct podcast_crawler::schema::episodes::columns::title\">title</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.active_rate.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::active_rate\">active_rate</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.authors_text.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::authors_text\">authors_text</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.avg_comment_count.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::avg_comment_count\">avg_comment_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.avg_duration.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::avg_duration\">avg_duration</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.avg_interact_indicator.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::avg_interact_indicator\">avg_interact_indicator</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.avg_open_rate.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::avg_open_rate\">avg_open_rate</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.avg_play_count.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::avg_play_count\">avg_play_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.avg_update_freq.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::avg_update_freq\">avg_update_freq</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.first_episode_post_time.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::first_episode_post_time\">first_episode_post_time</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.id.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::id\">id</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.last_release_date.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::last_release_date\">last_release_date</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.last_release_date_day_count.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::last_release_date_day_count\">last_release_date_day_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.links.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::links\">links</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.logo_url.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::logo_url\">logo_url</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.name.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::name\">name</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.primary_genre_name.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::primary_genre_name\">primary_genre_name</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.rank.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::rank\">rank</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.star.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::star\">star</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcast_rank/columns/struct.track_count.html\" title=\"struct podcast_crawler::schema::podcast_rank::columns::track_count\">track_count</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.author.html\" title=\"struct podcast_crawler::schema::podcasts::columns::author\">author</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.category.html\" title=\"struct podcast_crawler::schema::podcasts::columns::category\">category</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.copyright.html\" title=\"struct podcast_crawler::schema::podcasts::columns::copyright\">copyright</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.description.html\" title=\"struct podcast_crawler::schema::podcasts::columns::description\">description</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.explicit.html\" title=\"struct podcast_crawler::schema::podcasts::columns::explicit\">explicit</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.image_url.html\" title=\"struct podcast_crawler::schema::podcasts::columns::image_url\">image_url</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.keywords.html\" title=\"struct podcast_crawler::schema::podcasts::columns::keywords\">keywords</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.language.html\" title=\"struct podcast_crawler::schema::podcasts::columns::language\">language</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.last_build_date.html\" title=\"struct podcast_crawler::schema::podcasts::columns::last_build_date\">last_build_date</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.link.html\" title=\"struct podcast_crawler::schema::podcasts::columns::link\">link</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.owner_email.html\" title=\"struct podcast_crawler::schema::podcasts::columns::owner_email\">owner_email</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.owner_name.html\" title=\"struct podcast_crawler::schema::podcasts::columns::owner_name\">owner_name</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.podcast_id.html\" title=\"struct podcast_crawler::schema::podcasts::columns::podcast_id\">podcast_id</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.rss_feed_url.html\" title=\"struct podcast_crawler::schema::podcasts::columns::rss_feed_url\">rss_feed_url</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.star.html\" title=\"struct podcast_crawler::schema::podcasts::columns::star\">star</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.subtitle.html\" title=\"struct podcast_crawler::schema::podcasts::columns::subtitle\">subtitle</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.summary.html\" title=\"struct podcast_crawler::schema::podcasts::columns::summary\">summary</a>"],["impl Expression for <a class=\"struct\" href=\"podcast_crawler/schema/podcasts/columns/struct.title.html\" title=\"struct podcast_crawler::schema::podcasts::columns::title\">title</a>"]]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[14953]}