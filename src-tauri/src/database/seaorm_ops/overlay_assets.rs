use crate::database::models::{OvrAnthemAsset, OvrFlagAnimationAsset};
use crate::entity::{overlay_anthem, overlay_flag_animation};
use chrono::{TimeZone, Utc};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

fn to_i64(duration: Option<i32>) -> Option<i64> {
    duration.map(|value| i64::from(value))
}

fn clamp_duration(duration: Option<i64>) -> Option<i32> {
    duration.map(|value| value.clamp(0, i64::from(i32::MAX)) as i32)
}

fn map_flag_model(model: overlay_flag_animation::Model) -> OvrFlagAnimationAsset {
    OvrFlagAnimationAsset {
        id: model.id,
        ioc_code: model.ioc_code,
        file_name: model.file_name,
        file_path: model.file_path,
        display_name: model.display_name,
        duration_ms: to_i64(model.duration_ms),
        is_default: model.is_default,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn map_anthem_model(model: overlay_anthem::Model) -> OvrAnthemAsset {
    OvrAnthemAsset {
        id: model.id,
        ioc_code: model.ioc_code,
        file_name: model.file_name,
        file_path: model.file_path,
        display_name: model.display_name,
        duration_ms: to_i64(model.duration_ms),
        is_default: model.is_default,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn resolve_asset_id(raw_id: &str) -> String {
    let trimmed = raw_id.trim();
    if trimmed.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        trimmed.to_string()
    }
}

/// List all overlay flag animation assets ordered by IOC code then filename.
pub async fn list_flag_animations(
    conn: &DatabaseConnection,
) -> Result<Vec<OvrFlagAnimationAsset>, DbErr> {
    let records = overlay_flag_animation::Entity::find()
        .order_by_asc(overlay_flag_animation::Column::IocCode)
        .order_by_asc(overlay_flag_animation::Column::FileName)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_flag_model).collect())
}

/// Insert or update a flag animation asset, returning the saved record.
pub async fn upsert_flag_animation(
    conn: &DatabaseConnection,
    asset: &OvrFlagAnimationAsset,
) -> Result<OvrFlagAnimationAsset, DbErr> {
    let asset_id = resolve_asset_id(&asset.id);
    let existing = overlay_flag_animation::Entity::find_by_id(asset_id.clone())
        .one(conn)
        .await?;

    let duration = clamp_duration(asset.duration_ms);
    let now = Utc::now().naive_utc();

    let saved = if let Some(model) = existing {
        let mut active: overlay_flag_animation::ActiveModel = model.into();
        active.ioc_code = Set(asset.ioc_code.clone());
        active.file_name = Set(asset.file_name.clone());
        active.file_path = Set(asset.file_path.clone());
        active.display_name = Set(asset.display_name.clone());
        active.duration_ms = Set(duration);
        active.is_default = Set(asset.is_default);
        active.updated_at = Set(now);
        active.update(conn).await?
    } else {
        let active = overlay_flag_animation::ActiveModel {
            id: Set(asset_id.clone()),
            ioc_code: Set(asset.ioc_code.clone()),
            file_name: Set(asset.file_name.clone()),
            file_path: Set(asset.file_path.clone()),
            display_name: Set(asset.display_name.clone()),
            duration_ms: Set(duration),
            is_default: Set(asset.is_default),
            created_at: Set(asset.created_at.naive_utc()),
            updated_at: Set(now),
        };

        active.insert(conn).await?
    };

    if asset.is_default {
        overlay_flag_animation::Entity::update_many()
            .filter(overlay_flag_animation::Column::IocCode.eq(asset.ioc_code.clone()))
            .filter(overlay_flag_animation::Column::Id.ne(saved.id.clone()))
            .col_expr(
                overlay_flag_animation::Column::IsDefault,
                Expr::value(false),
            )
            .col_expr(overlay_flag_animation::Column::UpdatedAt, Expr::value(now))
            .exec(conn)
            .await?;
    }

    Ok(map_flag_model(saved))
}

/// Delete a flag animation asset by identifier.
pub async fn delete_flag_animation(conn: &DatabaseConnection, asset_id: &str) -> Result<(), DbErr> {
    overlay_flag_animation::Entity::delete_by_id(asset_id.to_string())
        .exec(conn)
        .await?;
    Ok(())
}

/// List all overlay anthem assets ordered by IOC code then filename.
pub async fn list_anthems(conn: &DatabaseConnection) -> Result<Vec<OvrAnthemAsset>, DbErr> {
    let records = overlay_anthem::Entity::find()
        .order_by_asc(overlay_anthem::Column::IocCode)
        .order_by_asc(overlay_anthem::Column::FileName)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_anthem_model).collect())
}

/// Insert or update an anthem asset, returning the persisted record.
pub async fn upsert_anthem(
    conn: &DatabaseConnection,
    asset: &OvrAnthemAsset,
) -> Result<OvrAnthemAsset, DbErr> {
    let asset_id = resolve_asset_id(&asset.id);
    let existing = overlay_anthem::Entity::find_by_id(asset_id.clone())
        .one(conn)
        .await?;

    let duration = clamp_duration(asset.duration_ms);
    let now = Utc::now().naive_utc();

    let saved = if let Some(model) = existing {
        let mut active: overlay_anthem::ActiveModel = model.into();
        active.ioc_code = Set(asset.ioc_code.clone());
        active.file_name = Set(asset.file_name.clone());
        active.file_path = Set(asset.file_path.clone());
        active.display_name = Set(asset.display_name.clone());
        active.duration_ms = Set(duration);
        active.is_default = Set(asset.is_default);
        active.updated_at = Set(now);
        active.update(conn).await?
    } else {
        let active = overlay_anthem::ActiveModel {
            id: Set(asset_id.clone()),
            ioc_code: Set(asset.ioc_code.clone()),
            file_name: Set(asset.file_name.clone()),
            file_path: Set(asset.file_path.clone()),
            display_name: Set(asset.display_name.clone()),
            duration_ms: Set(duration),
            is_default: Set(asset.is_default),
            created_at: Set(asset.created_at.naive_utc()),
            updated_at: Set(now),
        };

        active.insert(conn).await?
    };

    if asset.is_default {
        overlay_anthem::Entity::update_many()
            .filter(overlay_anthem::Column::IocCode.eq(asset.ioc_code.clone()))
            .filter(overlay_anthem::Column::Id.ne(saved.id.clone()))
            .col_expr(overlay_anthem::Column::IsDefault, Expr::value(false))
            .col_expr(overlay_anthem::Column::UpdatedAt, Expr::value(now))
            .exec(conn)
            .await?;
    }

    Ok(map_anthem_model(saved))
}

/// Delete an anthem asset by identifier.
pub async fn delete_anthem(conn: &DatabaseConnection, asset_id: &str) -> Result<(), DbErr> {
    overlay_anthem::Entity::delete_by_id(asset_id.to_string())
        .exec(conn)
        .await?;
    Ok(())
}
