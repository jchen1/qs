ALTER TABLE tokens
  ADD CONSTRAINT tokens_user_id_service_key UNIQUE (user_id, service);