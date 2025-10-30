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
use data_model::{classes, roles, schools, teacher_classes, user_roles, users};
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use validator::Validate;

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

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
    pub role_ids: Option<Vec<i32>>,
    pub class_ids: Option<Vec<i32>>,
    pub password: String,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserUpdatePayload {
    pub username: Option<String>,
    pub role_ids: Option<Vec<i32>>,
    pub class_ids: Option<Vec<i32>>,
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct ChangePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct UserClassInfo {
    pub class_id: i32,
    pub class_name: String,
    pub school_id: i32,
    pub school_name: String,
    pub grade: i32,
    pub class: i32,
    pub status: i32,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult)]
pub struct UserRoleInfo {
    pub role_id: i32,
    pub role_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub class_infos: Vec<UserClassInfo>,
    pub role_infos: Vec<UserRoleInfo>,
    pub created_at: DateTime<Utc>,
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
        None,
    )
    .await?;
    let user_info = get_by_id_impl(&state, new_user.id).await?;
    info!("User registered: {}", user_info.username);
    let role_ids = user_info.role_infos.iter().map(|r| r.role_id).collect();
    let token = create_jwt(user_info.id, role_ids, &state.config.jwt)
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
    let entity = add_impl(&state, req.into_inner(), None).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(
    state: &AppState,
    req: UserCreatePayload,
    insert_callback: Option<
        Box<dyn for<'a> FnOnce(&'a mut users::ActiveModel) -> BoxFuture<'a, Result<(), AppError>> + Send>,
    >,
) -> Result<users::Model, AppError> {
    let txn = state.db.begin().await?;
    let password_hash = bcrypt::hash(req.password, 10)?;

    // 1. Create user
    let mut new_user = users::ActiveModel {
        username: Set(req.username),
        password_hash: Set(password_hash),
        ..Default::default()
    };
    if let Some(callback) = insert_callback {
        callback(&mut new_user).await?;
    }
    let user_model = new_user.insert(&txn).await?;

    // 2. Assign role
    if let Some(role_ids) = req.role_ids {
        for role_id in role_ids {
            let new_user_role = user_roles::ActiveModel {
                user_id: Set(user_model.id),
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
                user_id: Set(user_model.id),
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
    Ok(user_model)
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
    if let Some(username) = req.username {
        user_active_model.username = Set(username);
    }

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

async fn enrich_users_with_details(
    state: &AppState,
    user_models: Vec<users::Model>,
) -> Result<Vec<UserInfo>, AppError> {
    if user_models.is_empty() {
        return Ok(vec![]);
    }
    let user_ids: Vec<i32> = user_models.iter().map(|u| u.id).collect();

    // Fetch all related data in batches
    let user_roles_list = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.is_in(user_ids.clone()))
        .all(&state.db)
        .await?;
    let teacher_classes_list = teacher_classes::Entity::find()
        .filter(teacher_classes::Column::UserId.is_in(user_ids.clone()))
        .all(&state.db)
        .await?;

    let role_ids: Vec<i32> = user_roles_list.iter().map(|ur| ur.role_id).collect();
    let class_ids: Vec<i32> = teacher_classes_list.iter().map(|tc| tc.class_id).collect();

    let roles_map: HashMap<i32, roles::Model> = if role_ids.is_empty() {
        HashMap::new()
    } else {
        roles::Entity::find()
            .filter(roles::Column::Id.is_in(role_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|r| (r.id, r))
            .collect()
    };

    let classes_map: HashMap<i32, classes::Model> = if class_ids.is_empty() {
        HashMap::new()
    } else {
        classes::Entity::find()
            .filter(classes::Column::Id.is_in(class_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|c| (c.id, c))
            .collect()
    };

    let school_ids: Vec<i32> = classes_map.values().map(|c| c.school_id).collect();
    let schools_map: HashMap<i32, schools::Model> = if school_ids.is_empty() {
        HashMap::new()
    } else {
        schools::Entity::find()
            .filter(schools::Column::Id.is_in(school_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s))
            .collect()
    };

    // Map data to UserInfo
    let list = user_models
        .into_iter()
        .map(|user| {
            let role_infos: Vec<UserRoleInfo> = user_roles_list
                .iter()
                .filter(|ur| ur.user_id == user.id)
                .filter_map(|ur| {
                    roles_map.get(&ur.role_id).map(|r| UserRoleInfo {
                        role_id: r.id,
                        role_name: r.name.clone(),
                    })
                })
                .collect();

            let class_infos: Vec<UserClassInfo> = teacher_classes_list
                .iter()
                .filter(|tc| tc.user_id == user.id)
                .filter_map(|tc| {
                    classes_map.get(&tc.class_id).and_then(|c| {
                        schools_map.get(&c.school_id).map(|s| UserClassInfo {
                            class_id: c.id,
                            class_name: c.name.clone(),
                            school_id: s.id,
                            school_name: s.name.clone(),
                            grade: c.grade,
                            class: c.class,
                            status: c.status,
                        })
                    })
                })
                .collect();

            UserInfo {
                id: user.id,
                username: user.username,
                created_at: user.created_at.into(),
                role_infos,
                class_infos,
            }
        })
        .collect();
    Ok(list)
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchUsersParams,
) -> Result<PagingResponse<UserInfo>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

    let mut query = users::Entity::find();
    crate::filter_if_some!(query, users::Column::Id, params.id, eq);
    crate::filter_if_some!(query, users::Column::Username, params.username, like);

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let user_models = paginator.fetch_page(page - 1).await?;
    let list = enrich_users_with_details(state, user_models).await?;

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
    let user = users::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;

    let mut user_infos = enrich_users_with_details(state, vec![user]).await?;
    if user_infos.is_empty() {
        return Err(AppError::not_found("users".to_string(), Some(id)));
    }
    Ok(user_infos.remove(0))
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct BindClassPayload {
    pub class_id: i32,
    pub password: String,
}

#[handler]
pub async fn bind_class(
    depot: &mut Depot,
    req: JsonBody<BindClassPayload>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();

    let class = classes::Entity::find()
        .filter(
            Condition::all()
                .add(classes::Column::Password.eq(&req.password))
                .add(classes::Column::Id.eq(req.class_id)),
        )
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("class with this password".to_string(), None))?;

    let existing_binding = teacher_classes::Entity::find()
        .filter(
            Condition::all()
                .add(teacher_classes::Column::UserId.eq(claims.user_id))
                .add(teacher_classes::Column::ClassId.eq(class.id)),
        )
        .one(&state.db)
        .await?;

    if existing_binding.is_some() {
        return Err(AppError::Message("Already bound to this class".to_string()));
    }

    let new_binding = teacher_classes::ActiveModel {
        user_id: Set(claims.user_id),
        class_id: Set(class.id),
    };
    new_binding.insert(&state.db).await?;

    Ok(ApiResponse::success(()))
}

#[handler]
pub async fn unbind_class(
    depot: &mut Depot,
    class_id: PathParam<i32>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();

    let result = teacher_classes::Entity::delete_many()
        .filter(
            Condition::all()
                .add(teacher_classes::Column::UserId.eq(claims.user_id))
                .add(teacher_classes::Column::ClassId.eq(class_id.into_inner())),
        )
        .exec(&state.db)
        .await?;

    if result.rows_affected == 0 {
        return Err(AppError::not_found(
            "binding for this class".to_string(),
            None,
        ));
    }

    Ok(ApiResponse::success(()))
}

#[handler]
pub async fn logout() -> Result<ApiResponse<()>, AppError> {
    Ok(ApiResponse::success(()))
}
