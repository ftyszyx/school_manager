use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::convert::from_str_optional;
use data_model::roles;
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct RoleCreatePayload {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct RoleUpdatePayload {
    #[validate(length(min = 1, max = 50))]
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult, ToSchema)]
pub struct RoleInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchRolesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub name: Option<String>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub id: Option<i32>,
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
    let new_role = roles::ActiveModel {
        name: Set(req.name),
        description: Set(req.description),
        ..Default::default()
    };
    let role = new_role.insert(&state.db).await?;
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
    let role = roles::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;

    let mut role_active_model: roles::ActiveModel = role.into();

    if let Some(name) = req.name {
        role_active_model.name = Set(name);
    }
    if let Some(description) = req.description {
        role_active_model.description = Set(Some(description));
    }

    let role = role_active_model.update(&state.db).await?;
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

    crate::filter_if_some!(query, roles::Column::Id, params.id, eq);
    crate::filter_if_some!(query, roles::Column::Name, params.name, like);

    let paginator = query.into_model().paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let list = paginator.fetch_page(page - 1).await?;

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
        .into_model::<RoleInfo>()
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;
    Ok(role)
}
