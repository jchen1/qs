use juniper::{FieldResult, RootNode};
use uuid::Uuid;

use super::Context;
use crate::db;
use crate::providers::fitbit::IntradayMetric;
use crate::queue::{QueueAction, QueueActionParams};
use chrono::{DateTime, NaiveDate, Utc};

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {
    field user(&executor, id: Option<Uuid>) -> FieldResult<Option<db::User>> {
        let conn = &executor.context().conn;
        let user = match id {
            Some(id) => db::User::find_one(conn, &id).ok(),
            None => executor.context().user.clone()
        };

        Ok(user)
    }
});

graphql_object!(db::User: Context as "User" |&self| {
    field steps(&executor, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>, only_populated = true: bool) -> FieldResult<Vec<db::Step>> {
        let conn = &executor.context().conn;
        let steps = db::Step::for_period(conn, &self.id, &start_time.unwrap_or_else(|| Utc::today().and_hms(0, 0, 0)), &end_time.unwrap_or_else(Utc::now)).unwrap_or_else(|_| vec![])
            .into_iter()
            .filter(|s| !only_populated || s.count > 0)
            .collect();

        Ok(steps)
    }

    field floors(&executor, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>, only_populated = true: bool) -> FieldResult<Vec<db::Floor>> {
        let conn = &executor.context().conn;
        let floors = db::Floor::for_period(conn, &self.id, &start_time.unwrap_or_else(|| Utc::today().and_hms(0, 0, 0)), &end_time.unwrap_or_else(Utc::now)).unwrap_or_else(|_| vec![])
            .into_iter()
            .filter(|f| !only_populated || f.count > 0)
            .collect();

        Ok(floors)
    }

    field distances(&executor, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>, only_populated = true: bool) -> FieldResult<Vec<db::Distance>> {
        let conn = &executor.context().conn;
        let distances = db::Distance::for_period(conn, &self.id, &start_time.unwrap_or_else(|| Utc::today().and_hms(0, 0, 0)), &end_time.unwrap_or_else(Utc::now)).unwrap_or_else(|_| vec![])
            .into_iter()
            .filter(|d| !only_populated || d.count > 0.0)
            .collect();

        Ok(distances)
    }

    field elevations(&executor, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>, only_populated = true: bool) -> FieldResult<Vec<db::Elevation>> {
        let conn = &executor.context().conn;
        let elevations = db::Elevation::for_period(conn, &self.id, &start_time.unwrap_or_else(|| Utc::today().and_hms(0, 0, 0)), &end_time.unwrap_or_else(Utc::now)).unwrap_or_else(|_| vec![])
            .into_iter()
            .filter(|e| !only_populated || e.count > 0.0)
            .collect();

        Ok(elevations)
    }

    field calories(&executor, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>, only_populated = true: bool) -> FieldResult<Vec<db::Calorie>> {
        let conn = &executor.context().conn;
        let calories = db::Calorie::for_period(conn, &self.id, &start_time.unwrap_or_else(|| Utc::today().and_hms(0, 0, 0)), &end_time.unwrap_or_else(Utc::now)).unwrap_or_else(|_| vec![])
            .into_iter()
            .filter(|c| !only_populated || c.count > 0.0)
            .collect();

        Ok(calories)
    }

    field email() -> &str {
        &self.email
    }

    field id() -> &Uuid {
        &self.id
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field ingest_intraday(&executor, service: String, measurement: IntradayMetric, date: Option<NaiveDate>, num_days = 1: i32) -> FieldResult<bool> {
        let producer = &executor.context().producer;
        let user_id = executor.context().user.clone().ok_or_else(|| "Not logged in".to_owned())?.id;

        match (service.as_str(), num_days < 0) {
            ("fitbit", false) => Ok(()),
            ("fitbit", true) => Err("num_days must be positive".to_owned()),
            _ => Err("only fitbit is supported".to_owned())
        }?;

        let action = QueueAction {
            id: Uuid::new_v4(),
            user_id: user_id,
            params: QueueActionParams::BulkIngestIntraday(
                measurement,
                // TODO naive_local or naive_utc?
                date.unwrap_or_else(|| Utc::now().naive_local().date()),
                num_days as u32
            )
        };

        producer.push(action)?;

        Ok(true)
    }
});

pub type Schema = RootNode<'static, QueryRoot, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {})
}
