use diesel::{Insertable, Queryable};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default, Queryable, Insertable)]
#[diesel(table_name = crate::schema::likes)]
pub struct Like{
    pub user_name:String,
    pub post_id:String,
}