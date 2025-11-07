use crate::entity::{settings_category, settings_history, settings_key, settings_value};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, Statement, TransactionTrait, Value,
};
use uuid::Uuid;

/// Seed definition for the built-in UI settings that ship with the desktop app.
struct SettingSeed {
    key_name: &'static str,
    display_name: &'static str,
    data_type: &'static str,
    default_value: Option<&'static str>,
    validation_rules: Option<&'static str>,
}

const UI_SETTINGS: &[SettingSeed] = &[
    SettingSeed {
        key_name: "window.position.x",
        display_name: "Window X Position",
        data_type: "integer",
        default_value: Some("100"),
        validation_rules: Some(r#"{"min": 0, "max": 9999}"#),
    },
    SettingSeed {
        key_name: "window.position.y",
        display_name: "Window Y Position",
        data_type: "integer",
        default_value: Some("100"),
        validation_rules: Some(r#"{"min": 0, "max": 9999}"#),
    },
    SettingSeed {
        key_name: "window.size.width",
        display_name: "Window Width",
        data_type: "integer",
        default_value: Some("1200"),
        validation_rules: Some(r#"{"min": 350, "max": 9999}"#),
    },
    SettingSeed {
        key_name: "window.size.height",
        display_name: "Window Height",
        data_type: "integer",
        default_value: Some("800"),
        validation_rules: Some(r#"{"min": 600, "max": 9999}"#),
    },
    SettingSeed {
        key_name: "window.fullscreen",
        display_name: "Fullscreen Mode",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "window.compact",
        display_name: "Compact Mode",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "theme.current",
        display_name: "Current Theme",
        data_type: "string",
        default_value: Some("dark"),
        validation_rules: Some(r#"{"enum": ["dark", "light", "auto"]}"#),
    },
    SettingSeed {
        key_name: "theme.auto_theme",
        display_name: "Auto Theme",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "theme.high_contrast",
        display_name: "High Contrast",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "layout.sidebar_position",
        display_name: "Sidebar Position",
        data_type: "string",
        default_value: Some("left"),
        validation_rules: Some(r#"{"enum": ["left", "right"]}"#),
    },
    SettingSeed {
        key_name: "layout.sidebar_width",
        display_name: "Sidebar Width",
        data_type: "integer",
        default_value: Some("300"),
        validation_rules: Some(r#"{"min": 200, "max": 500}"#),
    },
    SettingSeed {
        key_name: "layout.status_bar_visible",
        display_name: "Status Bar Visible",
        data_type: "boolean",
        default_value: Some("true"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "layout.task_bar_visible",
        display_name: "Task Bar Visible",
        data_type: "boolean",
        default_value: Some("true"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "advanced.show_advanced_panel",
        display_name: "Show Advanced Panel",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "advanced.debug_mode",
        display_name: "Debug Mode",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "advanced.verbose_logging",
        display_name: "Verbose Logging",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "animations.enabled",
        display_name: "Animations Enabled",
        data_type: "boolean",
        default_value: Some("true"),
        validation_rules: None,
    },
    SettingSeed {
        key_name: "animations.duration_ms",
        display_name: "Animation Duration",
        data_type: "integer",
        default_value: Some("300"),
        validation_rules: Some(r#"{"min": 0, "max": 2000}"#),
    },
    SettingSeed {
        key_name: "animations.reduce_motion",
        display_name: "Reduce Motion",
        data_type: "boolean",
        default_value: Some("false"),
        validation_rules: None,
    },
];

/// Ensure the canonical UI settings definitions exist and seed default values where missing.
pub async fn initialize_ui_settings(conn: &DatabaseConnection) -> Result<(), DbErr> {
    let txn = conn.begin().await?;
    backfill_missing_ids(&txn).await?;
    let category_id = get_or_create_category(&txn, "ui", "User Interface Settings", 5).await?;

    for seed in UI_SETTINGS {
        create_setting_key_if_not_exists(
            &txn,
            &category_id,
            seed.key_name,
            seed.display_name,
            seed.data_type,
            seed.default_value,
            seed.validation_rules,
        )
        .await?;
    }

    txn.commit().await
}

/// Retrieve a single setting value by its key name.
pub async fn get_ui_setting(
    conn: &DatabaseConnection,
    key_name: &str,
) -> Result<Option<String>, DbErr> {
    backfill_missing_ids(conn).await?;
    let record = settings_value::Entity::find()
        .inner_join(settings_key::Entity)
        .filter(settings_key::Column::KeyName.eq(key_name))
        .one(conn)
        .await?;

    Ok(record.map(|model| model.value))
}

/// Store or update a setting value and write an audit history row.
pub async fn set_ui_setting(
    conn: &DatabaseConnection,
    key_name: &str,
    value: &str,
    changed_by: &str,
    change_reason: Option<&str>,
) -> Result<(), DbErr> {
    let txn = conn.begin().await?;
    backfill_missing_ids(&txn).await?;

    let key = settings_key::Entity::find()
        .filter(settings_key::Column::KeyName.eq(key_name))
        .one(&txn)
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(format!("Setting key '{key_name}' not found")))?;

    let now_ts = Utc::now().timestamp();
    let key_id = key.id.clone();

    if let Some(existing) = settings_value::Entity::find()
        .filter(settings_value::Column::KeyId.eq(key_id.clone()))
        .one(&txn)
        .await?
    {
        let old_value = existing.value.clone();
        let mut active: settings_value::ActiveModel = existing.into();
        active.value = Set(value.to_string());
        active.updated = Set(Some(now_ts));
        active.update(&txn).await?;

        insert_history(
            &txn,
            &key_id,
            Some(old_value),
            Some(value.to_string()),
            changed_by,
            change_reason,
            now_ts,
        )
        .await?;
    } else {
        let value_model = settings_value::ActiveModel {
            id: Set(Uuid::new_v4().to_string()),
            key_id: Set(key_id.clone()),
            value: Set(value.to_string()),
            created: Set(Some(now_ts)),
            updated: Set(Some(now_ts)),
        };
        value_model.insert(&txn).await?;

        insert_history(
            &txn,
            &key_id,
            None,
            Some(value.to_string()),
            changed_by,
            change_reason,
            now_ts,
        )
        .await?;
    }

    txn.commit().await
}

/// Return all persisted settings values keyed by their canonical name.
pub async fn get_all_ui_settings(
    conn: &DatabaseConnection,
) -> Result<Vec<(String, String)>, DbErr> {
    backfill_missing_ids(conn).await?;
    let rows = settings_value::Entity::find()
        .find_also_related(settings_key::Entity)
        .all(conn)
        .await?;

    let mut result = Vec::with_capacity(rows.len());
    for (value_model, maybe_key) in rows {
        if let Some(key_model) = maybe_key {
            result.push((key_model.key_name, value_model.value));
        }
    }

    Ok(result)
}

/// Idempotently ensure a setting key exists for the UI category.
pub async fn ensure_key(
    conn: &DatabaseConnection,
    key_name: &str,
    display_name: &str,
    data_type: &str,
    default_value: Option<&str>,
) -> Result<(), DbErr> {
    let category_id = get_or_create_category(conn, "ui", "User Interface Settings", 5).await?;
    create_setting_key_if_not_exists(
        conn,
        &category_id,
        key_name,
        display_name,
        data_type,
        default_value,
        None,
    )
    .await
}

async fn get_or_create_category<C>(
    conn: &C,
    name: &str,
    description: &str,
    display_order: i32,
) -> Result<String, DbErr>
where
    C: ConnectionTrait,
{
    if let Some(existing) = settings_category::Entity::find()
        .filter(settings_category::Column::Name.eq(name))
        .one(conn)
        .await?
    {
        return Ok(existing.id);
    }

    let category = settings_category::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(name.to_string()),
        description: Set(Some(description.to_string())),
        display_order: Set(display_order),
        created: Set(Some(Utc::now().timestamp())),
    };

    let inserted = category.insert(conn).await?;
    Ok(inserted.id)
}

async fn create_setting_key_if_not_exists<C>(
    conn: &C,
    category_id: &str,
    key_name: &str,
    display_name: &str,
    data_type: &str,
    default_value: Option<&str>,
    validation_rules: Option<&str>,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    if let Some(existing) = settings_key::Entity::find()
        .filter(settings_key::Column::KeyName.eq(key_name))
        .one(conn)
        .await?
    {
        ensure_default_value(conn, &existing.id, default_value).await?;
        return Ok(());
    }

    let key_model = settings_key::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        category_id: Set(category_id.to_string()),
        key_name: Set(key_name.to_string()),
        display_name: Set(display_name.to_string()),
        description: Set(Some(format!("UI setting for {display_name}"))),
        data_type: Set(data_type.to_string()),
        default_value: Set(default_value.map(|val| val.to_string())),
        validation_rules: Set(validation_rules.map(|rules| rules.to_string())),
        is_required: Set(false),
        is_sensitive: Set(false),
        created: Set(Some(Utc::now().timestamp())),
    };

    let inserted = key_model.insert(conn).await?;
    ensure_default_value(conn, &inserted.id, default_value).await
}

async fn ensure_default_value<C>(
    conn: &C,
    key_id: &str,
    default_value: Option<&str>,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    backfill_missing_ids(conn).await?;
    let Some(default) = default_value else {
        return Ok(());
    };

    let existing = settings_value::Entity::find()
        .filter(settings_value::Column::KeyId.eq(key_id))
        .one(conn)
        .await?;

    if existing.is_some() {
        return Ok(());
    }

    let now_ts = Utc::now().timestamp();
    let value_model = settings_value::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        key_id: Set(key_id.to_string()),
        value: Set(default.to_string()),
        created: Set(Some(now_ts)),
        updated: Set(Some(now_ts)),
    };

    value_model.insert(conn).await.map(|_| ())
}

async fn backfill_missing_ids<C>(conn: &C) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    heal_table(conn, "settings_values").await?;
    heal_table(conn, "settings_history").await
}

async fn heal_table<C>(conn: &C, table: &str) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let backend = conn.get_database_backend();
    let select_sql = format!("SELECT rowid FROM {table} WHERE id IS NULL OR id = ''");
    let rows = conn
        .query_all(Statement::from_string(backend, select_sql))
        .await?;

    if rows.is_empty() {
        return Ok(());
    }

    for row in rows {
        let row_id: i64 = row.try_get("", "rowid")?;
        let update_sql = format!("UPDATE {table} SET id = ? WHERE rowid = ?");
        conn.execute(Statement::from_sql_and_values(
            backend,
            update_sql,
            vec![Value::from(Uuid::new_v4().to_string()), Value::from(row_id)],
        ))
        .await?;
    }

    Ok(())
}

async fn insert_history<C>(
    conn: &C,
    key_id: &str,
    old_value: Option<String>,
    new_value: Option<String>,
    changed_by: &str,
    change_reason: Option<&str>,
    timestamp: i64,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let history = settings_history::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        key_id: Set(key_id.to_string()),
        old_value: Set(old_value),
        new_value: Set(new_value),
        changed_by: Set(changed_by.to_string()),
        change_reason: Set(change_reason.map(|reason| reason.to_string())),
        created: Set(Some(timestamp)),
    };

    history.insert(conn).await.map(|_| ())
}
