## Setup

- Docker volumes
  - `docker volume create --name postgres-dev`
  - `docker volume create --name redis-dev`
- TimescaleDB
  `docker run -d --name timescaledb -p 127.0.0.1:5432:5432 -e POSTGRES_PASSWORD=password -v postgres-dev/data:/var/lib/postgresql/data timescale/timescaledb`
- Redis
  `docker run -d --name redis -p 127.0.0.1:6379:6379 -v redis-dev/data:/docker/host/dir:/data redis redis-server --appendonly yes`