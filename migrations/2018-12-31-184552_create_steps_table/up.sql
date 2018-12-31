CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE TABLE steps (
  time    TIMESTAMPTZ   NOT NULL,
  user_id UUID          REFERENCES users(id) NOT NULL,
  source  TEXT          NOT NULL, /* probably just fitbit... */
  count   INTEGER       CHECK (count >= 0),
  PRIMARY KEY (user_id, time)
);

CREATE INDEX ON steps (user_id, time DESC);

SELECT create_hypertable('steps', 'time');
