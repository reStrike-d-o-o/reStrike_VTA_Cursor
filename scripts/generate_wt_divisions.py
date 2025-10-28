#!/usr/bin/env python3
"""
Generate the WT divisions dataset JSON used by database migration 43.

The source data is provided as a semi-structured text file with the format:
    Gender | Discipline/Age | Weight/AgeCategory ; WT

This script classifies each row, assigns canonical discipline/age codes, and
produces a normalized JSON structure consumed by the Rust migration.
"""
from __future__ import annotations

import json
import re
from collections import OrderedDict
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Tuple

ROOT = Path(__file__).resolve().parents[1]
SOURCE = Path(r"C:\Users\Damjan\Documents\reStrikeVTA\tkd_divisions_official.txt")
TARGET = ROOT / "src-tauri" / "resources" / "wt_divisions.json"

# Translation map to sanitize the raw text
TRANS_TABLE = {
    ord("–"): "-",
    ord("—"): "-",
    ord("−"): "-",
    ord("‑"): "-",
    ord("“"): '"',
    ord("”"): '"',
    ord("’"): "'",
    ord("′"): "'",
    ord("″"): '"',
    ord("≤"): "<=",
    ord("≥"): ">=",
    ord("é"): "e",
    ord("á"): "a",
    ord("í"): "i",
    ord("ó"): "o",
    ord("ú"): "u",
    ord("ç"): "c",
    ord("ï"): "i",
    ord("ö"): "o",
    ord("ü"): "u",
}

# Predefined gender codes
GENDER_CODES = {
    "Men": ("M", "Men"),
    "Women": ("F", "Women"),
    "Mixed": ("MX", "Mixed"),
    "Mixed Pair": ("MP", "Mixed Pair"),
    "Mixed Team": ("MT", "Mixed Team"),
    "Women Team": ("WTM", "Women Team"),
    "Men Team": ("MTM", "Men Team"),
}

# Known age ranges for standard kyorugi
AGE_PRESETS = {
    "Senior": (17, None),
    "Junior": (15, 17),
    "Cadet": (12, 14),
    "U21": (18, 20),
    "Olympics": (17, None),
    "Youth Olympics": (14, 18),
    "Team Kyorugi": (None, None),
}

CODE_RE = re.compile(r"[^A-Za-z0-9]+")
AGE_RANGE_RE = re.compile(r"(\d+)\s*[-–]\s*(\d+)")
AGE_MIN_RE = re.compile(r"(\d+)\s*\+")


def normalize(text: str) -> str:
    return text.strip().translate(TRANS_TABLE).replace("\\u200b", "")


def make_code(text: str, prefix: str = "") -> str:
    cleaned = normalize(text)
    if not cleaned:
        cleaned = prefix or "CODE"
    code = CODE_RE.sub("_", cleaned.upper()).strip("_")
    if prefix and not code.startswith(prefix):
        code = f"{prefix}_{code}" if code else prefix
    if code and code[0].isdigit():
        code = f"_{code}"
    return (code or prefix or "CODE")[:48]


def parse_age_range(name: str) -> Tuple[Optional[int], Optional[int]]:
    cleaned = normalize(name)
    cleaned = cleaned.replace("(", " ").replace(")", " ")
    if cleaned in AGE_PRESETS:
        return AGE_PRESETS[cleaned]
    match = AGE_RANGE_RE.search(cleaned)
    if match:
        lower, upper = match.groups()
        return int(lower), int(upper)
    match = AGE_MIN_RE.search(cleaned)
    if match:
        lower = match.group(1)
        return int(lower), None
    return None, None


@dataclass
class Discipline:
    code: str
    name: str
    category: str


@dataclass
class AgeGroup:
    code: str
    name: str
    min_age: Optional[int]
    max_age: Optional[int]
    authority: str


@dataclass
class ClassEntry:
    gender_code: str
    discipline_code: str
    age_code: str
    code: str
    name: str
    min_kg: Optional[float]
    max_kg: Optional[float]
    authority: str


def detect_discipline(raw_second: str, raw_third: str) -> Tuple[Discipline, str]:
    second = normalize(raw_second)
    third = normalize(raw_third)
    lower = second.lower()

    if "poomsae recognized" in lower:
        return Discipline("POREC", "Poomsae Recognized", "poomsae"), third
    if "poomsae freestyle" in lower:
        return Discipline("POFR", "Poomsae Freestyle", "poomsae"), third
    if lower.startswith("para poomsae "):
        suffix = second.split()[-1].upper()
        code = suffix if suffix.startswith("PP") else f"PP{suffix}"
        return Discipline(code, second, "para_poomsae"), third
    if lower.startswith("para kyorugi "):
        suffix = second.split()[-1].upper()
        suffix = suffix.lstrip("K")
        code = f"PK{suffix}"
        return Discipline(code, second, "para_kyorugi"), second
    if "team kyorugi" in lower:
        return Discipline("TK", "Team Kyorugi", "team_kyorugi"), third
    return Discipline("KY", "Kyorugi", "kyorugi"), second


def is_weight_class(value: str) -> bool:
    check = normalize(value).lower()
    return "kg" in check or check.startswith(("+", "-")) or check.startswith((">=", "<="))


def parse_weight_range(value: str) -> Tuple[Optional[float], Optional[float]]:
    text = normalize(value).lower()
    text = text.replace("kilograms", "kg").replace("kgs", "kg")
    simple = text.split()
    if not text:
        return None, None

    def parse_float(segment: str) -> Optional[float]:
        segment = segment.replace("kg", "").strip()
        try:
            return float(segment)
        except ValueError:
            return None

    if text.startswith("-"):
        return None, parse_float(text[1:])
    if text.startswith("+"):
        return parse_float(text[1:]), None
    if text.startswith("<="):
        return None, parse_float(text[2:])
    if text.startswith(">="):
        return parse_float(text[2:]), None
    if "-" in text and "kg" in text:
        match = AGE_RANGE_RE.search(text)
        if match:
            lower, upper = match.groups()
            return float(lower), float(upper)
    if text.endswith("kg") and simple and simple[0].isdigit():
        value = parse_float(simple[0])
        return value, value
    return None, None


def main() -> None:
    genders: OrderedDict[str, Dict[str, str]] = OrderedDict()
    disciplines: OrderedDict[str, Dict[str, str]] = OrderedDict()
    age_groups: OrderedDict[str, Dict[str, Optional[int]]] = OrderedDict()
    classes: List[Dict[str, object]] = []

    with SOURCE.open("r", encoding="utf-8") as fh:
        header = fh.readline()
        for raw_line in fh:
            line = raw_line.strip()
            if not line or ";" not in line:
                continue
            main_part, org = line.split(";", 1)
            if normalize(org) != "WT":
                continue
            parts = [normalize(part) for part in main_part.split("|")]
            if len(parts) != 3:
                continue
            gender_name, second, third = parts

            if gender_name in GENDER_CODES:
                gender_code, gender_label = GENDER_CODES[gender_name]
            else:
                gender_code = make_code(gender_name)
                gender_label = gender_name
            genders.setdefault(gender_code, {"code": gender_code, "name": gender_label})

            discipline, age_name = detect_discipline(second, third)
            disciplines.setdefault(
                discipline.code,
                {
                    "code": discipline.code,
                    "name": discipline.name,
                    "category": discipline.category,
                },
            )

            age_code = make_code(age_name)
            min_age, max_age = parse_age_range(age_name) if not is_weight_class(third) else parse_age_range(age_name)
            age_groups.setdefault(
                age_code,
                {
                    "code": age_code,
                    "name": age_name,
                    "min_age": min_age,
                    "max_age": max_age,
                    "authority": "WT",
                },
            )

            class_code = make_code(third if not is_weight_class(third) else third)
            min_kg, max_kg = parse_weight_range(third) if is_weight_class(third) else (None, None)

            classes.append(
                {
                    "gender_code": gender_code,
                    "discipline_code": discipline.code,
                    "age_code": age_code,
                    "code": class_code,
                    "name": third,
                    "min_kg": min_kg,
                    "max_kg": max_kg,
                    "authority": "WT",
                }
            )

    payload = OrderedDict(
        [
            ("generated_at", datetime.now(timezone.utc).isoformat()),
            ("genders", list(genders.values())),
            ("disciplines", list(disciplines.values())),
            ("age_groups", list(age_groups.values())),
            ("classes", classes),
        ]
    )

    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"Wrote dataset to {TARGET} (classes={len(classes)})")


if __name__ == "__main__":
    main()
