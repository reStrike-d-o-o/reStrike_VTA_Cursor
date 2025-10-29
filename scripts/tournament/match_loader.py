#!/usr/bin/env python3
"""
Populates match-level tables (pss_matches, pss_match_athletes, pss_rounds,
pss_scores, pss_warnings) from Daedo GO2025 archives.

Workflow
--------
1. Parse canonical match data using the UDP-aware parser.
2. Resolve day/octagon/athlete references from the seeded scaffold.
3. Insert or update PSS tables with idempotent operations.

Usage examples
--------------
Inspect the archive against current DB (no writes):
    python scripts/tournament/match_loader.py inspect \
        --archive-root "C:/Users/Damjan/Documents/Daedo log files GO2025" \
        --db-path "src-tauri/restrike_vta.db" \
        --tournament-name "German Open - Hamburg 2025"

Apply the match ingestion:
    python scripts/tournament/match_loader.py apply \
        --archive-root "C:/Users/Damjan/Documents/Daedo log files GO2025" \
        --db-path "src-tauri/restrike_vta.db" \
        --tournament-name "German Open - Hamburg 2025"
"""

from __future__ import annotations

import argparse
import logging
import re
import sqlite3
import sys
import uuid
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, DefaultDict, Dict, Iterable, List, Optional, Sequence, Tuple

# Ensure repository root is on PYTHONPATH for intra-project imports
REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tournament import daedo_log_parser  # type: ignore  # pylint: disable=wrong-import-position


LOGGER = logging.getLogger("match_loader")

_FINAL_PATTERN = re.compile(r"\bFINAL(S)?\b")
_NON_CHAMPIONSHIP_PATTERN = re.compile(
    r"\b(SEMI|QUARTER|BRONZE|QUAL(?:IFIER)?|ELIMINATION)\s*FINAL(S)?\b"
)
_SEMIFINAL_PATTERN = re.compile(r"\bSEMI[-\s]?FINAL(S)?\b")
_BRONZE_PATTERN = re.compile(r"\bBRONZE\b")


def _utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


@dataclass
class RoundSnapshot:
    round_number: int
    start_iso: Optional[str]
    end_iso: Optional[str]
    duration_seconds: Optional[float]
    winner_position: int
    blue_score: Optional[int]
    red_score: Optional[int]
    blue_penalties: Optional[int]
    red_penalties: Optional[int]


@dataclass
class FinalSnapshot:
    blue_score: int
    red_score: int
    blue_penalties: int
    red_penalties: int
    timestamp_iso: Optional[str]


@dataclass
class MatchRecord:
    match_id: str
    day_folder: str
    court: str
    metadata: Dict[str, Any]
    config: Dict[str, Any]
    athletes: Dict[str, Dict[str, Optional[str]]]
    rounds: Dict[str, Any]
    events: Sequence[Dict[str, Any]]
    round_snapshots: List[RoundSnapshot]
    final_snapshot: FinalSnapshot


@dataclass
class MedalistEntry:
    category: Optional[str]
    match_uuid: str
    match_id: str
    corner_color: str
    athlete_name: Optional[str]
    athlete_country: Optional[str]
    blue_score: int
    red_score: int
    medal_type: str
    medal_rank: int


def _round_duration_seconds(config: Dict[str, Any]) -> Optional[int]:
    minutes = config.get("round_time", {}).get("minutes")
    seconds = config.get("round_time", {}).get("seconds")
    if minutes is None and seconds is None:
        return None
    minutes = minutes or 0
    seconds = seconds or 0
    return minutes * 60 + seconds


def _pull_round_winners(rounds_data: Dict[str, Any]) -> Dict[int, str]:
    winners: Dict[int, str] = {}
    inner = rounds_data.get("roundResults") or rounds_data.get("round_results")
    if isinstance(rounds_data, dict) and "roundResults" in rounds_data:
        inner = rounds_data["roundResults"]
    if isinstance(rounds_data, dict) and "roundResults" not in rounds_data and "round_results" in rounds_data:
        inner = rounds_data["round_results"]
    if not inner:
        return winners
    for entry in inner:
        try:
            winners[int(entry.get("round"))] = entry.get("roundWinner", "")
        except (TypeError, ValueError):
            continue
    return winners


def _position_for_winner(winner: str) -> int:
    if winner.upper() == "BLUE":
        return 1
    if winner.upper() == "RED":
        return 2
    return 0


def _extract_round_snapshots(
    events: Sequence[Dict[str, Any]],
    round_winners: Dict[int, str],
) -> Tuple[List[RoundSnapshot], FinalSnapshot]:
    round_data: Dict[int, Dict[str, Optional[float]]] = {}
    final_timestamp_iso: Optional[str] = None
    final_scores = {"blue": 0, "red": 0}
    final_penalties = {"blue": 0, "red": 0}

    for event in events:
        rnd = event.get("round_number")
        event_type = (event.get("event_type") or "").upper()
        event_time_iso = event.get("event_time_iso")
        event_time_ms = event.get("event_time_ms")

        score_snapshot = event.get("score_snapshot") or {}
        if isinstance(score_snapshot, dict):
            if "blue" in score_snapshot and score_snapshot["blue"] is not None:
                final_scores["blue"] = int(score_snapshot["blue"])
            if "red" in score_snapshot and score_snapshot["red"] is not None:
                final_scores["red"] = int(score_snapshot["red"])
        if event.get("blue_total_penalties") is not None:
            final_penalties["blue"] = int(event.get("blue_total_penalties") or 0)
        if event.get("red_total_penalties") is not None:
            final_penalties["red"] = int(event.get("red_total_penalties") or 0)
        if event_time_iso:
            final_timestamp_iso = event_time_iso

        if rnd is None:
            continue
        rnd = int(rnd)
        entry = round_data.setdefault(
            rnd,
            {
                "start_iso": None,
                "end_iso": None,
                "start_ms": None,
                "end_ms": None,
                "score_blue": None,
                "score_red": None,
                "pen_blue": None,
                "pen_red": None,
            },
        )

        if event_type == "START_ROUND" and entry["start_iso"] is None:
            entry["start_iso"] = event_time_iso
            entry["start_ms"] = event_time_ms
        elif event_type == "END_ROUND":
            entry["end_iso"] = event_time_iso
            entry["end_ms"] = event_time_ms

        if isinstance(score_snapshot, dict):
            if score_snapshot.get("blue") is not None:
                entry["score_blue"] = int(score_snapshot["blue"])
            if score_snapshot.get("red") is not None:
                entry["score_red"] = int(score_snapshot["red"])
        if event.get("blue_total_penalties") is not None:
            entry["pen_blue"] = int(event.get("blue_total_penalties") or 0)
        if event.get("red_total_penalties") is not None:
            entry["pen_red"] = int(event.get("red_total_penalties") or 0)

    snapshots: List[RoundSnapshot] = []
    for rnd, entry in sorted(round_data.items()):
        start_ms = entry.get("start_ms")
        end_ms = entry.get("end_ms")
        duration_seconds = (
            (end_ms - start_ms) / 1000.0 if isinstance(start_ms, (int, float)) and isinstance(end_ms, (int, float)) else None
        )
        winner_position = _position_for_winner(round_winners.get(rnd, ""))
        snapshots.append(
            RoundSnapshot(
                round_number=rnd,
                start_iso=entry.get("start_iso"),
                end_iso=entry.get("end_iso"),
                duration_seconds=duration_seconds,
                winner_position=winner_position,
                blue_score=entry.get("score_blue"),
                red_score=entry.get("score_red"),
                blue_penalties=entry.get("pen_blue"),
                red_penalties=entry.get("pen_red"),
            )
        )

    final_snapshot = FinalSnapshot(
        blue_score=final_scores["blue"],
        red_score=final_scores["red"],
        blue_penalties=final_penalties["blue"],
        red_penalties=final_penalties["red"],
        timestamp_iso=final_timestamp_iso,
    )
    return snapshots, final_snapshot


def load_matches(archive_root: Path) -> List[MatchRecord]:
    matches: List[MatchRecord] = []
    for match_log_path in sorted(archive_root.rglob("*-matchLog.csv")):
        if "-test" in match_log_path.name.lower():
            LOGGER.debug("Skipping test artefact %s", match_log_path.name)
            continue
        try:
            match_row = daedo_log_parser.parse_match_log(match_log_path)
            item_rows = daedo_log_parser.parse_match_log_items(
                match_log_path.with_name(match_log_path.name.replace("-matchLog.csv", "-matchLogItems.csv"))
            )
            canonical = daedo_log_parser._build_canonical_match(  # pylint: disable=protected-access
                match_row,
                item_rows,
                match_log_path,
            )
        except Exception as exc:  # pragma: no cover - diagnostics
            LOGGER.warning("Failed to parse %s: %s", match_log_path, exc)
            continue

        rounds_winners = canonical.get("rounds_winners") or {}
        round_winner_map = _pull_round_winners(rounds_winners)
        round_snapshots, final_snapshot = _extract_round_snapshots(canonical["events"], round_winner_map)

        matches.append(
            MatchRecord(
                match_id=canonical["match_id"],
                day_folder=canonical["source"]["day_folder"],
                court=canonical["source"]["court"],
                metadata=canonical["metadata"],
                config=canonical["configuration"],
                athletes=canonical["athletes"],
                rounds=canonical.get("rounds_winners", {}),
                events=canonical["events"],
                round_snapshots=round_snapshots,
                final_snapshot=final_snapshot,
            )
        )
    LOGGER.info("Loaded %d matches from archive", len(matches))
    return matches




def _ensure_champions_table(conn: sqlite3.Connection) -> None:
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS tournament_champions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tournament_uuid TEXT NOT NULL,
            category TEXT,
            match_uuid TEXT NOT NULL,
            match_id TEXT NOT NULL,
            winner_color TEXT NOT NULL,
            winner_name TEXT,
            winner_country_code TEXT,
            blue_score INTEGER NOT NULL,
            red_score INTEGER NOT NULL,
            medal_type TEXT NOT NULL DEFAULT 'gold',
            medal_rank INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        """
    )
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tournament_champions_tournament ON tournament_champions(tournament_uuid)"
    )
    existing_columns = {
        row[1] for row in conn.execute("PRAGMA table_info(tournament_champions)")
    }
    if "medal_type" not in existing_columns:
        conn.execute(
            "ALTER TABLE tournament_champions ADD COLUMN medal_type TEXT NOT NULL DEFAULT 'gold'"
        )
    if "medal_rank" not in existing_columns:
        conn.execute(
            "ALTER TABLE tournament_champions ADD COLUMN medal_rank INTEGER NOT NULL DEFAULT 1"
        )
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tournament_champions_category_rank "
        "ON tournament_champions(tournament_uuid, category, medal_rank)"
    )


def _pick_metadata_str(metadata: Dict[str, Any], *keys: str) -> Optional[str]:
    for key in keys:
        raw = metadata.get(key)
        if raw:
            value = str(raw).strip()
            if value:
                return value
    return None


def _normalize_phase(value: Optional[str]) -> str:
    if not value:
        return ""
    normalized = re.sub(r"[^A-Z0-9\s]", " ", value.upper())
    normalized = re.sub(r"\s+", " ", normalized).strip()
    return normalized


def _is_final_phase(value: Optional[str]) -> bool:
    normalized = _normalize_phase(value)
    if not normalized:
        return False
    if _NON_CHAMPIONSHIP_PATTERN.search(normalized):
        return False

    return bool(_FINAL_PATTERN.search(normalized))


def _is_semifinal_phase(value: Optional[str]) -> bool:
    normalized = _normalize_phase(value)
    if not normalized:
        return False
    return bool(_SEMIFINAL_PATTERN.search(normalized))


def _is_bronze_phase(value: Optional[str]) -> bool:
    normalized = _normalize_phase(value)
    if not normalized:
        return False
    if not _BRONZE_PATTERN.search(normalized):
        return False
    # Treat any explicit bronze finals/medal matches as bronze phases.
    if "FINAL" in normalized or "MEDAL" in normalized:
        return True
    # Fallback: keep bronze-only phases (e.g., "BRONZE MATCH").
    return True


def _resolve_winner_color(match: MatchRecord) -> str:
    winner_color = (
        _pick_metadata_str(
            match.metadata,
            "winner",
            "matchWinner",
            "match_winner",
            "matchWinnerColor",
        )
        or ""
    ).upper()

    blue_score = int(match.final_snapshot.blue_score)
    red_score = int(match.final_snapshot.red_score)

    if winner_color not in {"BLUE", "RED"}:
        if blue_score > red_score:
            winner_color = "BLUE"
        elif red_score > blue_score:
            winner_color = "RED"
        else:
            winner_color = "BLUE"
    return winner_color


def _resolve_corner_info(match: MatchRecord, corner_color: str) -> Tuple[Optional[str], Optional[str]]:
    corner_key = "blue" if corner_color.upper() == "BLUE" else "red"
    corner_info = match.athletes.get(corner_key, {})
    return corner_info.get("name"), corner_info.get("nation")


def _build_medalist_entry(
    *,
    category: Optional[str],
    match: MatchRecord,
    match_uuid: str,
    corner_color: str,
    medal_type: str,
    medal_rank: int,
) -> MedalistEntry:
    name, nation = _resolve_corner_info(match, corner_color)
    return MedalistEntry(
        category=category,
        match_uuid=match_uuid,
        match_id=match.match_id,
        corner_color=corner_color.upper(),
        athlete_name=name,
        athlete_country=nation,
        blue_score=int(match.final_snapshot.blue_score),
        red_score=int(match.final_snapshot.red_score),
        medal_type=medal_type,
        medal_rank=medal_rank,
    )


def _match_order_key(match: MatchRecord) -> Tuple[int, str, str]:
    match_number = match.metadata.get("match_number")
    if isinstance(match_number, str):
        try:
            match_number = int(match_number)
        except ValueError:
            match_number = None
    if not isinstance(match_number, int):
        match_number = -1
    timestamp = match.final_snapshot.timestamp_iso or ""
    return match_number, timestamp, match.match_id


def _select_latest_match(entries: Sequence[Tuple[MatchRecord, str]]) -> Optional[Tuple[MatchRecord, str]]:
    if not entries:
        return None
    return max(entries, key=lambda pair: _match_order_key(pair[0]))


def _derive_medalists_for_category(
    category: Optional[str],
    matches: Sequence[Tuple[MatchRecord, str]],
) -> List[MedalistEntry]:
    finals: List[Tuple[MatchRecord, str]] = []
    bronze_matches: List[Tuple[MatchRecord, str]] = []
    semifinals: List[Tuple[MatchRecord, str]] = []

    for match, match_uuid in matches:
        phase = _pick_metadata_str(
            match.metadata,
            "phase",
            "phaseName",
            "phase_name",
            "phase_label",
            "stage",
        )
        if _is_final_phase(phase):
            finals.append((match, match_uuid))
        elif _is_bronze_phase(phase):
            bronze_matches.append((match, match_uuid))
        elif _is_semifinal_phase(phase):
            semifinals.append((match, match_uuid))

    selected_final = _select_latest_match(finals)
    if not selected_final:
        return []

    final_match, final_uuid = selected_final
    winner_color = _resolve_winner_color(final_match)
    loser_color = "BLUE" if winner_color == "RED" else "RED"

    medalists: List[MedalistEntry] = [
        _build_medalist_entry(
            category=category,
            match=final_match,
            match_uuid=final_uuid,
            corner_color=winner_color,
            medal_type="gold",
            medal_rank=1,
        ),
        _build_medalist_entry(
            category=category,
            match=final_match,
            match_uuid=final_uuid,
            corner_color=loser_color,
            medal_type="silver",
            medal_rank=2,
        ),
    ]

    selected_bronze = _select_latest_match(bronze_matches)
    if selected_bronze:
        bronze_match, bronze_uuid = selected_bronze
        bronze_winner = _resolve_winner_color(bronze_match)
        bronze_loser = "BLUE" if bronze_winner == "RED" else "RED"
        medalists.append(
            _build_medalist_entry(
                category=category,
                match=bronze_match,
                match_uuid=bronze_uuid,
                corner_color=bronze_winner,
                medal_type="bronze1",
                medal_rank=3,
            )
        )
        medalists.append(
            _build_medalist_entry(
                category=category,
                match=bronze_match,
                match_uuid=bronze_uuid,
                corner_color=bronze_loser,
                medal_type="bronze2",
                medal_rank=4,
            )
        )
        return medalists

    if semifinals:
        semifinal_sorted = sorted(semifinals, key=lambda pair: _match_order_key(pair[0]))
        bronze_entries: List[MedalistEntry] = []
        for idx, (match, match_uuid) in enumerate(semifinal_sorted):
            winner = _resolve_winner_color(match)
            loser = "BLUE" if winner == "RED" else "RED"
            medal_type = "bronze1" if idx == 0 else "bronze2"
            medal_rank = 3 if idx == 0 else 4
            bronze_entries.append(
                _build_medalist_entry(
                    category=category,
                    match=match,
                    match_uuid=match_uuid,
                    corner_color=loser,
                    medal_type=medal_type,
                    medal_rank=medal_rank,
                )
            )
            if len(bronze_entries) == 2:
                break
        medalists.extend(bronze_entries)

    return medalists


def _store_medalists_for_tournament(
    conn: sqlite3.Connection,
    *,
    tournament_uuid: str,
    category_matches: DefaultDict[Optional[str], List[Tuple[MatchRecord, str]]],
) -> None:
    conn.execute(
        "DELETE FROM tournament_champions WHERE tournament_uuid = ?",
        (tournament_uuid,),
    )
    entries: List[MedalistEntry] = []
    for category, matches in category_matches.items():
        entries.extend(_derive_medalists_for_category(category, matches))

    if not entries:
        return

    payload = [
        (
            tournament_uuid,
            entry.category,
            entry.match_uuid,
            entry.match_id,
            entry.corner_color,
            entry.athlete_name,
            entry.athlete_country,
            entry.blue_score,
            entry.red_score,
            entry.medal_type,
            entry.medal_rank,
        )
        for entry in entries
    ]

    conn.executemany(
        """
        INSERT INTO tournament_champions (
            tournament_uuid, category, match_uuid, match_id, winner_color, winner_name,
            winner_country_code, blue_score, red_score, medal_type, medal_rank, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        """,
        payload,
    )


def _lookup_tournament(conn: sqlite3.Connection, name: str) -> Tuple[int, str]:
    row = conn.execute("SELECT id, uuid FROM tournaments WHERE name = ?", (name,)).fetchone()
    if not row:
        raise RuntimeError(f"Tournament '{name}' not found. Run scaffold_loader first.")
    return int(row[0]), row[1]


def _fetch_day_map(conn: sqlite3.Connection, tournament_id: int) -> Dict[str, int]:
    mapping: Dict[str, int] = {}
    for date_str, day_id in conn.execute(
        "SELECT date, id FROM tournament_days WHERE tournament_id = ?", (tournament_id,)
    ):
        mapping[date_str] = int(day_id)
    if not mapping:
        raise RuntimeError("No tournament days found. Seed scaffold before loading matches.")
    return mapping


def _fetch_octagon_map(conn: sqlite3.Connection, tournament_id: int) -> Dict[Tuple[int, str], int]:
    mapping: Dict[Tuple[int, str], int] = {}
    for octagon_id, day_id, number in conn.execute(
        "SELECT id, tournament_day_id, octagon_number FROM octagons WHERE tournament_id = ?", (tournament_id,)
    ):
        mapping[(int(day_id), number)] = int(octagon_id)
    return mapping


def _find_athlete_id(conn: sqlite3.Connection, entry: Dict[str, Optional[str]]) -> Optional[int]:
    wtid = (entry.get("wt_id") or entry.get("wtId") or "").strip()
    ioc = (entry.get("nation") or entry.get("ioc_code") or "").strip()
    display_name = (entry.get("name") or entry.get("display_name") or "").strip()

    if wtid:
        row = conn.execute("SELECT id FROM athletes WHERE wtid = ?", (wtid,)).fetchone()
        if row:
            return int(row[0])
    if ioc and display_name:
        row = conn.execute(
            "SELECT id FROM athletes WHERE ioc_code = ? AND display_name = ?",
            (ioc, display_name),
        ).fetchone()
        if row:
            return int(row[0])
    LOGGER.warning("Athlete '%s' (%s) not found in athletes table", display_name or "unknown", wtid or ioc or "?")
    return None


def _ensure_match(
    conn: sqlite3.Connection,
    *,
    tournament_db_id: int,
    tournament_uuid: str,
    match: MatchRecord,
) -> Tuple[int, str, bool]:
    row = conn.execute("SELECT id, uuid FROM pss_matches WHERE match_id = ?", (match.match_id,)).fetchone()
    match_number = match.metadata.get("match_number") or match.metadata.get("matchNumber")
    match_number = match_number if match_number is not None else match.metadata.get("match_number")
    category = match.metadata.get("category")
    weight_class = match.metadata.get("category")
    division = match.metadata.get("division")
    total_rounds = match.config.get("rounds")
    round_duration = _round_duration_seconds(match.config)
    countdown_type = "cntDown"
    format_type = None
    creation_mode = "replay"
    now = _utc_now_iso()

    if row:
        conn.execute(
            """
            UPDATE pss_matches
            SET tournament_id = ?,
                match_number = ?,
                category = ?,
                weight_class = ?,
                division = ?,
                total_rounds = ?,
                round_duration = ?,
                countdown_type = ?,
                format_type = ?,
                creation_mode = ?,
                updated_at = ?
            WHERE id = ?
            """,
            (
                tournament_uuid,
                str(match_number) if match_number is not None else None,
                category,
                weight_class,
                division,
                total_rounds,
                round_duration,
                countdown_type,
                format_type,
                creation_mode,
                now,
                int(row[0]),
            ),
        )
        return int(row[0]), row[1] or match.match_id, False

    match_uuid = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO pss_matches (
            uuid, tournament_id, match_id, match_number, category, weight_class, division,
            total_rounds, round_duration, countdown_type, format_type, creation_mode,
            created_at, updated_at, created, updated
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0)
        """,
        (
            match_uuid,
            tournament_uuid,
            match.match_id,
            str(match_number) if match_number is not None else None,
            category,
            weight_class,
            division,
            total_rounds,
            round_duration,
            countdown_type,
            format_type,
            creation_mode,
            now,
            now,
        ),
    )
    row_id = conn.execute("SELECT id FROM pss_matches WHERE uuid = ?", (match_uuid,)).fetchone()[0]
    return int(row_id), match_uuid, True


def _store_match_athletes(
    conn: sqlite3.Connection,
    *,
    match_ref: str,
    match: MatchRecord,
) -> None:
    conn.execute("DELETE FROM pss_match_athletes WHERE match_id = ?", (match_ref,))
    now = _utc_now_iso()

    blue_info = match.athletes.get("blue", {})
    red_info = match.athletes.get("red", {})

    blue_id = _find_athlete_id(conn, blue_info)
    red_id = _find_athlete_id(conn, red_info)

    rows_to_insert = []
    if blue_id:
        rows_to_insert.append(
            (match_ref, blue_id, 1, "#0000ff", "#ffffff", now)
        )
    if red_id:
        rows_to_insert.append(
            (match_ref, red_id, 2, "#ff0000", "#ffffff", now)
        )

    if rows_to_insert:
        conn.executemany(
            """
            INSERT INTO pss_match_athletes (
                match_id, athlete_id, athlete_position, bg_color, fg_color, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            """,
            rows_to_insert,
        )


def _store_rounds(conn: sqlite3.Connection, *, match_ref: str, match: MatchRecord) -> None:
    conn.execute("DELETE FROM pss_rounds WHERE match_id = ?", (match_ref,))
    for snapshot in match.round_snapshots:
        conn.execute(
            """
            INSERT INTO pss_rounds (
                match_id, round_number, start_time, end_time, duration, winner_athlete_position, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (
                match_ref,
                snapshot.round_number,
                snapshot.start_iso,
                snapshot.end_iso,
                snapshot.duration_seconds,
                snapshot.winner_position,
                _utc_now_iso(),
            ),
        )


def _store_scores(
    conn: sqlite3.Connection,
    *,
    match_ref: str,
    match: MatchRecord,
    tournament_uuid: str,
) -> None:
    conn.execute("DELETE FROM pss_scores WHERE match_id = ?", (match_ref,))
    rows = []

    for snapshot in match.round_snapshots:
        timestamp = snapshot.end_iso or _utc_now_iso()
        if snapshot.blue_score is not None:
            rows.append(
                (
                    match_ref,
                    snapshot.round_number,
                    1,
                    f"round{snapshot.round_number}",
                    snapshot.blue_score,
                    timestamp,
                    tournament_uuid,
                    _utc_now_iso(),
                )
            )
        if snapshot.red_score is not None:
            rows.append(
                (
                    match_ref,
                    snapshot.round_number,
                    2,
                    f"round{snapshot.round_number}",
                    snapshot.red_score,
                    timestamp,
                    tournament_uuid,
                    _utc_now_iso(),
                )
            )

    final_ts = match.final_snapshot.timestamp_iso or _utc_now_iso()
    rows.extend(
        [
            (
                match_ref,
                None,
                1,
                "total",
                match.final_snapshot.blue_score,
                final_ts,
                tournament_uuid,
                _utc_now_iso(),
            ),
            (
                match_ref,
                None,
                2,
                "total",
                match.final_snapshot.red_score,
                final_ts,
                tournament_uuid,
                _utc_now_iso(),
            ),
            (
                match_ref,
                None,
                1,
                "current",
                match.final_snapshot.blue_score,
                final_ts,
                tournament_uuid,
                _utc_now_iso(),
            ),
            (
                match_ref,
                None,
                2,
                "current",
                match.final_snapshot.red_score,
                final_ts,
                tournament_uuid,
                _utc_now_iso(),
            ),
        ]
    )

    conn.executemany(
        """
        INSERT INTO pss_scores (
            match_id, round_id, athlete_position, score_type, score_value, timestamp, tournament_id, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        rows,
    )


def _store_warnings(
    conn: sqlite3.Connection,
    *,
    match_ref: str,
    match: MatchRecord,
    tournament_uuid: str,
) -> None:
    conn.execute("DELETE FROM pss_warnings WHERE match_id = ?", (match_ref,))
    rows = []

    for snapshot in match.round_snapshots:
        timestamp = snapshot.end_iso or _utc_now_iso()
        if snapshot.blue_penalties is not None:
            rows.append(
                (
                    match_ref,
                    snapshot.round_number,
                    1,
                    "gam_jeom",
                    snapshot.blue_penalties,
                    timestamp,
                    tournament_uuid,
                    _utc_now_iso(),
                )
            )
        if snapshot.red_penalties is not None:
            rows.append(
                (
                    match_ref,
                    snapshot.round_number,
                    2,
                    "gam_jeom",
                    snapshot.red_penalties,
                    timestamp,
                    tournament_uuid,
                    _utc_now_iso(),
                )
            )

    final_ts = match.final_snapshot.timestamp_iso or _utc_now_iso()
    rows.extend(
        [
            (
                match_ref,
                None,
                1,
                "gam_jeom",
                match.final_snapshot.blue_penalties,
                final_ts,
                tournament_uuid,
                _utc_now_iso(),
            ),
            (
                match_ref,
                None,
                2,
                "gam_jeom",
                match.final_snapshot.red_penalties,
                final_ts,
                tournament_uuid,
                _utc_now_iso(),
            ),
        ]
    )

    conn.executemany(
        """
        INSERT INTO pss_warnings (
            match_id, round_id, athlete_position, warning_type, warning_count, timestamp, tournament_id, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """,
        rows,
    )


def apply_matches(
    *,
    conn: sqlite3.Connection,
    matches: Sequence[MatchRecord],
    tournament_name: str,
) -> Dict[str, int]:
    conn.execute("PRAGMA busy_timeout = 5000")
    tournament_db_id, tournament_uuid = _lookup_tournament(conn, tournament_name)
    _ensure_champions_table(conn)
    day_map = _fetch_day_map(conn, tournament_db_id)
    _ = _fetch_octagon_map(conn, tournament_db_id)  # Reserved for future use
    inserted = 0
    updated = 0
    category_matches: DefaultDict[Optional[str], List[Tuple[MatchRecord, str]]] = defaultdict(list)

    for match in matches:
        # Validate day mapping
        try:
            date_value = datetime.strptime(match.day_folder, "%Y%m%d").strftime("%Y-%m-%d")
        except ValueError:
            LOGGER.warning("Skipping match %s due to invalid day folder %s", match.match_id, match.day_folder)
            continue
        if date_value not in day_map:
            LOGGER.warning("No tournament day mapped for %s (match %s)", date_value, match.match_id)
        _, match_ref, created = _ensure_match(
            conn,
            tournament_db_id=tournament_db_id,
            tournament_uuid=tournament_uuid,
            match=match,
        )
        if created:
            inserted += 1
        else:
            updated += 1

        _store_match_athletes(conn, match_ref=match_ref, match=match)
        _store_rounds(conn, match_ref=match_ref, match=match)
        _store_scores(conn, match_ref=match_ref, match=match, tournament_uuid=tournament_uuid)
        _store_warnings(conn, match_ref=match_ref, match=match, tournament_uuid=tournament_uuid)
        category_key = _pick_metadata_str(
            match.metadata,
            "category",
            "categoryName",
            "category_name",
            "weight_class",
        )
        category_matches[category_key].append((match, match_ref))

    _store_medalists_for_tournament(
        conn,
        tournament_uuid=tournament_uuid,
        category_matches=category_matches,
    )
    conn.commit()
    return {"inserted_matches": inserted, "updated_matches": updated}


def cmd_inspect(args: argparse.Namespace) -> None:
    matches = load_matches(Path(args.archive_root).expanduser())
    print(
        {
            "matches_detected": len(matches),
            "sample_match_ids": [m.match_id for m in matches[:5]],
            "rounds_per_match": {m.match_id: len(m.round_snapshots) for m in matches[:3]},
        }
    )


def cmd_apply(args: argparse.Namespace) -> None:
    matches = load_matches(Path(args.archive_root).expanduser())
    conn = sqlite3.connect(args.db_path)
    try:
        conn.execute("PRAGMA foreign_keys = ON")
        stats = apply_matches(conn=conn, matches=matches, tournament_name=args.tournament_name)
    finally:
        conn.close()
    LOGGER.info("Match ingestion complete: %s", stats)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log-level", default="INFO")
    subparsers = parser.add_subparsers(dest="command", required=True)

    inspect_parser = subparsers.add_parser("inspect", help="Preview matches without modifying the DB.")
    inspect_parser.add_argument("--archive-root", required=True)
    inspect_parser.add_argument("--db-path", required=False)
    inspect_parser.set_defaults(func=cmd_inspect)

    apply_parser = subparsers.add_parser("apply", help="Apply matches to the database.")
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
