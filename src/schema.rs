// @generated automatically by Diesel CLI.

diesel::table! {
    chat_pairs (id) {
        id -> Varchar,
        user1 -> Varchar,
        user2 -> Varchar,
        last_message -> Nullable<Varchar>,
        all_read -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    chats (id) {
        id -> Varchar,
        sender -> Varchar,
        receiver -> Varchar,
        message -> Varchar,
        image -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        pair_id -> Varchar,
    }
}

diesel::table! {
    comments (id) {
        id -> Varchar,
        user_name -> Varchar,
        post_id -> Varchar,
        text -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    friend_requests (id) {
        id -> Varchar,
        user_name -> Varchar,
        requester -> Varchar,
        status -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    friends (id) {
        id -> Int4,
        user_username -> Varchar,
        friend_username -> Varchar,
    }
}

diesel::table! {
    likes (user_name, post_id) {
        user_name -> Varchar,
        post_id -> Varchar,
    }
}

diesel::table! {
    posts (id) {
        id -> Varchar,
        text -> Text,
        image -> Nullable<Varchar>,
        user_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    profiles (id) {
        id -> Varchar,
        user_name -> Varchar,
        bio -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        image -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        app_f_token -> Nullable<Varchar>,
    }
}

diesel::table! {
    system_data (id) {
        id -> Int4,
        price -> Numeric,
        android_app_version -> Varchar,
        trivia_win_amount -> Numeric,
    }
}

diesel::table! {
    users (id) {
        id -> Varchar,
        user_name -> Varchar,
        email -> Nullable<Varchar>,
        code -> Nullable<Int4>,
        created_at -> Timestamp,
        user_type -> Varchar,
        password_hash -> Text,
        is_deleted -> Bool,
    }
}

diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(likes -> posts (post_id));

diesel::allow_tables_to_appear_in_same_query!(
    chat_pairs,
    chats,
    comments,
    friend_requests,
    friends,
    likes,
    posts,
    profiles,
    system_data,
    users,
);
