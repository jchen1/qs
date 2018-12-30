ALTER TABLE users
  ADD COLUMN g_sub TEXT;

UPDATE users
  SET g_sub = '101568098750308133911'
  WHERE email = 'jeff.chen1994@gmail.com';

ALTER TABLE users
  ALTER COLUMN g_sub SET NOT NULL,
  ADD UNIQUE (g_sub);