CREATE TABLE tokens (
  user_id UUID REFERENCES users(id),
  service TEXT NOT NULL, /* fitbit, google, etc */
  service_userid TEXT NOT NULL,
  access_token TEXT NOT NULL,
  access_token_expiry TIMESTAMP NOT NULL,
  refresh_token TEXT NOT NULL
);