use crate::{
    db::{Conn, Step, Token},
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

fn ingest_steps_bulk(ctx: &WorkerContext, user_id: &Uuid, service: &str, start_date: &NaiveDate, num_days: &u32) -> Result<(), Error> {
    for i in 0..*num_days {
        let action = QueueAction {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            params: QueueActionParams::IngestSteps(
                service.to_string(),
                *start_date + Duration::days(i as i64)
            )
        };

        ctx.queue.push(action).map_err(error::ErrorInternalServerError)?;
    }

    Ok(())
}

fn ingest_steps(
    ctx: &WorkerContext,
    user_id: &Uuid,
    service: String,
    date: NaiveDate,
) -> Result<(), Error> {
    let token = Token::find_by_uid_service(&ctx.conn, user_id, &service)
        .map_err(error::ErrorInternalServerError)?;
    let steps_for_day = fitbit::steps_for_day(date, &token)?;
    Step::insert_many(&ctx.conn, &steps_for_day)
        .map_err(error::ErrorInternalServerError)
        .map(|_| ())
}

fn execute_one(
    ctx: &WorkerContext,
    user_id: &Uuid,
    action: &QueueActionParams,
) -> Result<(), Error> {
    match action {
        QueueActionParams::IngestSteps(service, date) => {
            ingest_steps(ctx, user_id, service.to_string(), date.clone())
        },
        QueueActionParams::BulkIngestSteps(service, start_date, num_days) => {
            ingest_steps_bulk(ctx, user_id, service, start_date, num_days)
        }
    }
}

pub fn pop_and_execute(ctx: &WorkerContext) -> Result<Option<()>, Error> {
    if let Some(task) = ctx.queue.next::<QueueAction>(5) {
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
