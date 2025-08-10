export type Player = 1 | 2;

export type Action =
  | 'moveX'
  | 'punch'
  | 'body'
  | 'head'
  | 'tech_body'
  | 'tech_head'
  | 'warning'
  | 'hit_level';

export interface ButtonBinding {
  type: 'button';
  index: number; // gamepad.buttons[index]
}

export interface AxisBinding {
  type: 'axis';
  index: number; // gamepad.axes[index]
  deadzone?: number; // default 0.25
}

export type Binding = ButtonBinding | AxisBinding;

export interface PlayerMapping {
  moveX: AxisBinding; // movement left/right
  punch: ButtonBinding;
  body: ButtonBinding;
  head: ButtonBinding;
  tech_body: ButtonBinding;
  tech_head: ButtonBinding;
  warning: ButtonBinding;
  hit_level: ButtonBinding; // press to emit hit_level event (fixed level)
  gamepadIndex: number; // which connected gamepad controls this player
}

export interface GamepadMappingConfig {
  player1: PlayerMapping;
  player2: PlayerMapping;
  hitLevelValue: number; // level sent when hit_level button is pressed
}

export const defaultMapping = (): GamepadMappingConfig => ({
  player1: {
    gamepadIndex: 0,
    moveX: { type: 'axis', index: 0, deadzone: 0.25 },
    punch: { type: 'button', index: 0 }, // A
    body: { type: 'button', index: 1 }, // B
    head: { type: 'button', index: 2 }, // X
    tech_body: { type: 'button', index: 3 }, // Y
    tech_head: { type: 'button', index: 5 }, // RB
    warning: { type: 'button', index: 4 }, // LB
    hit_level: { type: 'button', index: 6 }, // LT (digital on some pads)
  },
  player2: {
    gamepadIndex: 1,
    moveX: { type: 'axis', index: 0, deadzone: 0.25 },
    punch: { type: 'button', index: 0 },
    body: { type: 'button', index: 1 },
    head: { type: 'button', index: 2 },
    tech_body: { type: 'button', index: 3 },
    tech_head: { type: 'button', index: 5 },
    warning: { type: 'button', index: 4 },
    hit_level: { type: 'button', index: 6 },
  },
  hitLevelValue: 25,
});

const STORAGE_KEY = 'arcade_gamepad_mapping_v1';

export function loadMapping(): GamepadMappingConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return defaultMapping();
    const parsed = JSON.parse(raw);
    return { ...defaultMapping(), ...parsed };
  } catch {
    return defaultMapping();
  }
}

export function saveMapping(cfg: GamepadMappingConfig) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(cfg));
  } catch {}
}

export function listConnectedGamepads(): Gamepad[] {
  if (typeof navigator === 'undefined' || !navigator.getGamepads) return [] as any;
  return Array.from(navigator.getGamepads ? navigator.getGamepads() : []).filter(Boolean) as Gamepad[];
}

export function isButtonPressed(gp: Gamepad, index: number): boolean {
  const b = gp.buttons[index];
  return !!b && (b.pressed || b.value > 0.5);
}

export function axisValue(gp: Gamepad, index: number): number {
  return gp.axes[index] ?? 0;
}


