use crate::{
    database::models::{PssEventDetail, PssEventV2, PssScore, PssWarning},
    entity::{event, event_detail, event_warning, matches, score},
};
use chrono::{DateTime as ChronoDateTime, TimeZone, Utc};
use log::warn;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect,
};

/// Insert a PSS event into the canonical `event` table using SeaORM.
pub async fn insert_event(conn: &DatabaseConnection, event_model: &PssEventV2) -> Result<i64, DbErr> {
    let session_id = to_i32(event_model.session_id, "session_id")?;
    let match_id = opt_i64_to_i32(event_model.match_id, "match_id")?;
    let round_id = opt_i64_to_i32(event_model.round_id, "round_id")?;
    let event_type_id = to_i32(event_model.event_type_id, "event_type_id")?;

    let inserted = event::ActiveModel {
        session_id: Set(session_id),
        match_id: Set(match_id),
        round_id: Set(round_id),
        event_type_id: Set(event_type_id),
        timestamp: Set(event_model.timestamp.to_rfc3339()),
        raw_data: Set(event_model.raw_data.clone()),
        parsed_data: Set(event_model.parsed_data.clone()),
        event_sequence: Set(Some(event_model.event_sequence)),
        processing_time_ms: Set(event_model.processing_time_ms),
        is_valid: Set(event_model.is_valid),
        error_message: Set(event_model.error_message.clone()),
        recognition_status: Set(event_model.recognition_status.clone()),
        protocol_version: Set(event_model.protocol_version.clone()),
        parser_confidence: Set(event_model.parser_confidence),
        validation_errors: Set(event_model.validation_errors.clone()),
        tournament_uuid: Set(event_model.tournament_id.clone()),
        created_at: Set(event_model.created_at.naive_utc()),
        ..Default::default()
    }
    .insert(conn)
    .await?;

    Ok(inserted.id as i64)
}

/// Fetch PSS events for a given session ordered by newest sequence first.
pub async fn get_events_for_session(
    conn: &DatabaseConnection,
    session_id: i64,
    limit: Option<i64>,
) -> Result<Vec<PssEventV2>, DbErr> {
    let session_id = to_i32(session_id, "session_id")?;
    let lim = sanitize_limit(limit);

    let records = event::Entity::find()
        .filter(event::Column::SessionId.eq(session_id))
        .order_by_desc(event::Column::EventSequence)
        .order_by_desc(event::Column::Id)
        .limit(lim)
        .all(conn)
        .await?;

    records.into_iter().map(map_event).collect()
}

/// Fetch PSS events for a given match ordered by newest sequence first.
pub async fn get_events_for_match(
    conn: &DatabaseConnection,
    match_id: i64,
    limit: Option<i64>,
) -> Result<Vec<PssEventV2>, DbErr> {
    let match_id = to_i32(match_id, "match_id")?;
    let lim = sanitize_limit(limit);

    let records = event::Entity::find()
        .filter(event::Column::MatchId.eq(match_id))
        .order_by_desc(event::Column::EventSequence)
        .order_by_desc(event::Column::Id)
        .limit(lim)
        .all(conn)
        .await?;

    records.into_iter().map(map_event).collect()
}

/// Persist key-value metadata for an event.
pub async fn insert_event_details(
    conn: &DatabaseConnection,
    event_id: i64,
    details: &[(String, Option<String>, String)],
) -> Result<(), DbErr> {
    if details.is_empty() {
        return Ok(());
    }

    let event_id_i32 = to_i32(event_id, "event_id")?;

    // Ensure the event exists to avoid FK violations.
    if event::Entity::find_by_id(event_id_i32)
        .one(conn)
        .await?
        .is_none()
    {
        warn!(
            "Skipping event detail insert because event {} does not exist",
            event_id
        );
        return Ok(());
    }

    for (key, value, detail_type) in details {
        let existing = event_detail::Entity::find()
            .filter(event_detail::Column::EventId.eq(event_id_i32))
            .filter(event_detail::Column::DetailKey.eq(key.clone()))
            .one(conn)
            .await?;

        if let Some(existing_model) = existing {
            let mut active: event_detail::ActiveModel = existing_model.into();
            active.detail_value = Set(value.clone());
            active.detail_type = Set(detail_type.clone());
            active.update(conn).await?;
        } else {
            event_detail::ActiveModel {
                event_id: Set(event_id_i32),
                detail_key: Set(key.clone()),
                detail_value: Set(value.clone()),
                detail_type: Set(detail_type.clone()),
                created_at: Set(Utc::now().naive_utc()),
                ..Default::default()
            }
            .insert(conn)
            .await?;
        }
    }

    Ok(())
}

/// Retrieve metadata entries for an event ordered by detail key.
pub async fn get_event_details(
    conn: &DatabaseConnection,
    event_id: i64,
) -> Result<Vec<PssEventDetail>, DbErr> {
    let event_id = to_i32(event_id, "event_id")?;

    let records = event_detail::Entity::find()
        .filter(event_detail::Column::EventId.eq(event_id))
        .order_by_asc(event_detail::Column::DetailKey)
        .all(conn)
        .await?;

    Ok(records.into_iter().map(map_event_detail).collect())
}

/// Insert a score row for a match.
pub async fn insert_score(conn: &DatabaseConnection, score_model: &PssScore) -> Result<i64, DbErr> {
    let match_id = resolve_match_identifier(conn, &score_model.match_id).await?;
    let side = position_to_side(score_model.athlete_position)?;
    let round_id = opt_i64_to_i32(score_model.round_id, "round_id")?;
    let timestamp = score_model.timestamp.to_rfc3339();

    let inserted = score::ActiveModel {
        match_id: Set(match_id),
        round_id: Set(round_id),
        side: Set(side),
        r#type: Set(score_model.score_type.clone()),
        value: Set(score_model.score_value),
        timestamp: Set(Some(timestamp)),
        tournament_uuid: Set(score_model.tournament_id.clone()),
        created_at: Set(score_model.created_at.naive_utc()),
        ..Default::default()
    }
    .insert(conn)
    .await?;

    Ok(inserted.id as i64)
}

/// Retrieve the newest current scores for a match.
pub async fn get_current_scores_for_match(
    conn: &DatabaseConnection,
    match_id: i64,
) -> Result<Vec<PssScore>, DbErr> {
    let match_id = to_i32(match_id, "match_id")?;

    let entries = score::Entity::find()
        .filter(score::Column::MatchId.eq(match_id))
        .filter(score::Column::Type.eq("current"))
        .order_by_desc(score::Column::Timestamp)
        .order_by_desc(score::Column::Id)
        .limit(2)
        .find_also_related(matches::Entity)
        .all(conn)
        .await?;

    entries
        .into_iter()
        .map(|(score_model, match_model)| map_score(score_model, match_model))
        .collect()
}

/// Insert a warning entry for a match.
pub async fn insert_warning(
    conn: &DatabaseConnection,
    warning: &PssWarning,
) -> Result<i64, DbErr> {
    let match_id = resolve_match_identifier(conn, &warning.match_id).await?;
    let side = position_to_side(warning.athlete_position)?;
    let round_id = opt_i64_to_i32(warning.round_id, "round_id")?;
    let timestamp = warning.timestamp.to_rfc3339();

    let inserted = event_warning::ActiveModel {
        match_id: Set(match_id),
        round_id: Set(round_id),
        side: Set(side),
        warning_type: Set(warning.warning_type.clone()),
        warning_count: Set(warning.warning_count),
        timestamp: Set(Some(timestamp)),
        tournament_uuid: Set(warning.tournament_id.clone()),
        created_at: Set(warning.created_at.naive_utc()),
        ..Default::default()
    }
    .insert(conn)
    .await?;

    Ok(inserted.id as i64)
}

/// Retrieve warnings for a match ordered by most recent timestamp.
pub async fn get_current_warnings_for_match(
    conn: &DatabaseConnection,
    match_id: i64,
) -> Result<Vec<PssWarning>, DbErr> {
    let match_id = to_i32(match_id, "match_id")?;

    let entries = event_warning::Entity::find()
        .filter(event_warning::Column::MatchId.eq(match_id))
        .order_by_desc(event_warning::Column::Timestamp)
        .order_by_desc(event_warning::Column::Id)
        .find_also_related(matches::Entity)
        .all(conn)
        .await?;

    entries
        .into_iter()
        .map(|(warning_model, match_model)| map_warning(warning_model, match_model))
        .collect()
}

fn map_event(model: event::Model) -> Result<PssEventV2, DbErr> {
    let timestamp = parse_rfc3339(&model.timestamp, "timestamp")?;
    let created_at = Utc.from_utc_datetime(&model.created_at);

    Ok(PssEventV2 {
        id: Some(model.id as i64),
        session_id: model.session_id as i64,
        match_id: model.match_id.map(|v| v as i64),
        round_id: model.round_id.map(|v| v as i64),
        event_type_id: model.event_type_id as i64,
        tournament_id: model.tournament_uuid.clone(),
        timestamp,
        raw_data: model.raw_data,
        parsed_data: model.parsed_data,
        event_sequence: model.event_sequence.unwrap_or_default(),
        processing_time_ms: model.processing_time_ms,
        is_valid: model.is_valid,
        error_message: model.error_message,
        recognition_status: model.recognition_status,
        protocol_version: model.protocol_version,
        parser_confidence: model.parser_confidence,
        validation_errors: model.validation_errors,
        created_at,
        created: Some(created_at.timestamp()),
    })
}

fn map_event_detail(model: event_detail::Model) -> PssEventDetail {
    PssEventDetail {
        id: Some(model.id as i64),
        event_id: model.event_id as i64,
        detail_key: model.detail_key,
        detail_value: model.detail_value,
        detail_type: model.detail_type,
        created_at: Utc.from_utc_datetime(&model.created_at),
    }
}

fn map_score(
    model: score::Model,
    match_model: Option<matches::Model>,
) -> Result<PssScore, DbErr> {
    let timestamp = model
        .timestamp
        .as_deref()
        .map(|value| parse_rfc3339(value, "score.timestamp"))
        .transpose()?
        .unwrap_or_else(|| Utc.from_utc_datetime(&model.created_at));

    let match_code = match match_model {
        Some(ref record) => record.match_code.clone(),
        None => model.match_id.to_string(),
    };

    Ok(PssScore {
        id: Some(model.id as i64),
        match_id: match_code,
        round_id: model.round_id.map(|v| v as i64),
        athlete_position: side_to_position(&model.side),
        score_type: model.r#type,
        score_value: model.value,
        timestamp,
        created_at: Utc.from_utc_datetime(&model.created_at),
        tournament_id: model.tournament_uuid,
        tournament_day_id: None,
    })
}

fn map_warning(
    model: event_warning::Model,
    match_model: Option<matches::Model>,
) -> Result<PssWarning, DbErr> {
    let timestamp = model
        .timestamp
        .as_deref()
        .map(|value| parse_rfc3339(value, "warning.timestamp"))
        .transpose()?
        .unwrap_or_else(|| Utc.from_utc_datetime(&model.created_at));

    let match_code = match match_model {
        Some(ref record) => record.match_code.clone(),
        None => model.match_id.to_string(),
    };

    Ok(PssWarning {
        id: Some(model.id as i64),
        match_id: match_code,
        round_id: model.round_id.map(|v| v as i64),
        athlete_position: side_to_position(&model.side),
        warning_type: model.warning_type,
        warning_count: model.warning_count,
        timestamp,
        created_at: Utc.from_utc_datetime(&model.created_at),
        tournament_id: model.tournament_uuid,
        tournament_day_id: None,
    })
}

async fn resolve_match_identifier(
    conn: &DatabaseConnection,
    identifier: &str,
) -> Result<i32, DbErr> {
    if let Ok(parsed) = identifier.parse::<i64>() {
        if let Ok(value) = to_i32(parsed, "match_id") {
            if matches::Entity::find_by_id(value).one(conn).await?.is_some() {
                return Ok(value);
            }
        }
    }

    if let Some(model) = matches::Entity::find()
        .filter(matches::Column::MatchCode.eq(identifier))
        .one(conn)
        .await?
    {
        return Ok(model.id);
    }

    if let Some(model) = matches::Entity::find()
        .filter(matches::Column::Uuid.eq(identifier))
        .one(conn)
        .await?
    {
        return Ok(model.id);
    }

    Err(DbErr::Custom(format!(
        "Match not found for identifier '{}'",
        identifier
    )))
}

fn to_i32(value: i64, field: &str) -> Result<i32, DbErr> {
    i32::try_from(value).map_err(|_| {
        DbErr::Custom(format!(
            "{} value {} exceeds supported range for SeaORM operations",
            field, value
        ))
    })
}

fn opt_i64_to_i32(value: Option<i64>, field: &str) -> Result<Option<i32>, DbErr> {
    value
        .map(|inner| to_i32(inner, field))
        .transpose()
}

fn parse_rfc3339(value: &str, field: &str) -> Result<ChronoDateTime<Utc>, DbErr> {
    ChronoDateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| DbErr::Custom(format!("Invalid {} '{}': {}", field, value, err)))
}

fn position_to_side(position: i32) -> Result<String, DbErr> {
    match position {
        1 => Ok("blue".to_string()),
        2 => Ok("red".to_string()),
        other => Err(DbErr::Custom(format!(
            "Unsupported athlete position {} (expected 1 or 2)",
            other
        ))),
    }
}

fn side_to_position(side: &str) -> i32 {
    match side.to_ascii_lowercase().as_str() {
        "red" => 2,
        _ => 1,
    }
}

fn sanitize_limit(limit: Option<i64>) -> u64 {
    match limit {
        Some(value) if value > 0 => value as u64,
        _ => 100,
    }
}
