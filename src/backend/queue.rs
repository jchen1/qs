use chrono::{NaiveDate};
pub use oppgave::{Queue};
use redis::{Client};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum QueueAction {
  // IngestSteps(userid, service, date)
  IngestSteps(Uuid, String, NaiveDate)
}

pub fn init_queue(redis_url: String, queue_name: String) -> Queue {
  let client = Client::open(redis_url.as_str()).expect("Failed to connect to redis");
  Queue::new(queue_name, client)
}