use crate::database::{
    models::{
        Athlete, EventTrigger, MedalCeremony, MedalCeremonyDetail, MedalCeremonyDivision,
        MedalCeremonyDivisionDetail, MedalCeremonyMedalist, NetworkInterface, ObsRecordingConfig,
        ObsRecordingSession, Octagon, OverlayTemplate, OvrAnthemAsset, OvrCategory,
        OvrFlagAnimationAsset, OvrProvider, OvrTournament, PssAthlete, PssEventDetail,
        PssEventRecognitionHistory, PssEventStatistics, PssEventType, PssEventV2,
        PssEventValidationResult, PssEventValidationRule, PssMatch, PssMatchAthlete, PssScore,
        PssUnknownEvent, PssWarning, Tournament, TournamentDay, TournamentRanking,
        UdpClientConnection, UdpServerConfig, UdpServerSession,
    },
    DatabaseConnection, DatabaseError, DatabaseResult,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// PSS and UDP Subsystem Operations
pub struct PssUdpOperations;

impl PssUdpOperations {
    // Network Interface Operations

    /// Get all network interfaces
    pub fn get_network_interfaces(conn: &Connection) -> DatabaseResult<Vec<NetworkInterface>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM network_interfaces ORDER BY is_recommended DESC, is_active DESC, name",
        )?;

        let interfaces = stmt
            .query_map([], NetworkInterface::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(interfaces)
    }

    /// Get recommended network interface
    pub fn get_recommended_interface(
        conn: &Connection,
    ) -> DatabaseResult<Option<NetworkInterface>> {
        let interface = conn.query_row(
            "SELECT * FROM network_interfaces WHERE is_recommended = 1 AND is_active = 1 LIMIT 1",
            [],
            NetworkInterface::from_row
        ).optional()?;

        Ok(interface)
    }

    /// Add or update network interface
    pub fn upsert_network_interface(
        conn: &mut Connection,
        interface: &NetworkInterface,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        let interface_id = if let Some(id) = interface.id {
            // Update existing interface
            tx.execute(
                "UPDATE network_interfaces SET
                    name = ?, address = ?, netmask = ?, broadcast = ?, is_loopback = ?,
                    is_active = ?, is_recommended = ?, speed_mbps = ?, mtu = ?,
                    mac_address = ?, interface_type = ?, updated_at = ?
                WHERE id = ?",
                params![
                    interface.name,
                    interface.address,
                    interface.netmask,
                    interface.broadcast,
                    interface.is_loopback,
                    interface.is_active,
                    interface.is_recommended,
                    interface.speed_mbps,
                    interface.mtu,
                    interface.mac_address,
                    interface.interface_type,
                    Utc::now().to_rfc3339(),
                    id
                ],
            )?;
            id
        } else {
            // Insert new interface
            tx.execute(
                "INSERT INTO network_interfaces (
                    name, address, netmask, broadcast, is_loopback, is_active, is_recommended,
                    speed_mbps, mtu, mac_address, interface_type, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    interface.name,
                    interface.address,
                    interface.netmask,
                    interface.broadcast,
                    interface.is_loopback,
                    interface.is_active,
                    interface.is_recommended,
                    interface.speed_mbps,
                    interface.mtu,
                    interface.mac_address,
                    interface.interface_type,
                    interface.created_at.to_rfc3339(),
                    interface.updated_at.to_rfc3339()
                ],
            )?;
            tx.last_insert_rowid()
        };

        tx.commit()?;
        Ok(interface_id)
    }

    // UDP Server Configuration Operations

    /// Get all UDP server configurations
    pub fn get_udp_server_configs(conn: &Connection) -> DatabaseResult<Vec<UdpServerConfig>> {
        let mut stmt = conn.prepare("SELECT * FROM udp_server_configs ORDER BY name")?;

        let configs = stmt
            .query_map([], UdpServerConfig::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(configs)
    }

    /// Get UDP server configuration by ID
    pub fn get_udp_server_config(
        conn: &Connection,
        config_id: i64,
    ) -> DatabaseResult<Option<UdpServerConfig>> {
        let config = conn
            .query_row(
                "SELECT * FROM udp_server_configs WHERE id = ?",
                params![config_id],
                UdpServerConfig::from_row,
            )
            .optional()?;

        Ok(config)
    }

    /// Add or update UDP server configuration
    pub fn upsert_udp_server_config(
        conn: &mut Connection,
        config: &UdpServerConfig,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        let config_id = if let Some(id) = config.id {
            // Update existing config
            tx.execute(
                "UPDATE udp_server_configs SET
                    name = ?, port = ?, bind_address = ?, network_interface_id = ?,
                    enabled = ?, auto_start = ?, max_packet_size = ?, buffer_size = ?,
                    timeout_ms = ?, updated_at = ?
                WHERE id = ?",
                params![
                    config.name,
                    config.port,
                    config.bind_address,
                    config.network_interface_id,
                    config.enabled,
                    config.auto_start,
                    config.max_packet_size,
                    config.buffer_size,
                    config.timeout_ms,
                    Utc::now().to_rfc3339(),
                    id
                ],
            )?;
            id
        } else {
            // Check if config with same name exists
            let existing_id: Option<i64> = tx
                .query_row(
                    "SELECT id FROM udp_server_configs WHERE name = ?",
                    params![config.name],
                    |row| row.get(0),
                )
                .optional()?;

            if let Some(existing_id) = existing_id {
                // Update existing config
                tx.execute(
                    "UPDATE udp_server_configs SET
                        port = ?, bind_address = ?, network_interface_id = ?,
                        enabled = ?, auto_start = ?, max_packet_size = ?, buffer_size = ?,
                        timeout_ms = ?, updated_at = ?
                    WHERE id = ?",
                    params![
                        config.port,
                        config.bind_address,
                        config.network_interface_id,
                        config.enabled,
                        config.auto_start,
                        config.max_packet_size,
                        config.buffer_size,
                        config.timeout_ms,
                        Utc::now().to_rfc3339(),
                        existing_id
                    ],
                )?;
                existing_id
            } else {
                // Insert new config
                tx.execute(
                    "INSERT INTO udp_server_configs (
                        name, port, bind_address, network_interface_id, enabled, auto_start,
                        max_packet_size, buffer_size, timeout_ms, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        config.name,
                        config.port,
                        config.bind_address,
                        config.network_interface_id,
                        config.enabled,
                        config.auto_start,
                        config.max_packet_size,
                        config.buffer_size,
                        config.timeout_ms,
                        config.created_at.to_rfc3339(),
                        config.updated_at.to_rfc3339()
                    ],
                )?;
                tx.last_insert_rowid()
            }
        };
        tx.commit()?;
        Ok(config_id)
    }

    // UDP Server Session Operations

    /// Create new UDP server session
    pub fn create_udp_server_session(
        conn: &mut Connection,
        server_config_id: i64,
    ) -> DatabaseResult<i64> {
        let session = UdpServerSession::new(server_config_id);

        let session_id = conn.execute(
            "INSERT INTO udp_server_sessions (
                server_config_id, start_time, status, packets_received, packets_parsed,
                parse_errors, total_bytes_received, average_packet_size, max_packet_size_seen,
                min_packet_size_seen, unique_clients_count
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session.server_config_id,
                session.start_time.to_rfc3339(),
                session.status,
                session.packets_received,
                session.packets_parsed,
                session.parse_errors,
                session.total_bytes_received,
                session.average_packet_size,
                session.max_packet_size_seen,
                session.min_packet_size_seen,
                session.unique_clients_count
            ],
        )?;

        Ok(session_id as i64)
    }

    /// Update UDP server session statistics
    #[allow(clippy::too_many_arguments)]
    pub fn update_udp_server_session_stats(
        conn: &mut Connection,
        session_id: i64,
        packets_received: i32,
        packets_parsed: i32,
        parse_errors: i32,
        total_bytes_received: i32,
        average_packet_size: f64,
        max_packet_size_seen: i32,
        min_packet_size_seen: i32,
        unique_clients_count: i32,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE udp_server_sessions SET
                packets_received = ?, packets_parsed = ?, parse_errors = ?,
                total_bytes_received = ?, average_packet_size = ?, max_packet_size_seen = ?,
                min_packet_size_seen = ?, unique_clients_count = ?
            WHERE id = ?",
            params![
                packets_received,
                packets_parsed,
                parse_errors,
                total_bytes_received,
                average_packet_size,
                max_packet_size_seen,
                min_packet_size_seen,
                unique_clients_count,
                session_id
            ],
        )?;

        Ok(())
    }

    /// End UDP server session
    pub fn end_udp_server_session(
        conn: &mut Connection,
        session_id: i64,
        status: &str,
        error_message: Option<&str>,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE udp_server_sessions SET
                end_time = ?, status = ?, error_message = ?
            WHERE id = ?",
            params![Utc::now().to_rfc3339(), status, error_message, session_id],
        )?;

        Ok(())
    }

    /// Get UDP server session by ID
    pub fn get_udp_server_session(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<Option<UdpServerSession>> {
        let session = conn
            .query_row(
                "SELECT * FROM udp_server_sessions WHERE id = ?",
                params![session_id],
                UdpServerSession::from_row,
            )
            .optional()?;

        Ok(session)
    }

    /// Get recent UDP server sessions
    pub fn get_recent_udp_server_sessions(
        conn: &Connection,
        limit: i64,
    ) -> DatabaseResult<Vec<UdpServerSession>> {
        let mut stmt =
            conn.prepare("SELECT * FROM udp_server_sessions ORDER BY start_time DESC LIMIT ?")?;

        let sessions = stmt
            .query_map(params![limit], UdpServerSession::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    // UDP Client Connection Operations

    /// Add or update UDP client connection
    pub fn upsert_udp_client_connection(
        conn: &mut Connection,
        client: &UdpClientConnection,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        let client_id = if let Some(id) = client.id {
            // Update existing client connection
            tx.execute(
                "UPDATE udp_client_connections SET
                    last_seen = ?, packets_received = ?, total_bytes_received = ?, is_active = ?
                WHERE id = ?",
                params![
                    client.last_seen.to_rfc3339(),
                    client.packets_received,
                    client.total_bytes_received,
                    client.is_active,
                    id
                ],
            )?;
            id
        } else {
            // Insert new client connection
            tx.execute(
                "INSERT INTO udp_client_connections (
                    session_id, client_address, client_port, first_seen, last_seen,
                    packets_received, total_bytes_received, is_active
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    client.session_id,
                    client.client_address,
                    client.client_port,
                    client.first_seen.to_rfc3339(),
                    client.last_seen.to_rfc3339(),
                    client.packets_received,
                    client.total_bytes_received,
                    client.is_active
                ],
            )?;
            tx.last_insert_rowid()
        };

        tx.commit()?;
        Ok(client_id)
    }

    /// Get active client connections for a session
    pub fn get_active_client_connections(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<Vec<UdpClientConnection>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM udp_client_connections WHERE session_id = ? AND is_active = 1 ORDER BY last_seen DESC"
        )?;

        let clients = stmt
            .query_map(params![session_id], |row| {
                UdpClientConnection::from_row(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(clients)
    }

    // PSS Event Type Operations

    /// Get all PSS event types
    pub fn get_pss_event_types(conn: &Connection) -> DatabaseResult<Vec<PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_event_types WHERE is_active = 1 ORDER BY category, event_code",
        )?;

        let event_types = stmt
            .query_map([], PssEventType::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(event_types)
    }

    /// Get PSS event type by code
    pub fn get_pss_event_type_by_code(
        conn: &Connection,
        event_code: &str,
    ) -> DatabaseResult<Option<PssEventType>> {
        let event_type = conn
            .query_row(
                "SELECT * FROM pss_event_types WHERE event_code = ? AND is_active = 1",
                params![event_code],
                PssEventType::from_row,
            )
            .optional()?;

        Ok(event_type)
    }

    // PSS Match Operations

    /// Get or create PSS match
    pub fn get_or_create_pss_match(conn: &mut Connection, match_id: &str) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        // Try to get existing match
        let existing_match_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM pss_matches WHERE match_id = ?",
                params![match_id],
                |row| row.get(0),
            )
            .optional()?;

        let match_id = if let Some(id) = existing_match_id {
            id
        } else {
            // Create new match
            let match_obj = PssMatch::new(match_id.to_string());
            tx.execute(
                "INSERT INTO pss_matches (
                    uuid, match_id, total_rounds, created_at, updated_at
                ) VALUES (
                    lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',abs(random())%4+1,1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6))),
                    ?, ?, ?, ?
                )",
                params![
                    match_obj.match_id,
                    match_obj.total_rounds,
                    match_obj.created_at.to_rfc3339(),
                    match_obj.updated_at.to_rfc3339()
                ]
            )?;
            tx.last_insert_rowid()
        };

        tx.commit()?;
        Ok(match_id)
    }

    /// Update PSS match information
    pub fn update_pss_match(
        conn: &mut Connection,
        match_id: i64,
        match_data: &PssMatch,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE pss_matches SET
                match_number = ?, category = ?, weight_class = ?, division = ?,
                total_rounds = ?, round_duration = ?, countdown_type = ?, format_type = ?, updated_at = ?
            WHERE id = ?",
            params![
                match_data.match_number,
                match_data.category,
                match_data.weight_class,
                match_data.division,
                match_data.total_rounds,
                match_data.round_duration,
                match_data.countdown_type,
                match_data.format_type,
                Utc::now().to_rfc3339(),
                match_id
            ]
        )?;

        Ok(())
    }

    /// Get PSS match by database id
    pub fn get_pss_match_by_id(conn: &Connection, id: i64) -> DatabaseResult<Option<PssMatch>> {
        let mut stmt = conn.prepare("SELECT * FROM pss_matches WHERE id = ?")?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(PssMatch::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get PSS match by external match_id string
    pub fn get_pss_match_by_match_id(
        conn: &Connection,
        match_id: &str,
    ) -> DatabaseResult<Option<PssMatch>> {
        let mut stmt = conn.prepare("SELECT * FROM pss_matches WHERE match_id = ?")?;
        let mut rows = stmt.query([match_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(PssMatch::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    /// Rename the match_id string for an existing PSS match
    pub fn rename_pss_match_id(
        conn: &mut Connection,
        id: i64,
        new_match_id: &str,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE pss_matches SET match_id = ?, updated_at = ? WHERE id = ?",
            params![new_match_id, Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// Reassign all events from one match to another
    pub fn reassign_events_between_matches(
        conn: &mut Connection,
        from_match_id: i64,
        to_match_id: i64,
    ) -> DatabaseResult<usize> {
        let updated = conn.execute(
            "UPDATE pss_events SET match_id = ? WHERE match_id = ?",
            params![to_match_id, from_match_id],
        )?;
        Ok(updated)
    }

    // PSS Athlete Operations

    /// Get or create PSS athlete
    pub fn get_or_create_pss_athlete(
        conn: &mut Connection,
        athlete_code: &str,
        short_name: &str,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        // Try to get existing athlete
        let existing_athlete_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM pss_athletes WHERE athlete_code = ?",
                params![athlete_code],
                |row| row.get(0),
            )
            .optional()?;

        let athlete_id = if let Some(id) = existing_athlete_id {
            id
        } else {
            // Create new athlete
            let athlete = PssAthlete::new(athlete_code.to_string(), short_name.to_string());
            tx.execute(
                "INSERT INTO pss_athletes (
                    athlete_code, short_name, created_at, updated_at
                ) VALUES (?, ?, ?, ?)",
                params![
                    athlete.athlete_code,
                    athlete.short_name,
                    athlete.created_at.to_rfc3339(),
                    athlete.updated_at.to_rfc3339()
                ],
            )?;
            tx.last_insert_rowid()
        };

        tx.commit()?;
        Ok(athlete_id)
    }

    /// Update PSS athlete information
    pub fn update_pss_athlete(
        conn: &mut Connection,
        athlete_id: i64,
        athlete_data: &PssAthlete,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE pss_athletes SET
                long_name = ?, country_code = ?, flag_id = ?, updated_at = ?
            WHERE id = ?",
            params![
                athlete_data.long_name,
                athlete_data.country_code,
                athlete_data.flag_id,
                Utc::now().to_rfc3339(),
                athlete_id
            ],
        )?;

        Ok(())
    }

    // PSS Event Operations

    /// Store PSS event
    pub fn store_pss_event(conn: &mut Connection, event: &PssEventV2) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO pss_events (
                session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )",
            params![
                event.session_id,
                event.match_id,
                event.round_id,
                event.event_type_id,
                event.timestamp.to_rfc3339(),
                event.raw_data,
                event.parsed_data,
                event.event_sequence,
                event.processing_time_ms,
                event.is_valid,
                event.error_message,
                event.recognition_status,
                event.protocol_version,
                event.parser_confidence,
                event.validation_errors,
                event.tournament_id,
                event.created_at.to_rfc3339()
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get PSS events for a session
    pub fn get_pss_events_for_session(
        conn: &Connection,
        session_id: i64,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events WHERE session_id = ? ORDER BY event_sequence DESC LIMIT ?",
        )?;

        let events = stmt
            .query_map(params![session_id, limit], PssEventV2::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    /// Get PSS events for a match
    pub fn get_pss_events_for_match(
        conn: &Connection,
        match_id: i64,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_events WHERE match_id = ? ORDER BY timestamp DESC LIMIT ?",
        )?;

        let events = stmt
            .query_map(params![match_id, limit], PssEventV2::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(events)
    }

    // PSS Event Detail Operations

    /// Store PSS event details
    pub fn store_pss_event_details(
        conn: &mut Connection,
        event_id: i64,
        details: &[(String, Option<String>, String)],
    ) -> DatabaseResult<()> {
        // Defensive: ensure referenced event exists to avoid FK errors in edge cases
        let exists: i32 = conn
            .query_row(
                "SELECT COUNT(1) FROM pss_events WHERE id = ?",
                params![event_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            // Event row not found; skip details to maintain integrity
            log::warn!("pss_event_details skipped: event_id {event_id} not found");
            return Ok(());
        }

        let tx = conn.transaction()?;

        for (key, value, detail_type) in details {
            let detail =
                PssEventDetail::new(event_id, key.clone(), value.clone(), detail_type.clone());

            // Use INSERT OR REPLACE to handle duplicate key violations gracefully
            tx.execute(
                "INSERT OR REPLACE INTO pss_event_details (event_id, detail_key, detail_value, detail_type, created_at) VALUES (?, ?, ?, ?, ?)",
                params![
                    detail.event_id,
                    detail.detail_key,
                    detail.detail_value,
                    detail.detail_type,
                    detail.created_at.to_rfc3339()
                ]
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get PSS event details
    pub fn get_pss_event_details(
        conn: &Connection,
        event_id: i64,
    ) -> DatabaseResult<Vec<PssEventDetail>> {
        let mut stmt =
            conn.prepare("SELECT * FROM pss_event_details WHERE event_id = ? ORDER BY detail_key")?;

        let details = stmt
            .query_map(params![event_id], PssEventDetail::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(details)
    }

    // PSS Score Operations

    /// Store PSS score
    pub fn store_pss_score(conn: &mut Connection, score: &PssScore) -> DatabaseResult<i64> {
        let score_id = conn.execute(
            "INSERT INTO pss_scores (
                match_id, round_id, athlete_position, score_type, score_value, timestamp, created_at, tournament_id
            ) VALUES (
                (SELECT uuid FROM pss_matches WHERE id = ?), ?, ?, ?, ?, ?, ?,
                (SELECT tournament_id FROM pss_matches WHERE id = ?)
            )",
            params![
                score.match_id,
                score.round_id,
                score.athlete_position,
                score.score_type,
                score.score_value,
                score.timestamp.to_rfc3339(),
                score.created_at.to_rfc3339(),
                score.match_id,
                score.match_id
            ]
        )?;

        Ok(score_id as i64)
    }

    /// Get current scores for a match
    pub fn get_current_scores_for_match(
        conn: &Connection,
        match_id: i64,
    ) -> DatabaseResult<Vec<PssScore>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_scores WHERE match_id = (SELECT uuid FROM pss_matches WHERE id = ?) AND score_type = 'current' ORDER BY timestamp DESC LIMIT 2"
        )?;

        let scores = stmt
            .query_map(params![match_id], PssScore::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(scores)
    }

    // PSS Warning Operations

    /// Store PSS warning
    pub fn store_pss_warning(conn: &mut Connection, warning: &PssWarning) -> DatabaseResult<i64> {
        let warning_id = conn.execute(
            "INSERT INTO pss_warnings (
                match_id, round_id, athlete_position, warning_type, warning_count, timestamp, created_at, tournament_id
            ) VALUES (
                (SELECT uuid FROM pss_matches WHERE id = ?), ?, ?, ?, ?, ?, ?,
                (SELECT tournament_id FROM pss_matches WHERE id = ?)
            )",
            params![
                warning.match_id,
                warning.round_id,
                warning.athlete_position,
                warning.warning_type,
                warning.warning_count,
                warning.timestamp.to_rfc3339(),
                warning.created_at.to_rfc3339(),
                warning.match_id,
                warning.match_id
            ]
        )?;

        Ok(warning_id as i64)
    }

    /// Get current warnings for a match
    pub fn get_current_warnings_for_match(
        conn: &Connection,
        match_id: i64,
    ) -> DatabaseResult<Vec<PssWarning>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_warnings WHERE match_id = (SELECT uuid FROM pss_matches WHERE id = ?) ORDER BY timestamp DESC"
        )?;

        let warnings = stmt
            .query_map(params![match_id], PssWarning::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(warnings)
    }

    // Statistics and Analytics

    /// Get UDP server statistics
    pub fn get_udp_server_statistics(conn: &Connection) -> DatabaseResult<serde_json::Value> {
        // Get total sessions
        let total_sessions: i64 =
            conn.query_row("SELECT COUNT(*) FROM udp_server_sessions", [], |row| {
                row.get(0)
            })?;

        // Get active sessions
        let active_sessions: i64 = conn.query_row(
            "SELECT COUNT(*) FROM udp_server_sessions WHERE status = 'running'",
            [],
            |row| row.get(0),
        )?;

        // Get total events
        let total_events: i64 =
            conn.query_row("SELECT COUNT(*) FROM pss_events", [], |row| row.get(0))?;

        // Get total matches
        let total_matches: i64 =
            conn.query_row("SELECT COUNT(*) FROM pss_matches", [], |row| row.get(0))?;

        // Get recent activity (last 24 hours)
        let recent_events: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pss_events WHERE created_at > datetime('now', '-1 day')",
            [],
            |row| row.get(0),
        )?;

        Ok(serde_json::json!({
            "total_sessions": total_sessions,
            "active_sessions": active_sessions,
            "total_events": total_events,
            "total_matches": total_matches,
            "recent_events_24h": recent_events
        }))
    }

    pub fn get_pss_matches(conn: &Connection, limit: Option<i64>) -> DatabaseResult<Vec<PssMatch>> {
        let limit_clause = limit.map(|l| format!(" LIMIT {l}")).unwrap_or_default();
        let query = format!("SELECT * FROM pss_matches ORDER BY created_at DESC{limit_clause}");

        let mut stmt = conn.prepare(&query)?;
        let matches = stmt
            .query_map([], PssMatch::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(matches)
    }

    pub fn get_pss_matches_by_creation_mode(
        conn: &Connection,
        creation_mode: &str,
    ) -> DatabaseResult<Vec<PssMatch>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM pss_matches WHERE creation_mode = ? ORDER BY created_at DESC",
        )?;

        let matches = stmt
            .query_map([creation_mode], PssMatch::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(matches)
    }

    pub fn insert_pss_match(conn: &Connection, pss_match: &PssMatch) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO pss_matches (match_id, match_number, category, weight_class, division, total_rounds, round_duration, countdown_type, format_type, creation_mode, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                pss_match.match_id,
                pss_match.match_number,
                pss_match.category,
                pss_match.weight_class,
                pss_match.division,
                pss_match.total_rounds,
                pss_match.round_duration,
                pss_match.countdown_type,
                pss_match.format_type,
                pss_match.creation_mode,
                pss_match.created_at.to_rfc3339(),
                pss_match.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Set tournament context on an existing PSS match (id refers to DB id)
    pub fn set_pss_match_tournament_context(
        conn: &Connection,
        match_db_id: i64,
        tournament_id: Option<i64>,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE pss_matches SET
                tournament_id = COALESCE((SELECT uuid FROM tournaments WHERE id = ?), tournament_id),
                updated_at = ?
             WHERE id = ?",
            params![
                tournament_id,
                Utc::now().to_rfc3339(),
                match_db_id
            ],
        )?;
        Ok(())
    }

    pub fn insert_pss_athlete(conn: &Connection, athlete: &PssAthlete) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO pss_athletes (athlete_code, short_name, long_name, country_code, flag_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                athlete.athlete_code,
                athlete.short_name,
                athlete.long_name,
                athlete.country_code,
                athlete.flag_id,
                athlete.created_at.to_rfc3339(),
                athlete.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_pss_match_athlete(
        conn: &Connection,
        match_athlete: &PssMatchAthlete,
    ) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO pss_match_athletes (match_id, athlete_id, athlete_position, bg_color, fg_color, created_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                match_athlete.match_id,
                match_athlete.athlete_id,
                match_athlete.athlete_position,
                match_athlete.bg_color,
                match_athlete.fg_color,
                match_athlete.created_at.to_rfc3339(),
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    // (removed) Backfill pss_matches from recorded_videos; prefer purge strategy
    pub fn backfill_matches_tournament_from_recorded_videos(
        conn: &mut Connection,
    ) -> DatabaseResult<usize> {
        let sql = r#"
            UPDATE pss_matches AS m
            SET
                tournament_id = COALESCE(
                    m.tournament_id,
                    (
                        SELECT rv.tournament_id FROM recorded_videos rv
                        WHERE rv.match_id = m.id AND rv.tournament_id IS NOT NULL
                        ORDER BY rv.created_at DESC
                        LIMIT 1
                    )
                ),
                updated_at = ?
            WHERE (m.tournament_id IS NULL)
              AND EXISTS (
                SELECT 1 FROM recorded_videos rv
                WHERE rv.match_id = m.id AND (rv.tournament_id IS NOT NULL)
              )
        "#;
        let n = conn.execute(sql, [Utc::now().to_rfc3339()])?;
        Ok(n)
    }

    // (removed) Backfill pss_events from matches; prefer purge strategy
    pub fn backfill_events_tournament_from_matches(conn: &mut Connection) -> DatabaseResult<usize> {
        let sql = r#"
            UPDATE pss_events AS e
            SET
                tournament_id = COALESCE(
                    e.tournament_id,
                    (
                        SELECT m.tournament_id FROM pss_matches m
                        WHERE m.id = e.match_id
                    )
                )
            WHERE (e.tournament_id IS NULL)
              AND e.match_id IS NOT NULL
        "#;
        let n = conn.execute(sql, [])?;
        Ok(n)
    }

    // (removed) Optional fallback by date overlap; prefer purge strategy
    pub fn backfill_events_tournament_by_day_overlap(
        _conn: &mut Connection,
    ) -> DatabaseResult<usize> {
        // Set tournament_day_id by matching event timestamp to day date when tournament_id known but day missing
        Ok(0)
    }

    // (removed) Rebuild recorded_video_events; prefer purge strategy
    pub fn backfill_recorded_video_events(conn: &mut Connection) -> DatabaseResult<usize> {
        // For each recorded video, link matching events within its window using tournament constraints
        let sql = r#"
            INSERT OR IGNORE INTO recorded_video_events (recorded_video_id, event_id, offset_ms, created_at)
            SELECT rv.id,
                   e.id,
                   CAST((julianday(e.timestamp) - julianday(rv.start_time)) * 86400000 AS INTEGER) AS offset_ms,
                   ?
            FROM recorded_videos rv
            JOIN pss_events e ON e.match_id = rv.match_id
            JOIN pss_event_types t ON t.id = e.event_type_id
            WHERE e.timestamp >= rv.start_time
              AND e.timestamp <= datetime(rv.start_time, printf('+%d seconds', COALESCE(rv.duration_seconds, 0)))
              AND (rv.tournament_id IS NULL OR e.tournament_id = rv.tournament_id)
              AND t.event_code IN ('K','P','H','TH','TB','R')
        "#;
        let n = conn.execute(sql, [Utc::now().to_rfc3339()])?;
        Ok(n)
    }

    /// Purge all tournament-related and PSS historical data for a clean start
    pub fn purge_all_tournament_pss_data(conn: &mut Connection) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        // Order matters due to FKs
        tx.execute("DELETE FROM recorded_video_events", [])?;
        tx.execute("DELETE FROM recorded_videos", [])?;
        tx.execute("DELETE FROM pss_event_details", [])?;
        tx.execute("DELETE FROM pss_events", [])?;
        tx.execute("DELETE FROM pss_scores", [])?;
        tx.execute("DELETE FROM pss_warnings", [])?;
        tx.execute("DELETE FROM pss_match_athletes", [])?;
        tx.execute("DELETE FROM pss_rounds", [])?;
        tx.execute("DELETE FROM pss_athletes", [])?;
        tx.execute("DELETE FROM pss_matches", [])?;
        tx.execute("DELETE FROM tournaments", [])?;
        // Optional archives if present
        let _ = tx.execute("DELETE FROM pss_events_archive", []);
        let _ = tx.execute("DELETE FROM pss_event_details_archive", []);
        tx.commit()?;
        Ok(())
    }

    /// Get athletes for a specific match with their details
    pub fn get_pss_match_athletes(
        conn: &Connection,
        match_id: i64,
    ) -> DatabaseResult<Vec<(PssMatchAthlete, PssAthlete)>> {
        let mut stmt = conn.prepare(
            "SELECT ma.id, ma.match_id, ma.athlete_id, ma.athlete_position, ma.bg_color, ma.fg_color, ma.created_at,
                    a.id, a.athlete_code, a.short_name, a.long_name, a.country_code, a.flag_id, a.created_at, a.updated_at
             FROM pss_match_athletes ma
             JOIN pss_athletes a ON ma.athlete_id = a.id
             WHERE ma.match_id = ?
             ORDER BY ma.athlete_position"
        )?;

        let match_athletes = stmt
            .query_map([match_id], |row| {
                let match_athlete = PssMatchAthlete::from_row(row)?;
                let athlete = PssAthlete::from_row(row)?;
                Ok((match_athlete, athlete))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(match_athletes)
    }

    pub fn get_all_settings(conn: &Connection) -> DatabaseResult<serde_json::Value> {
        // Get all settings from the normalized settings system
        let mut stmt = conn.prepare(
            "SELECT c.name as category, k.key_name, k.display_name, v.value, k.data_type
             FROM settings_categories c
             JOIN settings_keys k ON c.id = k.category_id
             LEFT JOIN settings_values v ON k.id = v.key_id
             ORDER BY c.display_order, k.key_name",
        )?;

        let settings = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "category": row.get::<_, String>(0)?,
                    "key": row.get::<_, String>(1)?,
                    "display_name": row.get::<_, String>(2)?,
                    "value": row.get::<_, Option<String>>(3)?,
                    "data_type": row.get::<_, String>(4)?
                }))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(serde_json::json!({
            "settings": settings
        }))
    }
}

/// Tournament Operations for managing tournaments and tournament days
pub struct TournamentOperations;

impl TournamentOperations {
    /// Check if tournament with same name and start_date exists (if start_date provided)
    pub fn tournament_duplicate_exists(
        conn: &Connection,
        name: &str,
        start_date: &Option<String>,
    ) -> DatabaseResult<bool> {
        if let Some(sd) = start_date {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM tournaments WHERE name = ? AND start_date = ?",
                params![name, sd],
                |row| row.get(0),
            )?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// Create a new tournament
    pub fn create_tournament(
        conn: &mut Connection,
        tournament: &Tournament,
    ) -> DatabaseResult<i64> {
        // Allow duplicate names on different days; only block exact (name, start_date)
        let start_date_str = tournament.start_date.map(|d| d.to_rfc3339());
        if Self::tournament_duplicate_exists(conn, &tournament.name, &start_date_str)? {
            return Err(crate::database::DatabaseError::Sqlite(
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE),
                    Some("Tournament with same name and start date already exists".to_string()),
                ),
            ));
        }

        let location_json =
            serde_json::to_string(&tournament.location).unwrap_or_else(|_| "{}".to_string());
        let contact_json =
            serde_json::to_string(&tournament.contact).unwrap_or_else(|_| "{}".to_string());
        let oc_json = serde_json::to_string(&tournament.oc).unwrap_or_else(|_| "{}".to_string());
        let officials_json =
            serde_json::to_string(&tournament.officials).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "INSERT INTO tournaments (
                uuid,
                name,
                duration_days,
                city,
                country,
                country_code,
                logo_path,
                status,
                start_date,
                end_date,
                ranking_id,
                location,
                contact,
                oc,
                officials,
                banner,
                created_at,
                updated_at
            )
            VALUES (
                COALESCE(?, lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',abs(random())%4+1,1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6)))),
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )",
            params![
                tournament.uuid,
                tournament.name,
                tournament.duration_days,
                tournament.city,
                tournament.country,
                tournament.country_code,
                tournament.logo_path,
                tournament.status,
                tournament.start_date.map(|d| d.to_rfc3339()),
                tournament.end_date.map(|d| d.to_rfc3339()),
                tournament.ranking_id,
                location_json,
                contact_json,
                oc_json,
                officials_json,
                tournament.banner,
                tournament.created_at.to_rfc3339(),
                tournament.updated_at.to_rfc3339(),
            ],
        )?;
        // Return the actual inserted row id, not affected rows count
        Ok(conn.last_insert_rowid())
    }

    /// Get all tournaments
    pub fn get_tournaments(conn: &Connection) -> DatabaseResult<Vec<Tournament>> {
        let mut stmt = conn.prepare(
            "SELECT
                id,
                uuid,
                name,
                duration_days,
                city,
                country,
                country_code,
                logo_path,
                status,
                start_date,
                end_date,
                ranking_id,
                location,
                contact,
                oc,
                officials,
                banner,
                created_at,
                updated_at
            FROM tournaments
            ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], Tournament::from_row)?;

        let mut tournaments = Vec::new();
        for row in rows {
            tournaments.push(row?);
        }

        Ok(tournaments)
    }

    /// Get tournament by ID
    pub fn get_tournament(
        conn: &Connection,
        tournament_id: i64,
    ) -> DatabaseResult<Option<Tournament>> {
        let tournament = conn
            .query_row(
                "SELECT
                id,
                uuid,
                name,
                duration_days,
                city,
                country,
                country_code,
                logo_path,
                status,
                start_date,
                end_date,
                ranking_id,
                location,
                contact,
                oc,
                officials,
                banner,
                created_at,
                updated_at
            FROM tournaments WHERE id = ?",
                params![tournament_id],
                Tournament::from_row,
            )
            .optional()?;

        Ok(tournament)
    }

    /// Update tournament
    pub fn update_tournament(
        conn: &mut Connection,
        tournament_id: i64,
        tournament: &Tournament,
    ) -> DatabaseResult<()> {
        let location_json =
            serde_json::to_string(&tournament.location).unwrap_or_else(|_| "{}".to_string());
        let contact_json =
            serde_json::to_string(&tournament.contact).unwrap_or_else(|_| "{}".to_string());
        let oc_json = serde_json::to_string(&tournament.oc).unwrap_or_else(|_| "{}".to_string());
        let officials_json =
            serde_json::to_string(&tournament.officials).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            "UPDATE tournaments
             SET name = ?,
                 duration_days = ?,
                 city = ?,
                 country = ?,
                 country_code = ?,
                 logo_path = ?,
                 status = ?,
                 start_date = ?,
                 end_date = ?,
                 ranking_id = ?,
                 location = ?,
                 contact = ?,
                 oc = ?,
                 officials = ?,
                 banner = ?,
                 updated_at = ?
             WHERE id = ?",
            params![
                tournament.name,
                tournament.duration_days,
                tournament.city,
                tournament.country,
                tournament.country_code,
                tournament.logo_path,
                tournament.status,
                tournament.start_date.map(|d| d.to_rfc3339()),
                tournament.end_date.map(|d| d.to_rfc3339()),
                tournament.ranking_id,
                location_json,
                contact_json,
                oc_json,
                officials_json,
                tournament.banner,
                Utc::now().to_rfc3339(),
                tournament_id,
            ],
        )?;

        Ok(())
    }

    /// Delete tournament
    pub fn delete_tournament(conn: &mut Connection, tournament_id: i64) -> DatabaseResult<()> {
        conn.execute(
            "DELETE FROM tournaments WHERE id = ?",
            params![tournament_id],
        )?;
        Ok(())
    }

    /// Create tournament days for a tournament
    pub fn create_tournament_days(
        conn: &mut Connection,
        tournament_id: i64,
        start_date: chrono::DateTime<chrono::Utc>,
        duration_days: i32,
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;

        for day_number in 1..=duration_days {
            let day_date = start_date + chrono::Duration::days((day_number - 1) as i64);
            let tournament_day = TournamentDay::new(tournament_id, day_number, day_date);

            tx.execute(
                "INSERT INTO tournament_days (uuid, tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at) VALUES (lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',abs(random())%4+1,1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6))), ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    tournament_day.tournament_id,
                    tournament_day.day_number,
                    tournament_day.date.to_rfc3339(),
                    tournament_day.status,
                    tournament_day.start_time.map(|t| t.to_rfc3339()),
                    tournament_day.end_time.map(|t| t.to_rfc3339()),
                    tournament_day.created_at.to_rfc3339(),
                    tournament_day.updated_at.to_rfc3339(),
                ]
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get tournament days for a tournament
    pub fn get_tournament_days(
        conn: &Connection,
        tournament_id: i64,
    ) -> DatabaseResult<Vec<TournamentDay>> {
        let mut stmt = conn.prepare(
            "SELECT id, uuid, tournament_id, day_number, date, status, start_time, end_time, created_at, updated_at \
             FROM tournament_days WHERE tournament_id = ? ORDER BY day_number"
        )?;

        let rows = stmt.query_map(params![tournament_id], TournamentDay::from_row)?;

        let mut days = Vec::new();
        for row in rows {
            days.push(row?);
        }

        Ok(days)
    }

    /// Start a tournament day
    pub fn start_tournament_day(
        conn: &mut Connection,
        tournament_day_id: i64,
    ) -> DatabaseResult<()> {
        let now = Utc::now();

        // Update the tournament day status
        conn.execute(
            "UPDATE tournament_days SET status = ?, start_time = ?, updated_at = ? WHERE id = ?",
            params![
                "active",
                now.to_rfc3339(),
                now.to_rfc3339(),
                tournament_day_id
            ],
        )?;

        // Check if this is the first day and start the tournament
        let tournament_id: i64 = conn.query_row(
            "SELECT tournament_id FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0),
        )?;

        let day_number: i32 = conn.query_row(
            "SELECT day_number FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0),
        )?;

        if day_number == 1 {
            conn.execute(
                "UPDATE tournaments SET status = ?, start_date = ?, updated_at = ? WHERE id = ?",
                params!["active", now.to_rfc3339(), now.to_rfc3339(), tournament_id],
            )?;
        }

        Ok(())
    }

    /// End a tournament day
    pub fn end_tournament_day(conn: &mut Connection, tournament_day_id: i64) -> DatabaseResult<()> {
        let now = Utc::now();

        // Update the tournament day status
        conn.execute(
            "UPDATE tournament_days SET status = ?, end_time = ?, updated_at = ? WHERE id = ?",
            params![
                "completed",
                now.to_rfc3339(),
                now.to_rfc3339(),
                tournament_day_id
            ],
        )?;

        // Check if this is the last day and end the tournament
        let tournament_id: i64 = conn.query_row(
            "SELECT tournament_id FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0),
        )?;

        let day_number: i32 = conn.query_row(
            "SELECT day_number FROM tournament_days WHERE id = ?",
            params![tournament_day_id],
            |row| row.get(0),
        )?;

        let total_days: i32 = conn.query_row(
            "SELECT duration_days FROM tournaments WHERE id = ?",
            params![tournament_id],
            |row| row.get(0),
        )?;

        if day_number == total_days {
            conn.execute(
                "UPDATE tournaments SET status = ?, end_date = ?, updated_at = ? WHERE id = ?",
                params!["ended", now.to_rfc3339(), now.to_rfc3339(), tournament_id],
            )?;
        }

        Ok(())
    }

    /// Get active tournament
    pub fn get_active_tournament(conn: &Connection) -> DatabaseResult<Option<Tournament>> {
        let tournament = conn
            .query_row(
                "SELECT
                id,
                uuid,
                name,
                duration_days,
                city,
                country,
                country_code,
                logo_path,
                status,
                start_date,
                end_date,
                ranking_id,
                location,
                contact,
                oc,
                officials,
                banner,
                created_at,
                updated_at
            FROM tournaments
            WHERE status = 'running'
            ORDER BY created_at DESC
            LIMIT 1",
                [],
                Tournament::from_row,
            )
            .optional()?;

        Ok(tournament)
    }

    /// Get active tournament day
    pub fn get_active_tournament_day(
        conn: &Connection,
        tournament_id: i64,
    ) -> DatabaseResult<Option<TournamentDay>> {
        let day = conn
            .query_row(
                "SELECT
                id,
                uuid,
                tournament_id,
                day_number,
                date,
                status,
                start_time,
                end_time,
                created_at,
                updated_at
            FROM tournament_days
            WHERE tournament_id = ? AND status = 'running'
            ORDER BY day_number DESC
            LIMIT 1",
                params![tournament_id],
                TournamentDay::from_row,
            )
            .optional()?;

        Ok(day)
    }

    /// Update tournament logo
    pub fn update_tournament_logo(
        conn: &mut Connection,
        tournament_id: i64,
        logo_path: &str,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE tournaments SET logo_path = ?, updated_at = ? WHERE id = ?",
            params![logo_path, Utc::now().to_rfc3339(), tournament_id],
        )?;

        Ok(())
    }
}

/// Tournament ranking lookup operations
pub struct TournamentRankingOperations;

impl TournamentRankingOperations {
    pub fn list_all(conn: &Connection) -> DatabaseResult<Vec<TournamentRanking>> {
        let mut stmt = conn.prepare(
            "SELECT id, code, label, is_para, created_at, updated_at
             FROM tournament_rankings
             ORDER BY code",
        )?;

        let rows = stmt.query_map([], TournamentRanking::from_row)?;
        let mut rankings = Vec::new();
        for row in rows {
            rankings.push(row?);
        }
        Ok(rankings)
    }

    pub fn get_by_code(conn: &Connection, code: &str) -> DatabaseResult<Option<TournamentRanking>> {
        let ranking = conn
            .query_row(
                "SELECT id, code, label, is_para, created_at, updated_at
                 FROM tournament_rankings
                 WHERE code = ?1",
                params![code],
                TournamentRanking::from_row,
            )
            .optional()?;
        Ok(ranking)
    }
}

/// Operations for managing tournament octagons
pub struct OctagonOperations;

impl OctagonOperations {
    pub fn list_for_day(conn: &Connection, tournament_day_id: i64) -> DatabaseResult<Vec<Octagon>> {
        let mut stmt = conn.prepare(
            "SELECT id, tournament_id, tournament_day_id, octagon_number, created_at, updated_at
             FROM octagons
             WHERE tournament_day_id = ?1
             ORDER BY octagon_number",
        )?;

        let rows = stmt.query_map(params![tournament_day_id], Octagon::from_row)?;
        let mut octagons = Vec::new();
        for row in rows {
            octagons.push(row?);
        }
        Ok(octagons)
    }

    pub fn insert(
        conn: &mut Connection,
        tournament_id: i64,
        tournament_day_id: i64,
        octagon_number: &str,
    ) -> DatabaseResult<i64> {
        let octagon = Octagon::new(
            tournament_id,
            tournament_day_id,
            octagon_number.trim().to_string(),
        );

        conn.execute(
            "INSERT INTO octagons (tournament_id, tournament_day_id, octagon_number, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                octagon.tournament_id,
                octagon.tournament_day_id,
                octagon.octagon_number,
                octagon.created_at.to_rfc3339(),
                octagon.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }
}

/// Athlete master roster operations
pub struct AthleteOperations;

impl AthleteOperations {
    pub fn list_all(conn: &Connection, limit: Option<i64>) -> DatabaseResult<Vec<Athlete>> {
        let mut sql = String::from(
            "SELECT
                id,
                wtid,
                look_age_group_id,
                look_division_id,
                look_gender_id,
                look_weight_class_id,
                first_name,
                last_name,
                display_name,
                image,
                history,
                country,
                country_code,
                ioc_code,
                created_at,
                updated_at
             FROM athletes
             ORDER BY created_at DESC",
        );

        if limit.is_some() {
            sql.push_str(" LIMIT ?1");
        }

        let mut stmt = conn.prepare(&sql)?;
        let mapper = |row: &rusqlite::Row<'_>| Athlete::from_row(row);
        let rows = if let Some(limit) = limit {
            stmt.query_map(params![limit], mapper)?
        } else {
            stmt.query_map([], mapper)?
        };

        let mut athletes = Vec::new();
        for row in rows {
            athletes.push(row?);
        }
        Ok(athletes)
    }

    pub fn find_by_wtid(conn: &Connection, wtid: &str) -> DatabaseResult<Option<Athlete>> {
        let athlete = conn
            .query_row(
                "SELECT
                    id,
                    wtid,
                    look_age_group_id,
                    look_division_id,
                    look_gender_id,
                    look_weight_class_id,
                    first_name,
                    last_name,
                    display_name,
                    image,
                    history,
                    country,
                    country_code,
                    ioc_code,
                    created_at,
                    updated_at
                 FROM athletes
                 WHERE wtid = ?1",
                params![wtid],
                Athlete::from_row,
            )
            .optional()?;
        Ok(athlete)
    }
}

/// PSS Event Status Mark Operations for managing event recognition and validation
pub struct PssEventStatusOperations;

impl PssEventStatusOperations {
    /// Store a PSS event with status mark
    pub fn store_pss_event_with_status(
        conn: &mut Connection,
        event: &PssEventV2,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        let event_id = tx.execute(
            "INSERT INTO pss_events (
                session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                event.session_id,
                event.match_id,
                event.round_id,
                event.event_type_id,
                event.timestamp.to_rfc3339(),
                event.raw_data,
                event.parsed_data,
                event.event_sequence,
                event.processing_time_ms,
                event.is_valid,
                event.error_message,
                event.recognition_status,
                event.protocol_version,
                event.parser_confidence,
                event.validation_errors,
                event.tournament_id,
                event.created_at.to_rfc3339()
            ],
        )?;

        tx.commit()?;
        Ok(event_id as i64)
    }

    /// Update event recognition status and record history
    pub fn update_event_recognition_status(
        conn: &mut Connection,
        event_id: i64,
        new_status: &str,
        changed_by: &str,
        change_reason: Option<&str>,
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;

        // Get current status
        let current_status: String = tx.query_row(
            "SELECT recognition_status FROM pss_events WHERE id = ?",
            params![event_id],
            |row| row.get(0),
        )?;

        // Update event status
        tx.execute(
            "UPDATE pss_events SET recognition_status = ? WHERE id = ?",
            params![new_status, event_id],
        )?;

        // Record status change in history
        let history = PssEventRecognitionHistory::new(
            event_id,
            current_status,
            new_status.to_string(),
            changed_by.to_string(),
            "".to_string(), // We'll get raw_data separately if needed
        );

        tx.execute(
            "INSERT INTO pss_event_recognition_history (
                event_id, old_status, new_status, changed_by, change_reason,
                protocol_version, raw_data, parsed_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                history.event_id,
                history.old_status,
                history.new_status,
                history.changed_by,
                change_reason,
                history.protocol_version,
                history.raw_data,
                history.parsed_data,
                history.created_at.to_rfc3339()
            ],
        )?;

        tx.commit()?;
        Ok(())
    }

    /// Store unknown event
    pub fn store_unknown_event(
        conn: &mut Connection,
        unknown_event: &PssUnknownEvent,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        // Check if this pattern already exists
        let existing_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM pss_unknown_events WHERE session_id = ? AND raw_data = ?",
                params![unknown_event.session_id, unknown_event.raw_data],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(existing_id) = existing_id {
            // Update existing record
            tx.execute(
                "UPDATE pss_unknown_events SET
                    last_seen = ?, occurrence_count = occurrence_count + 1, updated_at = ?
                WHERE id = ?",
                params![
                    unknown_event.last_seen.to_rfc3339(),
                    unknown_event.updated_at.to_rfc3339(),
                    existing_id
                ],
            )?;
            tx.commit()?;
            Ok(existing_id)
        } else {
            // Insert new record
            let unknown_event_id = tx.execute(
                "INSERT INTO pss_unknown_events (
                    session_id, raw_data, first_seen, last_seen, occurrence_count,
                    pattern_hash, suggested_event_type, notes, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    unknown_event.session_id,
                    unknown_event.raw_data,
                    unknown_event.first_seen.to_rfc3339(),
                    unknown_event.last_seen.to_rfc3339(),
                    unknown_event.occurrence_count,
                    unknown_event.pattern_hash,
                    unknown_event.suggested_event_type,
                    unknown_event.notes,
                    unknown_event.created_at.to_rfc3339(),
                    unknown_event.updated_at.to_rfc3339()
                ],
            )?;

            tx.commit()?;
            Ok(unknown_event_id as i64)
        }
    }

    /// Get validation rules for an event type
    pub fn get_validation_rules(
        conn: &Connection,
        event_code: &str,
        protocol_version: &str,
    ) -> DatabaseResult<Vec<PssEventValidationRule>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_code, protocol_version, rule_name, rule_type, rule_definition,
                    error_message, is_active, created_at, updated_at
             FROM pss_event_validation_rules
             WHERE event_code = ? AND protocol_version = ? AND is_active = 1
             ORDER BY rule_name",
        )?;

        let rows = stmt.query_map(params![event_code, protocol_version], |row| {
            PssEventValidationRule::from_row(row)
        })?;

        let mut rules = Vec::new();
        for row in rows {
            rules.push(row?);
        }

        Ok(rules)
    }

    /// Store validation result
    pub fn store_validation_result(
        conn: &mut Connection,
        validation_result: &PssEventValidationResult,
    ) -> DatabaseResult<i64> {
        let validation_result_id = conn.execute(
            "INSERT INTO pss_event_validation_results (
                event_id, rule_id, validation_passed, error_message, validation_time_ms, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                validation_result.event_id,
                validation_result.rule_id,
                validation_result.validation_passed,
                validation_result.error_message,
                validation_result.validation_time_ms,
                validation_result.created_at.to_rfc3339()
            ],
        )?;

        Ok(validation_result_id as i64)
    }

    /// Update event statistics
    pub fn update_event_statistics(
        conn: &mut Connection,
        session_id: i64,
        event_type_id: Option<i64>,
        recognition_status: &str,
        processing_time_ms: Option<i32>,
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;

        // Get or create statistics record
        let stats_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM pss_event_statistics WHERE session_id = ? AND event_type_id IS ?",
                params![session_id, event_type_id],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(stats_id) = stats_id {
            // Update existing statistics
            let update_sql = match recognition_status {
                "recognized" => "recognized_events = recognized_events + 1",
                "unknown" => "unknown_events = unknown_events + 1",
                "partial" => "partial_events = partial_events + 1",
                "deprecated" => "deprecated_events = deprecated_events + 1",
                _ => "total_events = total_events + 1",
            };

            tx.execute(
                &format!(
                    "UPDATE pss_event_statistics SET
                    total_events = total_events + 1,
                    {update_sql},
                    updated_at = ?
                    WHERE id = ?"
                ),
                params![chrono::Utc::now().to_rfc3339(), stats_id],
            )?;

            // Update processing time statistics if available
            if let Some(processing_time) = processing_time_ms {
                tx.execute(
                    "UPDATE pss_event_statistics SET
                        average_processing_time_ms = (
                            (average_processing_time_ms * total_events + ?) / (total_events + 1)
                        ),
                        min_processing_time_ms = CASE
                            WHEN min_processing_time_ms IS NULL OR ? < min_processing_time_ms
                            THEN ? ELSE min_processing_time_ms END,
                        max_processing_time_ms = CASE
                            WHEN max_processing_time_ms IS NULL OR ? > max_processing_time_ms
                            THEN ? ELSE max_processing_time_ms END
                    WHERE id = ?",
                    params![
                        processing_time,
                        processing_time,
                        processing_time,
                        processing_time,
                        processing_time,
                        stats_id
                    ],
                )?;
            }
        } else {
            // Create new statistics record
            let stats = PssEventStatistics::new(session_id, event_type_id);
            let total_events = 1;
            let mut recognized_events = 0;
            let mut unknown_events = 0;
            let mut partial_events = 0;
            let mut deprecated_events = 0;

            match recognition_status {
                "recognized" => recognized_events = 1,
                "unknown" => unknown_events = 1,
                "partial" => partial_events = 1,
                "deprecated" => deprecated_events = 1,
                _ => {}
            }

            tx.execute(
                "INSERT INTO pss_event_statistics (
                    session_id, event_type_id, total_events, recognized_events, unknown_events,
                    partial_events, deprecated_events, validation_errors, parsing_errors,
                    average_processing_time_ms, min_processing_time_ms, max_processing_time_ms,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    stats.session_id,
                    stats.event_type_id,
                    total_events,
                    recognized_events,
                    unknown_events,
                    partial_events,
                    deprecated_events,
                    stats.validation_errors,
                    stats.parsing_errors,
                    processing_time_ms.unwrap_or(0) as f64,
                    processing_time_ms,
                    processing_time_ms,
                    stats.created_at.to_rfc3339(),
                    stats.updated_at.to_rfc3339()
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get event statistics for a session
    pub fn get_session_statistics(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<Vec<PssEventStatistics>> {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, event_type_id, total_events, recognized_events, unknown_events,
                    partial_events, deprecated_events, validation_errors, parsing_errors,
                    average_processing_time_ms, min_processing_time_ms, max_processing_time_ms,
                    created_at, updated_at
             FROM pss_event_statistics
             WHERE session_id = ?
             ORDER BY total_events DESC",
        )?;

        let rows = stmt.query_map(params![session_id], PssEventStatistics::from_row)?;

        let mut statistics = Vec::new();
        for row in rows {
            statistics.push(row?);
        }

        Ok(statistics)
    }

    /// Get unknown events for analysis
    pub fn get_unknown_events(
        conn: &Connection,
        session_id: Option<i64>,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PssUnknownEvent>> {
        let limit = limit.unwrap_or(100);

        let sql = if let Some(_session_id) = session_id {
            "SELECT id, session_id, raw_data, first_seen, last_seen, occurrence_count,
                    pattern_hash, suggested_event_type, notes, created_at, updated_at
             FROM pss_unknown_events
             WHERE session_id = ?
             ORDER BY occurrence_count DESC, last_seen DESC
             LIMIT ?"
        } else {
            "SELECT id, session_id, raw_data, first_seen, last_seen, occurrence_count,
                    pattern_hash, suggested_event_type, notes, created_at, updated_at
             FROM pss_unknown_events
             ORDER BY occurrence_count DESC, last_seen DESC
             LIMIT ?"
        };

        let mut stmt = conn.prepare(sql)?;

        let rows = if let Some(session_id) = session_id {
            stmt.query_map(params![session_id, limit], PssUnknownEvent::from_row)?
        } else {
            stmt.query_map(params![limit], PssUnknownEvent::from_row)?
        };

        let mut unknown_events = Vec::new();
        for row in rows {
            unknown_events.push(row?);
        }

        Ok(unknown_events)
    }

    /// Get recognition history for an event
    pub fn get_event_recognition_history(
        conn: &Connection,
        event_id: i64,
    ) -> DatabaseResult<Vec<PssEventRecognitionHistory>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_id, old_status, new_status, changed_by, change_reason,
                    protocol_version, raw_data, parsed_data, created_at
             FROM pss_event_recognition_history
             WHERE event_id = ?
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map(params![event_id], |row| {
            PssEventRecognitionHistory::from_row(row)
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }

        Ok(history)
    }

    /// Get events by recognition status
    pub fn get_events_by_status(
        conn: &Connection,
        session_id: i64,
        recognition_status: &str,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<PssEventV2>> {
        let limit = limit.unwrap_or(100);

        let mut stmt = conn.prepare(
            "SELECT id, session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                    parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                    recognition_status, protocol_version, parser_confidence, validation_errors, created_at
             FROM pss_events
             WHERE session_id = ? AND recognition_status = ?
             ORDER BY created_at DESC
             LIMIT ?"
        )?;

        let rows = stmt.query_map(params![session_id, recognition_status, limit], |row| {
            PssEventV2::from_row(row)
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    /// Get comprehensive event statistics with status breakdown
    pub fn get_comprehensive_event_statistics(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<serde_json::Value> {
        // Get overall statistics
        let overall_stats = conn.query_row(
            "SELECT
                COUNT(*) as total_events,
                SUM(CASE WHEN recognition_status = 'recognized' THEN 1 ELSE 0 END) as recognized_events,
                SUM(CASE WHEN recognition_status = 'unknown' THEN 1 ELSE 0 END) as unknown_events,
                SUM(CASE WHEN recognition_status = 'partial' THEN 1 ELSE 0 END) as partial_events,
                SUM(CASE WHEN recognition_status = 'deprecated' THEN 1 ELSE 0 END) as deprecated_events,
                AVG(parser_confidence) as avg_confidence,
                AVG(processing_time_ms) as avg_processing_time,
                MIN(processing_time_ms) as min_processing_time,
                MAX(processing_time_ms) as max_processing_time
            FROM pss_events
            WHERE session_id = ?",
            params![session_id],
            |row| {
                Ok(serde_json::json!({
                    "total_events": row.get::<_, i64>(0)?,
                    "recognized_events": row.get::<_, i64>(1)?,
                    "unknown_events": row.get::<_, i64>(2)?,
                    "partial_events": row.get::<_, i64>(3)?,
                    "deprecated_events": row.get::<_, i64>(4)?,
                    "avg_confidence": row.get::<_, Option<f64>>(5)?,
                    "avg_processing_time": row.get::<_, Option<f64>>(6)?,
                    "min_processing_time": row.get::<_, Option<i32>>(7)?,
                    "max_processing_time": row.get::<_, Option<i32>>(8)?
                }))
            }
        )?;

        // Get statistics by event type
        let mut event_type_stats = conn.prepare(
            "SELECT
                et.event_code,
                et.event_name,
                COUNT(*) as total,
                SUM(CASE WHEN e.recognition_status = 'recognized' THEN 1 ELSE 0 END) as recognized,
                SUM(CASE WHEN e.recognition_status = 'unknown' THEN 1 ELSE 0 END) as unknown,
                SUM(CASE WHEN e.recognition_status = 'partial' THEN 1 ELSE 0 END) as partial,
                AVG(e.parser_confidence) as avg_confidence,
                AVG(e.processing_time_ms) as avg_processing_time
            FROM pss_events e
            JOIN pss_event_types et ON e.event_type_id = et.id
            WHERE e.session_id = ?
            GROUP BY et.id, et.event_code, et.event_name
            ORDER BY total DESC",
        )?;

        let event_type_rows = event_type_stats.query_map(params![session_id], |row| {
            Ok(serde_json::json!({
                "event_code": row.get::<_, String>(0)?,
                "event_name": row.get::<_, String>(1)?,
                "total": row.get::<_, i64>(2)?,
                "recognized": row.get::<_, i64>(3)?,
                "unknown": row.get::<_, i64>(4)?,
                "partial": row.get::<_, i64>(5)?,
                "avg_confidence": row.get::<_, Option<f64>>(6)?,
                "avg_processing_time": row.get::<_, Option<f64>>(7)?
            }))
        })?;

        let mut event_type_stats_vec = Vec::new();
        for row in event_type_rows {
            event_type_stats_vec.push(row?);
        }

        // Get validation error breakdown
        let mut validation_errors = conn.prepare(
            "SELECT
                validation_errors,
                COUNT(*) as count
            FROM pss_events
            WHERE session_id = ? AND validation_errors IS NOT NULL
            GROUP BY validation_errors
            ORDER BY count DESC
            LIMIT 10",
        )?;

        let validation_rows = validation_errors.query_map(params![session_id], |row| {
            Ok(serde_json::json!({
                "error": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?
            }))
        })?;

        let mut validation_errors_vec = Vec::new();
        for row in validation_rows {
            validation_errors_vec.push(row?);
        }

        // Get unknown events summary
        let unknown_events_summary = conn
            .query_row(
                "SELECT
                COUNT(*) as total_unknown,
                COUNT(DISTINCT pattern_hash) as unique_patterns,
                MAX(occurrence_count) as max_occurrences
            FROM pss_unknown_events
            WHERE session_id = ?",
                params![session_id],
                |row| {
                    Ok(serde_json::json!({
                        "total_unknown": row.get::<_, i64>(0)?,
                        "unique_patterns": row.get::<_, i64>(1)?,
                        "max_occurrences": row.get::<_, i64>(2)?
                    }))
                },
            )
            .unwrap_or_else(|_| {
                serde_json::json!({
                    "total_unknown": 0,
                    "unique_patterns": 0,
                    "max_occurrences": 0
                })
            });

        Ok(serde_json::json!({
            "overall": overall_stats,
            "by_event_type": event_type_stats_vec,
            "validation_errors": validation_errors_vec,
            "unknown_events": unknown_events_summary
        }))
    }
}

/// PSS Event Operations for managing event types and basic event operations
pub struct PssEventOperations;

impl PssEventOperations {
    /// Get PSS event type by code
    pub fn get_pss_event_type_by_code(
        conn: &Connection,
        event_code: &str,
    ) -> DatabaseResult<Option<PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_code, event_name, description, category, is_active, created_at
             FROM pss_event_types
             WHERE event_code = ?",
        )?;

        let mut rows = stmt.query_map(params![event_code], PssEventType::from_row)?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// Upsert PSS event type
    pub fn upsert_pss_event_type(
        conn: &mut Connection,
        event_type: &PssEventType,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;

        // Check if event type already exists
        let existing_id: Option<i64> = tx
            .query_row(
                "SELECT id FROM pss_event_types WHERE event_code = ?",
                params![event_type.event_code],
                |row| row.get(0),
            )
            .optional()?;

        let event_type_id = if let Some(id) = existing_id {
            // Update existing event type - note: pss_event_types table doesn't have updated_at
            tx.execute(
                "UPDATE pss_event_types SET
                    event_name = ?, description = ?, category = ?, is_active = ?
                WHERE id = ?",
                params![
                    event_type.event_name,
                    event_type.description,
                    event_type.category,
                    event_type.is_active,
                    id
                ],
            )?;
            id
        } else {
            // Insert new event type
            tx.execute(
                "INSERT INTO pss_event_types (event_code, event_name, description, category, is_active, created_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    event_type.event_code,
                    event_type.event_name,
                    event_type.description,
                    event_type.category,
                    event_type.is_active,
                    event_type.created_at.to_rfc3339()
                ]
            )?;
            tx.last_insert_rowid()
        };

        tx.commit()?;
        Ok(event_type_id)
    }

    /// Get all PSS event types
    pub fn get_all_pss_event_types(conn: &Connection) -> DatabaseResult<Vec<PssEventType>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_code, event_name, description, category, is_active, created_at
             FROM pss_event_types
             ORDER BY event_code",
        )?;

        let rows = stmt.query_map([], PssEventType::from_row)?;

        let mut event_types = Vec::new();
        for row in rows {
            event_types.push(row?);
        }

        Ok(event_types)
    }

    /// Delete PSS event type
    pub fn delete_pss_event_type(conn: &mut Connection, event_type_id: i64) -> DatabaseResult<()> {
        conn.execute(
            "DELETE FROM pss_event_types WHERE id = ?",
            params![event_type_id],
        )?;
        Ok(())
    }
}

/// Phase 2 Optimization: Data Archival Strategy
/// Manages automatic archival of old events to improve performance
pub struct DataArchivalOperations;

impl DataArchivalOperations {
    /// Archive events older than specified days
    pub fn archive_old_events(
        conn: &mut rusqlite::Connection,
        days_old: i64,
    ) -> DatabaseResult<usize> {
        let start_time = std::time::Instant::now();
        let cutoff = format!("-{} days", days_old);

        // Create archive table if it doesn't exist (schema mirrors pss_events)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pss_events_archive (
                id INTEGER PRIMARY KEY,
                session_id INTEGER NOT NULL,
                match_id INTEGER,
                round_id INTEGER,
                event_type_id INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                raw_data TEXT NOT NULL,
                parsed_data TEXT,
                event_sequence INTEGER,
                processing_time_ms INTEGER,
                is_valid BOOLEAN NOT NULL,
                error_message TEXT,
                recognition_status TEXT NOT NULL,
                protocol_version TEXT,
                parser_confidence REAL,
                validation_errors TEXT,
                tournament_id TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create indices for archive table
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_archive_session_id ON pss_events_archive(session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_archive_created_at ON pss_events_archive(created_at)",
            [],
        )?;

        // Archive events older than specified days
        let archived_count = conn.execute(
            "INSERT INTO pss_events_archive (
                id, session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at
            )
             SELECT
                id, session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at
             FROM pss_events
             WHERE created_at < datetime('now', ?1)",
            params![cutoff.as_str()],
        )?;

        // Delete archived events from main table
        let deleted_count = conn.execute(
            "DELETE FROM pss_events
             WHERE created_at < datetime('now', ?1)",
            params![cutoff.as_str()],
        )?;

        // Archive related event details
        let archived_details = conn.execute(
            "INSERT INTO pss_event_details_archive (
                id, event_id, detail_key, detail_value, detail_type, created_at
            )
             SELECT
                id, event_id, detail_key, detail_value, detail_type, created_at
             FROM pss_event_details
             WHERE event_id IN (
                 SELECT id FROM pss_events_archive
                 WHERE created_at < datetime('now', ?1)
             )",
            params![cutoff.as_str()],
        )?;

        // Delete archived event details from main table
        let deleted_details = conn.execute(
            "DELETE FROM pss_event_details
             WHERE event_id IN (
                 SELECT id FROM pss_events_archive
                 WHERE created_at < datetime('now', ?1)
             )",
            params![cutoff.as_str()],
        )?;

        let duration = start_time.elapsed();
        log::info!(
            " Archived {archived_count} events and {archived_details} details in {duration:?} (deleted {deleted_count} events and {deleted_details} details)"
        );

        Ok(archived_count)
    }

    /// Get archive statistics
    pub fn get_archive_statistics(
        conn: &rusqlite::Connection,
    ) -> DatabaseResult<ArchiveStatistics> {
        let archived_events =
            conn.query_row("SELECT COUNT(*) FROM pss_events_archive", [], |row| {
                row.get(0)
            })?;

        let archived_details = conn.query_row(
            "SELECT COUNT(*) FROM pss_event_details_archive",
            [],
            |row| row.get(0),
        )?;

        let oldest_archived = conn.query_row(
            "SELECT MIN(created_at) FROM pss_events_archive",
            [],
            |row| row.get::<_, Option<String>>(0),
        )?;

        let newest_archived = conn.query_row(
            "SELECT MAX(created_at) FROM pss_events_archive",
            [],
            |row| row.get::<_, Option<String>>(0),
        )?;

        let archive_size = conn.query_row(
            "SELECT SUM(length(raw_data)) FROM pss_events_archive",
            [],
            |row| row.get::<_, Option<i64>>(0),
        )?;

        Ok(ArchiveStatistics {
            archived_events,
            archived_details,
            oldest_archived,
            newest_archived,
            archive_size_bytes: archive_size.unwrap_or(0),
        })
    }

    /// Restore events from archive (for data recovery)
    pub fn restore_from_archive(
        conn: &mut rusqlite::Connection,
        start_date: &str,
        end_date: &str,
    ) -> DatabaseResult<usize> {
        let start_time = std::time::Instant::now();

        // Restore events from archive
        let restored_events = conn.execute(
            "INSERT INTO pss_events (
                id, session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at
            )
             SELECT
                id, session_id, match_id, round_id, event_type_id, timestamp, raw_data,
                parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
                recognition_status, protocol_version, parser_confidence, validation_errors,
                tournament_id, created_at
             FROM pss_events_archive
             WHERE created_at BETWEEN ?1 AND ?2",
            params![start_date, end_date],
        )?;

        // Restore event details
        let restored_details = conn.execute(
            "INSERT INTO pss_event_details (
                id, event_id, detail_key, detail_value, detail_type, created_at
            )
             SELECT
                id, event_id, detail_key, detail_value, detail_type, created_at
             FROM pss_event_details_archive
             WHERE event_id IN (
                 SELECT id FROM pss_events_v2
                 WHERE created_at BETWEEN ?1 AND ?2
             )",
            params![start_date, end_date],
        )?;

        // Remove restored events from archive
        let _removed_from_archive = conn.execute(
            "DELETE FROM pss_events_archive
             WHERE created_at BETWEEN ? AND ?",
            [start_date, end_date],
        )?;

        let duration = start_time.elapsed();
        log::info!(
            " Restored {restored_events} events and {restored_details} details from archive in {duration:?}"
        );

        Ok(restored_events)
    }

    /// Clean up old archive data (permanent deletion)
    pub fn cleanup_old_archive_data(
        conn: &mut rusqlite::Connection,
        days_old: i64,
    ) -> DatabaseResult<usize> {
        let start_time = std::time::Instant::now();
        let cutoff = format!("-{} days", days_old);

        // Delete old archived events
        let deleted_events = conn.execute(
            "DELETE FROM pss_events_archive
             WHERE created_at < datetime('now', ?1)",
            params![cutoff.as_str()],
        )?;

        // Delete old archived event details
        let deleted_details = conn.execute(
            "DELETE FROM pss_event_details_archive
             WHERE event_id NOT IN (SELECT id FROM pss_events_archive)",
            [],
        )?;

        let duration = start_time.elapsed();
        log::info!(
            " Cleaned up {deleted_events} archived events and {deleted_details} details in {duration:?}"
        );

        Ok(deleted_events)
    }

    /// Optimize archive tables
    pub fn optimize_archive_tables(conn: &mut rusqlite::Connection) -> DatabaseResult<()> {
        log::info!("Optimizing archive tables...");

        // VACUUM archive tables
        conn.execute("VACUUM pss_events_archive", [])?;
        conn.execute("VACUUM pss_event_details_archive", [])?;

        // Analyze tables for better query planning
        conn.execute("ANALYZE pss_events_archive", [])?;
        conn.execute("ANALYZE pss_event_details_archive", [])?;

        // Optimize indices
        conn.execute("REINDEX pss_events_archive", [])?;
        conn.execute("REINDEX pss_event_details_archive", [])?;

        log::info!("Archive tables optimized successfully");
        Ok(())
    }
}

/// Archive statistics for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveStatistics {
    pub archived_events: i64,
    pub archived_details: i64,
    pub oldest_archived: Option<String>,
    pub newest_archived: Option<String>,
    pub archive_size_bytes: i64,
}

// ============================================================================
// TRIGGER SYSTEM OPERATIONS
// ============================================================================

impl DatabaseConnection {
    // ========================================================================
    // OVERLAY TEMPLATE OPERATIONS
    // ========================================================================

    /// Get all overlay templates
    pub async fn get_overlay_templates(&self) -> DatabaseResult<Vec<OverlayTemplate>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM overlay_templates ORDER BY name")?;

        let templates = stmt
            .query_map([], OverlayTemplate::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    /// Get active overlay templates only
    pub async fn get_active_overlay_templates(&self) -> DatabaseResult<Vec<OverlayTemplate>> {
        let conn = self.get_connection().await?;
        let mut stmt =
            conn.prepare("SELECT * FROM overlay_templates WHERE is_active = 1 ORDER BY name")?;

        let templates = stmt
            .query_map([], OverlayTemplate::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    /// Get overlay template by name
    pub async fn get_overlay_template_by_name(
        &self,
        name: &str,
    ) -> DatabaseResult<Option<OverlayTemplate>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare("SELECT * FROM overlay_templates WHERE name = ?")?;

        let template = stmt
            .query_row([name], OverlayTemplate::from_row)
            .optional()?;

        Ok(template)
    }

    /// Insert overlay template
    pub async fn insert_overlay_template(&self, template: &OverlayTemplate) -> DatabaseResult<i64> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        let id = conn.execute(
            "INSERT INTO overlay_templates (name, description, theme, colors, animation_type, duration_ms, is_active, url, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            [
                &template.name,
                &template.description.as_deref().unwrap_or("").to_string(),
                &template.theme,
                &template.colors.as_deref().unwrap_or("").to_string(),
                &template.animation_type,
                &template.duration_ms.to_string(),
                &(template.is_active as i32).to_string(),
                &template.url.as_deref().unwrap_or("").to_string(),
                &template.created_at.to_rfc3339(),
                &now,
            ],
        )?;

        Ok(id as i64)
    }

    /// Update overlay template
    pub async fn update_overlay_template(&self, template: &OverlayTemplate) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE overlay_templates SET description = ?, theme = ?, colors = ?, animation_type = ?, duration_ms = ?, is_active = ?, url = ?, updated_at = ?
             WHERE id = ?",
            [
                &template.description.as_deref().unwrap_or("").to_string(),
                &template.theme,
                &template.colors.as_deref().unwrap_or("").to_string(),
                &template.animation_type,
                &template.duration_ms.to_string(),
                &(template.is_active as i32).to_string(),
                &template.url.as_deref().unwrap_or("").to_string(),
                &now,
                &template.id.unwrap_or(0).to_string(),
            ],
        )?;

        Ok(())
    }

    /// Delete overlay template
    pub async fn delete_overlay_template(&self, id: i64) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;

        conn.execute("DELETE FROM overlay_templates WHERE id = ?", [id])?;

        Ok(())
    }

    // ========================================================================
    // EVENT TRIGGER OPERATIONS
    // ========================================================================

    /// Get all event triggers
    pub async fn get_event_triggers(&self) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt =
            conn.prepare("SELECT * FROM event_triggers ORDER BY priority DESC, event_type")?;

        let triggers = stmt
            .query_map([], EventTrigger::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(triggers)
    }

    /// Get event triggers for a specific tournament
    pub async fn get_event_triggers_for_tournament(
        &self,
        tournament_id: i64,
    ) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare(
            "SELECT * FROM event_triggers WHERE tournament_id = ? ORDER BY priority DESC, event_type"
        )?;

        let triggers = stmt
            .query_map([tournament_id], EventTrigger::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(triggers)
    }

    /// Get global event triggers (no tournament/day specified)
    pub async fn get_global_event_triggers(&self) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;
        let mut stmt = conn.prepare(
            "SELECT * FROM event_triggers WHERE tournament_id IS NULL ORDER BY priority DESC, event_type"
        )?;

        let triggers = stmt
            .query_map([], EventTrigger::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(triggers)
    }

    /// Get enabled event triggers for a specific event type
    pub async fn get_enabled_triggers_for_event(
        &self,
        event_type: &str,
        tournament_id: Option<i64>,
    ) -> DatabaseResult<Vec<EventTrigger>> {
        let conn = self.get_connection().await?;

        let mut query =
            String::from("SELECT * FROM event_triggers WHERE event_type = ? AND is_enabled = 1");
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(event_type.to_string())];

        if let Some(tid) = tournament_id {
            query.push_str(" AND (tournament_id = ? OR tournament_id IS NULL)");
            params.push(Box::new(tid));
        }

        query.push_str(" ORDER BY priority DESC");

        let mut stmt = conn.prepare(&query)?;
        let triggers = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                EventTrigger::from_row(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(triggers)
    }

    /// Insert event trigger
    pub async fn insert_event_trigger(&self, trigger: &EventTrigger) -> DatabaseResult<i64> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        use rusqlite::params;
        let id = conn.execute(
            "INSERT INTO event_triggers (
                tournament_id, event_type, trigger_type,
                obs_scene_id, overlay_template_id,
                action_kind, obs_connection_name,
                condition_round, condition_once_per, debounce_ms, cooldown_ms,
                is_enabled, priority, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                trigger.tournament_id,
                trigger.event_type,
                trigger.trigger_type,
                trigger.obs_scene_id,
                trigger.overlay_template_id,
                trigger.action_kind,
                trigger.obs_connection_name,
                trigger.condition_round,
                trigger.condition_once_per,
                trigger.debounce_ms,
                trigger.cooldown_ms,
                trigger.is_enabled,
                trigger.priority,
                trigger.created_at.to_rfc3339(),
                now,
            ],
        )?;

        Ok(id as i64)
    }

    /// Update event trigger
    pub async fn update_event_trigger(&self, trigger: &EventTrigger) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        use rusqlite::params;
        conn.execute(
            "UPDATE event_triggers SET
                tournament_id = ?, event_type = ?, trigger_type = ?,
                obs_scene_id = ?, overlay_template_id = ?,
                action_kind = ?, obs_connection_name = ?,
                condition_round = ?, condition_once_per = ?, debounce_ms = ?, cooldown_ms = ?,
                is_enabled = ?, priority = ?, updated_at = ?
             WHERE id = ?",
            params![
                trigger.tournament_id,
                trigger.event_type,
                trigger.trigger_type,
                trigger.obs_scene_id,
                trigger.overlay_template_id,
                trigger.action_kind,
                trigger.obs_connection_name,
                trigger.condition_round,
                trigger.condition_once_per,
                trigger.debounce_ms,
                trigger.cooldown_ms,
                trigger.is_enabled,
                trigger.priority,
                now,
                trigger.id,
            ],
        )?;

        Ok(())
    }

    /// Delete event trigger
    pub async fn delete_event_trigger(&self, id: i64) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;

        conn.execute("DELETE FROM event_triggers WHERE id = ?", [id])?;

        Ok(())
    }

    /// Enable/disable event trigger
    pub async fn set_event_trigger_enabled(&self, id: i64, enabled: bool) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE event_triggers SET is_enabled = ?, updated_at = ? WHERE id = ?",
            [&(enabled as i32).to_string(), &now, &id.to_string()],
        )?;

        Ok(())
    }

    /// Copy triggers from one tournament to another
    pub async fn copy_triggers_to_tournament(
        &self,
        source_tournament_id: i64,
        target_tournament_id: i64,
    ) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
                        "INSERT INTO event_triggers (tournament_id, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, created_at, updated_at)
                         SELECT ?, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, ?, ?
                            FROM event_triggers WHERE tournament_id = ?",
                        [&target_tournament_id.to_string(), &now, &now, &source_tournament_id.to_string()],
                )?;

        Ok(())
    }

    /// Save triggers as template (global triggers)
    pub async fn save_triggers_as_template(
        &self,
        tournament_id: i64,
        _template_name: &str,
    ) -> DatabaseResult<()> {
        let conn = self.get_connection().await?;
        let now = chrono::Utc::now().to_rfc3339();

        // Create a special template trigger with the template name
        conn.execute(
                        "INSERT INTO event_triggers (tournament_id, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, created_at, updated_at)
                         SELECT NULL, event_type, trigger_type, obs_scene_id, overlay_template_id, is_enabled, priority, ?, ?
                            FROM event_triggers WHERE tournament_id = ?",
                        [&now, &now, &tournament_id.to_string()],
                )?;

        Ok(())
    }
}

/// OBS Recording Operations for managing recording configuration and sessions
pub struct ObsRecordingOperations;

impl ObsRecordingOperations {
    /// Get all OBS recording configurations
    pub fn get_recording_configs(conn: &Connection) -> DatabaseResult<Vec<ObsRecordingConfig>> {
        let mut stmt =
            conn.prepare("SELECT * FROM obs_recording_config ORDER BY obs_connection_name")?;

        let configs = stmt
            .query_map([], ObsRecordingConfig::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(configs)
    }

    /// Get recording configuration for a specific OBS connection
    pub fn get_recording_config(
        conn: &Connection,
        obs_connection_name: &str,
    ) -> DatabaseResult<Option<ObsRecordingConfig>> {
        let mut stmt =
            conn.prepare("SELECT * FROM obs_recording_config WHERE obs_connection_name = ?")?;

        let config = stmt
            .query_row([obs_connection_name], |row| {
                ObsRecordingConfig::from_row(row)
            })
            .optional()?;

        Ok(config)
    }

    /// Create or update recording configuration
    pub fn upsert_recording_config(
        conn: &mut Connection,
        config: &ObsRecordingConfig,
    ) -> DatabaseResult<i64> {
        use rusqlite::params;
        let config_id = conn.execute(
            "INSERT OR REPLACE INTO obs_recording_config (
                obs_connection_name, recording_root_path, recording_format,
                replay_buffer_enabled, replay_buffer_duration,
                auto_start_recording, auto_start_replay_buffer, filename_template, folder_pattern, is_active,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                config.obs_connection_name,
                config.recording_root_path,
                config.recording_format,
                config.replay_buffer_enabled,
                config.replay_buffer_duration,
                config.auto_start_recording,
                config.auto_start_replay_buffer,
                config.filename_template,
                config.folder_pattern,
                config.is_active,
                config.created_at.to_rfc3339(),
                Utc::now().to_rfc3339(),
            ],
        )?;

        Ok(config_id as i64)
    }

    /// Delete recording configuration
    pub fn delete_recording_config(
        conn: &mut Connection,
        obs_connection_name: &str,
    ) -> DatabaseResult<()> {
        conn.execute(
            "DELETE FROM obs_recording_config WHERE obs_connection_name = ?",
            [obs_connection_name],
        )?;
        Ok(())
    }

    /// Get active recording sessions
    pub fn get_active_recording_sessions(
        conn: &Connection,
    ) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_session WHERE status IN ('pending', 'recording') ORDER BY created_at DESC"
        )?;

        let sessions = stmt
            .query_map([], ObsRecordingSession::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get recording sessions for a specific OBS connection
    pub fn get_recording_sessions_for_connection(
        conn: &Connection,
        obs_connection_name: &str,
    ) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_session WHERE obs_connection_name = ? ORDER BY created_at DESC"
        )?;

        let sessions = stmt
            .query_map([obs_connection_name], |row| {
                ObsRecordingSession::from_row(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Get recording sessions for a specific match
    pub fn get_recording_sessions_for_match(
        conn: &Connection,
        match_id: &str,
    ) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM obs_recording_session WHERE match_code = ? ORDER BY created_at DESC",
        )?;

        let sessions = stmt
            .query_map([match_id], ObsRecordingSession::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    /// Create new recording session
    pub fn create_recording_session(
        conn: &mut Connection,
        session: &ObsRecordingSession,
    ) -> DatabaseResult<i64> {
        conn.execute(
            "INSERT INTO obs_recording_session (
                obs_connection_name, tournament_id, match_code, match_number,
                player1_name, player1_flag, player2_name, player2_flag, recording_path,
                recording_filename, recording_start_time, recording_end_time, recording_duration,
                recording_size_bytes, replay_buffer_start_time, replay_buffer_end_time,
                replay_buffer_saved, replay_buffer_filename, status, error_message,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session.obs_connection_name,
                session.tournament_id,
                session.match_id,
                session.match_number,
                session.player1_name,
                session.player1_flag,
                session.player2_name,
                session.player2_flag,
                session.recording_path,
                session.recording_filename,
                session.recording_start_time.map(|dt| dt.to_rfc3339()),
                session.recording_end_time.map(|dt| dt.to_rfc3339()),
                session.recording_duration,
                session.recording_size_bytes,
                session.replay_buffer_start_time.map(|dt| dt.to_rfc3339()),
                session.replay_buffer_end_time.map(|dt| dt.to_rfc3339()),
                session.replay_buffer_saved,
                session.replay_buffer_filename.clone(),
                session.status,
                session.error_message.clone(),
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Update recording session
    pub fn update_recording_session(
        conn: &mut Connection,
        session_id: i64,
        session: &ObsRecordingSession,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE obs_recording_session SET
                obs_connection_name = ?, tournament_id = ?, match_code = ?, match_number = ?,
                player1_name = ?, player1_flag = ?, player2_name = ?, player2_flag = ?, recording_path = ?,
                recording_filename = ?, recording_start_time = ?, recording_end_time = ?, recording_duration = ?,
                recording_size_bytes = ?, replay_buffer_start_time = ?, replay_buffer_end_time = ?,
                replay_buffer_saved = ?, replay_buffer_filename = ?, status = ?, error_message = ?, updated_at = ?
            WHERE id = ?",
            params![
                session.obs_connection_name,
                session.tournament_id,
                session.match_id,
                session.match_number,
                session.player1_name,
                session.player1_flag,
                session.player2_name,
                session.player2_flag,
                session.recording_path,
                session.recording_filename,
                session.recording_start_time.map(|dt| dt.to_rfc3339()),
                session.recording_end_time.map(|dt| dt.to_rfc3339()),
                session.recording_duration,
                session.recording_size_bytes,
                session.replay_buffer_start_time.map(|dt| dt.to_rfc3339()),
                session.replay_buffer_end_time.map(|dt| dt.to_rfc3339()),
                session.replay_buffer_saved,
                session.replay_buffer_filename.clone(),
                session.status,
                session.error_message.clone(),
                Utc::now().to_rfc3339(),
                session_id,
            ],
        )?;

        Ok(())
    }

    /// Get recording session by ID
    pub fn get_recording_session(
        conn: &Connection,
        session_id: i64,
    ) -> DatabaseResult<Option<ObsRecordingSession>> {
        let mut stmt = conn.prepare("SELECT * FROM obs_recording_session WHERE id = ?")?;

        let session = stmt
            .query_row([session_id], ObsRecordingSession::from_row)
            .optional()?;

        Ok(session)
    }

    /// Update recording session status
    pub fn update_recording_session_status(
        conn: &mut Connection,
        session_id: i64,
        status: &str,
        error_message: Option<&str>,
    ) -> DatabaseResult<()> {
        conn.execute(
            "UPDATE obs_recording_session SET status = ?, error_message = ?, updated_at = ? WHERE id = ?",
            params![
                status,
                error_message,
                Utc::now().to_rfc3339(),
                session_id,
            ],
        )?;

        Ok(())
    }

    /// Start a recording session (set start time and status to recording)
    pub fn start_recording_session(conn: &mut Connection, session_id: i64) -> DatabaseResult<()> {
        let now = Utc::now();
        conn.execute(
            "UPDATE obs_recording_session SET recording_start_time = ?, status = ?, updated_at = ? WHERE id = ?",
            params![
                now.to_rfc3339(),
                "recording",
                Utc::now().to_rfc3339(),
                session_id,
            ],
        )?;

        log::info!("Started recording session {session_id} at {now}");
        Ok(())
    }

    /// Stop a recording session (set end time, calculate duration, and update status)
    pub fn stop_recording_session(
        conn: &mut Connection,
        session_id: i64,
        status: &str,
    ) -> DatabaseResult<()> {
        let now = Utc::now();

        // First get the start time to calculate duration
        let start_time: Option<String> = conn
            .query_row(
                "SELECT recording_start_time FROM obs_recording_session WHERE id = ?",
                [session_id],
                |row| row.get(0),
            )
            .optional()?;

        let duration_seconds = if let Some(start_time_str) = start_time {
            if let Ok(start_time) = DateTime::parse_from_rfc3339(&start_time_str) {
                let start_utc = start_time.with_timezone(&Utc);
                let duration = now.signed_duration_since(start_utc);
                duration.num_seconds() as i32
            } else {
                0
            }
        } else {
            0
        };

        conn.execute(
            "UPDATE obs_recording_session SET recording_end_time = ?, recording_duration = ?, status = ?, updated_at = ? WHERE id = ?",
            params![
                now.to_rfc3339(),
                duration_seconds,
                status,
                Utc::now().to_rfc3339(),
                session_id,
            ],
        )?;

        log::info!(
            "Stopped recording session {session_id} at {now} (duration: {duration_seconds}s)"
        );
        Ok(())
    }

    /// Get recent recording sessions
    pub fn get_recent_recording_sessions(
        conn: &Connection,
        limit: i64,
    ) -> DatabaseResult<Vec<ObsRecordingSession>> {
        let mut stmt =
            conn.prepare("SELECT * FROM obs_recording_session ORDER BY created_at DESC LIMIT ?")?;

        let sessions = stmt
            .query_map([limit], ObsRecordingSession::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }
}

// OVR Operations for external providers and scraped data
pub struct OvrOperations;

impl OvrOperations {
    // Providers
    pub fn ensure_default_providers(conn: &mut Connection) -> DatabaseResult<()> {
        let now = Utc::now().to_rfc3339();
        conn.execute(
			"INSERT OR IGNORE INTO ovr_providers(name, base_url, enabled, created_at, updated_at) VALUES(?, ?, 1, ?, ?)",
			params!["simplycompete", "https://www.simplycompete.com", &now, &now],
		)?;
        conn.execute(
			"INSERT OR IGNORE INTO ovr_providers(name, base_url, enabled, created_at, updated_at) VALUES(?, ?, 1, ?, ?)",
			params!["tpss", "https://www.tpss.eu", &now, &now],
		)?;
        conn.execute(
			"INSERT OR IGNORE INTO ovr_providers(name, base_url, enabled, created_at, updated_at) VALUES(?, ?, 1, ?, ?)",
			params!["martial.events", "https://martial.events", &now, &now],
		)?;
        conn.execute(
			"INSERT OR IGNORE INTO ovr_providers(name, base_url, enabled, created_at, updated_at) VALUES(?, ?, 1, ?, ?)",
			params!["etu", "https://europetaekwondo.org", &now, &now],
		)?;
        Ok(())
    }

    pub fn get_providers(conn: &Connection) -> DatabaseResult<Vec<OvrProvider>> {
        let mut stmt = conn.prepare("SELECT * FROM ovr_providers ORDER BY name")?;
        let res = stmt
            .query_map([], OvrProvider::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(res)
    }

    pub fn upsert_provider(conn: &mut Connection, p: &OvrProvider) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        let id_opt: Option<i64> = tx
            .query_row(
                "SELECT id FROM ovr_providers WHERE name = ?",
                params![p.name],
                |r| r.get(0),
            )
            .optional()?;
        let now = Utc::now().to_rfc3339();
        let id = if let Some(id) = id_opt {
            tx.execute(
				"UPDATE ovr_providers SET base_url = ?, enabled = ?, rate_limit_ms = ?, updated_at = ? WHERE id = ?",
				params![p.base_url, p.enabled, p.rate_limit_ms, now, id]
			)?;
            id
        } else {
            tx.execute(
            "INSERT INTO ovr_providers (name, base_url, enabled, rate_limit_ms, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            params![p.name, p.base_url, p.enabled, p.rate_limit_ms, now, now]
        )?;
            tx.last_insert_rowid()
        };
        tx.commit()?;
        Ok(id)
    }

    pub fn remove_provider(conn: &mut Connection, id: i64) -> DatabaseResult<()> {
        conn.execute("DELETE FROM ovr_providers WHERE id = ?", params![id])?;
        Ok(())
    }

    pub fn clear_all_tournaments(conn: &mut Connection) -> DatabaseResult<()> {
        conn.execute("DELETE FROM ovr_categories", [])?;
        conn.execute("DELETE FROM ovr_to_local_tournament", [])?;
        conn.execute("DELETE FROM ovr_tournaments", [])?;
        Ok(())
    }

    pub fn set_provider_refresh_status(
        conn: &mut Connection,
        id: i64,
        status: Option<&str>,
        err: Option<&str>,
    ) -> DatabaseResult<()> {
        conn.execute(
			"UPDATE ovr_providers SET last_refreshed_at = ?, last_status = ?, last_error = ?, updated_at = ? WHERE id = ?",
			params![
				Some(Utc::now().to_rfc3339()),
				status,
				err,
				Utc::now().to_rfc3339(),
				id
			]
		)?;
        Ok(())
    }

    // Tournaments
    pub fn upsert_tournament(conn: &mut Connection, t: &OvrTournament) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        // First, try exact provider match
        let id_opt: Option<i64> = tx.query_row(
			"SELECT id FROM ovr_tournaments WHERE provider_id = ? AND provider_tournament_id = ?",
			params![t.provider_id, t.provider_tournament_id],
			|r| r.get(0)
		).optional()?;
        // If not found, attempt cross-provider dedupe by hash
        let cross_id_opt: Option<i64> = if id_opt.is_none() {
            if let Some(ref h) = t.hash {
                tx.query_row(
                    "SELECT id FROM ovr_tournaments WHERE hash = ?",
                    params![h],
                    |r| r.get(0),
                )
                .optional()?
            } else {
                None
            }
        } else {
            None
        };
        let now = Utc::now().to_rfc3339();
        let id = if let Some(id) = id_opt.or(cross_id_opt) {
            tx.execute(
				"UPDATE ovr_tournaments SET name = ?, start_date = ?, end_date = ?, city = ?, country = ?, url = ?, status = ?, last_seen_at = ?, hash = ?, etag = ?, updated_at = ? WHERE id = ?",
				params![
					t.name,
					t.start_date.map(|d| d.to_rfc3339()),
					t.end_date.map(|d| d.to_rfc3339()),
					t.city,
					t.country,
					t.url,
					t.status,
					Some(now.clone()),
					t.hash,
					t.etag,
					now,
					id
				]
			)?;
            id
        } else {
            tx.execute(
            "INSERT INTO ovr_tournaments (provider_id, provider_tournament_id, name, start_date, end_date, city, country, url, status, last_seen_at, hash, etag, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                t.provider_id,
                t.provider_tournament_id,
                t.name,
                t.start_date.map(|d| d.to_rfc3339()),
                t.end_date.map(|d| d.to_rfc3339()),
                t.city,
                t.country,
                t.url,
                t.status,
                Some(now.clone()),
                t.hash,
                t.etag,
                now,
                now
            ]
        )?;
            tx.last_insert_rowid()
        };
        tx.commit()?;
        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn list_tournaments(
        conn: &Connection,
        provider_id: Option<i64>,
        q: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
        country: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> DatabaseResult<Vec<OvrTournament>> {
        let mut sql = String::from("SELECT * FROM ovr_tournaments WHERE 1=1");
        let mut args: Vec<rusqlite::types::Value> = Vec::new();
        if let Some(pid) = provider_id {
            sql.push_str(" AND provider_id = ?");
            args.push(rusqlite::types::Value::from(pid));
        }
        if let Some(qq) = q {
            sql.push_str(" AND name LIKE ?");
            args.push(rusqlite::types::Value::from(format!("%{qq}%")));
        }
        if let Some(f) = from {
            sql.push_str(" AND start_date >= ?");
            args.push(rusqlite::types::Value::from(String::from(f)));
        }
        if let Some(t_) = to {
            sql.push_str(" AND end_date <= ?");
            args.push(rusqlite::types::Value::from(String::from(t_)));
        }
        if let Some(cty) = country {
            sql.push_str(" AND country = ?");
            args.push(rusqlite::types::Value::from(String::from(cty)));
        }
        sql.push_str(" ORDER BY start_date DESC, created_at DESC");
        if let Some(lim) = limit {
            sql.push_str(" LIMIT ?");
            args.push(rusqlite::types::Value::from(lim));
        }
        if let Some(off) = offset {
            sql.push_str(" OFFSET ?");
            args.push(rusqlite::types::Value::from(off));
        }
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(args.iter()), |row| {
            OvrTournament::from_row(row)
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    // Categories
    pub fn replace_categories(
        conn: &mut Connection,
        tournament_id: i64,
        categories: &[OvrCategory],
    ) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM ovr_categories WHERE tournament_id = ?",
            params![tournament_id],
        )?;
        for c in categories {
            tx.execute(
            "INSERT INTO ovr_categories (tournament_id, discipline, age_group, gender, division, weight_class, bracket_stage, provider_raw, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                tournament_id,
                c.discipline,
                c.age_group,
                c.gender,
                c.division,
                c.weight_class,
                c.bracket_stage,
                c.provider_raw,
                Utc::now().to_rfc3339(),
                Utc::now().to_rfc3339(),
            ]
        )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_categories(
        conn: &Connection,
        tournament_id: i64,
    ) -> DatabaseResult<Vec<OvrCategory>> {
        let mut stmt =
            conn.prepare("SELECT * FROM ovr_categories WHERE tournament_id = ? ORDER BY id")?;
        let res = stmt
            .query_map(params![tournament_id], OvrCategory::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(res)
    }

    // Promotion bridge: adopt OVR tournament into local curated tournament
    pub fn promote_to_local(
        conn: &mut Connection,
        ovr_tournament_id: i64,
        local_name: Option<&str>,
    ) -> DatabaseResult<i64> {
        let tx = conn.transaction()?;
        let ovr: OvrTournament = tx.query_row(
            "SELECT * FROM ovr_tournaments WHERE id = ?",
            params![ovr_tournament_id],
            OvrTournament::from_row,
        )?;
        let t_name = local_name.unwrap_or(&ovr.name);
        // Insert directly within this transaction to avoid connection borrowing issues
        let local_id = {
            tx.execute(
            "INSERT INTO tournaments (name, duration_days, city, country, status, start_date, end_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                t_name,
                1,
                overv(ovr.city),
                overv(ovr.country),
                "pending",
                overv_dt(ovr.start_date),
                overv_dt(ovr.end_date),
                Utc::now().to_rfc3339(),
                Utc::now().to_rfc3339()
            ]
        )?;
            tx.last_insert_rowid()
        };
        // Bridge
        tx.execute(
			"INSERT OR REPLACE INTO ovr_to_local_tournament (ovr_tournament_id, local_tournament_id, created_at) VALUES (?, ?, ?)",
			params![ovr_tournament_id, local_id, Utc::now().to_rfc3339()]
		)?;
        tx.commit()?;
        Ok(local_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyDivisionOption {
    pub name: String,
    pub category: Option<String>,
    pub gender: Option<String>,
    pub weight_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedalCeremonyAthleteOption {
    pub id: i64,
    pub full_name: String,
    pub short_name: Option<String>,
    pub country_code: Option<String>,
    pub ioc_code: Option<String>,
    pub athlete_code: Option<String>,
}

pub struct MedalCeremonyOperations;

impl MedalCeremonyOperations {
    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    pub fn list_division_options(
        conn: &Connection,
    ) -> DatabaseResult<Vec<MedalCeremonyDivisionOption>> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT name, category, gender, weight_class
             FROM (
                SELECT
                    TRIM(division) AS name,
                    TRIM(category) AS category,
                    NULL AS gender,
                    TRIM(weight_class) AS weight_class
                FROM pss_matches
                WHERE division IS NOT NULL AND TRIM(division) <> ''
                UNION
                SELECT
                    TRIM(division) AS name,
                    TRIM(age_group) AS category,
                    TRIM(gender) AS gender,
                    TRIM(weight_class) AS weight_class
                FROM ovr_categories
                WHERE division IS NOT NULL AND TRIM(division) <> ''
             )
             WHERE name IS NOT NULL AND name <> ''
             ORDER BY LOWER(name), LOWER(IFNULL(category,'')), LOWER(IFNULL(gender,'')), LOWER(IFNULL(weight_class,''))",
        )?;
        let options = stmt
            .query_map([], |row| {
                Ok(MedalCeremonyDivisionOption {
                    name: row.get::<_, String>("name")?,
                    category: row.get("category")?,
                    gender: row.get("gender")?,
                    weight_class: row.get("weight_class")?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(options)
    }

    pub fn list_athlete_options(
        conn: &Connection,
        division_name: &str,
    ) -> DatabaseResult<Vec<MedalCeremonyAthleteOption>> {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT
                a.id AS athlete_id,
                COALESCE(NULLIF(TRIM(a.long_name), ''), TRIM(a.short_name)) AS full_name,
                a.short_name,
                a.country_code,
                a.country_code AS ioc_code,
                a.athlete_code
             FROM pss_match_athletes ma
             JOIN pss_matches m ON ma.match_id = m.id
             JOIN pss_athletes a ON ma.athlete_id = a.id
             WHERE m.division IS NOT NULL
               AND TRIM(m.division) <> ''
               AND LOWER(TRIM(m.division)) = LOWER(TRIM(?1))
             ORDER BY LOWER(full_name), LOWER(IFNULL(a.short_name,''))",
        )?;
        let options = stmt
            .query_map([division_name], |row| {
                Ok(MedalCeremonyAthleteOption {
                    id: row.get("athlete_id")?,
                    full_name: row
                        .get::<_, Option<String>>("full_name")?
                        .unwrap_or_else(|| "Unknown athlete".to_string()),
                    short_name: row.get("short_name")?,
                    country_code: row.get("country_code")?,
                    ioc_code: row.get("ioc_code")?,
                    athlete_code: row.get("athlete_code")?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(options)
    }

    fn fetch_divisions_with_medalists(
        conn: &Connection,
        ceremony_id: &str,
    ) -> DatabaseResult<Vec<MedalCeremonyDivisionDetail>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM medal_ceremony_divisions WHERE ceremony_id = ?1 ORDER BY order_index ASC, created_at ASC",
        )?;
        let divisions = stmt
            .query_map([ceremony_id], MedalCeremonyDivision::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        let mut details = Vec::with_capacity(divisions.len());
        let mut medal_stmt = conn.prepare(
            "SELECT * FROM medal_ceremony_medalists WHERE division_entry_id = ?1 ORDER BY medal_rank ASC",
        )?;

        for division in divisions {
            let medalists = medal_stmt
                .query_map([division.id.as_str()], |row| {
                    MedalCeremonyMedalist::from_row(row)
                })?
                .collect::<Result<Vec<_>, _>>()?;

            details.push(MedalCeremonyDivisionDetail {
                division,
                medalists,
            });
        }

        Ok(details)
    }

    pub fn list_ceremonies(conn: &Connection) -> DatabaseResult<Vec<MedalCeremony>> {
        let mut stmt = conn.prepare(
            "SELECT * FROM medal_ceremonies ORDER BY datetime(created_at) DESC, name ASC",
        )?;
        let items = stmt
            .query_map([], MedalCeremony::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    pub fn get_ceremony_detail(
        conn: &Connection,
        ceremony_id: &str,
    ) -> DatabaseResult<Option<MedalCeremonyDetail>> {
        let mut stmt = conn.prepare("SELECT * FROM medal_ceremonies WHERE id = ?1 LIMIT 1")?;
        let ceremony = stmt
            .query_row([ceremony_id], MedalCeremony::from_row)
            .optional()?;

        if let Some(ceremony) = ceremony {
            let divisions = Self::fetch_divisions_with_medalists(conn, &ceremony.id)?;
            Ok(Some(MedalCeremonyDetail {
                ceremony,
                divisions,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn upsert_ceremony(
        conn: &mut Connection,
        payload: &MedalCeremonyDetail,
    ) -> DatabaseResult<String> {
        let tx = conn.transaction()?;
        let now = Self::now();
        let ceremony = payload.ceremony.clone();
        let ceremony_id = if ceremony.id.trim().is_empty() {
            Uuid::new_v4().to_string()
        } else {
            ceremony.id.clone()
        };

        let show_external = if ceremony.show_external { 1 } else { 0 };
        let is_new = payload.ceremony.id.trim().is_empty();

        if is_new {
            tx.execute(
                "INSERT INTO medal_ceremonies (
                    id, tournament_id, name, background_path, break_path,
                    animation_duration, animation_speed, photo_time,
                    prepared_at, prepared_version, show_external,
                    created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10, ?11, ?12)",
                params![
                    ceremony_id,
                    ceremony.tournament_id,
                    ceremony.name,
                    ceremony.background_path,
                    ceremony.break_path,
                    ceremony.animation_duration,
                    ceremony.animation_speed,
                    ceremony.photo_time,
                    ceremony.prepared_version,
                    show_external,
                    now,
                    now
                ],
            )?;
        } else {
            tx.execute(
                "UPDATE medal_ceremonies SET
                    tournament_id = ?1,
                    name = ?2,
                    background_path = ?3,
                    break_path = ?4,
                    animation_duration = ?5,
                    animation_speed = ?6,
                    photo_time = ?7,
                    prepared_at = NULL,
                    prepared_version = ?8,
                    show_external = ?9,
                    updated_at = ?10
                 WHERE id = ?11",
                params![
                    ceremony.tournament_id,
                    ceremony.name,
                    ceremony.background_path,
                    ceremony.break_path,
                    ceremony.animation_duration,
                    ceremony.animation_speed,
                    ceremony.photo_time,
                    ceremony.prepared_version,
                    show_external,
                    now,
                    ceremony_id
                ],
            )?;
        }

        tx.execute(
            "DELETE FROM medal_ceremony_medalists
             WHERE division_entry_id IN (
                SELECT id FROM medal_ceremony_divisions WHERE ceremony_id = ?1
             )",
            params![ceremony_id],
        )?;
        tx.execute(
            "DELETE FROM medal_ceremony_divisions WHERE ceremony_id = ?1",
            params![ceremony_id],
        )?;

        for (idx, division_detail) in payload.divisions.iter().enumerate() {
            let mut division = division_detail.division.clone();
            if division.id.trim().is_empty() {
                division.id = Uuid::new_v4().to_string();
            }
            let order_index = if division.order_index > 0 {
                division.order_index
            } else {
                (idx + 1) as i64
            };

            tx.execute(
                "INSERT INTO medal_ceremony_divisions (
                    id, ceremony_id, division_id, division_name,
                    order_index, played_at, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    division.id,
                    ceremony_id,
                    division.division_id,
                    division.division_name,
                    order_index,
                    overv_dt(division.played_at),
                    now,
                    now
                ],
            )?;

            for medalist in &division_detail.medalists {
                let mut record = medalist.clone();
                if record.id.trim().is_empty() {
                    record.id = Uuid::new_v4().to_string();
                }
                tx.execute(
                    "INSERT INTO medal_ceremony_medalists (
                        id, division_entry_id, medal_type, medal_rank,
                        athlete_id, athlete_name, athlete_short_name,
                        ioc_code, flag_asset, anthem_asset,
                        created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        record.id,
                        division.id,
                        record.medal_type,
                        record.medal_rank,
                        record.athlete_id,
                        record.athlete_name,
                        record.athlete_short_name,
                        record.ioc_code,
                        record.flag_asset,
                        record.anthem_asset,
                        now,
                        now
                    ],
                )?;
            }
        }

        tx.commit()?;
        Ok(ceremony_id)
    }

    pub fn delete_ceremony(conn: &mut Connection, ceremony_id: &str) -> DatabaseResult<()> {
        conn.execute(
            "DELETE FROM medal_ceremonies WHERE id = ?1",
            params![ceremony_id],
        )?;
        Ok(())
    }

    pub fn prepare_playlist(
        conn: &mut Connection,
        ceremony_id: &str,
    ) -> DatabaseResult<Vec<MedalCeremonyDivisionDetail>> {
        let tx = conn.transaction()?;
        let now = Self::now();
        let current_version: Option<i64> = tx
            .query_row(
                "SELECT prepared_version FROM medal_ceremonies WHERE id = ?1",
                params![ceremony_id],
                |row| row.get(0),
            )
            .optional()?;

        let version = current_version.ok_or_else(|| {
            DatabaseError::Config(format!("Medal ceremony {ceremony_id} not found"))
        })?;

        tx.execute(
            "UPDATE medal_ceremonies
             SET prepared_at = ?1, prepared_version = ?2, updated_at = ?1
             WHERE id = ?3",
            params![now, version + 1, ceremony_id],
        )?;
        tx.commit()?;

        let detail = Self::get_ceremony_detail(conn, ceremony_id)?;
        Ok(detail.map(|d| d.divisions).unwrap_or_default())
    }

    pub fn mark_division_played(conn: &mut Connection, division_id: &str) -> DatabaseResult<()> {
        let tx = conn.transaction()?;
        let now = Self::now();
        let ceremony_id: Option<String> = tx
            .query_row(
                "SELECT ceremony_id FROM medal_ceremony_divisions WHERE id = ?1",
                params![division_id],
                |row| row.get(0),
            )
            .optional()?;

        let ceremony_id = ceremony_id.ok_or_else(|| {
            DatabaseError::Config(format!("Medal ceremony division {division_id} not found"))
        })?;

        tx.execute(
            "UPDATE medal_ceremony_divisions SET played_at = ?1, updated_at = ?1 WHERE id = ?2",
            params![now, division_id],
        )?;
        tx.execute(
            "UPDATE medal_ceremonies SET updated_at = ?1 WHERE id = ?2",
            params![now, ceremony_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn reset_playback(conn: &mut Connection, ceremony_id: &str) -> DatabaseResult<()> {
        let now = Self::now();
        conn.execute(
            "UPDATE medal_ceremony_divisions SET played_at = NULL, updated_at = ?1 WHERE ceremony_id = ?2",
            params![now, ceremony_id],
        )?;
        conn.execute(
            "UPDATE medal_ceremonies SET prepared_at = NULL, updated_at = ?1 WHERE id = ?2",
            params![Self::now(), ceremony_id],
        )?;
        Ok(())
    }

    pub fn update_show_external(
        conn: &mut Connection,
        ceremony_id: &str,
        enabled: bool,
    ) -> DatabaseResult<()> {
        let now = Self::now();
        conn.execute(
            "UPDATE medal_ceremonies SET show_external = ?1, updated_at = ?2 WHERE id = ?3",
            params![if enabled { 1 } else { 0 }, now, ceremony_id],
        )?;
        Ok(())
    }

    pub fn list_flag_animations(conn: &Connection) -> DatabaseResult<Vec<OvrFlagAnimationAsset>> {
        let mut stmt = conn.prepare(
            "SELECT
                id,
                ioc_code,
                file_name,
                file_path,
                COALESCE(display_name, country_name) AS display_name,
                duration_ms,
                is_default,
                created_at,
                updated_at
             FROM animations
             ORDER BY ioc_code ASC, file_name ASC",
        )?;
        let assets = stmt
            .query_map([], OvrFlagAnimationAsset::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(assets)
    }

    pub fn upsert_flag_animation(
        conn: &mut Connection,
        asset: &OvrFlagAnimationAsset,
    ) -> DatabaseResult<String> {
        let tx = conn.transaction()?;
        let now = Self::now();
        let asset_id = if asset.id.trim().is_empty() {
            Uuid::new_v4().to_string()
        } else {
            asset.id.clone()
        };

        let exists: Option<String> = tx
            .query_row(
                "SELECT id FROM animations WHERE id = ?1",
                params![asset_id.as_str()],
                |row| row.get(0),
            )
            .optional()?;

        if exists.is_some() {
            tx.execute(
                "UPDATE animations
                 SET ioc_code = ?1,
                     country_name = ?2,
                     file_name = ?3,
                     file_path = ?4,
                     filename = ?5,
                     display_name = ?6,
                     duration_ms = ?7,
                     is_default = ?8,
                     last_modified = ?9,
                     updated_at = ?9
                 WHERE id = ?10",
                params![
                    asset.ioc_code,
                    asset.display_name,
                    asset.file_name,
                    asset.file_path,
                    asset.file_name,
                    asset.display_name,
                    asset.duration_ms,
                    if asset.is_default { 1 } else { 0 },
                    now,
                    asset_id,
                ],
            )?;
        } else {
            tx.execute(
                "INSERT INTO animations (
                    id,
                    ioc_code,
                    country_name,
                    file_name,
                    file_path,
                    filename,
                    display_name,
                    duration_ms,
                    is_default,
                    upload_date,
                    last_modified,
                    created_at,
                    updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, ?10, ?10)",
                params![
                    asset_id,
                    asset.ioc_code,
                    asset.display_name,
                    asset.file_name,
                    asset.file_path,
                    asset.file_name,
                    asset.display_name,
                    asset.duration_ms,
                    if asset.is_default { 1 } else { 0 },
                    now.clone(),
                    now.clone(),
                    now.clone(),
                    now.clone()
                ],
            )?;
        }

        if asset.is_default {
            tx.execute(
                "UPDATE animations
                 SET is_default = 0, updated_at = ?1
                 WHERE ioc_code = ?2 AND id != ?3",
                params![Self::now(), asset.ioc_code, asset_id],
            )?;
        }

        tx.commit()?;
        Ok(asset_id)
    }

    pub fn delete_flag_animation(conn: &mut Connection, asset_id: &str) -> DatabaseResult<()> {
        conn.execute("DELETE FROM animations WHERE id = ?1", params![asset_id])?;
        Ok(())
    }

    pub fn list_anthems(conn: &Connection) -> DatabaseResult<Vec<OvrAnthemAsset>> {
        let mut stmt =
            conn.prepare("SELECT * FROM ovr_anthems ORDER BY ioc_code ASC, file_name ASC")?;
        let assets = stmt
            .query_map([], OvrAnthemAsset::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(assets)
    }

    pub fn upsert_anthem(conn: &mut Connection, asset: &OvrAnthemAsset) -> DatabaseResult<String> {
        let tx = conn.transaction()?;
        let now = Self::now();
        let asset_id = if asset.id.trim().is_empty() {
            Uuid::new_v4().to_string()
        } else {
            asset.id.clone()
        };

        let exists: Option<String> = tx
            .query_row(
                "SELECT id FROM ovr_anthems WHERE id = ?1",
                params![asset_id.as_str()],
                |row| row.get(0),
            )
            .optional()?;

        if exists.is_some() {
            tx.execute(
                "UPDATE ovr_anthems
                 SET ioc_code = ?1, file_name = ?2, file_path = ?3,
                     display_name = ?4, duration_ms = ?5, is_default = ?6,
                     updated_at = ?7
                 WHERE id = ?8",
                params![
                    asset.ioc_code,
                    asset.file_name,
                    asset.file_path,
                    asset.display_name,
                    asset.duration_ms,
                    if asset.is_default { 1 } else { 0 },
                    now,
                    asset_id
                ],
            )?;
        } else {
            tx.execute(
                "INSERT INTO ovr_anthems (
                    id, ioc_code, file_name, file_path, display_name,
                    duration_ms, is_default, created_at, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    asset_id,
                    asset.ioc_code,
                    asset.file_name,
                    asset.file_path,
                    asset.display_name,
                    asset.duration_ms,
                    if asset.is_default { 1 } else { 0 },
                    now.clone(),
                    now.clone()
                ],
            )?;
        }

        if asset.is_default {
            tx.execute(
                "UPDATE ovr_anthems
                 SET is_default = 0, updated_at = ?1
                 WHERE ioc_code = ?2 AND id != ?3",
                params![Self::now(), asset.ioc_code, asset_id],
            )?;
        }

        tx.commit()?;
        Ok(asset_id)
    }

    pub fn delete_anthem(conn: &mut Connection, asset_id: &str) -> DatabaseResult<()> {
        conn.execute("DELETE FROM ovr_anthems WHERE id = ?1", params![asset_id])?;
        Ok(())
    }
}

// Small helpers for optional values
fn overv<T: ToString>(v: Option<T>) -> Option<String> {
    v.map(|x| x.to_string())
}
fn overv_dt(v: Option<chrono::DateTime<chrono::Utc>>) -> Option<String> {
    v.map(|d| d.to_rfc3339())
}
