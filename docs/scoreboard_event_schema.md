# Scoreboard Event Schema

Source of truth: backend `PssProtocol` (WT UDP v2.3). Frontend overlays receive normalized events via the WebSocket bridge. This schema defines the TypeScript interfaces the new `scoreboard-core` controller will use and emit.

## Raw WebSocket Payload

```ts
interface PssEventPayload {
  type: PssEventType; // string literals below
  timestamp: number; // ms since epoch
  raw?: string;      // optional raw UDP packet for debugging
  data: Record<string, unknown>; // typed extension per event type
}
```

`type` values map directly to the enum variants from `PssProtocol`:

- `points`
- `hit_level`
- `warnings`
- `injury`
- `challenge`
- `break`
- `winner_rounds`
- `winner` (wmh/win combined)
- `athletes`
- `match_config`
- `scores`
- `current_scores`
- `clock`
- `round`
- `fight_loaded`
- `fight_ready`
- `supremacy`
- `video_time`
- `raw`

## Normalized State Model

`scoreboard-core` maintains an overlay-agnostic state derived from the above events. Each overlay subscribes to updates instead of directly parsing raw events.

```ts
interface ScoreboardState {
  match: MatchInfo;
  clock: ClockState;
  injury: InjuryState;
  scores: ScoreState;
  rounds: RoundState;
  warnings: WarningState;
  metadata: MetadataState;
}
```

### MatchInfo
```ts
interface MatchInfo {
  number?: string;
  category?: string;
  weight?: string;
  division?: string;
  format?: number;
  colors?: {
    blueBg: string;
    blueFg: string;
    redBg: string;
    redFg: string;
  };
  athletes?: {
    blue: AthleteInfo;
    red: AthleteInfo;
  };
}

interface AthleteInfo {
  short?: string;
  long?: string;
  country?: string;
}
```

### ClockState
```ts
interface ClockState {
  time: string;      // "m:ss"
  lastAction?: 'start' | 'stop';
}
```

### InjuryState
```ts
interface InjuryState {
  visible: boolean;
  athlete: 0 | 1 | 2; // 0=neutral
  time: string;       // last reported value
}
```

### ScoreState
```ts
interface ScoreState {
  current: {
    blue: number;
    red: number;
  };
  byRound: {
    blue: [number, number, number];
    red: [number, number, number];
  };
}
```

### RoundState
```ts
interface RoundState {
  current: number;
  winners: (0 | 1 | 2)[]; // length 3; values from winner_rounds
}
```

### WarningState
```ts
interface WarningState {
  blue: number;
  red: number;
}
```

### MetadataState
```ts
interface MetadataState {
  fightLoaded: boolean;
  fightReady: boolean;
  supremacy?: number;
  videoTime?: number;
  lastEventTs?: number;
}
```

## Update Rules / Precedence

1. **Match config reset:** When a `match_config` event arrives, clear `scores`, `warnings`, `rounds`, `clock`, and `injury` to defaults before applying new values.
2. **Clock events:** `clock` packets always override the `clock.time` string. `action` controls `lastAction` and determines overlay behaviours (e.g., injury toggles do not affect it).
3. **Injury:** `injury` events update `injury.time` and set `visible` based on action (`show`/`hide`/`reset`). `athlete` field picks which overlay section highlights.
4. **Scores:** `current_scores` overrides `ScoreState.current`. `scores` (round breakdown) updates `ScoreState.byRound`. Both maintain previous values when fields omitted.
5. **Warnings:** `warnings` event sets counts for both athletes. Packets missing one side default to prior value.
6. **Winner rounds:** Always replace the `winners` array using the latest payload.
7. **Athletes:** `athletes` event updates `MatchInfo.athletes` only; other fields persist.
8. **FightLoaded/Ready:** Set booleans and trigger overlay resets as needed (handled in controller).

This schema becomes the contract between the backend WebSocket payloads and the new overlay controllers. `scoreboard-core` should expose read-only accessors + event callbacks whenever any subset changes.
