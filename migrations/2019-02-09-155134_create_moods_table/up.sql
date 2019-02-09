CREATE TABLE moods (
  time    TIMESTAMPTZ   NOT NULL,
  user_id UUID          REFERENCES users(id) NOT NULL,
  mood  INTEGER         CHECK (mood > 0 AND mood <= 10) NOT NULL,
  note  TEXT            NOT NULL,
  PRIMARY KEY (user_id, time)
);

CREATE INDEX ON moods (user_id, time DESC);

SELECT create_hypertable('moods', 'time');
