CREATE TABLE tokens (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id) NOT NULL,
  service TEXT NOT NULL, /* fitbit, google, etc */
  service_userid TEXT NOT NULL,
  access_token TEXT NOT NULL,
  access_token_expiry TIMESTAMPTZ NOT NULL,
  refresh_token TEXT NOT NULL
);