use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::convert::from_str_optional;
use data_model::classes;
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct ClassCreatePayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub school_id: i32,
    pub status: Option<i32>,
    #[validate(length(max = 255))]
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct ClassUpdatePayload {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    pub grade: Option<i32>,
    pub class: Option<i32>,
    pub school_id: Option<i32>,
    pub status: Option<i32>,
    #[validate(length(max = 255))]
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult, ToSchema)]
pub struct ClassInfo {
    pub id: i32,
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub school_id: i32,
    pub status: i32,
    pub password: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchClassesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub name: Option<String>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub school_id: Option<i32>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub grade: Option<i32>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub class: Option<i32>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub id: Option<i32>,
}

// Create Class
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<ClassCreatePayload>,
) -> Result<ApiResponse<classes::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: ClassCreatePayload) -> Result<classes::Model, AppError> {
    let new_class = classes::ActiveModel {
        name: Set(req.name),
        grade: Set(req.grade),
        class: Set(req.class),
        school_id: Set(req.school_id),
        status: Set(req.status.unwrap_or(0)),
        password: Set(req.password.unwrap_or("".to_string())),
        ..Default::default()
    };
    let class = new_class.insert(&state.db).await?;
    Ok(class)
}

// Update Class
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<ClassUpdatePayload>,
) -> Result<ApiResponse<classes::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let class = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(class))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: ClassUpdatePayload,
) -> Result<classes::Model, AppError> {
    let class = classes::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("classes".to_string(), Some(id)))?;

    let mut class_active_model: classes::ActiveModel = class.into();

    if let Some(name) = req.name {
        class_active_model.name = Set(name);
    }
    if let Some(grade) = req.grade {
        class_active_model.grade = Set(grade);
    }
    if let Some(class) = req.class {
        class_active_model.class = Set(class);
    }
    if let Some(school_id) = req.school_id {
        class_active_model.school_id = Set(school_id);
    }
    if let Some(status) = req.status {
        class_active_model.status = Set(status);
    }
    if let Some(password) = req.password {
        class_active_model.password = Set(password);
    }

    let class = class_active_model.update(&state.db).await?;
    Ok(class)
}

// Delete Class
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    delete_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let class = classes::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("classes".to_string(), Some(id)))?;
    let _ = class.delete(&state.db).await?;
    Ok(())
}

// Get Classes List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<ClassInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchClassesParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchClassesParams,
) -> Result<PagingResponse<ClassInfo>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

    let mut query = classes::Entity::find();

    crate::filter_if_some!(query, classes::Column::Id, params.id, eq);
    crate::filter_if_some!(query, classes::Column::Name, params.name, like);
    crate::filter_if_some!(query, classes::Column::SchoolId, params.school_id, eq);
    crate::filter_if_some!(query, classes::Column::Grade, params.grade, eq);
    crate::filter_if_some!(query, classes::Column::Class, params.class, eq);

    let paginator = query.into_model().paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let list = paginator.fetch_page(page - 1).await?;

    Ok(PagingResponse { list, total, page })
}

// Get Class by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<ClassInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let class = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(class))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<ClassInfo, AppError> {
    let class = classes::Entity::find_by_id(id)
        .into_model::<ClassInfo>()
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("classes".to_string(), Some(id)))?;
    Ok(class)
}
