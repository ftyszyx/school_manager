use crate::apis::auth_middleware::Claims;
use crate::apis::list_api::ListParamsReq;
use crate::apis::list_api::PagingResponse;
use crate::core::app::AppState;
use crate::core::constants;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::convert::from_str_optional;
use crate::utils::jwt::create_jwt;
use bcrypt::verify;
use chrono::{DateTime, Utc};
use data_model::{roles, teacher_classes, user_roles, users};
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::sea_query::{Alias, Expr, JoinType};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Deserialize, ToSchema)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserCreatePayload {
    pub username: String,
    pub password: String,
    pub role_ids: Option<Vec<i32>>,
    pub class_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserUpdatePayload {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role_ids: Option<Vec<i32>>,
    pub class_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct ChangePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub role_ids: Vec<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchUsersParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub username: Option<String>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub id: Option<i32>,
}

#[handler]
pub async fn register(
    json: JsonBody<AuthPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user_exists = users::Entity::find()
        .filter(users::Column::Username.eq(&json.username))
        .one(&state.db)
        .await?;
    if user_exists.is_some() {
        return Err(AppError::user_already_exists());
    }
    let new_user = add_impl(
        &state,
        UserCreatePayload {
            username: json.username.clone(),
            password: json.password.clone(),
            role_ids: Some(vec![constants::DEFAULT_ROLE_ID]),
            class_ids: None,
        },
    )
    .await?;
    let user_info = get_by_id_impl(&state, new_user.id).await?;
    info!("User registered: {}", user_info.username);
    let token = create_jwt(user_info.id, user_info.role_ids, &state.config.jwt)
        .map_err(|_| AppError::auth_failed("Token creation failed"))?;
    Ok(ApiResponse::success(AuthResponse { token }))
}

#[handler]
pub async fn login(
    payload: JsonBody<AuthPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user_result = users::Entity::find()
        .filter(users::Column::Username.eq(&payload.username.clone()))
        .find_also_related(roles::Entity)
        .one(&state.db)
        .await?;
    let user_result = user_result.ok_or(AppError::NotFound {
        resource: "user".to_string(),
        id: None,
    })?;
    let is_valid = verify(&payload.password, &user_result.0.password_hash)
        .map_err(|_| AppError::auth_failed("User or password error"))?;
    if !is_valid {
        return Err(AppError::auth_failed("User or password error"));
    }
    tracing::info!("User logged in: {}", user_result.0.username);
    let user_id = user_result.0.id;
    let role_ids = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user_id))
        .all(&state.db)
        .await?;
    let role_ids = role_ids.iter().map(|r| r.role_id).collect();
    let token = create_jwt(user_id, role_ids, &state.config.jwt)
        .map_err(|_| AppError::auth_failed("Token creation failed"))?;
    Ok(ApiResponse::success(AuthResponse { token }))
}

#[handler]
pub async fn get_current_user(depot: &mut Depot) -> Result<ApiResponse<UserInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user_info = get_by_id_impl(&state, depot.obtain::<Claims>().unwrap().user_id).await?;
    Ok(ApiResponse::success(user_info))
}

#[handler]
pub async fn change_password(
    payload: JsonBody<ChangePasswordPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<bool>, AppError> {
    use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
    let state = depot.obtain::<AppState>().unwrap();
    let user = users::Entity::find_by_id(depot.obtain::<Claims>().unwrap().user_id)
        .one(&state.db)
        .await?;
    let user = user.ok_or(AppError::auth_failed("User not found"))?;
    let is_valid = verify(&payload.old_password, &user.password_hash)
        .map_err(|_| AppError::auth_failed("Old password incorrect"))?;
    if !is_valid {
        return Err(AppError::auth_failed("Old password incorrect"));
    }
    let mut active = user.into_active_model();
    active.password_hash = Set(bcrypt::hash(payload.new_password.clone(), 10)?);
    let _ = active.update(&state.db).await?;
    Ok(ApiResponse::success(true))
}

// Create User
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<UserCreatePayload>,
) -> Result<ApiResponse<users::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: UserCreatePayload) -> Result<users::Model, AppError> {
    let txn = state.db.begin().await?;
    let password_hash = bcrypt::hash(req.password, 10)?;

    // 1. Create user
    let new_user = users::ActiveModel {
        username: Set(req.username),
        password_hash: Set(password_hash),
        ..Default::default()
    };
    let user = new_user.insert(&txn).await?;

    // 2. Assign role
    if let Some(role_ids) = req.role_ids {
        for role_id in role_ids {
            let new_user_role = user_roles::ActiveModel {
                user_id: Set(user.id),
                role_id: Set(role_id),
            };
            new_user_role.insert(&txn).await?;
        }
    }

    // 3. Assign classes if provided
    if let Some(class_ids) = req.class_ids {
        let teacher_class_models: Vec<teacher_classes::ActiveModel> = class_ids
            .into_iter()
            .map(|class_id| teacher_classes::ActiveModel {
                user_id: Set(user.id),
                class_id: Set(class_id),
            })
            .collect();
        if !teacher_class_models.is_empty() {
            teacher_classes::Entity::insert_many(teacher_class_models)
                .exec(&txn)
                .await?;
        }
    }

    txn.commit().await?;
    Ok(user)
}

// Update User
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UserUpdatePayload>,
) -> Result<ApiResponse<users::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(user))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: UserUpdatePayload,
) -> Result<users::Model, AppError> {
    let txn = state.db.begin().await?;
    let user = users::Entity::find_by_id(id).one(&txn).await?;
    let user = user.ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;
    let mut user_active_model: users::ActiveModel = user.into();

    if let Some(password) = req.password {
        let hashed_password = bcrypt::hash(password, 10)?;
        user_active_model.password_hash = Set(hashed_password);
    }

    if let Some(role_ids) = req.role_ids {
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(id))
            .exec(&txn)
            .await?;
        for role_id in role_ids {
            user_roles::ActiveModel {
                user_id: Set(id),
                role_id: Set(role_id),
            }
            .insert(&txn)
            .await?;
        }
    }
    if let Some(class_ids) = req.class_ids {
        teacher_classes::Entity::delete_many()
            .filter(teacher_classes::Column::UserId.eq(id))
            .exec(&txn)
            .await?;

        if !class_ids.is_empty() {
            let models: Vec<teacher_classes::ActiveModel> = class_ids
                .into_iter()
                .map(|class_id| teacher_classes::ActiveModel {
                    user_id: Set(id),
                    class_id: Set(class_id),
                })
                .collect();
            teacher_classes::Entity::insert_many(models)
                .exec(&txn)
                .await?;
        }
    }

    let user = user_active_model.update(&txn).await?;
    txn.commit().await?;
    Ok(user)
}

// Delete User
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claim = depot.obtain::<Claims>().unwrap();
    let id = id.into_inner();
    //cant delete self
    if id == claim.user_id {
        return Err(AppError::Message("cannot delete self".to_string()));
    }
    delete_impl(&state, id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let user = users::Entity::find_by_id(id).one(&state.db).await?;
    let user = user.ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;
    let _ = user.delete(&state.db).await?;
    Ok(())
}

// Get Users List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<UserInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchUsersParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub fn get_query() -> Select<data_model::users::Entity> {
    let user_role_alias = Alias::new("user_role");
    let query = users::Entity::find()
        .join_as(
            JoinType::LeftJoin,
            users::Relation::UserRoles.def(),
            user_role_alias.clone(),
        )
        .select_only()
        .column_as(users::Column::Id, "id")
        .column_as(users::Column::Username, "username")
        .column_as(users::Column::CreatedAt, "created_at")
        .column_as(
            Expr::cust("COALESCE(ARRAY_AGG(user_role.role_id) FILTER (WHERE user_role.role_id IS NOT NULL), '{}'::integer[])"),
            "role_ids",
        )
        .group_by(users::Column::Id)
        .group_by(users::Column::Username)
        .group_by(users::Column::CreatedAt);
    query
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchUsersParams,
) -> Result<PagingResponse<UserInfo>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);
    let mut query = get_query();
    crate::filter_if_some!(query, users::Column::Id, params.id, eq);
    crate::filter_if_some!(query, users::Column::Username, params.username, like);
    let paginator = query
        .into_model::<UserInfo>()
        .paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let list = paginator.fetch_page(page - 1).await?;
    Ok(PagingResponse { list, total, page })
}

// Get User by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<UserInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(user))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<UserInfo, AppError> {
    let query = get_query();
    let query = query.filter(users::Column::Id.eq(id));
    let result: Option<UserInfo> = query.into_model::<UserInfo>().one(&state.db).await?;
    let user = result.ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;
    return Ok(user);
}
