use crate::database::models::OverlayTemplate as DbOverlayTemplate;
use crate::entity::overlay_template;
use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

fn map_model(model: overlay_template::Model) -> DbOverlayTemplate {
    DbOverlayTemplate {
        id: Some(model.id as i64),
        name: model.name,
        description: model.description,
        theme: model.theme,
        colors: model.colors,
        animation_type: model.animation_type,
        duration_ms: model.duration_ms,
        is_active: model.is_active,
        url: model.url,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

/// Fetch every overlay template sorted by name.
pub async fn list_templates(conn: &DatabaseConnection) -> Result<Vec<DbOverlayTemplate>, DbErr> {
    let records = overlay_template::Entity::find()
        .order_by_asc(overlay_template::Column::Name)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_model).collect())
}

/// Fetch only templates marked as active.
pub async fn list_active_templates(
    conn: &DatabaseConnection,
) -> Result<Vec<DbOverlayTemplate>, DbErr> {
    let records = overlay_template::Entity::find()
        .filter(overlay_template::Column::IsActive.eq(true))
        .order_by_asc(overlay_template::Column::Name)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_model).collect())
}

/// Locate an overlay template by its unique name.
pub async fn find_template_by_name(
    conn: &DatabaseConnection,
    name: &str,
) -> Result<Option<DbOverlayTemplate>, DbErr> {
    let record = overlay_template::Entity::find()
        .filter(overlay_template::Column::Name.eq(name))
        .one(conn)
        .await?;

    Ok(record.map(map_model))
}

/// Upsert an overlay template using its id or name.
pub async fn upsert_template(
    conn: &DatabaseConnection,
    data: &DbOverlayTemplate,
) -> Result<DbOverlayTemplate, DbErr> {
    let existing = if let Some(id) = data.id {
        overlay_template::Entity::find_by_id(id as i32)
            .one(conn)
            .await?
    } else {
        overlay_template::Entity::find()
            .filter(overlay_template::Column::Name.eq(data.name.clone()))
            .one(conn)
            .await?
    };

    let now = Utc::now().naive_utc();

    if let Some(model) = existing {
        let mut active: overlay_template::ActiveModel = model.into();
        active.name = Set(data.name.clone());
        active.description = Set(data.description.clone());
        active.theme = Set(data.theme.clone());
        active.colors = Set(data.colors.clone());
        active.animation_type = Set(data.animation_type.clone());
        active.duration_ms = Set(data.duration_ms);
        active.is_active = Set(data.is_active);
        active.url = Set(data.url.clone());
        active.updated_at = Set(now);

        let saved = active.update(conn).await?;
        Ok(map_model(saved))
    } else {
        let active = overlay_template::ActiveModel {
            name: Set(data.name.clone()),
            description: Set(data.description.clone()),
            theme: Set(data.theme.clone()),
            colors: Set(data.colors.clone()),
            animation_type: Set(data.animation_type.clone()),
            duration_ms: Set(data.duration_ms),
            is_active: Set(data.is_active),
            url: Set(data.url.clone()),
            created_at: Set(data.created_at.naive_utc()),
            updated_at: Set(now),
            ..Default::default()
        };

        let saved = active.insert(conn).await?;
        Ok(map_model(saved))
    }
}

/// Delete a template by its primary key.
pub async fn delete_template(conn: &DatabaseConnection, id: i64) -> Result<(), DbErr> {
    overlay_template::Entity::delete_by_id(id as i32)
        .exec(conn)
        .await?;
    Ok(())
}

/// Remove all overlay templates (used for seed/populate flows).
pub async fn clear_templates(conn: &DatabaseConnection) -> Result<(), DbErr> {
    overlay_template::Entity::delete_many().exec(conn).await?;
    Ok(())
}
