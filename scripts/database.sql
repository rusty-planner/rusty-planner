-- Create the database tables

CREATE TABLE sessions (
  session_id BIGINT NOT NULL GENERATED ALWAYS AS IDENTITY,
  session_updated TIMESTAMP NOT NULL DEFAULT NOW(),
  session_expires OID NOT NULL,
  session_token VARCHAR NOT NULL,
  session_refresh_token VARCHAR NOT NULL,
  session_cookie UUID NOT NULL,
  discord_id VARCHAR NOT NULL,
  discord_name VARCHAR NOT NULL,
  discord_avatar VARCHAR,
  PRIMARY KEY(session_id, session_cookie)
);

CREATE TABLE groups (
  group_id BIGINT GENERATED ALWAYS AS IDENTITY,
  guild_id VARCHAR UNIQUE NOT NULL,
  group_name VARCHAR NOT NULL,
  group_description VARCHAR NOT NULL,
  group_image VARCHAR,
  PRIMARY KEY(group_id)
);

CREATE TABLE permissions (
  group_id BIGINT NOT NULL,
  role_id VARCHAR NOT NULL,
  permission VARCHAR NOT NULL,
  FOREIGN KEY(group_id) REFERENCES groups(group_id)
);

CREATE TABLE events (
  event_id BIGINT GENERATED ALWAYS AS IDENTITY,
  event_name VARCHAR NOT NULL,
  event_description VARCHAR NOT NULL,
  event_time TIMESTAMP NOT NULL,
  event_link VARCHAR,
  group_id BIGINT NOT NULL,
  PRIMARY KEY(event_id),
  FOREIGN KEY(group_id) REFERENCES groups(group_id)
);

CREATE TABLE event_reviews (
  event_id BIGINT NOT NULL,
  review_author VARCHAR NOT NULL,
  review_content VARCHAR,
  review_stars INT,
  FOREIGN KEY(event_id) REFERENCES events(event_id)
);

CREATE TABLE notifications (
  group_id BIGINT NOT NULL,
  notification_type VARCHAR NOT NULL,
  channel_id VARCHAR NOT NULL,
  FOREIGN KEY(group_id) REFERENCES groups(group_id)
);
