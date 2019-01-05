use chrono::{NaiveDate};
pub use oppgave::{Queue};
use redis::{Client};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QueueActionParams {
  // service, date
  IngestSteps(String, NaiveDate)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueAction {
  pub id: Uuid,
  pub user_id: Uuid,
  pub params: QueueActionParams
}

pub fn init_queue(redis_url: String, queue_name: String) -> Queue {
  let client = Client::open(redis_url.as_str()).expect("Failed to connect to redis");
  Queue::new(queue_name, client)
}