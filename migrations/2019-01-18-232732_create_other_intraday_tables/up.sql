CREATE TABLE calories (
  time    TIMESTAMPTZ       NOT NULL,
  user_id UUID              REFERENCES users(id) NOT NULL,
  source  TEXT              NOT NULL, /* probably just fitbit... */
  count   DOUBLE PRECISION  CHECK (count >= 0) NOT NULL,
  level   INTEGER           NOT NULL,
  mets    INTEGER           NOT NULL,
  PRIMARY KEY (user_id, time)
);
CREATE INDEX ON calories (user_id, time DESC);
SELECT create_hypertable('calories', 'time');

CREATE TABLE distances (
  time    TIMESTAMPTZ       NOT NULL,
  user_id UUID              REFERENCES users(id) NOT NULL,
  source  TEXT              NOT NULL, /* probably just fitbit... */
  count   DOUBLE PRECISION  CHECK (count >= 0) NOT NULL,
  PRIMARY KEY (user_id, time)
);
CREATE INDEX ON distances (user_id, time DESC);
SELECT create_hypertable('distances', 'time');

CREATE TABLE floors (
  time    TIMESTAMPTZ   NOT NULL,
  user_id UUID          REFERENCES users(id) NOT NULL,
  source  TEXT          NOT NULL, /* probably just fitbit... */
  count   INTEGER       CHECK (count >= 0) NOT NULL,
  PRIMARY KEY (user_id, time)
);
CREATE INDEX ON floors (user_id, time DESC);
SELECT create_hypertable('floors', 'time');

CREATE TABLE elevations (
  time    TIMESTAMPTZ       NOT NULL,
  user_id UUID              REFERENCES users(id) NOT NULL,
  source  TEXT              NOT NULL, /* probably just fitbit... */
  count   DOUBLE PRECISION  CHECK (count >= 0) NOT NULL,
  PRIMARY KEY (user_id, time)
);
CREATE INDEX ON elevations (user_id, time DESC);
SELECT create_hypertable('elevations', 'time');
