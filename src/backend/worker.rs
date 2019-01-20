use crate::{
    db::{self, Conn, Step, Token, Calorie, Distance, Elevation, Floor},
    providers::fitbit,
    queue::{Queue, QueueAction, QueueActionParams},
};
use actix_web::{error, Error};
use chrono::{Duration, NaiveDate};
use uuid::Uuid;

pub struct WorkerContext {
    pub queue: Queue,
    pub conn: Conn,
}

fn ingest_intraday_bulk(
    ctx: &WorkerContext,
    user_id: &Uuid,
    metric: &fitbit::IntradayMetric,
    start_date: &NaiveDate,
    num_days: &u32,
) -> Result<(), Error> {
    for i in 0..*num_days {
        let action = QueueAction {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            params: QueueActionParams::IngestIntraday(
                metric.clone(),
                *start_date + Duration::days(i as i64),
            ),
        };

        ctx.queue
            .push(action)
            .map_err(error::ErrorInternalServerError)?;
    }

    Ok(())
}

fn ingest_intraday<T: fitbit::IntradayMeasurement + db::Object>(
    ctx: &WorkerContext,
    token: Token,
    date: NaiveDate,
) -> Result<(), Error> {
    let measurement = fitbit::measurement_for_day::<T>(date, &token)?;
    T::insert_many(&ctx.conn, &measurement)
        .map_err(error::ErrorInternalServerError)?;
    Ok(())
}

fn execute_one(
    ctx: &WorkerContext,
    user_id: &Uuid,
    action: &QueueActionParams,
) -> Result<(), Error> {
    match action {
        QueueActionParams::IngestIntraday(metric, date) => {
            let token = Token::find_by_uid_service(&ctx.conn, user_id, "fitbit")
                .map_err(error::ErrorInternalServerError)?;
            match metric {
                fitbit::IntradayMetric::Step => ingest_intraday::<Step>(ctx, token, date.clone()),
                fitbit::IntradayMetric::Calorie => ingest_intraday::<Calorie>(ctx, token, date.clone()),
                fitbit::IntradayMetric::Distance => ingest_intraday::<Distance>(ctx, token, date.clone()),
                fitbit::IntradayMetric::Elevation => ingest_intraday::<Elevation>(ctx, token, date.clone()),
                fitbit::IntradayMetric::Floor => ingest_intraday::<Floor>(ctx, token, date.clone()),
            }
        }
        QueueActionParams::BulkIngestIntraday(metric, start_date, num_days) => {
            ingest_intraday_bulk(ctx, user_id, metric, start_date, num_days)
        }
    }
}

pub fn pop_and_execute(ctx: &WorkerContext) -> Result<Option<()>, Error> {
    if let Some(task) = ctx.queue.next::<QueueAction>(1) {
        let task = task.map_err(error::ErrorInternalServerError)?;
        let QueueAction {
            id,
            user_id,
            params,
        } = task.inner();

        info!("Processing task {}...", id);
        match execute_one(ctx, user_id, &params) {
            Ok(_) => {
                info!("Processed task {}", id);
                Ok(Some(()))
            }
            Err(e) => {
                error!("Error processing task {}: {:?}", id, e);
                task.fail();
                Err(e)
            }
        }
    } else {
        info!("Timed out waiting for task");
        Ok(None)
    }
}
