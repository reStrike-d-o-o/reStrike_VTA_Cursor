#!/usr/bin/env python3
"""
Builds tournament scaffolding (tournament metadata, days, octagons, athletes)
from Daedo GO2025 log archives and applies it to the local SQLite database.

Usage examples
-------------
Inspect archive (no DB writes):
    python scripts/tournament/scaffold_loader.py inspect \
        --archive-root "C:/Users/Damjan/Documents/Daedo log files GO2025"

Apply scaffolding to the dev database:
    python scripts/tournament/scaffold_loader.py apply \
        --archive-root "C:/Users/Damjan/Documents/Daedo log files GO2025" \
        --db-path "src-tauri/restrike_vta.db" \
        --tournament-name "German Open - Hamburg 2025" \
        --city "Hamburg" \
        --country "Germany" \
        --country-code "DE" \
        --ranking-code "G2" \
        --location '{"venue": "Sporthalle Hamburg", "city": "Hamburg", "country": "Germany"}'
"""

from __future__ import annotations

import argparse
import json
import logging
import sqlite3
import sys
import uuid
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set, Tuple

# Ensure repository root is on sys.path for intra-project imports
REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.tournament import daedo_log_parser


LOGGER = logging.getLogger("scaffold_loader")


@dataclass(frozen=True)
class AthleteKey:
    wtid: Optional[str]
    ioc_code: str
    display_name: str


@dataclass
class AthleteInfo:
    key: AthleteKey
    first_name: Optional[str]
    last_name: Optional[str]
    country: Optional[str]
    country_code: Optional[str]


@dataclass
class ArchiveSummary:
    matches_scanned: int
    start_date: Optional[str]
    end_date: Optional[str]
    day_courts: Dict[str, Set[str]]
    athletes: Dict[AthleteKey, AthleteInfo]

    @property
    def day_count(self) -> int:
        return len(self.day_courts)

    @property
    def total_athletes(self) -> int:
        return len(self.athletes)


def _parse_date_from_filename(filename: str) -> Optional[str]:
    # Expect YYYYMMDDhhmmss-xxx
    try:
        dt = datetime.strptime(filename[:8], "%Y%m%d")
        return dt.strftime("%Y-%m-%d")
    except ValueError:
        return None


def _split_name(name: str) -> Tuple[Optional[str], Optional[str]]:
    parts = name.strip().split()
    if not parts:
        return None, None
    if len(parts) == 1:
        return None, parts[0]
    return " ".join(parts[:-1]), parts[-1]


def collect_archive_summary(root: Path) -> ArchiveSummary:
    LOGGER.info("Collecting archive summary from %s", root)
    matches_scanned = 0
    start_dates: List[str] = []
    end_dates: List[str] = []
    day_courts: Dict[str, Set[str]] = defaultdict(set)
    athletes: Dict[AthleteKey, AthleteInfo] = {}

    for match_log_path in sorted(root.rglob("*-matchLog.csv")):
        matches_scanned += 1
        # Use parser to read one row
        try:
            match_row = daedo_log_parser.parse_match_log(match_log_path)
        except Exception as exc:  # pragma: no cover - diagnostics
            LOGGER.warning("Failed to parse %s: %s", match_log_path, exc)
            continue

        filename = match_log_path.name
        match_start = match_row.get("matchStartTime", "").strip()
        if match_start:
            try:
                start_dt = datetime.strptime(match_start, "%d/%m/%Y %H:%M:%S:%f")
                start_dates.append(start_dt.strftime("%Y-%m-%d"))
            except ValueError:
                LOGGER.debug("Unable to parse start time '%s' from %s", match_start, filename)

        match_end = match_row.get("matchEndTime", "").strip()
        if match_end:
            try:
                end_dt = datetime.strptime(match_end, "%d/%m/%Y %H:%M:%S:%f")
                end_dates.append(end_dt.strftime("%Y-%m-%d"))
            except ValueError:
                LOGGER.debug("Unable to parse end time '%s' from %s", match_end, filename)

        if not end_dates:
            fallback_date = _parse_date_from_filename(filename)
            if fallback_date:
                end_dates.append(fallback_date)

        day_folder = match_log_path.parent.name
        day_courts[day_folder].add(match_log_path.parent.parent.name)

        for corner in ("blue", "red"):
            name = match_row.get(f"{corner}AthleteName", "").strip()
            if not name:
                continue
            wtid = match_row.get(f"{corner}AthleteWtfId", "").strip() or None
            ioc = match_row.get(f"{corner}AthleteFlagAbbreviation", "").strip() or "UNK"
            display = name
            key = AthleteKey(wtid=wtid, ioc_code=ioc, display_name=display)
            if key in athletes:
                continue
            first_name, last_name = _split_name(name)
            country = match_row.get(f"{corner}AthleteFlagName", "").strip() or None
            country_code = match_row.get(f"{corner}AthleteFlagAbbreviation", "").strip() or None
            athletes[key] = AthleteInfo(
                key=key,
                first_name=first_name,
                last_name=last_name,
                country=country,
                country_code=country_code,
            )

    start_date = min(start_dates) if start_dates else None
    end_date = max(end_dates) if end_dates else None

    LOGGER.info(
        "Archive summary collected: %d matches, %d unique days, %d athletes",
        matches_scanned,
        len(day_courts),
        len(athletes),
    )

    return ArchiveSummary(
        matches_scanned=matches_scanned,
        start_date=start_date,
        end_date=end_date,
        day_courts=day_courts,
        athletes=athletes,
    )


def _load_json_arg(value: Optional[str]) -> str:
    if not value:
        return "{}"
    path = Path(value)
    if path.exists():
        return path.read_text(encoding="utf-8")
    return json.dumps(json.loads(value))


def _ensure_ranking(conn: sqlite3.Connection, ranking_code: Optional[str]) -> Optional[int]:
    if not ranking_code:
        return None
    row = conn.execute(
        "SELECT id FROM tournament_rankings WHERE code = ?",
        (ranking_code,),
    ).fetchone()
    if not row:
        raise RuntimeError(f"Ranking code {ranking_code} not found in tournament_rankings")
    return row[0]


def _ensure_tournament(
    conn: sqlite3.Connection,
    *,
    name: str,
    city: str,
    country: str,
    country_code: Optional[str],
    summary: ArchiveSummary,
    status: str,
    ranking_id: Optional[int],
    location_json: str,
    contact_json: str,
    oc_json: str,
    officials_json: str,
    banner: Optional[str],
) -> int:
    row = conn.execute(
        "SELECT id, duration_days FROM tournaments WHERE name = ?",
        (name,),
    ).fetchone()

    data = {
        "duration_days": summary.day_count or 1,
        "city": city,
        "country": country,
        "country_code": country_code,
        "status": status,
        "start_date": summary.start_date,
        "end_date": summary.end_date,
        "ranking_id": ranking_id,
        "location": location_json,
        "contact": contact_json,
        "oc": oc_json,
        "officials": officials_json,
        "banner": banner,
    }

    if row:
        LOGGER.info("Updating existing tournament '%s' (id=%s)", name, row[0])
        conn.execute(
            """
            UPDATE tournaments
            SET duration_days = :duration_days,
                city = :city,
                country = :country,
                country_code = :country_code,
                status = :status,
                start_date = :start_date,
                end_date = :end_date,
                ranking_id = :ranking_id,
                location = :location,
                contact = :contact,
                oc = :oc,
                officials = :officials,
                banner = :banner,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = :id
            """,
            {**data, "id": row[0]},
        )
        return row[0]

    LOGGER.info("Creating tournament '%s'", name)
    tournament_uuid = str(uuid.uuid4())
    conn.execute(
        """
        INSERT INTO tournaments (
            uuid, name, duration_days, city, country, country_code, status,
            start_date, end_date, ranking_id, location, contact, oc, officials, banner,
            created_at, updated_at
        ) VALUES (
            :uuid, :name, :duration_days, :city, :country, :country_code, :status,
            :start_date, :end_date, :ranking_id, :location, :contact, :oc, :officials, :banner,
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        )
        """,
        {
            "uuid": tournament_uuid,
            "name": name,
            **data,
        },
    )
    return conn.execute("SELECT id FROM tournaments WHERE uuid = ?", (tournament_uuid,)).fetchone()[0]


def _ensure_tournament_days(
    conn: sqlite3.Connection,
    tournament_id: int,
    day_courts: Dict[str, Set[str]],
) -> Dict[str, int]:
    ids: Dict[str, int] = {}
    for day_number, day_folder in enumerate(sorted(day_courts.keys()), start=1):
        try:
            date_value = datetime.strptime(day_folder, "%Y%m%d").strftime("%Y-%m-%d")
        except ValueError:
            LOGGER.warning("Skipping unexpected day folder format: %s", day_folder)
            continue
        row = conn.execute(
            "SELECT id FROM tournament_days WHERE tournament_id = ? AND date = ?",
            (tournament_id, date_value),
        ).fetchone()
        if row:
            ids[day_folder] = row[0]
            conn.execute(
                """
                UPDATE tournament_days
                SET day_number = ?, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                """,
                (day_number, row[0]),
            )
            LOGGER.debug("Ensured tournament day %s (id=%s)", date_value, row[0])
            continue
        LOGGER.info("Creating tournament day %s (#%d)", date_value, day_number)
        day_uuid = str(uuid.uuid4())
        conn.execute(
            """
            INSERT INTO tournament_days (
                uuid, tournament_id, day_number, date, status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, 'ended', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            """,
            (day_uuid, tournament_id, day_number, date_value),
        )
        ids[day_folder] = conn.execute(
            "SELECT id FROM tournament_days WHERE uuid = ?",
            (day_uuid,),
        ).fetchone()[0]
    return ids


def _ensure_octagons(
    conn: sqlite3.Connection,
    tournament_id: int,
    day_ids: Dict[str, int],
    day_courts: Dict[str, Set[str]],
) -> None:
    for day_folder, courts in day_courts.items():
        day_id = day_ids.get(day_folder)
        if day_id is None:
            LOGGER.warning("No tournament day id found for folder %s; skipping octagons", day_folder)
            continue
        for court in sorted(courts):
            row = conn.execute(
                """
                SELECT id FROM octagons
                WHERE tournament_day_id = ? AND octagon_number = ?
                """,
                (day_id, court),
            ).fetchone()
            if row:
                LOGGER.debug("Octagon %s already exists for day %s", court, day_folder)
                conn.execute(
                    "UPDATE octagons SET updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                    (row[0],),
                )
                continue
            LOGGER.info("Creating octagon %s for day %s", court, day_folder)
            conn.execute(
                """
                INSERT INTO octagons (
                    tournament_id, tournament_day_id, octagon_number, created_at, updated_at
                ) VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                """,
                (tournament_id, day_id, court),
            )


def _ensure_athlete(conn: sqlite3.Connection, info: AthleteInfo) -> int:
    key = info.key
    if key.wtid:
        row = conn.execute(
            "SELECT id FROM athletes WHERE wtid = ?",
            (key.wtid,),
        ).fetchone()
    else:
        row = conn.execute(
            "SELECT id FROM athletes WHERE ioc_code = ? AND display_name = ?",
            (key.ioc_code, key.display_name),
        ).fetchone()

    payload = {
        "wtid": key.wtid,
        "first_name": info.first_name,
        "last_name": info.last_name,
        "display_name": key.display_name,
        "country": info.country,
        "country_code": info.country_code,
        "ioc_code": key.ioc_code,
    }

    if row:
        conn.execute(
            """
            UPDATE athletes
            SET first_name = COALESCE(:first_name, first_name),
                last_name = COALESCE(:last_name, last_name),
                country = COALESCE(:country, country),
                country_code = COALESCE(:country_code, country_code),
                display_name = :display_name,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = :id
            """,
            {**payload, "id": row[0]},
        )
        return row[0]

    conn.execute(
        """
        INSERT INTO athletes (
            wtid, first_name, last_name, display_name, history,
            country, country_code, ioc_code, created_at, updated_at
        ) VALUES (
            :wtid, :first_name, :last_name, :display_name, '[]',
            :country, :country_code, :ioc_code, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        )
        """,
        payload,
    )
    return conn.execute(
        "SELECT id FROM athletes WHERE ioc_code = ? AND display_name = ? ORDER BY id DESC LIMIT 1",
        (key.ioc_code, key.display_name),
    ).fetchone()[0]


def _ensure_athletes(conn: sqlite3.Connection, summary: ArchiveSummary) -> int:
    processed = 0
    for info in summary.athletes.values():
        athlete_id = _ensure_athlete(conn, info)
        if athlete_id:
            processed += 1
    LOGGER.info("Ensured %d athlete records", processed)
    return processed


def cmd_inspect(args: argparse.Namespace) -> None:
    summary = collect_archive_summary(Path(args.archive_root).expanduser())
    print(
        json.dumps(
            {
                "matches_scanned": summary.matches_scanned,
                "day_folders": sorted(summary.day_courts.keys()),
                "start_date": summary.start_date,
                "end_date": summary.end_date,
                "day_count": summary.day_count,
                "unique_athletes": summary.total_athletes,
            },
            indent=2,
        )
    )


def cmd_apply(args: argparse.Namespace) -> None:
    summary = collect_archive_summary(Path(args.archive_root).expanduser())
    conn = sqlite3.connect(args.db_path)
    try:
        conn.execute("PRAGMA foreign_keys = ON")
        ranking_id = _ensure_ranking(conn, args.ranking_code)
        tournament_id = _ensure_tournament(
            conn,
            name=args.tournament_name,
            city=args.city,
            country=args.country,
            country_code=args.country_code,
            summary=summary,
            status=args.status,
            ranking_id=ranking_id,
            location_json=_load_json_arg(args.location),
            contact_json=_load_json_arg(args.contact),
            oc_json=_load_json_arg(args.organizing_committee),
            officials_json=_load_json_arg(args.officials),
            banner=None,
        )
        day_ids = _ensure_tournament_days(conn, tournament_id, summary.day_courts)
        _ensure_octagons(conn, tournament_id, day_ids, summary.day_courts)
        _ensure_athletes(conn, summary)
        conn.commit()
        LOGGER.info("Scaffold applied successfully (tournament_id=%s)", tournament_id)
    finally:
        conn.close()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--log-level",
        default="INFO",
        help="Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL).",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    inspect_parser = subparsers.add_parser("inspect", help="Inspect archive contents.")
    inspect_parser.add_argument("--archive-root", required=True)
    inspect_parser.set_defaults(func=cmd_inspect)

    apply_parser = subparsers.add_parser("apply", help="Apply scaffold to SQLite DB.")
    apply_parser.add_argument("--archive-root", required=True)
    apply_parser.add_argument("--db-path", required=True)
    apply_parser.add_argument("--tournament-name", required=True)
    apply_parser.add_argument("--city", required=True)
    apply_parser.add_argument("--country", required=True)
    apply_parser.add_argument("--country-code")
    apply_parser.add_argument("--ranking-code", default="G2")
    apply_parser.add_argument("--status", default="ended")
    apply_parser.add_argument("--location")
    apply_parser.add_argument("--contact")
    apply_parser.add_argument("--organizing-committee")
    apply_parser.add_argument("--officials")
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
