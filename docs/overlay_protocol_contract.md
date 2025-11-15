# Overlay ↔ Protocol Contract

This document binds every scoreboard SVG element to the exact WT UDP protocol stream and field defined in `protocol/pss_v2.3.txt`. It is the canonical mapping used by the new scoreboard controllers.

Notation:
- **Stream** refers to the UDP prefix (e.g., `at1`, `mch`, `clk`, `wrd`).
- **Field** is the positional argument in the stream.
- Where logic is required (e.g., deriving gender labels), the rule references the protocol fields explicitly.

---

## Modern Overlay (`ui/public/assets/scoreboard/modern/modern_scoreboard.svg`)

| SVG ID | Protocol Stream / Field | Notes |
| --- | --- | --- |
| `athlete1Name` | `at1`: `athlete1_short` | Short name from Athletes stream |
| `athlete2Name` | `at1`: `athlete2_short` | — |
| `athlete1Score` | `sc1`: current score | Live score |
| `athlete2Score` | `sc2`: current score | — |
| `athlete1Rounds` | `wrd`: round winners (value == 1) | Count of rounds where winner value is `1` |
| `athlete2Rounds` | `wrd`: round winners (value == 2) | Count where winner value is `2` |
| `athlete1Warnings` | `wg1`: athlete1 warnings | Display as integer (max 5) |
| `athlete2Warnings` | `wg2`: athlete2 warnings | — |
| `athlete1Flag`, `athlete1FlagPlaceholder` | `at1`: `athlete1_country` | IOC code, used to select SVG flag |
| `athlete2Flag`, `athlete2FlagPlaceholder` | `at1`: `athlete2_country` | — |
| `roundNumber` | `rnd`: current round integer | Format per overlay (ordinal label) |
| `matchNumber` | `mch`: match number | Strip leading zeros |
| `matchTimer` | `clk`: time (`m:ss`) | Always display last clock value, even when stopped |
| `categoryGender` | `mch`: weight string | Rule: If weight starts with `M`, use “Men’s”; `W` becomes “Women’s”; then append `over` for `+`, `under` for `-` |
| `categoryWeight` | `mch`: weight string | Number after `+`/`-` |
| `injuryTimer` / `injury_x5F_bg` | `ij*`: time + action | Show/hide timer based on `show`/`hide`/`reset`; time from payload |

---

## Olympic Overlay (`ui/public/assets/scoreboard/olympic/olympic_scoreboard.svg`)

| SVG ID | Protocol Stream / Field | Notes |
| --- | --- | --- |
| `athlete1Country` | `at1`: `athlete1_country` | Text |
| `athlete2Country` | `at1`: `athlete2_country` | — |
| `athlete1Score` | `sc1` | — |
| `athlete2Score` | `sc2` | — |
| `athlete1Rounds` | `wrd` winner counts (value==1) | As modern |
| `athlete2Rounds` | `wrd` winner counts (value==2) | — |
| `athlete1Warnings` | `wg1` | — |
| `athlete2Warnings` | `wg2` | — |
| `athlete1Flag` (+ placeholder) | `at1`: `athlete1_country` | Render flag with IOC |
| `athlete2Flag` (+ placeholder) | `at1`: `athlete2_country` | — |
| `roundNumber` | `rnd` | Display as `ROUND {n}` |
| `matchNumber` | `mch`: match number | 4-digit text block |
| `matchTime` | `clk`: time | Always reflect most recent `clk` |
| `categoryGender` | `mch`: weight | Same derivation as modern |
| `categoryWeight` | `mch`: weight | Numeric portion |
| `injuryTime`, `injury_x5F_time_x5F_bg` | `ij*` | Show/hide per action |
| `logo_x5F_position1`/`logo_x5F_position2` | `ij*` | Toggle based on injury visibility |

---

## Arcade Overlay (`ui/public/assets/scoreboard/arcade/arcade_scoreboard.svg`)

| SVG ID | Protocol Stream / Field | Notes |
| --- | --- | --- |
| `athlete1Name` | `at1`: `athlete1_short` | Uppercase on render |
| `athlete2Name` | `at1`: `athlete2_short` | — |
| `athlete1Score` | `sc1` | — |
| `athlete2Score` | `sc2` | — |
| `athlete1_round_won` | `wrd`: count value==1 | — |
| `athlete2_round_won` | `wrd`: count value==2 | — |
| `athlete1Flag`, placeholder | `at1`: `athlete1_country` | — |
| `athlete2Flag`, placeholder | `at1`: `athlete2_country` | — |
| `athlete1_warning1-0/1` … `athlete1_warning5-0/1`, `athlete1_warnings_on/off` | `wg1` | Turn indicator on/off up to count |
| Equivalent `athlete2_*` warnings | `wg2` | — |
| `roundNumber` | `rnd` | Use ordinal text |
| `matchNumber` | `mch`: match number | Numeric center label |
| `matchTimer` / `matchTimer2` | `clk` | `matchTimer` for standard display, `matchTimer2` used when injury block visible |
| `injuryTime`, `injury_x5F_time_x5F_bg`, `time_x5F_block`, `time_x5F_and_x5F_injury`, `time_x5F_divider` | `ij*` | Toggle combined/standard timer groups when injury shown |
| `category_weight` | `mch`: weight string | Render “Men’s/Women’s over|under {kg} {value}` per rule |

---

Any SVG element not listed above is outside scoreboard scope and should remain untouched. Use this contract when implementing the protocol-driven controllers so every overlay updates directly from its source stream without heuristics.
