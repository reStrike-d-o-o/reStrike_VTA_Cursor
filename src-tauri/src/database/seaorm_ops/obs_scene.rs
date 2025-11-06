use crate::database::models::ObsScene as DbObsScene;
use crate::entity::obs_scene;
use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

fn parse_last_seen(value: &str) -> Result<DateTime<Utc>, DbErr> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| DbErr::Custom(err.to_string()))
}

fn map_model(model: obs_scene::Model) -> Result<DbObsScene, DbErr> {
    Ok(DbObsScene {
        id: Some(model.id as i64),
        scene_name: model.scene_name,
        scene_id: model.scene_id,
        is_active: model.is_active,
        last_seen_at: parse_last_seen(&model.last_seen_at)?,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    })
}

/// Retrieve all OBS scenes sorted by name.
pub async fn list_scenes(conn: &DatabaseConnection) -> Result<Vec<DbObsScene>, DbErr> {
    let records = obs_scene::Entity::find()
        .order_by_asc(obs_scene::Column::SceneName)
        .all(conn)
        .await?;

    let mut scenes = Vec::with_capacity(records.len());
    for record in records {
        scenes.push(map_model(record)?);
    }
    Ok(scenes)
}

/// Retrieve only scenes flagged as active.
pub async fn list_active_scenes(conn: &DatabaseConnection) -> Result<Vec<DbObsScene>, DbErr> {
    let records = obs_scene::Entity::find()
        .filter(obs_scene::Column::IsActive.eq(true))
        .order_by_asc(obs_scene::Column::SceneName)
        .all(conn)
        .await?;

    let mut scenes = Vec::with_capacity(records.len());
    for record in records {
        scenes.push(map_model(record)?);
    }
    Ok(scenes)
}

/// Find a scene via its unique name.
pub async fn find_scene_by_name(
    conn: &DatabaseConnection,
    scene_name: &str,
) -> Result<Option<DbObsScene>, DbErr> {
    let record = obs_scene::Entity::find()
        .filter(obs_scene::Column::SceneName.eq(scene_name))
        .one(conn)
        .await?;

    record.map(map_model).transpose()
}

/// Insert or update an OBS scene record.
pub async fn upsert_scene(
    conn: &DatabaseConnection,
    data: &DbObsScene,
) -> Result<DbObsScene, DbErr> {
    let existing = if let Some(id) = data.id {
        obs_scene::Entity::find_by_id(id as i32).one(conn).await?
    } else {
        obs_scene::Entity::find()
            .filter(obs_scene::Column::SceneName.eq(data.scene_name.clone()))
            .one(conn)
            .await?
    };

    let now = Utc::now().naive_utc();
    let last_seen = data.last_seen_at.to_rfc3339();

    if let Some(model) = existing {
        let mut active: obs_scene::ActiveModel = model.into();
        active.scene_name = Set(data.scene_name.clone());
        active.scene_id = Set(data.scene_id.clone());
        active.is_active = Set(data.is_active);
        active.last_seen_at = Set(last_seen);
        active.updated_at = Set(now);

        let saved = active.update(conn).await?;
        map_model(saved)
    } else {
        let created_at = data.created_at.naive_utc();
        let active = obs_scene::ActiveModel {
            scene_name: Set(data.scene_name.clone()),
            scene_id: Set(data.scene_id.clone()),
            is_active: Set(data.is_active),
            last_seen_at: Set(last_seen),
            created_at: Set(created_at),
            updated_at: Set(now),
            ..Default::default()
        };

        let saved = active.insert(conn).await?;
        map_model(saved)
    }
}

/// Mark a scene as inactive.
pub async fn mark_scene_inactive(conn: &DatabaseConnection, scene_name: &str) -> Result<(), DbErr> {
    if let Some(model) = obs_scene::Entity::find()
        .filter(obs_scene::Column::SceneName.eq(scene_name))
        .one(conn)
        .await?
    {
        let mut active: obs_scene::ActiveModel = model.into();
        active.is_active = Set(false);
        active.updated_at = Set(Utc::now().naive_utc());
        active.update(conn).await?;
    }
    Ok(())
}

/// Refresh the last-seen timestamp for a scene.
pub async fn update_scene_last_seen(
    conn: &DatabaseConnection,
    scene_name: &str,
) -> Result<(), DbErr> {
    if let Some(model) = obs_scene::Entity::find()
        .filter(obs_scene::Column::SceneName.eq(scene_name))
        .one(conn)
        .await?
    {
        let now = Utc::now();
        let mut active: obs_scene::ActiveModel = model.into();
        active.last_seen_at = Set(now.to_rfc3339());
        active.updated_at = Set(now.naive_utc());
        active.update(conn).await?;
    }
    Ok(())
}

/// Mark all scenes inactive, then re-activate provided names.
pub async fn sync_scenes(
    conn: &DatabaseConnection,
    active_scene_names: &[String],
) -> Result<(), DbErr> {
    let now = Utc::now();
    let updated_at = now.naive_utc();
    let last_seen = now.to_rfc3339();

    let scenes = obs_scene::Entity::find().all(conn).await?;
    for model in scenes {
        let mut active: obs_scene::ActiveModel = model.into();
        active.is_active = Set(false);
        active.updated_at = Set(updated_at);
        active.update(conn).await?;
    }

    for name in active_scene_names {
        if let Some(model) = obs_scene::Entity::find()
            .filter(obs_scene::Column::SceneName.eq(name.clone()))
            .one(conn)
            .await?
        {
            let mut active: obs_scene::ActiveModel = model.into();
            active.is_active = Set(true);
            active.last_seen_at = Set(last_seen.clone());
            active.updated_at = Set(updated_at);
            active.update(conn).await?;
        }
    }

    Ok(())
}
