# Overlay Field Mapping Template

Fill in the **Data Field(s)** column for every SVG element below using the exact field names emitted by the PSS/UDP events (e.g. `athlete1_long`, `athlete1_score`, `current_round`). Once you finish, save this file and let Codex know so the code can be wired directly to your mapping.

---

## Modern Overlay (`ui/public/assets/scoreboard/modern/modern_scoreboard.svg`)

| SVG ID | Data Field(s) |
| --- | --- |
| `athlete1Name` | `athlete1_short`|
| `athlete1Score` |`athlete1_score` |
| `athlete1Rounds` | UDP Event: WinnerRounds { round1_winner: 1, round2_winner: 0, round3_winner: 0 }| - rule value of round{N}_winner can be 0, 1, 2. If the value of round{N}_winner is different than 0 and {N} == `roundNumber` then add 1 to athlete determined by the value of round{N}_winner - 1 athlete1Rounds, 2 athlete2Rounds
| `athlete1Warnings` |`athlete1_warnings` |
| `athlete1Flag` / `athlete1FlagPlaceholder` |`athlete1_country` |
| `athlete2Name` |`athlete2_short` |
| `athlete2Score` |`athlete2_score` |
| `athlete2Rounds` | UDP Event: WinnerRounds { round1_winner: 1, round2_winner: 0, round3_winner: 0 }| - rule value of round{N}_winner can be 0, 1, 2. If the value of round{N}_winner is different than 0 and {N} == `roundNumber` then add 1 to athlete determined by the value of round{N}_winner - 1 athlete1Rounds, 2 athlete2Rounds
| `athlete2Warnings` |`athlete2_warnings` |
| `athlete2Flag` / `athlete2FlagPlaceholder` |`athlete2_country` |
| `roundNumber` | `current_round`|
| `matchNumber` |`number` |
| `matchTimer` |`time` |
| `categoryGender` |`weight` | (rule: if it starts with M you write Men's, if it starts with W you write Women's, then if after space comes + append word over, in case of - append word under)
| `categoryWeight` |`weight` | (rule: you just use text after + or - . Example +78kg, use just 78kg)
| `injuryTimer` |`time` | (rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer)
| `injury_x5F_bg` | (rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer)|
| `matchInfo_x5F_bg` (text source or visibility rule) | |

---

## Olympic Overlay (`ui/public/assets/scoreboard/olympic/olympic_scoreboard.svg`)

| SVG ID | Data Field(s) |
| --- | --- |
| `athlete1Country` |`athlete1_country` |
| `athlete1Score` | `athlete1_score`|
| `athlete1Rounds` | UDP Event: WinnerRounds { round1_winner: 1, round2_winner: 0, round3_winner: 0 }| - rule value of round{N}_winner can be 0, 1, 2. If the value of round{N}_winner is different than 0 and {N} == `roundNumber` then add 1 to athlete determined by the value of round{N}_winner - 1 athlete1Rounds, 2 athlete2Rounds
| `athlete1Warnings` |`athlete1_warnings` |
| `athlete1Flag` / `athlete1FlagPlaceholder` | `athlete1_country`|
| `athlete2Country` | `athlete2_country`|
| `athlete2Score` |`athlete2_score`|
| `athlete2Rounds` | UDP Event: WinnerRounds { round1_winner: 1, round2_winner: 0, round3_winner: 0 }| - rule value of round{N}_winner can be 0, 1, 2. If the value of round{N}_winner is different than 0 and {N} == `roundNumber` then add 1 to athlete determined by the value of round{N}_winner - 1 athlete1Rounds, 2 athlete2Rounds
| `athlete2Warnings` | `athlete2_warnings`|
| `athlete2Flag` / `athlete2FlagPlaceholder` | `athlete2_country`|
| `roundNumber` |`current_round` |
| `matchNumber` | `number`|
| `matchTime` |`time` |
| `categoryGender` |`weight` | (rule: if it starts with M you write Men's, if it starts with W you write Women's, then if after space comes + append word over, in case of - append word under) |
| `categoryWeight` | `weight` | (rule: you just use text after + or - . Example +78kg, use just 78kg)
| `injuryTime` |`time` | (rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer)
| `injury_x5F_time_x5F_bg` | (rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer) |
| `logo_x5F_position1` (when visible?) |always |
| `logo_x5F_position2` (when visible?) |always |

---

## Arcade Overlay (`ui/public/assets/scoreboard/arcade/arcade_scoreboard.svg`)

| SVG ID | Data Field(s) |
| --- | --- |
| `athlete1Name` |`athlete1_short` |
| `athlete1Score` |`athlete1_score` |
| `athlete1_round_won` | UDP Event: WinnerRounds { round1_winner: 1, round2_winner: 0, round3_winner: 0 }| - rule value of round{N}_winner can be 0, 1, 2. If the value of round{N}_winner is different than 0 and {N} == `roundNumber` then add 1 to athlete determined by the value of round{N}_winner - 1 athlete1_round_won, 2 athlete1_round_won
| `athlete1Flag` / `athlete1FlagPlaceholder` |`athlete1_country` |
| `athlete1_warning1-0/1` … `athlete1_warning5-0/1`, `athlete1_warnings_on/off` |`athlete1_warnings` |
| `athlete1_power1` … `athlete1_power10`, `athlete1_power_levels` | |
| `athlete2Name` |`athlete2_short` |
| `athlete2Score` | `athlete2_score`|
| `athlete2_round_won` | UDP Event: WinnerRounds { round1_winner: 1, round2_winner: 0, round3_winner: 0 }| - rule value of round{N}_winner can be 0, 1, 2. If the value of round{N}_winner is different than 0 and {N} == `roundNumber` then add 1 to athlete determined by the value of round{N}_winner - 1 athlete1_round_won, 2 athlete1_round_won
| `athlete2Flag` / `athlete2FlagPlaceholder` |`athlete2_country` |
| `athlete2_warning1-0/1` … `athlete2_warning5-0/1`, `athlete2_warnings-on/off` |`athlete2_warnings` |
| `athlete2_power1` … `athlete2_power10`, `athlete2_power-levels` | |
| `roundNumber` |`current_round` |
| `matchNumber` | `number`|
| `matchTimer` |`time` |
| `matchTimer2` |`time` |
| `injuryTime` |`time` |(rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer)
| `injury_x5F_time_x5F_bg` |(rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer) |
| `category_weight` |`weight` | (rule: if it starts with M you write Men's, if it starts with W you write Women's, then if after space comes + append word over, in case of - append word under, then you just use text after + or - . Example +78kg, use just 78kg)
| `time_x5F_block`, `time_x5F_and_x5F_injury`, `time_x5F_divider` (rule: UDP Event: Injury { athlete: 0, time: "01:00", action: Some("show") } -> show the injury timer and start countdown, UDP Event: Injury { athlete: 0, time: "00:39", action: Some("hide") } -> hide the injury timer) |`time` |

---

### Available Data Fields (copy/paste as needed)

- `athlete1_long`, `athlete1_short`, `athlete1_country`, `athlete2_long`, `athlete2_short`, `athlete2_country`
- `athlete1_score`, `athlete2_score`
- `athlete1_warnings`, `athlete2_warnings`
- `round1_winner`, `round2_winner`, `round3_winner`
- `current_round`
- `time`, `action` (from `clock`, `break`, `timeout`, etc.)
- `number`, `category`, `division`, `weight`, `round_duration`, `countdown_type`, `count_up`
- `value` (supremacy)
- Any additional `structured_data` fields emitted in your events

After completing the table, let Codex know so the implementation can be updated accordingly.
