use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::convert::from_str_optional;
use data_model::schools;
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, ToSchema)]
pub struct SchoolCreatePayload {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct SchoolUpdatePayload {
    pub name: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, FromQueryResult, ToSchema)]
pub struct SchoolInfo {
    pub id: i32,
    pub name: String,
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchSchoolsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub name: Option<String>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub id: Option<i32>,
}

// Create School
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<SchoolCreatePayload>,
) -> Result<ApiResponse<schools::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: SchoolCreatePayload) -> Result<schools::Model, AppError> {
    let new_school = schools::ActiveModel {
        name: Set(req.name),
        ..Default::default()
    };
    let school = new_school.insert(&state.db).await?;
    Ok(school)
}

// Update School
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<SchoolUpdatePayload>,
) -> Result<ApiResponse<schools::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let school = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(school))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: SchoolUpdatePayload,
) -> Result<schools::Model, AppError> {
    let school = schools::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("schools".to_string(), Some(id)))?;

    let mut school_active_model: schools::ActiveModel = school.into();

    if let Some(name) = req.name {
        school_active_model.name = Set(name);
    }

    let school = school_active_model.update(&state.db).await?;
    Ok(school)
}

// Delete School
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    delete_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let school = schools::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("schools".to_string(), Some(id)))?;
    let _ = school.delete(&state.db).await?;
    Ok(())
}

// Get Schools List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<SchoolInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchSchoolsParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

#[handler]
pub async fn get_all_schools(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<SchoolInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchSchoolsParams>()?;
    let res= get_list_impl(&state, params).await?;
    let list= res.list.into_iter().map(|s| SchoolInfo {
        id: s.id,
        name: s.name.clone(),
        password: None,
    }).collect();
    Ok(ApiResponse::success(PagingResponse { list, total: res.total, page: res.page }))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchSchoolsParams,
) -> Result<PagingResponse<SchoolInfo>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

    let mut query = schools::Entity::find();

    crate::filter_if_some!(query, schools::Column::Id, params.id, eq);
    crate::filter_if_some!(query, schools::Column::Name, params.name, like);

    let paginator = query.into_model().paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let list = paginator.fetch_page(page - 1).await?;

    Ok(PagingResponse { list, total, page })
}

// Get School by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<SchoolInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let school = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(school))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<SchoolInfo, AppError> {
    let school = schools::Entity::find_by_id(id)
        .into_model::<SchoolInfo>()
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("schools".to_string(), Some(id)))?;
    Ok(school)
}
