# WT UDP Protocol Summary (v2.3)

Source: `protocol/pss_v2.3.txt` (2024). This document lists every stream, its meaning, required arguments, optional arguments, and example payloads. Use it as the ground-truth contract for the scoreboard rewrite; no assumptions outside this spec.

| Stream | Purpose | Required Arguments (in order) | Optional Arguments | Example Payload |
| --- | --- | --- | --- | --- |
| `pt1`, `pt2` | Points awarded per athlete | point-code (1 punch, 2 body, 3 head, 4 technical body, 5 technical head) | — | `pt1;3;` |
| `hl1`, `hl2` | Hit level (1–100) | hit level integer | — | `hl2;50;` |
| `wg1/wg2` | Gam-jeom / warnings | `wg1;<a1>;wg2;<a2>` | — | `wg1;1;wg2;2;` |
| `ij0`, `ij1`, `ij2` | Injury timer (unassigned, athlete1, athlete2) | `time (m:ss)` | `show`, `hide`, `reset` | `ij2;0:44;hide;` |
| `ch0`, `ch1`, `ch2` | Challenge / IVR status (ref, ath1, ath2) | `state` (1 accept, 0 deny, -1 cancel) | `result` (1 won, 0 lost) | `ch1;1;0;` |
| `brk` | Break (between rounds) | clock string or seconds | `stop`, `stopEnd`, etc., depending on vendor text | `brk;0:59;` |
| `wrd` | Winner of each round | `rd1;<winner>` `rd2;<winner>` `rd3;<winner>` (winner: 0 none, 1 blue, 2 red) | — | `wrd;rd1;2;rd2;1;rd3;0;` |
| `wmh` | Winner metadata | winner long name, classification points (optional) | — | `wmh;Nicolas DESMOND;2-0 PTF;` |
| `at1 ... at2` | Athlete info | `at1;<short1>;<long1>;<country1>;at2;<short2>;<long2>;<country2>` | — | `at1;N. DESMOND;Nicolas DESMOND;MRN;at2;M. THIBAULT;Marcel THIBAULT;SUI;` |
| `mch` | Match configuration | `number;category;weight;rounds;color1_bg;color1_fg;color2_bg;color2_fg;match_id;division;total_rounds;round_duration;countdown_type;count_up;format` | — | `mch;101;Round of 16;M- 80 kg;1;#0000ff;#FFFFFF;#ff0000;#FFFFFF;a14ddd5c;Senior;3;120;cntDown;18;1;` |
| `s11/s21...s13/s23` | Per-round scores | `score` | — | `s11;3;` |
| `sc1`, `sc2` | Current total scores | `current score` | — | `sc1;3;` |
| `avt` | Athlete video time | `value` | — | `avt;0;` |
| `clk` | Match clock | `time (m:ss)` | `start`, `stop` | `clk;2:00;start;` |
| `rnd` | Current round | `round number` | — | `rnd;1;` |
| `rdy` | Fight ready status | `FightReady` literal | — | `rdy;FightReady;` |
| `pre` | Fight loaded status | `FightLoaded` literal | — | `pre;FightLoaded;` |
| `win` | Winner color | `BLUE` or `RED` | — | `win;BLUE;` |
| `pt*`, `hl*`, `avt` etc. | Additional telemetry beyond scoreboard scope but available if needed. | | | |

Example flow (abridged):
1. `Udp Port XXXX connected;`
2. `pre;FightLoaded;`
3. `at1;...;at2;...;`
4. `mch;...;`
5. `wg1;0;wg2;0;`
6. `wrd;rd1;0;rd2;0;rd3;0;`
7. `sc1;0;sc2;0;`
8. `clk;2:00;`
9. `rnd;1;`
10. `rdy;FightReady;`

During match:
- Clock ticks via repeated `clk;mm:ss;` packets, plus `start`/`stop`.
- Scores update via `s**` and `sc*`.
- Round winners update via `wrd`.
- Injury via `ij*`.
- Breaks via `brk`.
- Winner via `win` + `wmh`.

Use this summary to map every overlay element directly to its source stream and arguments.
