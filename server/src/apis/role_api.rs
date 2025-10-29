use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::apis::permission_api::{PermissionInfo, get_permission_infos};
use data_model::{role_permissions, roles};
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct RoleCreatePayload {
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct RoleUpdatePayload {
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Serialize, Debug,  ToSchema, Clone)]
pub struct RoleInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permission_infos: Option<Vec<PermissionInfo>>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchRolesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub name: Option<String>,
}

// Create Role
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<RoleCreatePayload>,
) -> Result<ApiResponse<roles::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: RoleCreatePayload) -> Result<roles::Model, AppError> {
    let txn = state.db.begin().await?;
    let new_role = roles::ActiveModel {
        name: Set(req.name),
        description: Set(req.description),
        ..Default::default()
    };
    let role = new_role.insert(&txn).await?;
    if let Some(permission_ids) = req.permission_ids {
        for permission_id in permission_ids {
            let new_role_permission = role_permissions::ActiveModel {
                role_id: Set(role.id),
                permission_id: Set(permission_id),
            };
            new_role_permission.insert(&txn).await?;
        }
    }
    txn.commit().await?;
    Ok(role)
}

// Update Role
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<RoleUpdatePayload>,
) -> Result<ApiResponse<roles::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let role = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(role))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: RoleUpdatePayload,
) -> Result<roles::Model, AppError> {
    let txn = state.db.begin().await?;
    let role = roles::Entity::find_by_id(id)
        .one(&txn)
        .await?
        .ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;

    let mut role_active_model: roles::ActiveModel = role.into();

    if let Some(name) = req.name {
        role_active_model.name = Set(name);
    }
    if let Some(description) = req.description {
        role_active_model.description = Set(Some(description));
    }
    if let Some(permission_ids) = req.permission_ids {
        role_permissions::Entity::delete_many()
            .filter(role_permissions::Column::RoleId.eq(id))
            .exec(&txn)
            .await?;
        for permission_id in permission_ids {
            let new_role_permission = role_permissions::ActiveModel {
                role_id: Set(id),
                permission_id: Set(permission_id),
            };
            new_role_permission.insert(&txn).await?;
        }
    }
    let role = role_active_model.update(&state.db).await?;
    txn.commit().await?;
    Ok(role)
}

// Delete Role
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    delete_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let role = roles::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;
    let _ = role.delete(&state.db).await?;
    Ok(())
}

// Get Roles List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<RoleInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchRolesParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchRolesParams,
) -> Result<PagingResponse<RoleInfo>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

    let mut query = roles::Entity::find();
    crate::filter_if_some!(query, roles::Column::Name, params.name, like);
    let paginator = query.paginate(&state.db,page_size);
    let total = paginator.num_items().await?;
    let role_models = paginator.fetch_page(page - 1).await?;
    let list = enrich_roles_with_details(state, role_models).await?;
    Ok(PagingResponse { list, total, page })
}

// Get Role by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<RoleInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let role = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(role))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<RoleInfo, AppError> {
    let role = roles::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;
    let mut role_infos=enrich_roles_with_details(state, vec![role]).await?;
    Ok(role_infos.remove(0))
}

pub async fn enrich_roles_with_details(state: &AppState, roles: Vec<roles::Model>) -> Result<Vec<RoleInfo>, AppError> {
    let role_ids=roles.iter().map(|r| r.id).collect();
    let permission_infos = get_permission_infos(&state, role_ids).await?;
    Ok(roles.into_iter().map(|r| RoleInfo {
        id: r.id,
        name: r.name,
        description: r.description,
        permission_infos: Some(permission_infos.clone()),
    }).collect())
}