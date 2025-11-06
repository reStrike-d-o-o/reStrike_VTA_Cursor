use crate::database::models::OvrProvider as DbOvrProvider;
use crate::entity::overlay_provider;
use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};

const DEFAULT_PROVIDERS: &[(&str, &str)] = &[
    ("simplycompete", "https://www.simplycompete.com"),
    ("tpss", "https://www.tpss.eu"),
    ("martial.events", "https://martial.events"),
    ("etu", "https://europetaekwondo.org"),
];

fn parse_optional_datetime(value: Option<String>) -> Option<DateTime<Utc>> {
    value
        .and_then(|raw| DateTime::parse_from_rfc3339(&raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn format_optional_datetime(value: Option<DateTime<Utc>>) -> Option<String> {
    value.map(|dt| dt.to_rfc3339())
}

fn clamp_rate_limit(ms: i64) -> i32 {
    let clamped = ms.clamp(0, i64::from(i32::MAX));
    clamped as i32
}

fn map_model(model: overlay_provider::Model) -> DbOvrProvider {
    DbOvrProvider {
        id: Some(model.id as i64),
        name: model.name,
        base_url: model.base_url,
        enabled: model.enabled,
        rate_limit_ms: i64::from(model.rate_limit_ms),
        last_refreshed_at: parse_optional_datetime(model.last_refreshed_at),
        last_status: model.last_status,
        last_error: model.last_error,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

/// Ensure that all default overlay providers are present.
pub async fn ensure_default_providers(conn: &DatabaseConnection) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    for (name, base_url) in DEFAULT_PROVIDERS {
        let existing = overlay_provider::Entity::find()
            .filter(overlay_provider::Column::Name.eq(name.to_string()))
            .one(conn)
            .await?;

        if existing.is_none() {
            let active = overlay_provider::ActiveModel {
                name: Set(name.to_string()),
                base_url: Set(Some(base_url.to_string())),
                enabled: Set(true),
                rate_limit_ms: Set(1000),
                last_refreshed_at: Set(None::<String>),
                last_status: Set(None::<String>),
                last_error: Set(None::<String>),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };

            active.insert(conn).await?;
        }
    }

    Ok(())
}

/// List overlay providers sorted by name.
pub async fn list_providers(conn: &DatabaseConnection) -> Result<Vec<DbOvrProvider>, DbErr> {
    let records = overlay_provider::Entity::find()
        .order_by_asc(overlay_provider::Column::Name)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_model).collect())
}

/// Look up an overlay provider by its primary key.
pub async fn find_provider_by_id(
    conn: &DatabaseConnection,
    id: i64,
) -> Result<Option<DbOvrProvider>, DbErr> {
    let record = overlay_provider::Entity::find_by_id(id as i32)
        .one(conn)
        .await?;

    Ok(record.map(map_model))
}

/// Insert or update an overlay provider.
pub async fn upsert_provider(
    conn: &DatabaseConnection,
    data: &DbOvrProvider,
) -> Result<DbOvrProvider, DbErr> {
    let existing = if let Some(id) = data.id {
        overlay_provider::Entity::find_by_id(id as i32)
            .one(conn)
            .await?
    } else {
        overlay_provider::Entity::find()
            .filter(overlay_provider::Column::Name.eq(data.name.clone()))
            .one(conn)
            .await?
    };

    let now = Utc::now().naive_utc();
    let rate_limit = clamp_rate_limit(data.rate_limit_ms);
    let last_refreshed = format_optional_datetime(data.last_refreshed_at);

    if let Some(model) = existing {
        let mut active: overlay_provider::ActiveModel = model.into();
        active.name = Set(data.name.clone());
        active.base_url = Set(data.base_url.clone());
        active.enabled = Set(data.enabled);
        active.rate_limit_ms = Set(rate_limit);
        active.last_refreshed_at = Set(last_refreshed);
        active.last_status = Set(data.last_status.clone());
        active.last_error = Set(data.last_error.clone());
        active.updated_at = Set(now);

        let saved = active.update(conn).await?;
        Ok(map_model(saved))
    } else {
        let active = overlay_provider::ActiveModel {
            name: Set(data.name.clone()),
            base_url: Set(data.base_url.clone()),
            enabled: Set(data.enabled),
            rate_limit_ms: Set(rate_limit),
            last_refreshed_at: Set(last_refreshed),
            last_status: Set(data.last_status.clone()),
            last_error: Set(data.last_error.clone()),
            created_at: Set(data.created_at.naive_utc()),
            updated_at: Set(now),
            ..Default::default()
        };

        let saved = active.insert(conn).await?;
        Ok(map_model(saved))
    }
}

/// Delete an overlay provider by its primary key.
pub async fn delete_provider(conn: &DatabaseConnection, id: i64) -> Result<(), DbErr> {
    overlay_provider::Entity::delete_by_id(id as i32)
        .exec(conn)
        .await?;
    Ok(())
}

/// Update the refresh status metadata for a provider.
pub async fn set_provider_refresh_status(
    conn: &DatabaseConnection,
    id: i64,
    status: Option<&str>,
    error: Option<&str>,
) -> Result<(), DbErr> {
    let Some(model) = overlay_provider::Entity::find_by_id(id as i32)
        .one(conn)
        .await?
    else {
        return Err(DbErr::RecordNotFound(format!(
            "Overlay provider {id} not found"
        )));
    };

    let mut active: overlay_provider::ActiveModel = model.into();
    active.last_status = Set(status.map(|s| s.to_string()));
    active.last_error = Set(error.map(|s| s.to_string()));
    active.last_refreshed_at = Set(Some(Utc::now().to_rfc3339()));
    active.updated_at = Set(Utc::now().naive_utc());

    active.update(conn).await.map(|_| ())
}
