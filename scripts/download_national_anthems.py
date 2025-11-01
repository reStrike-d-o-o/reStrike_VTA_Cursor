#!/usr/bin/env python3
"""
Bulk downloader for national anthem audio assets.

Workflow:
1. Read a list of MP3 URLs (one per line).
2. Download each MP3 into the configured output directory.
3. Fetch the companion HTML page to extract the canonical country name.
4. Rename the downloaded file to the country name, then to the matching IOC code,
   using local IOC mapping reports.
5. Log any failures (download errors, HTML parsing problems, missing IOC mapping)
   to a Markdown file for later review.
"""

from __future__ import annotations

import argparse
import html
import logging
import re
import sys
import time
import shutil
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Set
from urllib.parse import unquote, urljoin, urlparse

import requests

# Default HTTP headers emulate a modern browser to avoid being blocked by mod_security.
HTML_HEADERS = {
    "User-Agent": (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
        "AppleWebKit/537.36 (KHTML, like Gecko) "
        "Chrome/124.0.0.0 Safari/537.36"
    ),
    "Accept": (
        "text/html,application/xhtml+xml,application/xml;q=0.9,"
        "image/avif,image/webp,*/*;q=0.8"
    ),
    "Accept-Language": "en-US,en;q=0.9",
    "Accept-Encoding": "gzip, deflate, br",
    "Referer": "https://nationalanthems.info/",
}

MP3_HEADERS = {
    "User-Agent": HTML_HEADERS["User-Agent"],
    "Accept": "audio/mpeg,audio/*;q=0.9,*/*;q=0.8",
    "Accept-Language": HTML_HEADERS["Accept-Language"],
    "Referer": HTML_HEADERS["Referer"],
}


@dataclass
class CountryPage:
    title: str
    mp3_urls: List[str]


@dataclass
class AnthemResult:
    url: str
    success: bool
    final_path: Optional[Path]
    reason: Optional[str] = None
    country_name: Optional[str] = None
    ioc_code: Optional[str] = None
    version_index: Optional[int] = None


VERSION_FILENAME_RE = re.compile(r"^(?P<ioc>[a-z]{3})-v(?P<version>\d+)\.mp3$", re.IGNORECASE)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Download national anthem MP3s and rename them to IOC codes."
    )
    parser.add_argument(
        "--url-list",
        type=Path,
        default=Path(r"C:\Users\Damjan\Downloads\anthems_audio\visited_pages.txt"),
        help="Path to text file containing anthem MP3 URLs (one per line).",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path(
            r"C:\Users\Damjan\source\repos\reStrikeVTA_WO\reStrike_VTA_Cursor"
            r"\ui\public\assets\anthems"
        ),
        help="Directory where MP3 files will be stored.",
    )
    parser.add_argument(
        "--failed-log",
        type=Path,
        default=Path("failed_links.md"),
        help="Markdown file where failed downloads or lookups are recorded.",
    )
    parser.add_argument(
        "--official-report",
        type=Path,
        default=Path(
            "ui/public/assets/flags/OFFICIAL_IOC_FLAGS_DOWNLOAD_REPORT.md"
        ),
        help="Primary IOC mapping markdown report.",
    )
    parser.add_argument(
        "--fallback-report",
        type=Path,
        default=Path("ui/public/assets/flags/IOC_FLAGS_DOWNLOAD_REPORT.md"),
        help="Fallback IOC mapping markdown report.",
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=0.6,
        help="Delay in seconds between successive downloads (helps avoid rate limits).",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Re-download files even if the final IOC file already exists.",
    )
    return parser.parse_args()


def load_url_list(path: Path) -> List[str]:
    if not path.exists():
        raise FileNotFoundError(f"URL list not found: {path}")
    urls: List[str] = []
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            url = line.strip()
            if not url or not url.lower().endswith(".mp3"):
                continue
            urls.append(url)
    return urls


def read_ioc_mapping(paths: Iterable[Path]) -> Dict[str, str]:
    mapping: Dict[str, str] = {}
    for path in paths:
        if not path or not path.exists():
            logging.warning("IOC mapping file missing: %s", path)
            continue
        with path.open("r", encoding="utf-8") as handle:
            for raw_line in handle:
                line = raw_line.strip()
                if not line.startswith("|"):
                    continue
                cells = [cell.strip() for cell in line.strip("|").split("|")]
                if len(cells) < 2:
                    continue
                code = cells[0].strip().upper()
                name = cells[1].strip()
                if not code or code == "IOC CODE":
                    continue
                for variant in generate_name_variants(name):
                    key = normalize_country_key(variant)
                    mapping.setdefault(key, code)
    if not mapping:
        raise RuntimeError("No IOC entries parsed from mapping reports.")
    return mapping


def generate_name_variants(name: str) -> Set[str]:
    variants: Set[str] = set()
    cleaned = html.unescape(name).strip()
    if not cleaned:
        return variants
    variants.add(cleaned)

    # Drop bracketed and parenthetical suffixes.
    without_paren = re.sub(r"\s*\([^)]*\)", "", cleaned).strip()
    if without_paren:
        variants.add(without_paren)

    # Handle comma-separated "Country, Descriptor" patterns.
    if "," in cleaned:
        parts = [p.strip() for p in cleaned.split(",") if p.strip()]
        if len(parts) == 2:
            variants.add(f"{parts[1]} {parts[0]}")
            variants.add(parts[0])
            variants.add(parts[1])

    # Expand common synonyms manually.
    synonyms = {
        "Korea (Republic of)": "South Korea",
        "Korea (Democratic People's Republic of)": "North Korea",
        "Russian Federation": "Russia",
        "United States of America": "United States",
        "Viet Nam": "Vietnam",
        "Ivory Coast": "Cote d'Ivoire",
        "Congo, Democratic Republic of the": "Democratic Republic of the Congo",
        "Congo, Republic of the": "Republic of the Congo",
        "Lao People's Democratic Republic": "Laos",
        "Syrian Arab Republic": "Syria",
        "Iran (Islamic Republic of)": "Iran",
        "Moldova, Republic of": "Moldova",
        "Tanzania, United Republic of": "Tanzania",
        "Bolivia (Plurinational State of)": "Bolivia",
        "Micronesia (Federated States of)": "Micronesia",
        "Marshall Islands": "Marshall Islands",
        "Venezuela (Bolivarian Republic of)": "Venezuela",
        "Palestine, State of": "Palestine",
        "United Kingdom of Great Britain and Northern Ireland": "United Kingdom",
        "Hong Kong, China": "Hong Kong",
        "Macao, China": "Macau",
    }
    key = cleaned.replace("–", "-")
    if key in synonyms:
        variants.add(synonyms[key])
    for value in synonyms.values():
        variants.add(value)
    return variants


def normalize_country_key(name: str) -> str:
    text = html.unescape(name)
    text = re.sub(r"\s*\([^)]*\)", "", text)
    text = text.replace("&", " and ")
    text = text.replace("–", " ")
    text = text.replace("-", " ")
    text = text.replace("'", "")
    text = text.replace("’", "")
    text = text.replace("`", "")
    text = text.lower()
    text = unicodedata.normalize("NFKD", text)
    text = "".join(ch for ch in text if not unicodedata.combining(ch))
    text = re.sub(r"[^a-z0-9 ]+", " ", text)
    text = re.sub(r"\bthe\b", " ", text)
    text = re.sub(r"\s+", " ", text)
    return text.strip()


def sanitize_filename(name: str) -> str:
    text = html.unescape(name)
    text = re.sub(r"[\\/:*?\"<>|]", " ", text)
    text = re.sub(r"\s+", " ", text).strip()
    if not text:
        text = "unknown"
    return text


def normalize_mp3_url(url: str) -> str:
    parsed = urlparse(url)
    path = unquote(parsed.path or "")
    return path.strip().lower()


def existing_version_max(output_dir: Path, ioc_code: str) -> int:
    prefix = ioc_code.lower()
    max_version = 0
    for candidate in output_dir.glob(f"{prefix}-v*.mp3"):
        match = VERSION_FILENAME_RE.match(candidate.name)
        if match and match.group("ioc").lower() == prefix:
            max_version = max(max_version, int(match.group("version")))
    return max_version


def ensure_directory(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)


def download_file(session: requests.Session, url: str, destination: Path) -> None:
    with session.get(url, headers=MP3_HEADERS, stream=True, timeout=60) as response:
        if response.status_code != 200:
            raise RuntimeError(f"HTTP {response.status_code}")
        total = int(response.headers.get("Content-Length", 0))
        with destination.open("wb") as handle:
            for chunk in response.iter_content(chunk_size=64 * 1024):
                if not chunk:
                    continue
                handle.write(chunk)
        if total and destination.stat().st_size != total:
            raise RuntimeError("Downloaded size mismatch")


def fetch_country_page(session: requests.Session, url: str) -> CountryPage:
    response = session.get(url, headers=HTML_HEADERS, timeout=30)
    if response.status_code != 200:
        raise RuntimeError(f"HTML request failed with HTTP {response.status_code}")
    text = response.text
    match = re.search(r"<title>(.*?)</title>", text, re.IGNORECASE | re.DOTALL)
    if not match:
        raise RuntimeError("No <title> found in HTML")
    title = html.unescape(match.group(1)).strip()
    if " – " in title:
        title = title.split(" – ", 1)[0].strip()
    elif " - " in title:
        title = title.split(" - ", 1)[0].strip()
    title = re.sub(r"\s+", " ", title)

    mp3_candidates: List[str] = []
    for href_match in re.finditer(r"""href=["']([^"']+\.mp3)["']""", text, re.IGNORECASE):
        href = html.unescape(href_match.group(1))
        absolute = urljoin(url, href)
        mp3_candidates.append(absolute)

    ordered_links: List[str] = []
    seen: Set[str] = set()
    for candidate in mp3_candidates:
        norm = normalize_mp3_url(candidate)
        if not norm or norm in seen:
            continue
        seen.add(norm)
        ordered_links.append(candidate)

    return CountryPage(title=title, mp3_urls=ordered_links)


def resolve_html_url(mp3_url: str) -> List[str]:
    base = mp3_url.rsplit(".mp3", 1)[0]
    return [f"{base}.htm", f"{base}.html", base]


def map_country_to_ioc(country: str, mapping: Dict[str, str]) -> Optional[str]:
    key = normalize_country_key(country)
    if key in mapping:
        return mapping[key]
    # Attempt relaxed search by removing words like "state", "republic".
    relaxed = re.sub(
        r"\b(state|states|republic|kingdom|federation|federal|plurinational|democratic|arab|people|united)\b",
        " ",
        key,
    )
    relaxed = re.sub(r"\s+", " ", relaxed).strip()
    if relaxed and relaxed in mapping:
        return mapping[relaxed]
    return None


def process_url(
    session: requests.Session,
    url: str,
    output_dir: Path,
    mapping: Dict[str, str],
    overwrite: bool = False,
) -> AnthemResult:
    parsed = urlparse(url)
    filename = Path(parsed.path).name
    if not filename:
        return AnthemResult(url=url, success=False, final_path=None, reason="No filename in URL")
    parsed_name = Path(filename)
    if parsed_name.suffix.lower() != ".mp3":
        return AnthemResult(url=url, success=False, final_path=None, reason="Not an MP3 link")
    temp_path = output_dir / parsed_name.name

    try:
        download_file(session, url, temp_path)
    except Exception as exc:
        if temp_path.exists():
            temp_path.unlink(missing_ok=True)
        return AnthemResult(url=url, success=False, final_path=None, reason=f"Download failed: {exc}")

    country_page: Optional[CountryPage] = None
    html_errors: List[str] = []
    for html_url in resolve_html_url(url):
        try:
            country_page = fetch_country_page(session, html_url)
            break
        except Exception as exc:
            html_errors.append(f"{html_url}: {exc}")
            continue

    if not country_page:
        temp_path.unlink(missing_ok=True)
        return AnthemResult(
            url=url,
            success=False,
            final_path=None,
            reason="; ".join(["HTML title fetch failed"] + html_errors),
        )

    sanitized = sanitize_filename(country_page.title)
    interim_path = output_dir / f"{sanitized}.mp3"
    if interim_path.exists():
        interim_path.unlink()
    temp_path.rename(interim_path)

    ioc_code = map_country_to_ioc(country_page.title, mapping)
    if not ioc_code:
        return AnthemResult(
            url=url,
            success=False,
            final_path=interim_path,
            reason=f"No IOC mapping for '{country_page.title}'",
            country_name=country_page.title,
        )

    normalized_links = [normalize_mp3_url(link) for link in country_page.mp3_urls]
    current_norm = normalize_mp3_url(url)
    if current_norm in normalized_links:
        version_index = normalized_links.index(current_norm) + 1
    else:
        version_index = existing_version_max(output_dir, ioc_code) + 1
        logging.debug(
            "URL %s not located in HTML list for %s; assigning version %d",
            url,
            country_page.title,
            version_index,
        )

    version_filename = f"{ioc_code.lower()}-v{version_index}.mp3"
    version_path = output_dir / version_filename
    if version_path.exists():
        if overwrite:
            version_path.unlink()
        else:
            interim_path.unlink(missing_ok=True)
            return AnthemResult(
                url=url,
                success=True,
                final_path=version_path,
                country_name=country_page.title,
                ioc_code=ioc_code,
                version_index=version_index,
            )

    interim_path.rename(version_path)

    return AnthemResult(
        url=url,
        success=True,
        final_path=version_path,
        country_name=country_page.title,
        ioc_code=ioc_code,
        version_index=version_index,
    )


def write_failure_report(failed_log: Path, failures: List[AnthemResult]) -> None:
    if not failures:
        if failed_log.exists():
            failed_log.unlink()
        return
    lines = ["# Failed Anthem Downloads", ""]
    for result in failures:
        reason = result.reason or "Unknown error"
        lines.append(f"- {result.url} - {reason}")
    failed_log.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    logging.basicConfig(
        level=logging.INFO,
        format="[%(asctime)s] %(levelname)s - %(message)s",
        datefmt="%H:%M:%S",
    )

    ensure_directory(args.output_dir)

    try:
        url_list = load_url_list(args.url_list)
    except Exception as exc:
        logging.error("Failed to load URL list: %s", exc)
        return 1

    if not url_list:
        logging.error("No MP3 URLs found in %s", args.url_list)
        return 1

    try:
        mapping = read_ioc_mapping([args.official_report, args.fallback_report])
    except Exception as exc:
        logging.error("Failed to read IOC mapping files: %s", exc)
        return 1

    session = requests.Session()
    session.headers.update({"Connection": "keep-alive"})

    successes: List[AnthemResult] = []
    failures: List[AnthemResult] = []

    for index, url in enumerate(url_list, start=1):
        logging.info("Processing %s (%d/%d)", url, index, len(url_list))
        result = process_url(
            session=session,
            url=url,
            output_dir=args.output_dir,
            mapping=mapping,
            overwrite=args.overwrite,
        )
        if result.success:
            logging.info(
                "Success: %s -> %s (IOC: %s)",
                url,
                result.final_path,
                result.ioc_code or "unknown",
            )
            successes.append(result)
        else:
            logging.warning("Failed: %s (%s)", url, result.reason)
            failures.append(result)
        time.sleep(max(args.delay, 0.0))

    write_failure_report(args.failed_log, failures)

    # Ensure the latest version for each IOC is copied to the canonical filename.
    latest_by_ioc: Dict[str, AnthemResult] = {}
    for result in successes:
        if not result.ioc_code or result.version_index is None:
            continue
        key = result.ioc_code.upper()
        current = latest_by_ioc.get(key)
        if current is None or (result.version_index or 0) > (current.version_index or 0):
            latest_by_ioc[key] = result

    for ioc_code, latest in latest_by_ioc.items():
        current_path = args.output_dir / f"{ioc_code}.mp3"
        if current_path.exists():
            current_path.unlink()
        shutil.copy2(latest.final_path, current_path)
        logging.info(
            "Set current anthem for %s to version v%d (%s)",
            ioc_code,
            latest.version_index or 0,
            current_path.name,
        )

    logging.info("Completed: %d success, %d failures.", len(successes), len(failures))
    if failures:
        logging.info("Review failure details in %s", args.failed_log)
    return 0 if not failures else 2


if __name__ == "__main__":
    sys.exit(main())
