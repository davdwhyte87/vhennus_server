use std::fmt::Write;
use chrono::NaiveDateTime;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default,)]
pub struct User {
    pub id: String,
    pub user_name: String,
    pub email: Option<String>,
    pub code: Option<i32>,
    pub created_at: NaiveDateTime,
    pub user_type: String, // 0 user, 1 admin
    pub password_hash: String,
    pub is_deleted: bool,
    pub email_confirmed: bool,
}

// #[derive(Debug,  Clone, Copy, AsExpression, FromSqlRow, Default, Serialize, Deserialize)]
// #[diesel(sql_type = diesel::sql_types::Integer)]
// pub enum UserType{
//     #[default]
//     User = 0,
//     Admin = 1
// }

// impl<'a> FromSql<Integer, Pg> for UserType {
//     fn from_sql(bytes: <Pg as Backend>::RawValue<'a>) -> deserialize::Result<Self> {
//         let value = <i32 as FromSql<Integer, Pg>>::from_sql(bytes)?;
//         match value {
//             0 => Ok(UserType::User),
//             1 => Ok(UserType::Admin),
//             _ => Err("Unrecognized enum variant".into()),
//         }
//     }
// }

// impl<DB: Backend> ToSql<Integer, DB> for UserType
// where
//     i32: ToSql<Integer, DB>,
// {
//     fn to_sql<'a>(&self, out: &mut Output<'a, '_, DB>) -> serialize::Result {
//         let value = *self as i32;
//         value.to_sql(out) // No need to clone
//     }
// }

