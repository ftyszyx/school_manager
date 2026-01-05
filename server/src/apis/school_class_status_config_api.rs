use crate::core::app::AppState;
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use data_model::{school_class_status_configs, schools};
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
pub struct SchoolClassStatusConfigItem {
    pub status: i32,
    pub label: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub sort_order: i32,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct UpdateSchoolClassStatusConfigsPayload {
    pub configs: Vec<SchoolClassStatusConfigItem>,
}

pub async fn get_configs_by_school_id(
    state: &AppState,
    school_id: i32,
) -> Result<Vec<SchoolClassStatusConfigItem>, AppError> {
    let configs = school_class_status_configs::Entity::find()
        .filter(school_class_status_configs::Column::SchoolId.eq(school_id))
        .order_by_asc(school_class_status_configs::Column::SortOrder)
        .all(&state.db)
        .await?;

    Ok(configs
        .into_iter()
        .map(|c| SchoolClassStatusConfigItem {
            status: c.status,
            label: c.label,
            r#type: c.r#type,
            sort_order: c.sort_order,
        })
        .collect())
}

#[handler]
pub async fn get_by_school_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<Vec<SchoolClassStatusConfigItem>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let school_id = id.into_inner();

    // Ensure school exists
    schools::Entity::find_by_id(school_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("schools".to_string(), Some(school_id)))?;

    let list = get_configs_by_school_id(state, school_id).await?;
    Ok(ApiResponse::success(list))
}

#[handler]
pub async fn update_by_school_id(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UpdateSchoolClassStatusConfigsPayload>,
) -> Result<ApiResponse<Vec<SchoolClassStatusConfigItem>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let school_id = id.into_inner();

    // Ensure school exists
    schools::Entity::find_by_id(school_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("schools".to_string(), Some(school_id)))?;

    let txn = state.db.begin().await?;

    for cfg in req.configs.iter() {
        let existing = school_class_status_configs::Entity::find()
            .filter(
                Condition::all()
                    .add(school_class_status_configs::Column::SchoolId.eq(school_id))
                    .add(school_class_status_configs::Column::Status.eq(cfg.status)),
            )
            .one(&txn)
            .await?;

        match existing {
            Some(model) => {
                let mut active: school_class_status_configs::ActiveModel = model.into();
                active.label = Set(cfg.label.clone());
                active.r#type = Set(cfg.r#type.clone());
                active.sort_order = Set(cfg.sort_order);
                let _ = active.update(&txn).await?;
            }
            None => {
                let active = school_class_status_configs::ActiveModel {
                    school_id: Set(school_id),
                    status: Set(cfg.status),
                    label: Set(cfg.label.clone()),
                    r#type: Set(cfg.r#type.clone()),
                    sort_order: Set(cfg.sort_order),
                    ..Default::default()
                };
                let _ = active.insert(&txn).await?;
            }
        }
    }

    txn.commit().await?;

    let list = get_configs_by_school_id(state, school_id).await?;
    Ok(ApiResponse::success(list))
}
