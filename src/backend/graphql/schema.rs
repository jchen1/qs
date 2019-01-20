use diesel::prelude::*;
use juniper::{FieldResult, RootNode};
use std::ops::Deref;
use uuid::Uuid;

use super::Context;
use crate::db;
use crate::queue::{QueueAction, QueueActionParams};
use crate::providers::fitbit::{IntradayMetric};
use chrono::{DateTime, NaiveDate, Utc};

#[derive(GraphQLInputObject)]
#[graphql(description = "A user")]
struct NewUser {
    id: Uuid,
    email: String,
    g_sub: String,
}

#[derive(GraphQLObject)]
#[graphql(description = "A token")]
struct Token {
    pub id: Uuid,
    pub service: String,
    pub access_token: String,
    pub access_token_expiry: DateTime<Utc>,
}

impl From<&db::Token> for Token {
    fn from(token: &db::Token) -> Self {
        Token {
            id: token.id,
            service: token.service.clone(),
            access_token: token.access_token.clone(),
            access_token_expiry: token.access_token_expiry,
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(description = "A user")]
struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub g_sub: String,
    pub tokens: Vec<Token>,
    pub steps: Vec<db::Step>,
}

impl User {
    pub fn new(user: db::User, tokens: Vec<db::Token>, steps: Vec<db::Step>) -> User {
        User {
            id: user.id,
            email: user.email,
            name: "Jeff Chen".to_string(), // todo
            g_sub: user.g_sub,
            tokens: tokens.iter().map(|t| Token::from(t)).collect(),
            steps: steps,
        }
    }
}

pub struct QueryRoot;

graphql_object!(QueryRoot: Context |&self| {

    field user(&executor, id: Option<String>, start_time: Option<DateTime<Utc>>, end_time: Option<DateTime<Utc>>) -> FieldResult<Option<User>> {
        let conn = &executor.context().conn;
        let user = match id {
            Some(id) => db::User::find_one(conn, &Uuid::parse_str(&id)?).ok(),
            None => executor.context().user.clone()
        };

        if let Some(user) = user {
            let tokens = db::Token::belonging_to(&user).load::<db::Token>(conn.deref()).ok();
            let fitbit_token = db::Token::find_by_uid_service(conn, &user.id, "fitbit").map_err(|_| "no token!".to_owned())?;
            let steps = db::Step::for_period(conn, &user.id, &start_time.unwrap_or(Utc::today().and_hms(0, 0, 0)), &end_time.unwrap_or(Utc::now())).unwrap_or(vec![]);

            let only_populated = steps.into_iter().filter(|s| s.count > 0).collect();

            Ok(Some(User::new(user, tokens.unwrap_or(vec![]), only_populated)))
        } else {
            Ok(None)
        }
    }
});

pub struct MutationRoot;

graphql_object!(MutationRoot: Context |&self| {
    field create_user(&executor, new_user: NewUser) -> FieldResult<User> {
        Ok(User{
            id: Uuid::new_v4(),
            email: new_user.email,
            name: "Jeff Chen".to_string(),
            g_sub: new_user.g_sub,
            tokens: vec![],
            steps: vec![]
        })
    }

    // field refresh_token(&executor, token_id: Uuid) -> FieldResult<Token> {
    //     let conn = &executor.context().conn;

    //     let token = db::Token::find_one(conn, token_id)?;
    //     let new_token = oauth::refresh_token(OAuthToken::from(token))?;
    //     let updated = db::Token::update(conn, token_id, db::UpdateToken {
    //         access_token: Some(&new_token.access_token),
    //         access_token_expiry: Some(&new_token.expiration),
    //         service_userid: Some(&new_token.user_id),
    //         refresh_token: match new_token.refresh_token.as_str() {
    //             "" => None,
    //             e => Some(&e)
    //         }
    //     })?;
    //     Ok(Token::from(&updated))
    // }

    field ingest_data(&executor, service: String, measurement: String, date: NaiveDate) -> FieldResult<bool> {
        let producer = &executor.context().producer;
        let user_id = executor.context().user.clone().ok_or("Not logged in".to_owned())?.id;

        let measurement = match (service.as_str(), measurement.as_str()) {
            ("fitbit", "steps") => Ok(IntradayMetric::Step),
            ("fitbit", "calories") => Ok(IntradayMetric::Calorie),
            ("fitbit", "distances") => Ok(IntradayMetric::Distance),
            ("fitbit", "elevations") => Ok(IntradayMetric::Elevation),
            ("fitbit", "floors") => Ok(IntradayMetric::Floor),
            ("fitbit", _) => Err("unsupported metric".to_owned()),
            _ => Err("only fitbit is supported".to_owned())
        }?;

        let action = QueueAction {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            params: QueueActionParams::IngestIntraday(
                measurement,
                date
            )
        };

        producer.push(action)?;

        Ok(true)
    }

    field ingest_data_bulk(&executor, service: String, measurement: String, date: NaiveDate, num_days: i32) -> FieldResult<bool> {
        let producer = &executor.context().producer;
        let user_id = executor.context().user.clone().ok_or("Not logged in".to_owned())?.id;
        match (service.as_str(), num_days < 0) {
            ("fitbit", false) => Ok(()),
            ("fitbit", true) => Err("num_days must be positive".to_owned()),
            _ => Err("only fitbit is supported".to_owned())
        }?;

        let measurement = match measurement.as_str() {
            "steps" => Ok(IntradayMetric::Step),
            "calories" => Ok(IntradayMetric::Calorie),
            "distances" => Ok(IntradayMetric::Distance),
            "elevations" => Ok(IntradayMetric::Elevation),
            "floors" => Ok(IntradayMetric::Floor),
            _ => Err("unsupported metric!".to_owned())
        }?;

        let action = QueueAction {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            params: QueueActionParams::BulkIngestIntraday(
                measurement,
                date,
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
