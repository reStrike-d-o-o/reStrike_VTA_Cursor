#!/usr/bin/env python3
"""
Utilities for reverse-engineering Daedo GO2025 match logs.

The parser handles the non-standard CSV layout used by the `matchLog.csv`
files, where the `roundsWinners` field embeds JSON-like content without
quoting, and produces a canonical match payload suited for tooling and
migrations.

Example usage:
    python scripts/tournament/daedo_log_parser.py extract \
        "C:/Users/Damjan/Documents/Daedo log files GO2025/Court01/20250913/20250913091016-101-matchLog.csv" \
        --output match_101.json

    python scripts/tournament/daedo_log_parser.py summarize \
        "C:/Users/Damjan/Documents/Daedo log files GO2025"
"""

from __future__ import annotations

import argparse
import csv
import json
from collections import Counter
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

import logging

LOGGER = logging.getLogger("daedo_log_parser")


POINT_CODE_MAP = {
    "BODY_POINT": "2",
    "BODY_TECH_POINT": "4",
    "HEAD_POINT": "3",
    "HEAD_TECH_POINT": "5",
    "PUNCH_POINT": "1",
}

HIT_EVENT_SUFFIXES = {
    "BODY_HIT",
    "HEAD_HIT",
    "SENSOR_PUNCH_HIT",
    "JUDGE_PUNCH",
    "JUDGE_BODY_TECH",
    "JUDGE_HEAD_TECH",
    "JUDGE_BODY_POINT",
    "JUDGE_HEAD_POINT",
    "ADD_NEAR_MISS_HIT",
}

PENALTY_EVENT_SUFFIXES = {
    "ADD_GAME_JEON",
    "REMOVE_GAME_JEON",
    "ADD_PENALTY",
    "REMOVE_PENALTY",
}


def _split_match_log_line(line: str) -> List[str]:
    """Split a matchLog.csv row while respecting unquoted JSON blobs."""
    fields: List[str] = []
    current: List[str] = []
    depth_brace = 0
    depth_bracket = 0

    for char in line:
        if char == "," and depth_brace == 0 and depth_bracket == 0:
            fields.append("".join(current))
            current = []
            continue

        if char == "{":
            depth_brace += 1
        elif char == "}":
            depth_brace = max(0, depth_brace - 1)
        elif char == "[":
            depth_bracket += 1
        elif char == "]":
            depth_bracket = max(0, depth_bracket - 1)

        current.append(char)

    fields.append("".join(current))
    return fields


def _ensure_unique_headers(headers: Sequence[str]) -> List[str]:
    """Rename duplicate CSV headers by adding suffixes."""
    seen: Dict[str, int] = {}
    result: List[str] = []
    for header in headers:
        if header in seen:
            seen[header] += 1
            new_header = f"{header}_{seen[header]}"
            result.append(new_header)
        else:
            seen[header] = 0
            result.append(header)
    return result


def _parse_bool(value: str) -> bool:
    return value.strip().lower() == "true"


def _parse_optional_int(value: str) -> Optional[int]:
    value = value.strip()
    if not value:
        return None
    try:
        return int(value)
    except ValueError:
        return None


def _parse_score(value: str) -> Tuple[int, int]:
    value = value.strip()
    if not value or "-" not in value:
        return (0, 0)
    left, right = value.split("-", 1)
    return (int(left), int(right))


def _parse_rounds_winners(raw_value: str) -> Dict[str, Any]:
    normalized = raw_value.replace('""', '"').strip()
    if not normalized:
        return {}
    try:
        return json.loads(normalized)
    except json.JSONDecodeError:
        return {}


def _ms_to_iso8601(value: str) -> Optional[str]:
    value = value.strip()
    if not value:
        return None
    try:
        millis = int(value)
    except ValueError:
        return None
    seconds = millis / 1000.0
    dt = datetime.fromtimestamp(seconds, tz=timezone.utc)
    return dt.isoformat().replace("+00:00", "Z")


@dataclass
class CanonicalEvent:
    sequence: int
    event_time_ms: Optional[int]
    event_time_iso: Optional[str]
    round_number: Optional[int]
    round_time_ms: Optional[int]
    system_time_ms: Optional[int]
    event_type: str
    entry_value: Optional[str]
    entry_value_aux: Optional[str]
    blue_add_points: Optional[int]
    red_add_points: Optional[int]
    score_snapshot: Tuple[int, int]
    blue_general_points: Optional[int]
    red_general_points: Optional[int]
    blue_points: Optional[int]
    red_points: Optional[int]
    blue_penalties: Optional[int]
    red_penalties: Optional[int]
    blue_total_penalties: Optional[int]
    red_total_penalties: Optional[int]
    blue_gp_hits: Optional[int]
    red_gp_hits: Optional[int]
    blue_gp_penalties: Optional[int]
    red_gp_penalties: Optional[int]
    golden_point_round: bool
    blue_video_quota: Optional[int]
    red_video_quota: Optional[int]
    udp_packets: Optional[List[Dict[str, Any]]] = None
    udp_mapping_status: str = "unmapped"
    udp_mapping_notes: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        data = {
            "sequence": self.sequence,
            "event_time_ms": self.event_time_ms,
            "event_time_iso": self.event_time_iso,
            "round_number": self.round_number,
            "round_time_ms": self.round_time_ms,
            "system_time_ms": self.system_time_ms,
            "event_type": self.event_type,
            "entry_value": self.entry_value,
            "entry_value_aux": self.entry_value_aux,
            "blue_add_points": self.blue_add_points,
            "red_add_points": self.red_add_points,
            "score_snapshot": {
                "blue": self.score_snapshot[0],
                "red": self.score_snapshot[1],
            },
            "blue_general_points": self.blue_general_points,
            "red_general_points": self.red_general_points,
            "blue_points": self.blue_points,
            "red_points": self.red_points,
            "blue_penalties": self.blue_penalties,
            "red_penalties": self.red_penalties,
            "blue_total_penalties": self.blue_total_penalties,
            "red_total_penalties": self.red_total_penalties,
            "blue_golden_point_hits": self.blue_gp_hits,
            "red_golden_point_hits": self.red_gp_hits,
            "blue_golden_point_penalties": self.blue_gp_penalties,
            "red_golden_point_penalties": self.red_gp_penalties,
            "golden_point_round": self.golden_point_round,
            "blue_video_quota": self.blue_video_quota,
            "red_video_quota": self.red_video_quota,
        }
        udp_info: Dict[str, Any] = {"status": self.udp_mapping_status}
        if self.udp_packets:
            udp_info["packets"] = self.udp_packets
        if self.udp_mapping_notes:
            udp_info["notes"] = self.udp_mapping_notes
        if udp_info["status"] != "unmapped" or udp_info.get("packets") or udp_info.get("notes"):
            data["udp"] = udp_info
        return data


def parse_match_log(path: Path) -> Dict[str, Any]:
    LOGGER.debug("Reading match log %s", path)
    lines = path.read_text(encoding="utf-8-sig").splitlines()
    if not lines:
        raise ValueError(f"Empty match log: {path}")

    headers = lines[0].split(",")
    records = []
    for raw_line in lines[1:]:
        if not raw_line.strip():
            continue
        values = _split_match_log_line(raw_line.rstrip())
        if len(values) != len(headers):
            raise ValueError(
                f"Header/row mismatch in {path} ({len(headers)} vs {len(values)})"
            )
        records.append(dict(zip(headers, values)))

    if not records:
        raise ValueError(f"No data rows in match log: {path}")

    return records[0]


def parse_match_log_items(path: Path) -> List[Dict[str, Any]]:
    LOGGER.debug("Reading match log items %s", path)
    with path.open("r", encoding="utf-8-sig", newline="") as handle:
        reader = csv.reader(handle)
        try:
            headers = next(reader)
        except StopIteration:
            raise ValueError(f"Empty match log items file: {path}") from None

        headers = _ensure_unique_headers(headers)
        rows: List[Dict[str, Any]] = []
        for row in reader:
            if not any(cell.strip() for cell in row):
                continue
            rows.append({headers[idx]: value for idx, value in enumerate(row)})
    return rows


def _normalize_competitor(event_type: str) -> Tuple[str, str, bool]:
    overtime = False
    base_type = event_type
    if base_type.startswith("OT_"):
        overtime = True
        base_type = base_type[3:]
    if base_type.startswith("BLUE_"):
        return "BLUE", base_type[len("BLUE_"):], overtime
    if base_type.startswith("RED_"):
        return "RED", base_type[len("RED_"):], overtime
    if base_type.startswith("CR_BLUE_"):
        return "CENTER_BLUE", base_type[len("CR_BLUE_"):], overtime
    if base_type.startswith("CR_RED_"):
        return "CENTER_RED", base_type[len("CR_RED_"):], overtime
    return "UNKNOWN", base_type, overtime


def _safe_str(value: Optional[str]) -> Optional[str]:
    if value is None:
        return None
    value = value.strip()
    return value or None


def _map_udp_packets(event_type: str, row: Dict[str, Any]) -> Tuple[Optional[List[Dict[str, Any]]], str, Optional[str]]:
    competitor, suffix, overtime = _normalize_competitor(event_type)
    suffix_upper = suffix.upper()

    if competitor in ("BLUE", "RED") and suffix_upper in POINT_CODE_MAP:
        stream = "pt1" if competitor == "BLUE" else "pt2"
        packet = {
            "stream": stream,
            "arguments": [POINT_CODE_MAP[suffix_upper]],
            "source": "auto",
        }
        notes = "overtime" if overtime else None
        return [packet], "mapped", notes

    if competitor in ("BLUE", "RED") and suffix_upper in HIT_EVENT_SUFFIXES:
        stream = "hl1" if competitor == "BLUE" else "hl2"
        hit_level = _safe_str(row.get("entryValue")) or _safe_str(row.get("entryValue_1"))
        if not hit_level:
            notes = "hit level missing"
            return None, "partial", notes
        packet = {
            "stream": stream,
            "arguments": [hit_level],
            "source": "auto",
        }
        notes = "overtime" if overtime else None
        return [packet], "mapped", notes

    if suffix_upper in PENALTY_EVENT_SUFFIXES:
        blue_total = _safe_str(row.get("blueTotalPenalties")) or "0"
        red_total = _safe_str(row.get("redTotalPenalties")) or "0"
        packets = [
            {"stream": "wg1", "arguments": [blue_total], "source": "auto"},
            {"stream": "wg2", "arguments": [red_total], "source": "auto"},
        ]
        action = "increase" if "ADD" in suffix_upper else "decrease"
        notes = f"{competitor.lower()} penalty {action}" if competitor in ("BLUE", "RED") else f"penalty {action}"
        if overtime:
            notes = f"{notes} (overtime)"
        return packets, "mapped", notes

    if suffix_upper in {"VIDEO_REQUEST", "VIDEO_QUOTA_CHANGED", "VIDEO_QUOTA_ACCEPTED"}:
        effective_competitor = competitor
        if competitor in {"CENTER_BLUE", "CENTER_RED"}:
            effective_competitor = "UNKNOWN"
        stream = "ch1" if effective_competitor == "BLUE" else "ch2" if effective_competitor == "RED" else "ch0"
        if competitor == "BLUE":
            quota = _safe_str(row.get("blueVideoQuota"))
        elif competitor == "RED":
            quota = _safe_str(row.get("redVideoQuota"))
        else:
            quota = None
        args = []
        if quota is not None and suffix_upper == "VIDEO_QUOTA_CHANGED":
            args = [quota]
        packet = {"stream": stream, "arguments": args, "source": "inferred"}
        if competitor in ("BLUE", "RED"):
            notes = "coach review"
        elif competitor in ("CENTER_BLUE", "CENTER_RED"):
            notes = "central review (team-specific)"
        else:
            notes = "central review"
        return [packet], "mapped", notes

    if suffix_upper in {"KYE_SHI", "DOCTOR", "DOCTOR_QUIT"}:
        stream = "ij1" if competitor == "BLUE" else "ij2" if competitor == "RED" else "ij0"
        action_map = {
            "KYE_SHI": "start",
            "DOCTOR": "show",
            "DOCTOR_QUIT": "hide",
        }
        packet = {"stream": stream, "arguments": [action_map[suffix_upper]], "source": "inferred"}
        return [packet], "mapped", None

    if suffix_upper in {"TIMEOUT", "RESUME"}:
        action = "stop" if suffix_upper == "TIMEOUT" else "start"
        packet = {"stream": "clk", "arguments": [action], "source": "inferred"}
        return [packet], "mapped", None

    if suffix_upper == "END_ROUND":
        packet = {"stream": "clk", "arguments": ["stopEnd"], "source": "inferred"}
        return [packet], "mapped", None

    if event_type in {"START_MATCH", "MATCH_FINISHED", "MATCH_FINAL_NEEDS_CONFIRM_DECISION"}:
        stream = "wmh" if event_type == "MATCH_FINISHED" else "pre"
        arguments: List[str] = []
        if event_type == "MATCH_FINISHED":
            winner_summary = _safe_str(row.get("entryValue")) or _safe_str(row.get("score"))
            if winner_summary:
                arguments.append(winner_summary)
        packet = {"stream": stream, "arguments": arguments, "source": "inferred"}
        status = "mapped" if arguments or stream == "pre" else "partial"
        return [packet], status, None

    return None, "unmapped", None


def _build_canonical_match(
    match_row: Dict[str, Any],
    item_rows: Sequence[Dict[str, Any]],
    source_path: Path,
) -> Dict[str, Any]:
    start = match_row.get("matchStartTime", "").strip()
    end = match_row.get("matchEndTime", "").strip()
    score = _parse_score(match_row.get("matchResult", ""))

    canonical_events: List[CanonicalEvent] = []
    for idx, row in enumerate(item_rows, start=1):
        event_time_ms = _parse_optional_int(row.get("eventTime", ""))
        udp_packets, udp_status, udp_notes = _map_udp_packets(row.get("matchLogItemType", "").strip(), row)
        canonical_events.append(
            CanonicalEvent(
                sequence=idx,
                event_time_ms=event_time_ms,
                event_time_iso=_ms_to_iso8601(row.get("eventTime", "")),
                round_number=_parse_optional_int(row.get("roundNumber", "")),
                round_time_ms=_parse_optional_int(row.get("roundTime", "")),
                system_time_ms=_parse_optional_int(row.get("systemTime", "")),
                event_type=row.get("matchLogItemType", "").strip(),
                entry_value=row.get("entryValue", "").strip() or None,
                entry_value_aux=row.get("entryValue_1", "").strip() or None,
                blue_add_points=_parse_optional_int(row.get("blueAddPoints", "")),
                red_add_points=_parse_optional_int(row.get("redAddPoints", "")),
                score_snapshot=_parse_score(row.get("score", "")),
                blue_general_points=_parse_optional_int(
                    row.get("blueGeneralPoints", "")
                ),
                red_general_points=_parse_optional_int(row.get("redGeneralPoints", "")),
                blue_points=_parse_optional_int(row.get("bluePoints", "")),
                red_points=_parse_optional_int(row.get("redPoints", "")),
                blue_penalties=_parse_optional_int(row.get("bluePenalties", "")),
                red_penalties=_parse_optional_int(row.get("redPenalties", "")),
                blue_total_penalties=_parse_optional_int(
                    row.get("blueTotalPenalties", "")
                ),
                red_total_penalties=_parse_optional_int(
                    row.get("redTotalPenalties", "")
                ),
                blue_gp_hits=_parse_optional_int(row.get("blueGoldenPointHits", "")),
                red_gp_hits=_parse_optional_int(row.get("redGoldenPointHits", "")),
                blue_gp_penalties=_parse_optional_int(
                    row.get("blueGoldenPointPenalties", "")
                ),
                red_gp_penalties=_parse_optional_int(
                    row.get("redGoldenPointPenalties", "")
                ),
                golden_point_round=_parse_bool(row.get("goldenPointRound", "")),
                blue_video_quota=_parse_optional_int(row.get("blueVideoQuota", "")),
                red_video_quota=_parse_optional_int(row.get("redVideoQuota", "")),
                udp_packets=udp_packets,
                udp_mapping_status=udp_status,
                udp_mapping_notes=udp_notes,
            )
        )

    file_stem = source_path.name.replace("-matchLog.csv", "")
    court = source_path.parent.parent.name if source_path.parents else ""
    day = source_path.parent.name if source_path.parents else ""

    canonical = {
        "match_id": file_stem,
        "source": {
            "court": court,
            "day_folder": day,
            "match_log": str(source_path),
        },
        "metadata": {
            "match_number": _parse_optional_int(match_row.get("matchNumber", "")),
            "phase": match_row.get("phaseName", "").strip() or None,
            "division": match_row.get("subCategoryName", "").strip() or None,
            "category": match_row.get("categoryName", "").strip() or None,
            "gender": match_row.get("categoryGender", "").strip() or None,
            "winner": match_row.get("matchWinner", "").strip() or None,
            "winner_by": match_row.get("matchWinnerBy", "").strip() or None,
            "victory_criteria": match_row.get("matchVictoryCriteria", "").strip()
            or None,
            "result": match_row.get("matchResult", "").strip() or None,
            "start_raw": start or None,
            "end_raw": end or None,
        },
        "configuration": {
            "rounds": _parse_optional_int(match_row.get("roundsConfig.rounds", "")),
            "round_time": {
                "minutes": _parse_optional_int(
                    match_row.get("roundsConfig.roundTimeMinutes", "")
                ),
                "seconds": _parse_optional_int(
                    match_row.get("roundsConfig.roundTimeSeconds", "")
                ),
            },
            "rest_time": {
                "minutes": _parse_optional_int(
                    match_row.get("roundsConfig.restTimeMinutes", "")
                ),
                "seconds": _parse_optional_int(
                    match_row.get("roundsConfig.restTimeSeconds", "")
                ),
            },
            "kye_shi": {
                "minutes": _parse_optional_int(
                    match_row.get("roundsConfig.kyeShiTimeMinutes", "")
                ),
                "seconds": _parse_optional_int(
                    match_row.get("roundsConfig.kyeShiTimeSeconds", "")
                ),
            },
            "golden_point_enabled": _parse_bool(
                match_row.get("roundsConfig.goldenPointEnabled", "")
            ),
            "golden_point_time": {
                "minutes": _parse_optional_int(
                    match_row.get("roundsConfig.goldenPointTimeMinutes", "")
                ),
                "seconds": _parse_optional_int(
                    match_row.get("roundsConfig.goldenPointTimeSeconds", "")
                ),
            },
            "max_gam_jeoms": _parse_optional_int(
                match_row.get("maxAllowedGamJeoms", "")
            ),
            "ceiling_score": _parse_optional_int(match_row.get("ceilingScore", "")),
            "differential_score": _parse_optional_int(
                match_row.get("differencialScore", "")
            ),
            "para_tkd_match": _parse_bool(match_row.get("paraTkdMatch", "")),
        },
        "athletes": {
            "blue": {
                "name": match_row.get("blueAthleteName", "").strip() or None,
                "wt_id": match_row.get("blueAthleteWtfId", "").strip() or None,
                "nation": match_row.get("blueAthleteFlagAbbreviation", "").strip()
                or None,
            },
            "red": {
                "name": match_row.get("redAthleteName", "").strip() or None,
                "wt_id": match_row.get("redAthleteWtfId", "").strip() or None,
                "nation": match_row.get("redAthleteFlagAbbreviation", "").strip()
                or None,
            },
        },
        "rounds_winners": _parse_rounds_winners(match_row.get("roundsWinners", "")),
        "tie_breaker": {
            "have_tie_breaker": _parse_bool(
                match_row.get("goldenPointTieBreakerInfo.haveTieBreaker", "")
            ),
            "blue": {
                "punches": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.bluePunches", "")
                ),
                "round_wins": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.blueRoundWins", "")
                ),
                "hits": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.blueHits", "")
                ),
                "penalties": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.bluePenalties", "")
                ),
                "para_tech_points": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.bluePARATechPoints", "")
                ),
            },
            "red": {
                "punches": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.redPunches", "")
                ),
                "round_wins": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.redRoundWins", "")
                ),
                "hits": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.redHits", "")
                ),
                "penalties": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.redPenalties", "")
                ),
                "para_tech_points": _parse_optional_int(
                    match_row.get("goldenPointTieBreakerInfo.redPARATechPoints", "")
                ),
            },
        },
        "punch_configuration": {
            "enabled": _parse_bool(match_row.get("punchEnabled", "")),
            "mode": match_row.get("punchMode", "").strip() or None,
            "near_miss_level": _parse_optional_int(
                match_row.get("punchNearMissLevel", "")
            ),
            "min_level": _parse_optional_int(match_row.get("minPunchLevel", "")),
        },
        "sensors": {
            "body": {
                "enabled": _parse_bool(match_row.get("bodySensorsEnabled", "")),
                "points": _parse_optional_int(match_row.get("bodyPoints", "")),
                "tech_points": _parse_optional_int(match_row.get("bodyTechPoints", "")),
                "min_level": _parse_optional_int(match_row.get("minBodyLevel", "")),
            },
            "head": {
                "enabled": _parse_bool(match_row.get("headSensorsEnabled", "")),
                "points": _parse_optional_int(match_row.get("headPoints", "")),
                "tech_points": _parse_optional_int(match_row.get("headTechPoints", "")),
                "min_level": _parse_optional_int(match_row.get("minHeadLevel", "")),
            },
        },
        "events": [event.to_dict() for event in canonical_events],
        "final_score": {"blue": score[0], "red": score[1]},
    }

    return canonical


def extract_command(args: argparse.Namespace) -> None:
    match_log_path = Path(args.match_log).resolve()
    LOGGER.info("Extracting canonical payload for %s", match_log_path)
    if match_log_path.name.endswith("-matchLogItems.csv"):
        match_log_path = match_log_path.with_name(
            match_log_path.name.replace("-matchLogItems.csv", "-matchLog.csv")
        )
    if not match_log_path.exists():
        raise SystemExit(f"match log not found: {match_log_path}")

    items_path = match_log_path.with_name(
        match_log_path.name.replace("-matchLog.csv", "-matchLogItems.csv")
    )
    if not items_path.exists():
        raise SystemExit(f"match log items not found: {items_path}")

    match_row = parse_match_log(match_log_path)
    item_rows = parse_match_log_items(items_path)
    canonical = _build_canonical_match(match_row, item_rows, match_log_path)

    output = json.dumps(canonical, indent=args.indent, ensure_ascii=False)
    if args.output:
        Path(args.output).write_text(output, encoding="utf-8")
    else:
        print(output)
    LOGGER.info("Extraction complete for %s", match_log_path.name)


def summarize_command(args: argparse.Namespace) -> None:
    root = Path(args.archive_root).expanduser().resolve()
    LOGGER.info("Summarizing archive at %s", root)
    if not root.exists():
        raise SystemExit(f"Archive root does not exist: {root}")

    victory_criteria: Counter[str] = Counter()
    winner_by: Counter[str] = Counter()
    phases: Counter[str] = Counter()
    punch_modes: Counter[str] = Counter()
    item_types: Counter[str] = Counter()
    tie_breaker_flags: Counter[str] = Counter()

    for match_log_path in root.rglob("*-matchLog.csv"):
        try:
            match_row = parse_match_log(match_log_path)
            items_path = match_log_path.with_name(
                match_log_path.name.replace("-matchLog.csv", "-matchLogItems.csv")
            )
            if not items_path.exists():
                continue
            item_rows = parse_match_log_items(items_path)
        except Exception as exc:  # pragma: no cover - diagnostics only
            if args.verbose:
                print(f"[warn] skipping {match_log_path}: {exc}")
            continue

        victory_criteria[match_row.get("matchVictoryCriteria", "").strip()] += 1
        winner_by[match_row.get("matchWinnerBy", "").strip()] += 1
        phases[match_row.get("phaseName", "").strip()] += 1
        punch_modes[match_row.get("punchMode", "").strip()] += 1
        tie_breaker_flags[
            match_row.get("goldenPointTieBreakerInfo.haveTieBreaker", "").strip()
        ] += 1

        for row in item_rows:
            item_types[row.get("matchLogItemType", "").strip()] += 1

    summary = {
        "matches_scanned": sum(victory_criteria.values()),
        "match_phases": _counter_to_sorted_list(phases),
        "victory_criteria": _counter_to_sorted_list(victory_criteria),
        "winner_by": _counter_to_sorted_list(winner_by),
        "punch_modes": _counter_to_sorted_list(punch_modes),
        "tie_breaker_have_flag": _counter_to_sorted_list(tie_breaker_flags),
        "event_types": _counter_to_sorted_list(item_types),
    }

    print(json.dumps(summary, indent=args.indent, ensure_ascii=False))
    LOGGER.info("Summarized %d matches", summary["matches_scanned"])


def _counter_to_sorted_list(counter: Counter[str]) -> List[Dict[str, Any]]:
    items = [
        {"value": key or "(empty)", "count": count}
        for key, count in counter.items()
    ]
    items.sort(key=lambda item: item["count"], reverse=True)
    return items


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Daedo GO2025 log parser for tournament reverse engineering."
    )
    parser.add_argument("--indent", type=int, default=2, help="JSON indentation level")
    parser.add_argument(
        "--log-level",
        default="INFO",
        help="Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL).",
    )
    subparsers = parser.add_subparsers(dest="command")

    extract_parser = subparsers.add_parser(
        "extract", help="Extract canonical match payload for a single bout."
    )
    extract_parser.add_argument(
        "match_log",
        help="Path to *-matchLog.csv (or *-matchLogItems.csv / prefix).",
    )
    extract_parser.add_argument(
        "-o",
        "--output",
        help="Optional output file path (stdout when omitted).",
    )
    extract_parser.set_defaults(func=extract_command)

    summarize_parser = subparsers.add_parser(
        "summarize", help="Scan an archive root and print dataset statistics."
    )
    summarize_parser.add_argument(
        "archive_root", help="Root directory that contains CourtXX folders."
    )
    summarize_parser.add_argument(
        "--verbose", action="store_true", help="Show warnings during traversal."
    )
    summarize_parser.set_defaults(func=summarize_command)

    return parser


def main(argv: Optional[Sequence[str]] = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)
    log_level = getattr(logging, str(args.log_level).upper(), logging.INFO)
    logging.basicConfig(
        level=log_level,
        format="[%(asctime)s] %(levelname)s - %(message)s",
    )
    if not hasattr(args, "func"):
        parser.print_help()
        return
    args.func(args)


if __name__ == "__main__":  # pragma: no cover
    main()
