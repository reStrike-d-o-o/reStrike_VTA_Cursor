use crate::{
    database::models::{
        PssEventRecognitionHistory, PssEventStatistics, PssEventValidationResult,
        PssEventValidationRule, PssEventV2, PssUnknownEvent,
    },
    entity::{
        event, event_recognition_history, event_statistic, event_unknown,
        event_validation_result, event_validation_rule,
    },
};
use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    DbBackend, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
};
use serde_json::json;

use super::pss;

/// Insert an event with recognition status tracking. This mirrors the legacy helper that wrote
/// directly into `pss_events`.
pub async fn store_event_with_status(
    conn: &DatabaseConnection,
    event_model: &PssEventV2,
) -> Result<i64, DbErr> {
    super::pss::insert_event(conn, event_model).await
}

/// Update the recognition status of an event and append a history entry describing the transition.
pub async fn update_event_recognition_status(
    conn: &DatabaseConnection,
    event_id: i64,
    new_status: &str,
    changed_by: &str,
    change_reason: Option<&str>,
) -> Result<(), DbErr> {
    let txn = conn.begin().await?;
    let event_id_i32 = pss::to_i32(event_id, "event_id")?;

    let Some(model) = event::Entity::find_by_id(event_id_i32)
        .one(&txn)
        .await?
    else {
        txn.rollback().await?;
        return Err(DbErr::Custom(format!("Event {event_id} not found")));
    };

    let current_status = model.recognition_status.clone();
    if current_status == new_status {
        txn.commit().await?;
        return Ok(());
    }

    let mut active: event::ActiveModel = model.clone().into();
    active.recognition_status = Set(new_status.to_string());
    active.update(&txn).await?;

    let history = event_recognition_history::ActiveModel {
        event_id: Set(event_id_i32),
        old_status: Set(current_status),
        new_status: Set(new_status.to_string()),
        changed_by: Set(changed_by.to_string()),
        change_reason: Set(change_reason.map(|s| s.to_string())),
        protocol_version: Set(model.protocol_version.clone()),
        raw_data: Set(model.raw_data.clone()),
        parsed_data: Set(model.parsed_data.clone()),
        created_at: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    history.insert(&txn).await?;

    txn.commit().await?;
    Ok(())
}

/// Store or update an unknown event pattern.
pub async fn store_unknown_event(
    conn: &DatabaseConnection,
    unknown_event: &PssUnknownEvent,
) -> Result<i64, DbErr> {
    let txn = conn.begin().await?;
    let session_id = pss::to_i32(unknown_event.session_id, "session_id")?;

    let existing = event_unknown::Entity::find()
        .filter(event_unknown::Column::SessionId.eq(session_id))
        .filter(event_unknown::Column::RawData.eq(unknown_event.raw_data.clone()))
        .one(&txn)
        .await?;

    let id = if let Some(model) = existing {
        let mut active: event_unknown::ActiveModel = model.clone().into();
        active.last_seen = Set(unknown_event.last_seen.to_rfc3339());
        active.occurrence_count = Set(model.occurrence_count + 1);
        active.updated_at = Set(unknown_event.updated_at.naive_utc());
        if unknown_event.suggested_event_type.is_some() {
            active.suggested_event_type = Set(unknown_event.suggested_event_type.clone());
        }
        if unknown_event.notes.is_some() {
            active.notes = Set(unknown_event.notes.clone());
        }
        active.update(&txn).await?.id as i64
    } else {
        let inserted = event_unknown::ActiveModel {
            session_id: Set(session_id),
            raw_data: Set(unknown_event.raw_data.clone()),
            first_seen: Set(unknown_event.first_seen.to_rfc3339()),
            last_seen: Set(unknown_event.last_seen.to_rfc3339()),
            occurrence_count: Set(unknown_event.occurrence_count),
            pattern_hash: Set(unknown_event.pattern_hash.clone()),
            suggested_event_type: Set(unknown_event.suggested_event_type.clone()),
            notes: Set(unknown_event.notes.clone()),
            created_at: Set(unknown_event.created_at.naive_utc()),
            updated_at: Set(unknown_event.updated_at.naive_utc()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        inserted.id as i64
    };

    txn.commit().await?;
    Ok(id)
}

/// Fetch active validation rules for a given event code and protocol version.
pub async fn get_validation_rules(
    conn: &DatabaseConnection,
    event_code: &str,
    protocol_version: &str,
) -> Result<Vec<PssEventValidationRule>, DbErr> {
    let records = event_validation_rule::Entity::find()
        .filter(event_validation_rule::Column::EventCode.eq(event_code))
        .filter(event_validation_rule::Column::ProtocolVersion.eq(protocol_version))
        .filter(event_validation_rule::Column::IsActive.eq(true))
        .order_by_asc(event_validation_rule::Column::RuleName)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_validation_rule).collect())
}

/// Persist a validation result row.
pub async fn store_validation_result(
    conn: &DatabaseConnection,
    validation_result: &PssEventValidationResult,
) -> Result<i64, DbErr> {
    let event_id = pss::to_i32(validation_result.event_id, "event_id")?;
    let rule_id = pss::to_i32(validation_result.rule_id, "rule_id")?;

    let inserted = event_validation_result::ActiveModel {
        event_id: Set(event_id),
        rule_id: Set(rule_id),
        validation_passed: Set(validation_result.validation_passed),
        error_message: Set(validation_result.error_message.clone()),
        validation_time_ms: Set(validation_result.validation_time_ms),
        created_at: Set(validation_result.created_at.naive_utc()),
        ..Default::default()
    }
    .insert(conn)
    .await?;

    Ok(inserted.id as i64)
}

/// Update the rolling statistics for a given session/event type combination.
pub async fn update_event_statistics(
    conn: &DatabaseConnection,
    session_id: i64,
    event_type_id: Option<i64>,
    recognition_status: &str,
    processing_time_ms: Option<i32>,
) -> Result<(), DbErr> {
    let txn = conn.begin().await?;
    let session_id_i32 = pss::to_i32(session_id, "session_id")?;
    let event_type_i32 = pss::opt_i64_to_i32(event_type_id, "event_type_id")?;

    let mut condition = Condition::all().add(event_statistic::Column::SessionId.eq(session_id_i32));
    condition = match event_type_i32 {
        Some(value) => condition.add(event_statistic::Column::EventTypeId.eq(value)),
        None => condition.add(event_statistic::Column::EventTypeId.is_null()),
    };

    if let Some(model) = event_statistic::Entity::find()
        .filter(condition.clone())
        .one(&txn)
        .await?
    {
        let mut active: event_statistic::ActiveModel = model.clone().into();

        let prev_total = model.total_events;
        let new_total = prev_total + 1;

        let mut recognized = model.recognized_events;
        let mut unknown = model.unknown_events;
        let mut partial = model.partial_events;
        let mut deprecated = model.deprecated_events;

        match recognition_status {
            "recognized" => recognized += 1,
            "unknown" => unknown += 1,
            "partial" => partial += 1,
            "deprecated" => deprecated += 1,
            _ => {}
        }

        active.total_events = Set(new_total);
        active.recognized_events = Set(recognized);
        active.unknown_events = Set(unknown);
        active.partial_events = Set(partial);
        active.deprecated_events = Set(deprecated);
        active.updated_at = Set(Utc::now().naive_utc());

        if let Some(processing_time) = processing_time_ms {
            let prev_avg = model.average_processing_time_ms;
            let new_avg = if prev_total > 0 {
                ((prev_avg * prev_total as f64) + processing_time as f64) / new_total as f64
            } else {
                processing_time as f64
            };

            let min = match model.min_processing_time_ms {
                Some(current) => Some(current.min(processing_time)),
                None => Some(processing_time),
            };
            let max = match model.max_processing_time_ms {
                Some(current) => Some(current.max(processing_time)),
                None => Some(processing_time),
            };

            active.average_processing_time_ms = Set(new_avg);
            active.min_processing_time_ms = Set(min);
            active.max_processing_time_ms = Set(max);
        }

        active.update(&txn).await?;
    } else {
        let mut stats = PssEventStatistics::new(session_id, event_type_id);
        stats.total_events = 1;
        match recognition_status {
            "recognized" => stats.recognized_events = 1,
            "unknown" => stats.unknown_events = 1,
            "partial" => stats.partial_events = 1,
            "deprecated" => stats.deprecated_events = 1,
            _ => {}
        }
        if let Some(processing_time) = processing_time_ms {
            stats.average_processing_time_ms = processing_time as f64;
            stats.min_processing_time_ms = Some(processing_time);
            stats.max_processing_time_ms = Some(processing_time);
        }

        event_statistic::ActiveModel {
            session_id: Set(session_id_i32),
            event_type_id: Set(event_type_i32),
            total_events: Set(stats.total_events),
            recognized_events: Set(stats.recognized_events),
            unknown_events: Set(stats.unknown_events),
            partial_events: Set(stats.partial_events),
            deprecated_events: Set(stats.deprecated_events),
            validation_errors: Set(stats.validation_errors),
            parsing_errors: Set(stats.parsing_errors),
            average_processing_time_ms: Set(stats.average_processing_time_ms),
            min_processing_time_ms: Set(stats.min_processing_time_ms),
            max_processing_time_ms: Set(stats.max_processing_time_ms),
            created_at: Set(stats.created_at.naive_utc()),
            updated_at: Set(stats.updated_at.naive_utc()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }

    txn.commit().await?;
    Ok(())
}

/// Retrieve statistics for every event type within a session.
pub async fn get_session_statistics(
    conn: &DatabaseConnection,
    session_id: i64,
) -> Result<Vec<PssEventStatistics>, DbErr> {
    let session_id = pss::to_i32(session_id, "session_id")?;
    let records = event_statistic::Entity::find()
        .filter(event_statistic::Column::SessionId.eq(session_id))
        .order_by_desc(event_statistic::Column::TotalEvents)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_event_statistic).collect())
}

/// Fetch unknown events, optionally scoped by session.
pub async fn get_unknown_events(
    conn: &DatabaseConnection,
    session_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<PssUnknownEvent>, DbErr> {
    let limit = pss::sanitize_limit(limit);
    let mut query = event_unknown::Entity::find()
        .order_by_desc(event_unknown::Column::OccurrenceCount)
        .order_by_desc(event_unknown::Column::LastSeen)
        .limit(limit);

    if let Some(session_id) = session_id {
        query = query.filter(event_unknown::Column::SessionId.eq(pss::to_i32(
            session_id,
            "session_id",
        )?));
    }

    let records = query.all(conn).await?;
    let mut result = Vec::with_capacity(records.len());
    for model in records {
        result.push(map_unknown_event(model)?);
    }
    Ok(result)
}

/// Fetch recognition history entries for a specific event.
pub async fn get_event_recognition_history(
    conn: &DatabaseConnection,
    event_id: i64,
) -> Result<Vec<PssEventRecognitionHistory>, DbErr> {
    let event_id = pss::to_i32(event_id, "event_id")?;
    let records = event_recognition_history::Entity::find()
        .filter(event_recognition_history::Column::EventId.eq(event_id))
        .order_by_desc(event_recognition_history::Column::CreatedAt)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_recognition_history).collect())
}

/// Fetch events filtered by recognition status.
pub async fn get_events_by_status(
    conn: &DatabaseConnection,
    session_id: i64,
    recognition_status: &str,
    limit: Option<i64>,
) -> Result<Vec<PssEventV2>, DbErr> {
    let limit = pss::sanitize_limit(limit);
    let session_id = pss::to_i32(session_id, "session_id")?;

    let records = event::Entity::find()
        .filter(event::Column::SessionId.eq(session_id))
        .filter(event::Column::RecognitionStatus.eq(recognition_status))
        .order_by_desc(event::Column::CreatedAt)
        .order_by_desc(event::Column::Id)
        .limit(limit)
        .all(conn)
        .await?;

    records.into_iter().map(pss::map_event).collect()
}

/// Aggregate comprehensive statistics for dashboards/analytics.
pub async fn get_comprehensive_event_statistics(
    conn: &DatabaseConnection,
    session_id: i64,
) -> Result<serde_json::Value, DbErr> {
    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        r#"
        SELECT
            COUNT(*) AS total_events,
            SUM(CASE WHEN recognition_status = 'recognized' THEN 1 ELSE 0 END) AS recognized_events,
            SUM(CASE WHEN recognition_status = 'unknown' THEN 1 ELSE 0 END) AS unknown_events,
            SUM(CASE WHEN recognition_status = 'partial' THEN 1 ELSE 0 END) AS partial_events,
            SUM(CASE WHEN recognition_status = 'deprecated' THEN 1 ELSE 0 END) AS deprecated_events,
            AVG(parser_confidence) AS avg_confidence,
            AVG(processing_time_ms) AS avg_processing_time,
            MIN(processing_time_ms) AS min_processing_time,
            MAX(processing_time_ms) AS max_processing_time
        FROM event
        WHERE session_id = ?
        "#,
        vec![session_id.into()],
    );
    let overall = conn
        .query_one(stmt)
        .await?
        .map(|row| {
            json!({
                "total_events": row.try_get::<i64>("", "total_events").unwrap_or(0),
                "recognized_events": row.try_get::<i64>("", "recognized_events").unwrap_or(0),
                "unknown_events": row.try_get::<i64>("", "unknown_events").unwrap_or(0),
                "partial_events": row.try_get::<i64>("", "partial_events").unwrap_or(0),
                "deprecated_events": row.try_get::<i64>("", "deprecated_events").unwrap_or(0),
                "avg_confidence": row.try_get::<Option<f64>>("", "avg_confidence").unwrap_or(None),
                "avg_processing_time": row
                    .try_get::<Option<f64>>("", "avg_processing_time")
                    .unwrap_or(None),
                "min_processing_time": row
                    .try_get::<Option<i32>>("", "min_processing_time")
                    .unwrap_or(None),
                "max_processing_time": row
                    .try_get::<Option<i32>>("", "max_processing_time")
                    .unwrap_or(None),
            })
        })
        .unwrap_or_else(|| {
            json!({
                "total_events": 0,
                "recognized_events": 0,
                "unknown_events": 0,
                "partial_events": 0,
                "deprecated_events": 0,
                "avg_confidence": Option::<f64>::None,
                "avg_processing_time": Option::<f64>::None,
                "min_processing_time": Option::<i32>::None,
                "max_processing_time": Option::<i32>::None,
            })
        });

    let stmt_event_type = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        r#"
        SELECT
            et.code AS event_code,
            et.name AS event_name,
            COUNT(*) AS total,
            SUM(CASE WHEN e.recognition_status = 'recognized' THEN 1 ELSE 0 END) AS recognized,
            SUM(CASE WHEN e.recognition_status = 'unknown' THEN 1 ELSE 0 END) AS unknown,
            SUM(CASE WHEN e.recognition_status = 'partial' THEN 1 ELSE 0 END) AS partial,
            AVG(e.parser_confidence) AS avg_confidence,
            AVG(e.processing_time_ms) AS avg_processing_time
        FROM event e
        JOIN event_type et ON e.event_type_id = et.id
        WHERE e.session_id = ?
        GROUP BY et.id, et.code, et.name
        ORDER BY total DESC
        "#,
        vec![session_id.into()],
    );
    let event_type_rows = conn.query_all(stmt_event_type).await?;
    let event_type_stats: Vec<serde_json::Value> = event_type_rows
        .into_iter()
        .map(|row| {
            json!({
                "event_code": row.try_get::<String>("", "event_code").unwrap_or_default(),
                "event_name": row.try_get::<String>("", "event_name").unwrap_or_default(),
                "total": row.try_get::<i64>("", "total").unwrap_or(0),
                "recognized": row.try_get::<i64>("", "recognized").unwrap_or(0),
                "unknown": row.try_get::<i64>("", "unknown").unwrap_or(0),
                "partial": row.try_get::<i64>("", "partial").unwrap_or(0),
                "avg_confidence": row.try_get::<Option<f64>>("", "avg_confidence").unwrap_or(None),
                "avg_processing_time": row
                    .try_get::<Option<f64>>("", "avg_processing_time")
                    .unwrap_or(None),
            })
        })
        .collect();

    let stmt_validation = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        r#"
        SELECT
            validation_errors AS error,
            COUNT(*) AS count
        FROM event
        WHERE session_id = ? AND validation_errors IS NOT NULL
        GROUP BY validation_errors
        ORDER BY count DESC
        LIMIT 10
        "#,
        vec![session_id.into()],
    );
    let validation_rows = conn.query_all(stmt_validation).await?;
    let validation_errors: Vec<serde_json::Value> = validation_rows
        .into_iter()
        .map(|row| {
            json!({
                "error": row.try_get::<String>("", "error").unwrap_or_default(),
                "count": row.try_get::<i64>("", "count").unwrap_or(0),
            })
        })
        .collect();

    let stmt_unknown = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        r#"
        SELECT
            COUNT(*) AS total_unknown,
            COUNT(DISTINCT pattern_hash) AS unique_patterns,
            MAX(occurrence_count) AS max_occurrences
        FROM event_unknown
        WHERE session_id = ?
        "#,
        vec![session_id.into()],
    );
    let unknown = conn
        .query_one(stmt_unknown)
        .await?
        .map(|row| {
            json!({
                "total_unknown": row.try_get::<i64>("", "total_unknown").unwrap_or(0),
                "unique_patterns": row.try_get::<i64>("", "unique_patterns").unwrap_or(0),
                "max_occurrences": row.try_get::<i64>("", "max_occurrences").unwrap_or(0),
            })
        })
        .unwrap_or_else(|| json!({"total_unknown":0,"unique_patterns":0,"max_occurrences":0}));

    Ok(json!({
        "overall": overall,
        "by_event_type": event_type_stats,
        "validation_errors": validation_errors,
        "unknown_events": unknown
    }))
}

fn map_validation_rule(model: event_validation_rule::Model) -> PssEventValidationRule {
    PssEventValidationRule {
        id: Some(model.id as i64),
        event_code: model.event_code,
        protocol_version: model.protocol_version,
        rule_name: model.rule_name,
        rule_type: model.rule_type,
        rule_definition: model.rule_definition,
        error_message: model.error_message,
        is_active: model.is_active,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn map_event_statistic(model: event_statistic::Model) -> PssEventStatistics {
    PssEventStatistics {
        id: Some(model.id as i64),
        session_id: model.session_id as i64,
        event_type_id: model.event_type_id.map(|v| v as i64),
        total_events: model.total_events,
        recognized_events: model.recognized_events,
        unknown_events: model.unknown_events,
        partial_events: model.partial_events,
        deprecated_events: model.deprecated_events,
        validation_errors: model.validation_errors,
        parsing_errors: model.parsing_errors,
        average_processing_time_ms: model.average_processing_time_ms,
        min_processing_time_ms: model.min_processing_time_ms,
        max_processing_time_ms: model.max_processing_time_ms,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    }
}

fn map_unknown_event(model: event_unknown::Model) -> Result<PssUnknownEvent, DbErr> {
    Ok(PssUnknownEvent {
        id: Some(model.id as i64),
        session_id: model.session_id as i64,
        raw_data: model.raw_data,
        first_seen: pss::parse_rfc3339(&model.first_seen, "event_unknown.first_seen")?,
        last_seen: pss::parse_rfc3339(&model.last_seen, "event_unknown.last_seen")?,
        occurrence_count: model.occurrence_count,
        pattern_hash: model.pattern_hash,
        suggested_event_type: model.suggested_event_type,
        notes: model.notes,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    })
}

fn map_recognition_history(
    model: event_recognition_history::Model,
) -> PssEventRecognitionHistory {
    PssEventRecognitionHistory {
        id: Some(model.id as i64),
        event_id: model.event_id as i64,
        old_status: model.old_status,
        new_status: model.new_status,
        changed_by: model.changed_by,
        change_reason: model.change_reason,
        protocol_version: model.protocol_version,
        raw_data: model.raw_data,
        parsed_data: model.parsed_data,
        created_at: Utc.from_utc_datetime(&model.created_at),
    }
}
