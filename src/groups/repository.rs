use std::collections::HashMap;
use actix_web::App;
use log::{debug, error};
use sqlx::{query_as, PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::groups::models::{Group, MyGroupsView, Room, RoomMessage, RoomView, RoomWithMembersView, UserRoom};
use crate::models::app_error::AppError;
use crate::services::profile_service::MiniProfile;
use crate::utils::general::get_time_naive;



pub struct GroupRepo{

}
impl GroupRepo{
    pub async fn get_room_by_id(pool:&PgPool, room_id:String)->Result<Room, AppError>{
        let room = query_as!(Room, "
        SELECT * FROM rooms
        WHERE id=$1
        ",
            room_id
        ).fetch_one(pool).await.map_err(|err|
            match err{
                sqlx::Error::RowNotFound=>{
                    error!("room not found {}", room_id);
                    AppError::NotFoundError("rooms".to_string(), room_id)
                },
                others=>{
                    error!("error {}", others);
                    AppError::FetchDataError
                }
            })?;
        Ok(room)
    }

    pub async fn get_room_by_code(pool:&PgPool, code:String)->Result<Room, AppError>{
        let room = query_as!(Room, "
        SELECT * FROM rooms
        WHERE code=$1
        ",
            code
        ).fetch_one(pool).await.map_err(|err|
            match err{
                sqlx::Error::RowNotFound=>{
                    error!("code room not found {}", code);
                    AppError::NotFoundError("rooms".to_string(), code)
                },
                others=>{
                    error!("error {}", others);
                    AppError::FetchDataError
                }
            })?;
        Ok(room)
    }


    pub async fn get_group_by_id(pool:&PgPool, group_id:String)->Result<Group, AppError>{
        let group = query_as!(Group, "
        SELECT * FROM groups
        WHERE id=$1
        ",
            group_id
        ).fetch_one(pool).await.map_err(|err|
        match err{
            sqlx::Error::RowNotFound=>{
                error!("Group not found {}", group_id);
                AppError::NotFoundError("groups".to_string(), group_id)
            },
            others=>{
                error!("error {}", others);
                AppError::FetchDataError
            }
        })?;
        Ok(group)
    }

    pub async fn get_all_rooms_by_group_id(pool:&PgPool, group_id:String)->Result<Vec<Room>, AppError>{
        let rooms = query_as!(Room, "
        SELECT * FROM rooms
        WHERE group_id=$1
         ",
            group_id
        ).fetch_all(pool).await;
        let rooms = match rooms{
            Ok(data)=>{data},
            Err(err)=>{
                return Err(AppError::FetchDataError);
            }
        };

        Ok(rooms)
    }

    pub async fn create_group(pool:&PgPool, group:&Group)->Result<(), AppError>{
        let mut tx: Transaction<'_, Postgres> = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => return Err(AppError::CreateTransactionError),
        };

        let c_group_res = query_as!(Group, "
        INSERT INTO groups(id, user_name, name, description, is_private, image,category, created_at, updated_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        ",
            group.id.to_owned(),
            group.user_name.to_owned(),
            group.name.to_owned(),
            group.description.to_owned(),
            group.is_private,
            group.image.to_owned(),
            &group.category.to_owned(),
            group.created_at.to_owned(),
            group.updated_at.to_owned()
        ).execute(&mut *tx).await;

        if c_group_res.is_err(){
            let err = c_group_res.err().unwrap();
            error!("{}", err);
            let _= tx.rollback().await;
            if let Some(code) = err.as_database_error().unwrap().code() {
                debug!("db error code {}", code);
                if code == "23505" {
                    return Err(AppError::AlreadyExistsError);
                }
            }
            return Err(AppError::DBInsertError);
        }

        let public_room = Room{
            id: Uuid::new_v4().to_string(),
            group_id: group.id.to_owned(),
            name: "public".to_string(),
            description: Some("public room where all group members can meet and discuss.".to_string()),
            is_private: false,
            created_by: group.user_name.to_owned(),
            code:None,
            member_count: 0,
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };

        let c_create_room_res = query_as!(Room, "
        INSERT INTO rooms(id,group_id, name, description, is_private,created_by,created_at, updated_at)
        VALUES($1,$2,$3,$4,$5,$6,$7,$8)
        ",
          public_room.id.to_owned(),
            group.id.to_owned(),
            public_room.name.to_owned(),
            public_room.description.to_owned(),
            public_room.is_private,
            public_room.created_by.to_owned(),
            public_room.created_at.to_owned(),
            public_room.updated_at.to_owned()
        ).execute(&mut *tx).await;

        if c_create_room_res.is_err(){
            error!("{}", c_create_room_res.err().unwrap());
            let _= tx.rollback().await;
            return Err(AppError::DBInsertError);
        }

        let user_room = UserRoom{
            user_name: group.user_name.to_owned(),
            room_id: public_room.id.to_owned(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        let jroom_res = query_as!(UserRoom, "
        INSERT INTO user_rooms (user_name, room_id,created_at,updated_at)
        VALUES ($1,$2,$3,$4)
        ",
            user_room.user_name.to_owned(),
            user_room.room_id.to_owned(),
            user_room.created_at.to_owned(),
            user_room.updated_at.to_owned()
        ).execute(&mut *tx).await;

        match jroom_res {
            Ok(_) => {},
            Err(err)=>{
                error!("db error: {:?}", err);
                tx.rollback().await.unwrap();
                return Err(AppError::DBInsertError);
            }
        }

        match tx.commit().await{
            Ok(_) => {},
            Err(err) =>{
                error!("{}", err);
                return Err(AppError::CreateTransactionError)
            }
        }
        Ok(())
    }


    pub async fn update_group(pool:&PgPool, room:&Group)->Result<(), AppError>{
        let update_time = get_time_naive();
        let c_group_res = query_as!(Group, "
        UPDATE groups
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_private = COALESCE($4, is_private),
            updated_at = COALESCE($5, updated_at)
        WHERE id =$1
        ",
            room.id.to_owned(),
            room.name.to_owned(),
            room.description.to_owned(),
            room.is_private,
            room.updated_at.to_owned()
        ).execute(pool).await;

        Ok(())

    }
    pub async fn update_room(pool:&PgPool, room:&Room)->Result<(), AppError>{
        let update_time = get_time_naive();
        let c_group_res = query_as!(Room, "
        UPDATE rooms
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            is_private = COALESCE($4, is_private),
            updated_at = COALESCE($5, updated_at),
            code = COALESCE($6, code)
        WHERE id =$1
        ",
            room.id.to_owned(),
            room.name.to_owned(),
            room.description.to_owned(),
            room.is_private,
            room.updated_at.to_owned(),
            room.code.to_owned()
        ).execute(pool).await;
        match c_group_res {
            Ok(_) => {},
            Err(err)=>{
                error!("db error: {:?}", err);
                return Err(AppError::DBUpdateError);
            }
        }
        Ok(())
    }
    pub async fn create_room(pool:&PgPool, room:&Room)->Result<(), AppError>{
        let mut tx: Transaction<'_, Postgres> = match pool.begin().await {
            Ok(tx) => tx,
            Err(err) => {
                error!("db-transaction error{}", err);
                return Err(AppError::CreateTransactionError)
            }
        };
        let c_group_res = query_as!(Room, "
        INSERT INTO rooms(id, group_id, name, description, is_private,created_by, created_at, updated_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ",
            room.id.to_owned(),
            room.group_id.to_owned(),
            room.name.to_owned(),
            room.description.to_owned(),
            room.is_private,
            room.created_by.to_owned(),
            room.created_at.to_owned(),
            room.updated_at.to_owned()
        ).execute(&mut *tx).await;

        match c_group_res {
            Ok(_) => {},
            Err(err)=>{
                error!("db error: {:?}", err);
                tx.rollback().await.unwrap();
                return Err(AppError::DBInsertError);
            }
        }

        let user_room = UserRoom{
            user_name: room.created_by.to_owned(),
            room_id: room.id.to_owned(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        let jroom_res = query_as!(UserRoom, "
        INSERT INTO user_rooms (user_name, room_id,created_at,updated_at)
        VALUES ($1,$2,$3,$4)
        ",
            user_room.user_name.to_owned(),
            user_room.room_id.to_owned(),
            user_room.created_at.to_owned(),
            user_room.updated_at.to_owned()
        ).execute(&mut *tx).await;

        match jroom_res {
            Ok(_) => {},
            Err(err)=>{
                error!("db error: {:?}", err);
                tx.rollback().await.unwrap();
                return Err(AppError::DBInsertError);
            }
        }

        match tx.commit().await{
            Ok(_) => {},
            Err(err) =>{
                error!("{}", err);
                return Err(AppError::CreateTransactionError)
            }
        }
        Ok(())
    }

    pub async fn join_room(pool:&PgPool, room:&UserRoom)->Result<(), AppError>{
        let res = query_as!(UserRoom, "
        INSERT INTO user_rooms(user_name, room_id, created_at, updated_at)
        VALUES ($1,$2,$3,$4)
        ",
            room.user_name.to_owned(),
            room.room_id.to_owned(),
            room.created_at.to_owned(),
            room.updated_at.to_owned(),
        ).execute(pool).await;


        match res {
            Ok(data) => {
                if data.rows_affected() >0{
                    // silently run query to update member count
                    match query_as!(Room,
                        "
                        UPDATE rooms
                        SET member_count = GREATEST(member_count + 1, 0)
                        WHERE id = $1
                        ", room.room_id.to_owned(),
                    ).execute(pool).await{
                        Ok(_) => {},
                        Err(err)=>{
                            error!("{}", err);
                        }
                    };
                }
            },
            Err(err)=>{
                error!("db error: {:?}", err);
                if let Some(code) = err.as_database_error().unwrap().code() {
                    if code == "23505" {
                        return Err(AppError::AlreadyExistsError);
                    }
                }
                return Err(AppError::DBInsertError);
            }
        }
        Ok(())
    }

    pub async fn exit_room(pool:&PgPool, user_name:String, room_id:String)->Result<(), AppError>{
        let res = query_as!(UserRoom, "
        DELETE FROM user_rooms
        WHERE user_name = $1 AND room_id = $2
        ",
            user_name.to_owned(),
            room_id.to_owned(),
        ).execute(pool).await;
        match res {
            Ok(data) => {
                if data.rows_affected() >0{
                    // silently run query to update member count
                    match query_as!(Room,
                        "
                        UPDATE rooms
                        SET member_count = GREATEST(member_count - 1, 0)
                        WHERE id = $1
                        ", room_id.to_owned(),
                    ).execute(pool).await{
                        Ok(_) => {},
                        Err(err)=>{
                            error!("{}", err);
                        }
                    };
                }
            },
            Err(err)=>{
                error!("delete room error: {:?}", err);
                return Err(AppError::DBDeleteError);
            }
        }

        Ok(())
    }
    async fn fetch_rooms_for_groups(pool: &PgPool, group_ids: &[String])
                                    -> Result<Vec<Room>, AppError>
    {
        // In PostgreSQL, ANY($1) expects $1 to be a slice that implements `Into<sqlx::types::Array>`.
        let rooms = sqlx::query_as!(
        Room,
        r#"
        SELECT
          r.id,
          r.group_id,
          r.name,
          r.description,
          r.is_private,
          r.created_by,
          r.code,
          r.created_at,
          r.updated_at,
          r.member_count
        FROM rooms AS r
        WHERE r.group_id = ANY($1)
        ORDER BY r.group_id, r.name;
        "#,
        group_ids
    )
            .fetch_all(pool)
            .await;
        let rooms = match rooms{
            Ok(data)=>{data},
            Err(err)=>{
                error!("{}", err);
                return Err(AppError::FetchDataError);
            }
        };
        Ok(rooms)
    }
    pub async fn get_my_groups(pool:&PgPool,user_name:String )->Result<Vec<MyGroupsView>, AppError>{
        let groups = query_as!(Group, "
        SELECT DISTINCT
          g.id,
          g.name,
          g.description,
          g.user_name,
          g.is_private,
          g.image,
          g.category,
          g.created_at,
          g.updated_at
        FROM groups AS g
        JOIN rooms   AS r ON r.group_id = g.id
        JOIN user_rooms AS ur ON ur.room_id = r.id
        WHERE ur.user_name = $1
        ORDER BY g.name;
        ",
            user_name.clone()
        ).fetch_all(pool).await;
        let groups = match groups{
            Ok(data)=>{data},
            Err(err)=>{
                error!("{}", err);
                return Err(AppError::FetchDataError);
            }
        };
        // get rooms for each group
        let group_ids: Vec<String> = groups.iter().map(|g| g.id.clone()).collect();
        let rooms: Vec<Room> = Self::fetch_rooms_for_groups(pool, &group_ids).await?;
        let mut rooms_by_group: HashMap<String, Vec<Room>> = HashMap::new();
        for room in rooms {
            rooms_by_group
                .entry(room.group_id.clone())
                .or_default()
                .push(room);
        }

        let mut result: Vec<MyGroupsView> = Vec::with_capacity(groups.len());
        for g in groups {
            let group_rooms = rooms_by_group.remove(&g.id).unwrap_or_default();
            result.push(MyGroupsView {
                id:          g.id.clone(),
                name:        g.name.clone(),
                description: g.description.clone(),
                is_private:  g.is_private,
                created_by:  g.user_name.clone(),
                rooms:       group_rooms,
            });
        }
        Ok(result)
    }

    pub async fn get_group_with_rooms_by_id(pool:&PgPool, group_id:String)->Result<MyGroupsView, AppError>{
        // Get the group by ID
        let group = Self::get_group_by_id(pool, group_id.clone()).await?;

        // Get all rooms for this group
        let rooms = Self::get_all_rooms_by_group_id(pool, group_id.clone()).await?;

        // Create and return the MyGroupsView
        let result = MyGroupsView {
            id: group.id.clone(),
            name: group.name.clone(),
            description: group.description.clone(),
            is_private: group.is_private,
            created_by: group.user_name.clone(),
            rooms: rooms,
        };

        Ok(result)
    }

    pub async fn get_room_with_members(pool: &PgPool, room_id: String) -> Result<RoomWithMembersView, AppError> {
        // Get the room by ID
        let room = Self::get_room_by_id(pool, room_id.clone()).await?;

        // Get all members (usernames) for this room
        let members = sqlx::query_as!(
            MiniProfile,
            r#"
            SELECT 
                p.user_name, 
                p.image, 
                p.bio, 
                p.name
            FROM 
                user_rooms ur
            JOIN 
                profiles p ON ur.user_name = p.user_name
            WHERE 
                ur.room_id = $1
            "#,
            room_id
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("Error fetching room members: {}", err);
            AppError::FetchDataError
        })?;

        // Create and return the RoomWithMembersView
        let result = RoomWithMembersView {
            id: room.id,
            group_id: room.group_id,
            name: room.name,
            description: room.description,
            is_private: room.is_private,
            created_by: room.created_by,
            code: room.code,
            created_at: room.created_at,
            updated_at: room.updated_at,
            member_count: room.member_count,
            members,
        };

        Ok(result)
    }

    pub async fn create_room_message(pool: &PgPool, message: RoomMessage) -> Result<RoomMessage, AppError> {
        // Execute the insert query
        let result = sqlx::query!(
            r#"
            INSERT INTO room_messages (id, user_name, text, image, room_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            message.id,
            message.user_name,
            message.text,
            message.image,
            message.room_id,
            message.created_at,
            message.updated_at
        )
        .execute(pool)
        .await
        .map_err(|err| {
            error!("Error creating room message: {}", err);
            if let Some(db_err) = err.as_database_error() {
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        return AppError::AlreadyExistsError;
                    }
                }
            }
            AppError::DBInsertError
        })?;

        // Return the message that was inserted
        Ok(message)
    }

    pub async fn get_rooms_by_user_name(pool: &PgPool, user_name: String) -> Result<Vec<Room>, AppError> {
        // Query to get all rooms a user belongs to
        let rooms = sqlx::query_as!(
            Room,
            r#"
            SELECT 
                r.id, 
                r.group_id, 
                r.name, 
                r.description, 
                r.is_private, 
                r.created_by, 
                r.code, 
                r.member_count, 
                r.created_at, 
                r.updated_at
            FROM 
                rooms r
            JOIN 
                user_rooms ur ON r.id = ur.room_id
            WHERE 
                ur.user_name = $1
            "#,
            user_name
        )
        .fetch_all(pool)
        .await
        .map_err(|err| {
            error!("Error fetching rooms for user {}: {}", user_name, err);
            AppError::FetchDataError
        })?;

        Ok(rooms)
    }

}
