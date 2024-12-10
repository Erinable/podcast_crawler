// @generated automatically by Diesel CLI.

diesel::table! {
    episode_rank (id) {
        id -> Int4,
        title -> Nullable<Varchar>,
        podcast_id -> Nullable<Varchar>,
        podcast_name -> Nullable<Varchar>,
        logo_url -> Nullable<Varchar>,
        link -> Nullable<Varchar>,
        play_count -> Nullable<Int4>,
        comment_count -> Nullable<Int4>,
        subscription -> Nullable<Int4>,
        duration -> Nullable<Int4>,
        post_time -> Nullable<Timestamptz>,
        primary_genre_name -> Nullable<Varchar>,
        total_episodes_count -> Nullable<Int4>,
        open_rate -> Nullable<Float8>,
        last_release_date_day_count -> Nullable<Float8>,
    }
}

diesel::table! {
    episodes (episode_id) {
        episode_id -> Int4,
        podcast_id -> Nullable<Int4>,
        #[max_length = 1024]
        episode_image_url -> Nullable<Varchar>,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 1024]
        link -> Nullable<Varchar>,
        pub_date -> Nullable<Timestamptz>,
        #[max_length = 255]
        guid -> Nullable<Varchar>,
        #[max_length = 1024]
        enclosure_url -> Nullable<Varchar>,
        #[max_length = 50]
        enclosure_type -> Nullable<Varchar>,
        enclosure_length -> Nullable<Int8>,
        explicit -> Nullable<Bool>,
        subtitle -> Nullable<Text>,
        #[max_length = 255]
        author -> Nullable<Varchar>,
        summary -> Nullable<Text>,
        keywords -> Nullable<Array<Nullable<Text>>>,
        category -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 255]
        duration -> Nullable<Varchar>,
    }
}

diesel::table! {
    podcast_rank (id) {
        id -> Varchar,
        rank -> Nullable<Int4>,
        name -> Nullable<Varchar>,
        logo_url -> Nullable<Varchar>,
        primary_genre_name -> Nullable<Varchar>,
        authors_text -> Nullable<Varchar>,
        track_count -> Nullable<Int4>,
        last_release_date -> Nullable<Timestamptz>,
        last_release_date_day_count -> Nullable<Float8>,
        first_episode_post_time -> Nullable<Timestamptz>,
        active_rate -> Nullable<Float8>,
        avg_duration -> Nullable<Int4>,
        avg_play_count -> Nullable<Int4>,
        avg_update_freq -> Nullable<Int4>,
        avg_comment_count -> Nullable<Int4>,
        avg_interact_indicator -> Nullable<Float8>,
        avg_open_rate -> Nullable<Float8>,
        links -> Nullable<Jsonb>,
    }
}

diesel::table! {
    podcasts (podcast_id) {
        podcast_id -> Int4,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        #[max_length = 1024]
        link -> Nullable<Varchar>,
        last_build_date -> Nullable<Timestamptz>,
        #[max_length = 20]
        language -> Nullable<Varchar>,
        #[max_length = 255]
        copyright -> Nullable<Varchar>,
        #[max_length = 1024]
        image_url -> Nullable<Varchar>,
        #[max_length = 1024]
        rss_feed_url -> Nullable<Varchar>,
        category -> Nullable<Array<Nullable<Text>>>,
        #[max_length = 255]
        author -> Nullable<Varchar>,
        #[max_length = 255]
        owner_name -> Nullable<Varchar>,
        #[max_length = 255]
        owner_email -> Nullable<Varchar>,
        keywords -> Nullable<Array<Nullable<Text>>>,
        explicit -> Nullable<Bool>,
        summary -> Nullable<Text>,
        subtitle -> Nullable<Text>,
    }
}

diesel::joinable!(episodes -> podcasts (podcast_id));

diesel::allow_tables_to_appear_in_same_query!(episode_rank, episodes, podcast_rank, podcasts,);
