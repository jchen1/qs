use actix::prelude::*;
use actix_web::{error, Error};
use chrono::{NaiveDate};
pub use oppgave::{Queue};
use redis::{Client};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QueueAction {
  // IngestSteps(userid, service, date)
  IngestSteps(Uuid, String, NaiveDate)
}

pub struct QueueExecutor(pub Queue);

impl Actor for QueueExecutor {
  type Context = SyncContext<Self>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendMessage {
  action: QueueAction
}

pub fn init_queue(redis_url: String, queue_name: String) -> Queue {
  let client = Client::open(redis_url.as_str()).expect("Failed to connect to redis");
  Queue::new(queue_name, client)
}

impl Message for SendMessage {
  type Result = Result<(), Error>;
}

impl Handler<SendMessage> for QueueExecutor {
  type Result = Result<(), Error>;

  fn handle(&mut self, msg: SendMessage, _: &mut Self::Context) -> Self::Result {
    let queue = &self.0;

    queue.push(msg);
    Ok(())
  }
}