-- Add up migration script here
create type gender as enum (
    'female',
    'male',
    'unknown'
);

create table user_stats (
    email varchar(128) not null primary key,
    name varchar(64) not null,
    gender gender default 'unknown',
    created_at timestamptz not null default current_timestamp,
    last_visited_at timestamptz,
    last_watched_at timestamptz,
    recent_watched int[],
    viewed_but_not_started int[],
    started_but_not_finished int[],
    finished int[],
    last_email_notification timestamptz,
    last_in_app_notification timestamptz,
    last_sms_notification timestamptz
);

create index user_stats_created_at_idx on user_stats (created_at);
create index user_stats_last_visited_at_idx on user_stats (last_visited_at);
create index user_stats_last_watched_at_idx on user_stats (last_watched_at);
create index user_stats_recent_watched_idx on user_stats using gin (recent_watched);
create index user_stats_viewed_but_not_started_idx on user_stats using gin (viewed_but_not_started);
create index user_stats_started_but_not_finished_idx on user_stats using gin (started_but_not_finished);
create index user_stats_last_email_notification_idx on user_stats (last_email_notification);
create index user_stats_last_in_app_notification_idx on user_stats (last_in_app_notification);
create index user_stats_last_sms_notification_idx on user_stats (last_sms_notification);
