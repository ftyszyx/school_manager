use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::convert::from_str_optional;
use data_model::{classes, schools, teacher_classes, users};
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;
use crate::apis::auth_middleware::Claims;

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


#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct ClassBulkCreatePayload {
    pub classes: Vec<ClassCreatePayload>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UserClassInfo {
    pub user_id: i32,
    pub user_name: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct ClassInfo {
    pub id: i32,
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub school_id: i32,
    pub school_name: String,
    pub status: i32,
    pub password: String,
    pub teacher_infos: Vec<UserClassInfo>,
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
    #[serde(deserialize_with = "from_str_optional", default)]
    pub status: Option<i32>,
}

#[derive(Deserialize, Debug, Validate, ToSchema)]
pub struct ClassStatusUpdatePayload {
    pub status: i32,
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

#[handler]
pub async fn add_bulk(
    depot: &mut Depot,
    req: JsonBody<ClassBulkCreatePayload>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let new_classes: Vec<classes::ActiveModel> = req
        .classes
        .iter()
        .map(|c| classes::ActiveModel {
            name: Set(c.name.clone()),
            grade: Set(c.grade),
            class: Set(c.class),
            school_id: Set(c.school_id),
            status: Set(c.status.unwrap_or(0)),
            password: Set(c.password.clone().unwrap_or("".to_string())),
            ..Default::default()
        })
        .collect();

    classes::Entity::insert_many(new_classes).exec(&state.db).await?;
    Ok(ApiResponse::success(()))
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

async fn enrich_classes_with_details(
    state: &AppState,
    class_models: Vec<classes::Model>,
) -> Result<Vec<ClassInfo>, AppError> {
    if class_models.is_empty() {
        return Ok(vec![]);
    }
    let class_ids: Vec<i32> = class_models.iter().map(|c| c.id).collect();

    let school_ids: Vec<i32> = class_models.iter().map(|c| c.school_id).collect();
    let schools_map: HashMap<i32, schools::Model> = if school_ids.is_empty() {
        HashMap::new()
    } else {
        schools::Entity::find()
            .filter(schools::Column::Id.is_in(school_ids.clone()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|s| (s.id, s))
            .collect()
    };

    let teacher_classes_list = teacher_classes::Entity::find()
        .filter(teacher_classes::Column::ClassId.is_in(class_ids.clone()))
        .all(&state.db)
        .await?;
    
    let user_ids: Vec<i32> = teacher_classes_list.iter().map(|tc| tc.user_id).collect();

    let users_map: HashMap<i32, users::Model> = if user_ids.is_empty() {
        HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(user_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    let list = class_models
        .into_iter()
        .map(|class| {
            let teacher_infos: Vec<UserClassInfo> = teacher_classes_list
                .iter()
                .filter(|tc| tc.class_id == class.id)
                .filter_map(|tc| {
                    users_map.get(&tc.user_id).map(|u| UserClassInfo {
                        user_id: u.id,
                        user_name: u.username.clone(),
                    })
                })
                .collect();
            
            let school_name = schools_map.get(&class.school_id).map(|s| s.name.clone()).unwrap_or_default();
            ClassInfo {
                id: class.id,
                name: class.name,
                grade: class.grade,
                class: class.class,
                school_id: class.school_id,
                school_name,
                status: class.status,
                password: class.password,
                teacher_infos,
            }
        })
        .collect();

    Ok(list)
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
    crate::filter_if_some!(query, classes::Column::Status, params.status, eq);

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await?;
    let class_models = paginator.fetch_page(page - 1).await?;
    
    let list = enrich_classes_with_details(state, class_models).await?;

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
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("classes".to_string(), Some(id)))?;
    
    let mut class_infos = enrich_classes_with_details(state, vec![class]).await?;
    if class_infos.is_empty() {
        return Err(AppError::not_found("classes".to_string(), Some(id)));
    }
    Ok(class_infos.remove(0))
}

#[handler]
pub async fn update_status(
    depot: &mut Depot,
    class_id: PathParam<i32>,
    req: JsonBody<ClassStatusUpdatePayload>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    //check if the user is a teacher
    let claims = depot.obtain::<Claims>().unwrap();
    let teacher_class = teacher_classes::Entity::find()
        .filter(teacher_classes::Column::UserId.eq(claims.user_id))
        .filter(teacher_classes::Column::ClassId.eq(class_id.into_inner()))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("users".to_string(), Some(claims.user_id)))?;
    let class = classes::Entity::find_by_id(teacher_class.class_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("classes".to_string(), Some(teacher_class.class_id)))?;
    let mut class_active_model: classes::ActiveModel = class.into();
    class_active_model.status = Set(req.status);
    class_active_model.update(&state.db).await?;
    Ok(ApiResponse::success(()))
}
