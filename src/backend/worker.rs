use crate::{db::{self, Conn, Step, Token}, queue::{QueueAction, Queue, QueueActionParams}, providers::{fitbit}};
use actix_web::{error, Error};
use uuid::Uuid;
use chrono::NaiveDate;

pub struct WorkerContext {
    pub queue: Queue,
    pub conn: Conn
}

fn ingest_steps(ctx: &WorkerContext, user_id: &Uuid, service: String, date: NaiveDate) -> Result<(), Error> {
    let token = Token::find_by_uid_service(&ctx.conn, user_id, &service).map_err(error::ErrorInternalServerError)?;
    let steps_for_day = fitbit::steps_for_day(date, &token)?;
    Step::insert_many(&ctx.conn, &steps_for_day).map_err(error::ErrorInternalServerError).map(|_| { () })
}

fn execute_one(ctx: &WorkerContext, user_id: &Uuid, action: &QueueActionParams) -> Result<(), Error> {
    match action {
        QueueActionParams::IngestSteps(service, date) => ingest_steps(ctx, user_id, service.to_string(), date.clone())
    }
}

pub fn pop_and_execute(ctx: &WorkerContext) -> Result<(), Error> {
    if let Some(task) = ctx.queue.next::<QueueAction>() {
        let task = task.map_err(error::ErrorInternalServerError)?;
        let QueueAction { id, user_id, params } = task.inner();

        info!("Processing task {}...", id);
        match execute_one(ctx, user_id, &params) {
            Ok(_) => {
                info!("Processed task {}", id);
                Ok(())
            },
            Err(e) => {
                error!("Error processing task {}: {:?}", id, e);
                task.fail();
                Err(e)
            }
        }
    } else {
        Ok(())
    }
}