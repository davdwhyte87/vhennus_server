use actix_web::{web, App, HttpResponse};
use actix_web::web::ReqData;
use bigdecimal::num_traits::clamp_min;
use futures_util::future::err;
use log::{debug, error};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use sqlx::PgPool;
use uuid::Uuid;
use crate::groups::models::{Group, Room, UserRoom};
use crate::groups::repository::GroupRepo;
use crate::models::app_error::AppError;
use crate::req_models::requests::{CreateGroupReq, CreateRoomReq, UpdateGroupReq, UpdateProfileReq, UpdateRoomReq};
use crate::utils::auth::Claims;
use crate::utils::general::get_time_naive;

pub struct GroupService{

}
impl GroupService{
    pub async fn create_group(
        req: &CreateGroupReq,
        pool:&PgPool,
        claim:&Claims
    )->Result<(), AppError>{

        let group = Group{
            id: Uuid::new_v4().to_string(),
            user_name: claim.user_name.clone(),
            name: req.name.to_owned(),
            description: req.description.to_owned(),
            is_private: req.is_private,
            image: req.image.to_owned(),
            category: req.category.to_owned(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        match GroupRepo::create_group(pool, &group).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error creating group {}{}", group.name, err );
                return Err(err.into());
            }
        };
        return Ok(())
    }

    pub async fn create_room(
        pool:&PgPool,
        req:&CreateRoomReq,
        claim: &Claims
    )->Result<(), AppError>{
        // make sure the req user is the owner of the group
        // get the group 
        let group =  GroupRepo::get_group_by_id(pool, req.group_id.clone()).await
            .map_err(|err| err)?;

        if group.user_name != claim.user_name{
            return Err(AppError::UnauthorizedError)
        }

        // make sure this room does not exist before
        // get all the rooms
        let rooms = match GroupRepo::get_all_rooms_by_group_id(pool, req.group_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        for room in rooms{
            if room.name == req.name{
                return Err(AppError::AlreadyExistsError)
            }
        }
        let room = Room{
            id: Uuid::new_v4().to_string(),
            group_id:req.group_id.to_owned(),
            name: req.name.to_owned(),
            description: req.description.to_owned(),
            is_private: req.is_private,
            created_by: claim.user_name.clone(),
            code:None,
            member_count: 0,
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        GroupRepo::create_room(&pool, &room).await.map_err(|err| err)?;

        Ok(())
    }

    pub async fn join_room(
        pool:&PgPool,
        claim: &Claims,
        room_id:&String
    )->Result<(), AppError>{
        // make sure room exists
        let room = match GroupRepo::get_room_by_id(pool, room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };

        // user can only join if the room is public
        if room.is_private{
            return Err(AppError::UnauthorizedError)
        }

        let room_user = UserRoom{
            user_name:claim.user_name.clone() ,
            room_id: room.id.clone(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        match GroupRepo::join_room(pool,&room_user ).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error joining room {}{}", room_user.user_name, err );
                return Err(err.into());
            }
        };
        Ok(())
    }

    pub async fn generate_code(
        pool:&PgPool,
        room_id:&String,
        claims: &Claims
    )->Result<String, AppError>{
        let mut room = match GroupRepo::get_room_by_id(pool, room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        debug!("gen by user {}", claims.user_name.clone());
        if claims.user_name.clone() != room.created_by{
            return Err(AppError::UnauthorizedError)
        }
        let code = thread_rng().sample_iter(&Alphanumeric).take(10).map(char::from).collect::<String>();
        room.code = Some(code.clone());
        match GroupRepo::update_room(&pool, &room).await{
            Ok(_)=>{},
            Err(err)=>{
                return Err(err);
            }
        }
        Ok(code)
    }

    pub async fn join_room_with_code(
        pool:&PgPool,
        claim: &Claims,
        code:Option<String>
    )->Result<(), AppError>{

        if code.is_none(){
            return Err(AppError::BadRequestError("code is missing".to_string()));
        }

        // make sure room exists
        let room = match GroupRepo::get_room_by_code(pool, code.clone().unwrap_or_default()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };

        if room.code.is_none(){
            return Err(AppError::BadRequestError("room has no code".to_string()));
        }

        if room.code.unwrap_or_default() != code.clone().unwrap() {
            return Err(AppError::UnauthorizedError);
        }


        let room_user = UserRoom{
            user_name:claim.user_name.clone() ,
            room_id: room.id.clone(),
            created_at: get_time_naive(),
            updated_at: get_time_naive(),
        };
        match GroupRepo::join_room(pool,&room_user ).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error joining room {}{}", room_user.user_name, err );
                return Err(err.into());
            }
        };
        Ok(())
    }

    pub async fn update_group(
        pool:&PgPool,
        claim: &Claims,
        req:UpdateGroupReq
    )->Result<(),AppError>{
        // get group
        let mut group = match GroupRepo::get_group_by_id(pool, req.group_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        if group.user_name != claim.user_name{
            return Err(AppError::UnauthorizedError)
        }

        if req.name.is_some(){
            group.name = req.name.unwrap();
        }
        if req.description.is_some(){
            group.description = req.description;
        }
        if req.is_private.is_some(){
            group.is_private = req.is_private.unwrap();
        }


        group.updated_at = get_time_naive();
        match GroupRepo::update_group(pool, &group).await{
            Ok(_)=>{},
            Err(err)=>{
                return Err(err);
            }
        };
        Ok(())
    }

    pub async fn update_room(
        pool:&PgPool,
        claim: &Claims,
        req:UpdateRoomReq
    )->Result<(),AppError>{
        // get group
        let mut room = match GroupRepo::get_room_by_id(pool, req.room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };
        if room.created_by != claim.user_name{
            return Err(AppError::UnauthorizedError)
        }

        if req.name.is_some(){
            room.name = req.name.unwrap();
        }
        if req.description.is_some(){
            room.description = req.description;
        }
        if req.is_private.is_some(){
            room.is_private = req.is_private.unwrap();
        }


        room.updated_at = get_time_naive();
        match GroupRepo::update_room(pool, &room).await{
            Ok(_)=>{},
            Err(err)=>{
                return Err(err);
            }
        };
        Ok(())
    }

    pub async fn leave_room(
        pool:&PgPool,
        claim: &Claims,
        room_id:&String
    )->Result<(), AppError>{
        // make sure room exists
        let room = match GroupRepo::get_room_by_id(pool, room_id.clone()).await{
            Ok(data) => data,
            Err(err)=>{
                return Err(err);
            }
        };

        // Call the repository function to exit the room
        match GroupRepo::exit_room(pool, claim.user_name.clone(), room_id.clone()).await{
            Ok(_)=>{},
            Err(err)=>{
                error!("error leaving room {}{}", claim.user_name, err );
                return Err(err);
            }
        };
        Ok(())
    }
}
