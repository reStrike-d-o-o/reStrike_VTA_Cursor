#!/usr/bin/env python3
"""
Reconstructs PSS event timelines from Daedo GO2025 archives and stores them in
pss_events / pss_event_details for the seeded tournament.

This enables deterministic replay by preserving the original event ordering,
timestamps, and inferred WT UDP packet mappings.

Usage
-----
Inspect only:
    python scripts/tournament/event_loader.py inspect \
        --archive-root "C:/Users/Damjan/Documents/Daedo log files GO2025" \
        --db-path "src-tauri/restrike_vta.db" \
        --tournament-name "German Open - Hamburg 2025"

Apply ingestion:
    python scripts/tournament/event_loader.py apply \
        --archive-root "C:/Users/Damjan/Documents/Daedo log files GO2025" \
        --db-path "src-tauri/restrike_vta.db" \
        --tournament-name "German Open - Hamburg 2025"
"""

from __future__ import annotations

import argparse
import json
import logging
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, Iterable, Optional, Sequence, Tuple

# Ensure repository root on sys.path for sibling imports
REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tournament.match_loader import MatchRecord, load_matches  # type: ignore  # pylint: disable=wrong-import-position


LOGGER = logging.getLogger("event_loader")

REPLAY_SESSION_LABEL = "replay_seed"
DEFAULT_SERVER_CONFIG_ID = 1
DEFAULT_RECOGNITION_STATUS = "reconstructed"
DEFAULT_PROTOCOL_VERSION = "2.3"


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def _normalize_timestamp(event: Dict[str, any]) -> Optional[str]:
    iso = event.get("event_time_iso")
    if iso:
        return iso
    millis = event.get("event_time_ms")
    if isinstance(millis, int):
        return datetime.fromtimestamp(millis / 1000.0, tz=timezone.utc).isoformat()
    return None


def _ensure_session(conn: sqlite3.Connection) -> int:
    row = conn.execute(
        "SELECT id FROM udp_server_sessions WHERE status = ? LIMIT 1",
        (REPLAY_SESSION_LABEL,),
    ).fetchone()
    if row:
        return int(row[0])

    start = _utc_now_iso()
    conn.execute(
        """
        INSERT INTO udp_server_sessions (
            server_config_id, start_time, end_time, status,
            packets_received, packets_parsed, parse_errors,
            total_bytes_received, average_packet_size, max_packet_size_seen,
            min_packet_size_seen, unique_clients_count, error_message, created, updated
        ) VALUES (?, ?, ?, ?, 0, 0, 0, 0, 0.0, 0, 0, 0, NULL, strftime('%s','now'), strftime('%s','now'))
        """,
        (DEFAULT_SERVER_CONFIG_ID, start, start, REPLAY_SESSION_LABEL),
    )
    session_id = conn.execute(
        "SELECT id FROM udp_server_sessions WHERE status = ? ORDER BY id DESC LIMIT 1",
        (REPLAY_SESSION_LABEL,),
    ).fetchone()[0]
    LOGGER.info("Created UDP replay session id=%s", session_id)
    return int(session_id)


def _ensure_event_type(conn: sqlite3.Connection, code: str) -> int:
    row = conn.execute("SELECT id FROM pss_event_types WHERE event_code = ?", (code,)).fetchone()
    if row:
        return int(row[0])
    conn.execute(
        """
        INSERT INTO pss_event_types (
            event_code, event_name, description, category, is_active, created_at, created
        ) VALUES (?, ?, ?, ?, 1, ?, 0)
        """,
        (code, code.replace("_", " ").title(), "Reconstructed from Daedo GO2025 logs", "replay", _utc_now_iso()),
    )
    event_type_id = conn.execute(
        "SELECT id FROM pss_event_types WHERE event_code = ?",
        (code,),
    ).fetchone()[0]
    return int(event_type_id)


def _fetch_match_lookup(conn: sqlite3.Connection, tournament_name: str) -> Dict[str, Dict[str, any]]:
    lookup: Dict[str, Dict[str, any]] = {}
    rows = conn.execute(
        """
        SELECT pm.id, pm.uuid, pm.match_id, pm.tournament_id
        FROM pss_matches pm
        JOIN tournaments t ON pm.tournament_id = CAST(t.id AS TEXT)
        WHERE t.name = ?
        """,
        (tournament_name,),
    ).fetchall()
    for match_id, uuid_value, external_id, tournament_id in rows:
        lookup[external_id] = {
            "match_db_id": int(match_id),
            "match_uuid": uuid_value,
            "tournament_id": tournament_id,
        }
    return lookup


def _fetch_round_lookup(conn: sqlite3.Connection, match_db_id: int) -> Dict[int, int]:
    round_map: Dict[int, int] = {}
    for round_id, round_number in conn.execute(
        "SELECT id, round_number FROM pss_rounds WHERE match_id = (SELECT uuid FROM pss_matches WHERE id = ?)",
        (match_db_id,),
    ):
        if round_number is not None:
            round_map[int(round_number)] = int(round_id)
    return round_map


def _delete_existing_events(conn: sqlite3.Connection, match_db_id: int) -> None:
    event_ids = [row[0] for row in conn.execute("SELECT id FROM pss_events WHERE match_id = ?", (match_db_id,))]
    if not event_ids:
        return
    conn.executemany("DELETE FROM pss_event_details WHERE event_id = ?", [(event_id,) for event_id in event_ids])
    conn.execute("DELETE FROM pss_events WHERE match_id = ?", (match_db_id,))


def _store_event_details(conn: sqlite3.Connection, event_id: int, event: Dict[str, any]) -> None:
    udp_info = event.get("udp")
    if udp_info:
        conn.execute(
            """
            INSERT INTO pss_event_details (
                event_id, detail_key, detail_value, detail_type, created_at, created
            ) VALUES (?, ?, ?, 'json', ?, 0)
            """,
            (event_id, "udp_mapping", json.dumps(udp_info, ensure_ascii=False), _utc_now_iso()),
        )


def _insert_event(
    conn: sqlite3.Connection,
    *,
    session_id: int,
    match_db_id: int,
    round_map: Dict[int, int],
    event: Dict[str, any],
    event_type_id: int,
    tournament_id: Optional[str],
) -> Optional[int]:
    timestamp = _normalize_timestamp(event)
    if not timestamp:
        LOGGER.warning("Skipping event sequence=%s due to missing timestamp", event.get("sequence"))
        return None

    round_number = event.get("round_number")
    round_id = round_map.get(int(round_number)) if isinstance(round_number, int) or (isinstance(round_number, str) and round_number.isdigit()) else None

    raw_json = json.dumps(event, ensure_ascii=False)
    conn.execute(
        """
        INSERT INTO pss_events (
            session_id, match_id, round_id, event_type_id, timestamp, raw_data,
            parsed_data, event_sequence, processing_time_ms, is_valid, error_message,
            recognition_status, protocol_version, parser_confidence, validation_errors,
            tournament_id, created_at, created
        ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, NULL, 1, NULL,
            ?, ?, ?, NULL,
            ?, ?, 0
        )
        """,
        (
            session_id,
            match_db_id,
            round_id,
            event_type_id,
            timestamp,
            raw_json,
            raw_json,
            event.get("sequence", 0),
            DEFAULT_RECOGNITION_STATUS,
            DEFAULT_PROTOCOL_VERSION,
            1.0,
            tournament_id,
            _utc_now_iso(),
        ),
    )
    return conn.execute("SELECT last_insert_rowid()").fetchone()[0]


def _ingest_match_events(
    conn: sqlite3.Connection,
    *,
    session_id: int,
    match_record: MatchRecord,
    match_context: Dict[str, any],
) -> Tuple[int, int]:
    match_db_id = match_context["match_db_id"]
    tournament_id = match_context["tournament_id"]
    round_map = _fetch_round_lookup(conn, match_db_id)
    _delete_existing_events(conn, match_db_id)

    inserted = 0
    skipped = 0

    for event in match_record.events:
        event_type_code = event.get("event_type") or event.get("matchLogItemType") or "UNKNOWN"
        event_type_id = _ensure_event_type(conn, event_type_code)
        event_id = _insert_event(
            conn,
            session_id=session_id,
            match_db_id=match_db_id,
            round_map=round_map,
            event=event,
            event_type_id=event_type_id,
            tournament_id=tournament_id,
        )
        if event_id is None:
            skipped += 1
            continue
        _store_event_details(conn, int(event_id), event)
        inserted += 1

    return inserted, skipped


def apply_events(
    *,
    conn: sqlite3.Connection,
    matches: Sequence[MatchRecord],
    tournament_name: str,
) -> Dict[str, int]:
    lookup = _fetch_match_lookup(conn, tournament_name)
    if not lookup:
        raise RuntimeError(f"No pss_matches found for tournament '{tournament_name}'. Run match_loader first.")
    session_id = _ensure_session(conn)

    total_inserted = 0
    total_skipped = 0
    missing_matches = 0

    for match in matches:
        context = lookup.get(match.match_id)
        if not context:
            LOGGER.warning("Match %s not present in DB; skipping events", match.match_id)
            missing_matches += 1
            continue
        inserted, skipped = _ingest_match_events(
            conn,
            session_id=session_id,
            match_record=match,
            match_context=context,
        )
        total_inserted += inserted
        total_skipped += skipped

    conn.commit()
    return {
        "matches_processed": len(matches) - missing_matches,
        "events_inserted": total_inserted,
        "events_skipped": total_skipped,
        "matches_missing": missing_matches,
    }


def cmd_inspect(args: argparse.Namespace) -> None:
    matches = load_matches(Path(args.archive_root).expanduser())
    event_counts = {m.match_id: len(m.events) for m in matches[:5]}
    print(
        {
            "matches_detected": len(matches),
            "sample_event_counts": event_counts,
        }
    )


def cmd_apply(args: argparse.Namespace) -> None:
    matches = load_matches(Path(args.archive_root).expanduser())
    conn = sqlite3.connect(args.db_path)
    try:
        conn.execute("PRAGMA foreign_keys = ON")
        stats = apply_events(conn=conn, matches=matches, tournament_name=args.tournament_name)
    finally:
        conn.close()
    LOGGER.info("Event ingestion complete: %s", stats)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log-level", default="INFO")
    subparsers = parser.add_subparsers(dest="command", required=True)

    inspect_parser = subparsers.add_parser("inspect", help="List match / event counts without DB writes.")
    inspect_parser.add_argument("--archive-root", required=True)
    inspect_parser.add_argument("--db-path", required=False)
    inspect_parser.add_argument("--tournament-name", required=False)
    inspect_parser.set_defaults(func=cmd_inspect)

    apply_parser = subparsers.add_parser("apply", help="Apply event ingestion to the database.")
    apply_parser.add_argument("--archive-root", required=True)
    apply_parser.add_argument("--db-path", required=True)
    apply_parser.add_argument("--tournament-name", required=True)
    apply_parser.set_defaults(func=cmd_apply)

    return parser


def main(argv: Optional[Iterable[str]] = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)
    logging.basicConfig(
        level=getattr(logging, str(args.log_level).upper(), logging.INFO),
        format="[%(asctime)s] %(levelname)s - %(message)s",
    )
    args.func(args)


if __name__ == "__main__":  # pragma: no cover
    main()

