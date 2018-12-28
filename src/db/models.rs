use super::schema::users;

#[derive(Serialize, Queryable)]
pub struct User {
    pub id: String,
    pub email: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub email: &'a str,
}