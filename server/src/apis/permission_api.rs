use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use data_model::permissions;
use data_model::role_permissions;
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use validator::Validate;
use wildmatch::WildMatch;
use std::time::Duration;

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct PermissionCreatePayload {
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct PermissionUpdatePayload {
    pub name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult, ToSchema)]
pub struct PermissionInfo {
    pub id: i32,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchPermissionsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

// Create Permission
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<PermissionCreatePayload>,
) -> Result<ApiResponse<permissions::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(
    state: &AppState,
    req: PermissionCreatePayload,
) -> Result<permissions::Model, AppError> {
    let new_permission = permissions::ActiveModel {
        name: Set(req.name),
        resource: Set(req.resource),
        action: Set(req.action),
        description: Set(req.description),
        ..Default::default()
    };
    let permission = new_permission.insert(&state.db).await?;
    Ok(permission)
}

// Update Permission
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<PermissionUpdatePayload>,
) -> Result<ApiResponse<permissions::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let permission = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(permission))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: PermissionUpdatePayload,
) -> Result<permissions::Model, AppError> {
    let permission = permissions::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("permissions".to_string(), Some(id)))?;

    let mut permission_active_model: permissions::ActiveModel = permission.into();

    if let Some(name) = req.name {
        permission_active_model.name = Set(name);
    }
    if let Some(resource) = req.resource {
        permission_active_model.resource = Set(resource);
    }
    if let Some(action) = req.action {
        permission_active_model.action = Set(action);
    }
    if let Some(description) = req.description {
        permission_active_model.description = Set(Some(description));
    }

    let permission = permission_active_model.update(&state.db).await?;
    Ok(permission)
}

// Delete Permission
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    delete_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let permission = permissions::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("permissions".to_string(), Some(id)))?;
    let _ = permission.delete(&state.db).await?;
    Ok(())
}

// Get Permissions List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<PermissionInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchPermissionsParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchPermissionsParams,
) -> Result<PagingResponse<PermissionInfo>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

    let mut query = permissions::Entity::find();

    crate::filter_if_some!(query, permissions::Column::Name, params.name, like);
    crate::filter_if_some!(query, permissions::Column::Resource, params.resource, eq);
    crate::filter_if_some!(query, permissions::Column::Action, params.action, eq);


    let paginator = query.into_model().paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let list = paginator.fetch_page(page - 1).await?;

    Ok(PagingResponse { list, total, page })
}

// Get Permission by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<PermissionInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let permission = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(permission))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<PermissionInfo, AppError> {
    let permission = permissions::Entity::find_by_id(id)
        .into_model::<PermissionInfo>()
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("permissions".to_string(), Some(id)))?;
    Ok(permission)
}


pub async fn get_role_permissions(state: &AppState, role_ids: Vec<i32>) -> Result<Vec<permissions::Model>, AppError> {
    let role_permissions = role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.is_in(role_ids))
        .all(&state.db)
        .await?;
    let permission_ids: Vec<i32> = role_permissions.iter().map(|rp| rp.permission_id).collect();
    let permissions = permissions::Entity::find()
        .filter(permissions::Column::Id.is_in(permission_ids))
        .all(&state.db)
        .await?;
    Ok(permissions)
}


pub async  fn check_path_permission_and_cache(
    state: &AppState,
    user_id: i32,
    role_ids: Vec<i32>,
    method: &str,
    path: &str,
) -> Result<bool, AppError> {
    //get permission ids by role ids
    let action=get_path_action(method).await?;
    let cache_key = format!("user_permissions:{}", user_id);
    let cache_result = state.redis.get::<Vec<permissions::Model>>(&cache_key).await?;
    if cache_result.is_some() {
        let result=check_permission(path,&action,&cache_result.unwrap()).await?;
        return Ok(result);
    }
    let permissions=get_role_permissions(state, role_ids).await?;
    let result=check_permission(path,&action,&permissions).await?;
    state.redis.set(&cache_key, &permissions, Some(Duration::from_secs(60 * 60 * 24))).await?;
    Ok(result)
}

pub async  fn clean_user_permissions_cache(
    state: &AppState,
    user_id: i32,
) -> Result<(), AppError> {
    let cache_key = format!("user_permissions:{}", user_id);
    state.redis.del(&cache_key).await?;
    Ok(())
}

async  fn get_path_action(method: &str) -> Result<String, AppError> {
    let action=match method.to_uppercase().as_str() {
        "GET" => "READ",
        "POST" => "CREATE",
        "PUT" => "UPDATE",
        "DELETE" => "DELETE",
        _ => return Err(AppError::InternalError { message: "Invalid method".to_string() }),
    };
    Ok(action.to_string())
}

async  fn check_permission(res:&str,action:&str,permissions:&Vec<permissions::Model>) -> Result<bool, AppError> {
    for permissioninfo in permissions{
        if permissioninfo.action=="*" || permissioninfo.action==action{
            if WildMatch::new(&permissioninfo.resource).matches(res){
                return Ok(true);
            }
        }
    }
    Ok(false)
}
