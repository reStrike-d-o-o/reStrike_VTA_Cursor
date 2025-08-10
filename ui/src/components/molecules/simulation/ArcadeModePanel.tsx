import React, { useEffect, useRef, useState } from 'react';
import { useSimulationStore } from '../../../stores/simulationStore';
import { usePssMatchStore } from '../../../stores/pssMatchStore';
import Button from '../../atoms/Button';
import ArcadeBindingsPanel from './ArcadeBindingsPanel';
import { loadMapping, listConnectedGamepads, isButtonPressed, axisValue } from './gamepad';

type Fighter = {
  x: number;
  y: number;
  dir: 1 | -1; // 1 right, -1 left
  color: 'blue' | 'red';
};

const WIDTH = 720;
const HEIGHT = 260;

const ArcadeModePanel: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const { sendManualEvent } = useSimulationStore();
  const [running, setRunning] = useState(true);

  const [b, setB] = useState<Fighter>({ x: 120, y: 180, dir: 1, color: 'blue' });
  const [r, setR] = useState<Fighter>({ x: 600, y: 180, dir: -1, color: 'red' });
  const [score, setScore] = useState<{ blue: number; red: number }>({ blue: 0, red: 0 });
  const [cooldown, setCooldown] = useState<{ blue: number; red: number }>({ blue: 0, red: 0 });
  const [flash, setFlash] = useState<{ blueHit?: number; redHit?: number; blueWarn?: number; redWarn?: number }>({});
  const matchStore = usePssMatchStore();
  const mappingRef = useRef(loadMapping());
  // Retro FX/animation
  const animRef = useRef<{ blue: number; red: number }>({ blue: 0, red: 0 });
  type Spark = { x: number; y: number; t: number; color: string };
  const sparksRef = useRef<Spark[]>([]);

  // Draw loop
  useEffect(() => {
    const ctx = canvasRef.current?.getContext('2d');
    if (!ctx) return;
    let raf = 0;
    let last = performance.now();

    const drawBackground = () => {
      // sky gradient
      const g = ctx.createLinearGradient(0, 0, 0, HEIGHT);
      g.addColorStop(0, '#0b1220');
      g.addColorStop(1, '#0e1621');
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, WIDTH, HEIGHT);
      // skyline
      ctx.fillStyle = 'rgba(255,255,255,0.05)';
      for (let i = 0; i < 18; i++) {
        const w = 20 + Math.random() * 30;
        const h = 40 + Math.random() * 60;
        const x = ((i * 40) % WIDTH);
        ctx.fillRect(x, 120 - h, w, h);
      }
      // sun
      ctx.beginPath();
      ctx.arc(WIDTH * 0.75, 60, 18, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(255, 200, 60, 0.2)';
      ctx.fill();
      // floor: checker
      for (let y = 220; y < HEIGHT; y += 10) {
        for (let x = 0; x < WIDTH; x += 20) {
          ctx.fillStyle = ((x / 20 + y / 10) % 2 === 0) ? '#132034' : '#0f1a2a';
          ctx.fillRect(x, y, 20, 10);
        }
      }
    };

    const drawShadow = (x: number) => {
      ctx.fillStyle = 'rgba(0,0,0,0.35)';
      ctx.beginPath();
      ctx.ellipse(x, 220, 28, 8, 0, 0, Math.PI * 2);
      ctx.fill();
    };

    const drawFighter = (f: Fighter, punchAmt: number) => {
      const isBlue = f.color === 'blue';
      const base = isBlue ? '#3b82f6' : '#ef4444';
      const dark = isBlue ? '#1e40af' : '#7f1d1d';
      // torso
      ctx.fillStyle = base;
      ctx.fillRect(f.x - 16, f.y - 36, 32, 30);
      // belt
      ctx.fillStyle = dark;
      ctx.fillRect(f.x - 16, f.y - 14, 32, 4);
      // head
      ctx.fillStyle = '#e8e6e3';
      ctx.beginPath();
      ctx.arc(f.x, f.y - 45, 10, 0, Math.PI * 2);
      ctx.fill();
      // face stripe
      ctx.strokeStyle = '#0a0a0a';
      ctx.beginPath();
      ctx.moveTo(f.x, f.y - 45);
      ctx.lineTo(f.x + (f.dir === 1 ? 6 : -6), f.y - 45);
      ctx.stroke();
      // legs
      ctx.strokeStyle = base;
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.moveTo(f.x - 8, f.y - 6);
      ctx.lineTo(f.x - 10, f.y + 4);
      ctx.moveTo(f.x + 8, f.y - 6);
      ctx.lineTo(f.x + 10, f.y + 4);
      ctx.stroke();
      // arms
      ctx.strokeStyle = base;
      ctx.lineWidth = 4;
      ctx.beginPath();
      // rear arm
      ctx.moveTo(f.x - f.dir * 6, f.y - 26);
      ctx.lineTo(f.x - f.dir * (14 + punchAmt * 2), f.y - 18);
      // lead arm
      ctx.moveTo(f.x + f.dir * 6, f.y - 26);
      ctx.lineTo(f.x + f.dir * (18 + punchAmt * 10), f.y - (20 - punchAmt * 3));
      ctx.stroke();
      ctx.lineWidth = 1;
    };

    const drawHealthBars = () => {
      const s = matchStore.getTotalScore?.();
      const blueScore = s ? s.athlete1 : score.blue;
      const redScore = s ? s.athlete2 : score.red;
      const a1 = matchStore.getAthlete1?.();
      const a2 = matchStore.getAthlete2?.();
      // panel bg
      ctx.fillStyle = 'rgba(0,0,0,0.35)';
      ctx.fillRect(8, 8, WIDTH - 16, 24);
      // names
      ctx.fillStyle = '#cbd5e1';
      ctx.font = '12px monospace';
      ctx.fillText(`${a1?.short || 'BLUE'}`, 14, 24);
      const wName = ctx.measureText(`${a2?.short || 'RED'}`).width;
      ctx.fillText(`${a2?.short || 'RED'}`, WIDTH - 14 - wName, 24);
      // segmented bars (points)
      const segW = 10; const segGap = 2; const maxSeg = 20;
      for (let i = 0; i < maxSeg; i++) {
        // blue
        ctx.fillStyle = i < blueScore ? '#60a5fa' : '#1f2a44';
        ctx.fillRect(120 + i * (segW + segGap), 12, segW, 8);
        // red (right to left)
        ctx.fillStyle = i < redScore ? '#f87171' : '#3a2020';
        const rx = WIDTH - 120 - (i + 1) * (segW + segGap);
        ctx.fillRect(rx, 12, segW, 8);
      }
    };

    const drawOverlays = (now: number) => {
      // HUD flashes
      const flashAlpha = (t?: number) => (t && now - t < 600 ? 1 - (now - t) / 600 : 0);
      const fb = flashAlpha(flash.blueHit);
      const fr = flashAlpha(flash.redHit);
      if (fb > 0) { ctx.fillStyle = `rgba(59,130,246,${fb * 0.6})`; ctx.fillRect(0, 0, WIDTH / 2, 6); }
      if (fr > 0) { ctx.fillStyle = `rgba(239,68,68,${fr * 0.6})`; ctx.fillRect(WIDTH / 2, 0, WIDTH / 2, 6); }
      const wb = flashAlpha(flash.blueWarn);
      const wr = flashAlpha(flash.redWarn);
      if (wb > 0) { ctx.fillStyle = `rgba(234,179,8,${wb * 0.6})`; ctx.fillRect(0, HEIGHT - 6, WIDTH / 2, 6); }
      if (wr > 0) { ctx.fillStyle = `rgba(234,179,8,${wr * 0.6})`; ctx.fillRect(WIDTH / 2, HEIGHT - 6, WIDTH / 2, 6); }
      // sparks
      const life = 280;
      sparksRef.current = sparksRef.current.filter(s => now - s.t < life);
      for (const s of sparksRef.current) {
        const p = (now - s.t) / life;
        ctx.strokeStyle = s.color;
        ctx.lineWidth = 2;
        ctx.beginPath();
        ctx.moveTo(s.x - 6 - p * 10, s.y - 8 + p * 4);
        ctx.lineTo(s.x + 6 + p * 10, s.y + 8 - p * 4);
        ctx.moveTo(s.x - 6 - p * 10, s.y + 8 - p * 4);
        ctx.lineTo(s.x + 6 + p * 10, s.y - 8 + p * 4);
        ctx.stroke();
      }
      // scanlines
      ctx.fillStyle = 'rgba(255,255,255,0.03)';
      for (let y = 0; y < HEIGHT; y += 3) ctx.fillRect(0, y, WIDTH, 1);
      // vignette
      const rad = ctx.createRadialGradient(WIDTH/2, HEIGHT/2, Math.min(WIDTH, HEIGHT)/3, WIDTH/2, HEIGHT/2, Math.max(WIDTH, HEIGHT)/1.1);
      rad.addColorStop(0, 'rgba(0,0,0,0)');
      rad.addColorStop(1, 'rgba(0,0,0,0.35)');
      ctx.fillStyle = rad;
      ctx.fillRect(0, 0, WIDTH, HEIGHT);
    };

    const draw = () => {
      if (!running) { raf = requestAnimationFrame(draw); return; }
      const now = performance.now();
      const dt = Math.min(32, now - last);
      last = now;

      ctx.clearRect(0, 0, WIDTH, HEIGHT);
      drawBackground();
      drawHealthBars();

      // shadows + fighters with simple punch animation
      drawShadow(b.x);
      drawShadow(r.x);
      animRef.current.blue = Math.max(0, animRef.current.blue - dt);
      animRef.current.red = Math.max(0, animRef.current.red - dt);
      const punchB = animRef.current.blue > 0 ? 1 - animRef.current.blue / 250 : 0;
      const punchR = animRef.current.red > 0 ? 1 - animRef.current.red / 250 : 0;
      drawFighter(b, punchB);
      drawFighter(r, punchR);

      // cooldown text
      ctx.fillStyle = '#e5e7eb';
      ctx.font = '11px monospace';
      if (cooldown.blue > 0) ctx.fillText(`CD ${cooldown.blue}`, 10, 46);
      if (cooldown.red > 0) ctx.fillText(`CD ${cooldown.red}`, WIDTH - 70, 46);

      drawOverlays(performance.now());
      raf = requestAnimationFrame(draw);
    };

    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, [b, r, running, cooldown, flash, matchStore]);

  // Cooldown timer
  useEffect(() => {
    const t = setInterval(() => {
      setCooldown(prev => ({ blue: Math.max(0, prev.blue - 100), red: Math.max(0, prev.red - 100) }));
    }, 100);
    return () => clearInterval(t);
  }, []);

  // Key bindings → PSS events with range + cooldown
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!running) return;
      // Movement
      const move = (f: Fighter, dx: number) => {
        const nx = Math.max(40, Math.min(WIDTH - 40, f.x + dx));
        f.dir = dx > 0 ? 1 : -1;
        return { ...f, x: nx };
      };
      const inRange = (attacker: Fighter, defender: Fighter) => Math.abs(attacker.x - defender.x) < 70;
      const hit = (athlete: 1 | 2, point_type: number) => {
        // cooldown check
        if (athlete === 1 && cooldown.blue > 0) return;
        if (athlete === 2 && cooldown.red > 0) return;
        // range check
        if (athlete === 1 && !inRange(b, r)) return;
        if (athlete === 2 && !inRange(r, b)) return;
        sendManualEvent('point', { athlete, point_type });
        if (athlete === 1) { setScore(s => ({ ...s, blue: s.blue + point_type })); setCooldown(c => ({ ...c, blue: 1200 })); }
        else { setScore(s => ({ ...s, red: s.red + point_type })); setCooldown(c => ({ ...c, red: 1200 })); }
        // animate + spark
        if (athlete === 1) { animRef.current.blue = 250; sparksRef.current.push({ x: (b.x + r.x)/2, y: b.y - 26, t: performance.now(), color: '#60a5fa' }); }
        else { animRef.current.red = 250; sparksRef.current.push({ x: (b.x + r.x)/2, y: r.y - 26, t: performance.now(), color: '#f87171' }); }
      };
      switch (e.code) {
        // Blue movement
        case 'KeyA': setB(prev => move(prev, -12)); break;
        case 'KeyD': setB(prev => move(prev, 12)); break;
        // Red movement
        case 'ArrowLeft': setR(prev => move(prev, -12)); break;
        case 'ArrowRight': setR(prev => move(prev, 12)); break;

        // Blue attacks
        case 'KeyJ': hit(1, 1); break; // punch
        case 'KeyK': hit(1, 2); break; // body kick
        case 'KeyL': hit(1, 3); break; // head kick
        case 'KeyU': hit(1, 4); break; // tech body
        case 'KeyI': hit(1, 5); break; // tech head

        // Red attacks (numpad digits as alternative to top row)
        case 'Digit1':
        case 'Numpad1': hit(2, 1); break;
        case 'Digit2':
        case 'Numpad2': hit(2, 2); break;
        case 'Digit3':
        case 'Numpad3': hit(2, 3); break;
        case 'Digit4':
        case 'Numpad4': hit(2, 4); break;
        case 'Digit5':
        case 'Numpad5': hit(2, 5); break;

        // Extras
        case 'KeyQ': sendManualEvent('warning', { athlete: 1 }); break;
        case 'Slash': sendManualEvent('warning', { athlete: 2 }); break;
        case 'KeyZ': sendManualEvent('hit_level', { athlete: 1, level: 25 }); break;
        case 'Period': sendManualEvent('hit_level', { athlete: 2, level: 25 }); break;
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [running, sendManualEvent]);

  // Listen to browser PSS events for HUD flashes
  useEffect(() => {
    const onPss = (ev: any) => {
      const e = ev.detail;
      if (!e || !e.type) return;
      if (e.type === 'points') {
        if (e.athlete === 'athlete1' || e.athlete === 1 || e.athlete === 'blue') setFlash(f => ({ ...f, blueHit: Date.now() }));
        else setFlash(f => ({ ...f, redHit: Date.now() }));
      } else if (e.type === 'warnings') {
        if (e.athlete === 'athlete1' || e.athlete === 1 || e.athlete === 'blue') setFlash(f => ({ ...f, blueWarn: Date.now() }));
        else setFlash(f => ({ ...f, redWarn: Date.now() }));
      } else if (e.type === 'hit_level') {
        if (e.athlete === 1 || e.athlete === 'blue') setFlash(f => ({ ...f, blueHit: Date.now() }));
        else setFlash(f => ({ ...f, redHit: Date.now() }));
      } else if (e.type === 'current_scores') {
        // optional: update local score fallback from store handled via getTotalScore
      }
    };
    window.addEventListener('pss-event', onPss as EventListener);
    return () => window.removeEventListener('pss-event', onPss as EventListener);
  }, []);

  // Gamepad polling and actions
  useEffect(() => {
    let raf = 0;
    let lastPress: Record<string, boolean> = {};
    const poll = () => {
      if (!running) { raf = requestAnimationFrame(poll); return; }
      const pads = listConnectedGamepads();
      const cfg = mappingRef.current;

      const handlePlayer = (player: 1 | 2, px: Fighter, setP: React.Dispatch<React.SetStateAction<Fighter>>) => {
        const map = player === 1 ? cfg.player1 : cfg.player2;
        const gp = pads[map.gamepadIndex];
        if (!gp) return;
        // movement
        const ax = axisValue(gp, map.moveX.index);
        const dz = map.moveX.deadzone ?? 0.25;
        if (Math.abs(ax) > dz) setP(prev => ({ ...prev, x: Math.max(40, Math.min(WIDTH - 40, prev.x + Math.sign(ax) * 6)), dir: ax > 0 ? 1 : -1 }));
        // helper for button edge
        const edge = (key: string, pressed: boolean) => {
          const was = lastPress[key] || false;
          lastPress[key] = pressed;
          return pressed && !was;
        };
        const press = (name: string, index: number) => edge(name, isButtonPressed(gp, index));
        const inRange = (attacker: Fighter, defender: Fighter) => Math.abs(attacker.x - defender.x) < 70;
        const tryHit = (pt: number) => {
          if (player === 1) {
            if (cooldown.blue > 0) return;
            if (!inRange(b, r)) return;
            sendManualEvent('point', { athlete: 1, point_type: pt });
          } else {
            if (cooldown.red > 0) return;
            if (!inRange(r, b)) return;
            sendManualEvent('point', { athlete: 2, point_type: pt });
          }
        };
        if (press(`p${player}-punch`, map.punch.index)) tryHit(1);
        if (press(`p${player}-body`, map.body.index)) tryHit(2);
        if (press(`p${player}-head`, map.head.index)) tryHit(3);
        if (press(`p${player}-tb`, map.tech_body.index)) tryHit(4);
        if (press(`p${player}-th`, map.tech_head.index)) tryHit(5);
        if (press(`p${player}-warn`, map.warning.index)) sendManualEvent('warning', { athlete: player });
        if (press(`p${player}-hl`, map.hit_level.index)) sendManualEvent('hit_level', { athlete: player, level: cfg.hitLevelValue });
      };

      handlePlayer(1, b, setB);
      handlePlayer(2, r, setR);
      raf = requestAnimationFrame(poll);
    };
    raf = requestAnimationFrame(poll);
    const onConnect = () => { /* trigger mapping reload/display updates elsewhere */ };
    window.addEventListener('gamepadconnected', onConnect);
    window.addEventListener('gamepaddisconnected', onConnect);
    return () => { cancelAnimationFrame(raf); window.removeEventListener('gamepadconnected', onConnect); window.removeEventListener('gamepaddisconnected', onConnect); };
  }, [running, b, r, cooldown, sendManualEvent]);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <div className="text-sm text-gray-300">Arcade Mode – Blue: A/D J K L U I | Red: ←/→ 1..5</div>
        <Button size="sm" variant={running ? 'secondary' : 'primary'} onClick={() => setRunning(v => !v)}>
          {running ? 'Pause' : 'Resume'}
        </Button>
      </div>
      <canvas ref={canvasRef} width={WIDTH} height={HEIGHT} className="border border-gray-700 bg-[#0d131a]" />
      <ArcadeBindingsPanel />
    </div>
  );
};

export default ArcadeModePanel;


