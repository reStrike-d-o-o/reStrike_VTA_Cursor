import React, { useEffect, useRef, useState } from 'react';
import { useSimulationStore } from '../../../stores/simulationStore';
import { usePssMatchStore } from '../../../stores/pssMatchStore';
import Button from '../../atoms/Button';
import ArcadeBindingsPanel from './ArcadeBindingsPanel';
import { loadMapping, listConnectedGamepads, isButtonPressed, axisValue } from './gamepad';
import retroSound from './sound';

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
  const containerRef = useRef<HTMLDivElement | null>(null);
  const widthRef = useRef<number>(WIDTH);
  const heightRef = useRef<number>(HEIGHT * 2);
  const dprRef = useRef<number>(Math.max(1, window.devicePixelRatio || 1));
  const { sendManualEvent } = useSimulationStore();
  const [running, setRunning] = useState(true);
  const [mute, setMute] = useState(false);
  const [music, setMusic] = useState(true);

  const [b, setB] = useState<Fighter>({ x: 120, y: 180, dir: 1, color: 'blue' });
  const [r, setR] = useState<Fighter>({ x: 600, y: 180, dir: -1, color: 'red' });
  const [score, setScore] = useState<{ blue: number; red: number }>({ blue: 0, red: 0 });
  const [cooldown, setCooldown] = useState<{ blue: number; red: number }>({ blue: 0, red: 0 });
  const [flash, setFlash] = useState<{ blueHit?: number; redHit?: number; blueWarn?: number; redWarn?: number }>({});
  const matchStore = usePssMatchStore();
  const mappingRef = useRef(loadMapping());
  // Retro FX/animation
  type AnimState = { t: number; pose: 'idle' | 'punch' | 'kick_body' | 'kick_head' | 'spin' };
  const animRef = useRef<{ blue: AnimState; red: AnimState }>({ blue: { t: 0, pose: 'idle' }, red: { t: 0, pose: 'idle' } });
  type Spark = { x: number; y: number; t: number; color: string };
  const sparksRef = useRef<Spark[]>([]);
  const skylineRef = useRef<Array<{ x: number; w: number; h: number }>>([]);
  type Floater = { x: number; y: number; t: number; text: string; color: string };
  const floatersRef = useRef<Floater[]>([]);
  type Dust = { x: number; y: number; t: number };
  const dustRef = useRef<Dust[]>([]);
  const lastPosRef = useRef<{ bx: number; rx: number }>({ bx: 0, rx: 0 });
  const shakeRef = useRef<{ ax: number; ay: number }>({ ax: 0, ay: 0 });
  const emitLocal = (evt: any) => {
    try {
      const ev = new CustomEvent('pss-event', { detail: evt, bubbles: true, cancelable: true });
      window.dispatchEvent(ev);
    } catch {}
  };

  // --- Retro pixel sprites (idle/punch) ---
  const PIX = 4; // pixel scale
  const SPR_W = 24;
  const SPR_H = 24;
  // Taekwondo fighter sprites (idle/punch/kick), 24x24 pixels. Legend:
  // W/w: white dobok (light/shadow), C/c: chest protector (blue/red variant),
  // S: skin, H/h: headgear light/shadow, K: belt, B: outline, G: glove/foot pad
  const idleSprite: string[] = [
    '           HHHHHH           ',
    '          HSSSSSH          ',
    '          SSSSSSS          ',
    '           SSSSS           ',
    '         WWWWWWWW          ',
    '        WWWWWWWWWW         ',
    '       WWCCCCCCWW          ',
    '       WwCccccCWw          ',
    '       WwCccccCWw          ',
    '       WWCCCCCCWW          ',
    '        WWWKKWWWW          ',
    '        WWWWWWWWW          ',
    '      WwWW    WWwW         ',
    '     WwWW      WWwW        ',
    '     WwW        WwW        ',
    '     GGG        GGG        ',
    '     G G        G G        ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
  ];
  const punchSprite: string[] = [
    '           HHHHHH           ',
    '          HSSSSSH          ',
    '          SSSSSSS          ',
    '           SSSSS           ',
    '         WWWWWWWW          ',
    '        WWWWWWWWWW         ',
    '       WWCCCCCCWW   GGG    ',
    '       WwCccccCWw   GGG    ',
    '       WwCccccCWw          ',
    '       WWCCCCCCWW          ',
    '        WWWKKWWWW          ',
    '        WWWWWWWWW          ',
    '      WwWW      WWwW       ',
    '     WwWW        WWwW      ',
    '     WwW          WwW      ',
    '     GGG          GGG      ',
    '     G G          G G      ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
  ];
  const kickBodySprite: string[] = [
    '           HHHHHH           ',
    '          HSSSSSH          ',
    '          SSSSSSS          ',
    '           SSSSS           ',
    '         WWWWWWWW          ',
    '        WWWWWWWWWW         ',
    '       WWCCCCCCWW          ',
    '       WwCccccCWw          ',
    '       WwCccccCWw          ',
    '       WWCCCCCCWW          ',
    '        WWWKKWWWW   GGGGG  ',
    '        WWWWWWWWW   GGGGG  ',
    '      WwWW                ',
    '     WwWW                 ',
    '     WwW                  ',
    '     GGG                  ',
    '     G G                  ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
  ];
  const kickHeadSprite: string[] = [
    '           HHHHHH           ',
    '          HSSSSSH          ',
    '          SSSSSSS      GGG ',
    '           SSSSS       GGG ',
    '         WWWWWWWW          ',
    '        WWWWWWWWWW         ',
    '       WWCCCCCCWW          ',
    '       WwCccccCWw          ',
    '       WwCccccCWw          ',
    '       WWCCCCCCWW          ',
    '        WWWKKWWWW          ',
    '        WWWWWWWWW          ',
    '      WwWW                ',
    '     WwWW                 ',
    '     WwW                  ',
    '     GGG                  ',
    '     G G                  ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
  ];
  const spinKickSprite: string[] = [
    '           HHHHHH           ',
    '          HSSSSSH          ',
    '      GGG SSSSSSS          ',
    '      GGG  SSSSS           ',
    '     WWWWWWWWWWWW          ',
    '    WWWWWWWWWWWWWW         ',
    '   WWCCCCCCCCCCCCWW        ',
    '   WwCccccccccccCWw        ',
    '   WwCccccccccccCWw        ',
    '   WWCCCCCCCCCCCCWW  GGG   ',
    '    WWWKKWWWWWWWWWW  GGG   ',
    '    WWWWWWWWWWWWWWW        ',
    '  WwWW                  ',
    ' WwWW                   ',
    ' WwW                    ',
    ' GGG                    ',
    ' G G                    ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
    '                          ',
  ];

  const basePalette = {
    ' ': 'transparent',
    H: '#e5e7eb',
    h: '#cbd5e1',
    S: '#f5d0a6',
    B: '#111827',
    W: '#f3f4f6',
    w: '#d1d5db',
    K: '#1f2937',
    G: '#9ca3af',
  } as const;
  const variantPalette = (color: 'blue' | 'red') => ({
    ...basePalette,
    C: color === 'blue' ? '#2563eb' : '#dc2626',
    c: color === 'blue' ? '#1e40af' : '#7f1d1d',
  });

  const drawPixelSprite = (
    ctx: CanvasRenderingContext2D,
    grid: string[],
    palette: Record<string, string>,
    x: number,
    y: number,
    dir: 1 | -1
  ) => {
    const anchorX = (SPR_W * PIX) / 2;
    for (let row = 0; row < SPR_H; row++) {
      const line = grid[row] || '';
      for (let col = 0; col < SPR_W; col++) {
        const ch = line[col] || ' ';
        if (ch === ' ') continue;
        const fill = palette[ch] || 'transparent';
        if (fill === 'transparent') continue;
        ctx.fillStyle = fill;
        const drawCol = dir === 1 ? col : SPR_W - 1 - col;
        const dx = x - anchorX + drawCol * PIX;
        const dy = y - SPR_H * PIX + row * PIX;
        ctx.fillRect(dx, dy, PIX, PIX);
      }
    }
    // simple outline pass
    ctx.strokeStyle = 'rgba(0,0,0,0.4)';
    for (let row = 0; row < SPR_H; row++) {
      const line = grid[row] || '';
      for (let col = 0; col < SPR_W; col++) {
        const ch = line[col] || ' ';
        if (ch === ' ') continue;
        const drawCol = dir === 1 ? col : SPR_W - 1 - col;
        const dx = x - anchorX + drawCol * PIX;
        const dy = y - SPR_H * PIX + row * PIX;
        ctx.strokeRect(dx + 0.5, dy + 0.5, PIX, PIX);
      }
    }
  };

  // Draw loop
  useEffect(() => {
    const ctx = canvasRef.current?.getContext('2d');
    if (!ctx) return;
    let raf = 0;
    let last = performance.now();

    // prepare skyline once
    if (skylineRef.current.length === 0) {
      const arr: Array<{ x: number; w: number; h: number }> = [];
      for (let i = 0; i < 18; i++) {
        const w = 20 + Math.random() * 30;
        const h = 40 + Math.random() * 60;
        const x = (i * 40) % WIDTH;
        arr.push({ x, w, h });
      }
      skylineRef.current = arr;
    }

    const drawBackground = () => {
      const W = widthRef.current; const H = heightRef.current;
      // sky gradient
      const g = ctx.createLinearGradient(0, 0, 0, H);
      g.addColorStop(0, '#0b1220');
      g.addColorStop(1, '#0e1621');
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, W, H);
      // skyline parallax
      ctx.fillStyle = 'rgba(255,255,255,0.04)';
      const t = performance.now() * 0.0005;
      skylineRef.current.forEach((b, i) => {
        const px = (b.x + (i % 3) * (t * 20)) % W;
        ctx.fillRect(px, 120 - b.h, b.w, b.h);
      });
      // sun
      ctx.beginPath();
      ctx.arc(W * 0.75, 60, 18, 0, Math.PI * 2);
      const sunPhase = (Math.sin(performance.now() * 0.002) * 0.5 + 0.5) * 0.3 + 0.2;
      ctx.fillStyle = `rgba(255, 200, 60, ${sunPhase.toFixed(2)})`;
      ctx.fill();
      // floor: checker
      for (let y = 220; y < H; y += 10) {
        for (let x = 0; x < W; x += 20) {
          ctx.fillStyle = ((x / 20 + y / 10) % 2 === 0) ? '#132034' : '#0f1a2a';
          ctx.fillRect(x, y, 20, 10);
        }
      }
      // crowd silhouettes
      ctx.fillStyle = 'rgba(20,30,50,0.8)';
      for (let i = 0; i < 12; i++) {
        const cx = (i * 60 + (Math.sin(t + i) * 10)) % W;
        ctx.beginPath();
        const radius = 8 + (i % 3);
        ctx.arc(cx, 110 + Math.sin(t * 2 + i) * 2, radius, 0, Math.PI * 2);
        ctx.fill();
        // shoulders
        ctx.fillRect(cx - radius - 4, 110 + radius, (radius + 4) * 2, 4);
      }
    };

    const drawShadow = (x: number) => {
      ctx.fillStyle = 'rgba(0,0,0,0.35)';
      ctx.beginPath();
      ctx.ellipse(x, 220, 28, 8, 0, 0, Math.PI * 2);
      ctx.fill();
    };

    const drawFighter = (f: Fighter, anim: AnimState) => {
      // Choose sprite based on current pose
      let grid = idleSprite;
      if (anim.pose === 'punch') grid = punchSprite;
      else if (anim.pose === 'kick_body') grid = kickBodySprite;
      else if (anim.pose === 'kick_head') grid = kickHeadSprite;
      else if (anim.pose === 'spin') grid = spinKickSprite;
      drawPixelSprite(ctx, grid, variantPalette(f.color), f.x, f.y, f.dir);
    };

    const drawHealthBars = () => {
      const W = widthRef.current;
      const s = matchStore.getTotalScore?.();
      const blueScore = s ? s.athlete1 : score.blue;
      const redScore = s ? s.athlete2 : score.red;
      const a1 = matchStore.getAthlete1?.();
      const a2 = matchStore.getAthlete2?.();
      // panel bg
      ctx.fillStyle = 'rgba(0,0,0,0.35)';
      ctx.fillRect(8, 8, W - 16, 24);
      // names
      ctx.fillStyle = '#cbd5e1';
      ctx.font = '12px monospace';
      ctx.fillText(`${a1?.short || 'BLUE'}`, 14, 24);
      const wName = ctx.measureText(`${a2?.short || 'RED'}`).width;
      ctx.fillText(`${a2?.short || 'RED'}`, W - 14 - wName, 24);
      // segmented bars (points)
      const segW = 10; const segGap = 2; const maxSeg = 20;
      for (let i = 0; i < maxSeg; i++) {
        // blue
        ctx.fillStyle = i < blueScore ? '#60a5fa' : '#1f2a44';
        ctx.fillRect(120 + i * (segW + segGap), 12, segW, 8);
        // red (right to left)
        ctx.fillStyle = i < redScore ? '#f87171' : '#3a2020';
        const rx = W - 120 - (i + 1) * (segW + segGap);
        ctx.fillRect(rx, 12, segW, 8);
      }
    };

    const drawOverlays = (now: number) => {
      const W = widthRef.current; const H = heightRef.current;
      // HUD flashes
      const flashAlpha = (t?: number) => (t && now - t < 600 ? 1 - (now - t) / 600 : 0);
      const fb = flashAlpha(flash.blueHit);
      const fr = flashAlpha(flash.redHit);
      if (fb > 0) { ctx.fillStyle = `rgba(59,130,246,${fb * 0.6})`; ctx.fillRect(0, 0, W / 2, 6); }
      if (fr > 0) { ctx.fillStyle = `rgba(239,68,68,${fr * 0.6})`; ctx.fillRect(W / 2, 0, W / 2, 6); }
      const wb = flashAlpha(flash.blueWarn);
      const wr = flashAlpha(flash.redWarn);
      if (wb > 0) { ctx.fillStyle = `rgba(234,179,8,${wb * 0.6})`; ctx.fillRect(0, H - 6, W / 2, 6); }
      if (wr > 0) { ctx.fillStyle = `rgba(234,179,8,${wr * 0.6})`; ctx.fillRect(W / 2, H - 6, W / 2, 6); }
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
      // score floaters
      const flife = 900;
      floatersRef.current = floatersRef.current.filter(f => now - f.t < flife);
      for (const f of floatersRef.current) {
        const p = (now - f.t) / flife;
        ctx.globalAlpha = 1 - p;
        ctx.fillStyle = f.color;
        ctx.font = 'bold 14px monospace';
        ctx.fillText(f.text, f.x - 8, f.y - p * 40);
        ctx.globalAlpha = 1;
      }
      // scanlines
      ctx.fillStyle = 'rgba(255,255,255,0.035)';
      for (let y = 0; y < H; y += 3) {
        ctx.fillRect(0, y, W, 1);
      }
      // vignette
      const rad = ctx.createRadialGradient(W/2, H/2, Math.min(W, H)/3, W/2, H/2, Math.max(W, H)/1.1);
      rad.addColorStop(0, 'rgba(0,0,0,0)');
      rad.addColorStop(1, 'rgba(0,0,0,0.4)');
      ctx.fillStyle = rad;
      ctx.fillRect(0, 0, W, H);
    };

    const draw = () => {
      if (!running) { raf = requestAnimationFrame(draw); return; }
      const now = performance.now();
      const dt = Math.min(32, now - last);
      last = now;

      // responsive sizing
      const parentW = containerRef.current?.clientWidth || WIDTH;
      const W = parentW;
      const H = Math.max(HEIGHT * 2, Math.round((parentW / WIDTH) * HEIGHT * 2));
      widthRef.current = W; heightRef.current = H;
      const dpr = dprRef.current;
      if (canvasRef.current) {
        const c = canvasRef.current;
        if (c.width !== Math.floor(W * dpr) || c.height !== Math.floor(H * dpr)) {
          c.width = Math.floor(W * dpr);
          c.height = Math.floor(H * dpr);
        }
        c.style.width = '100%';
        c.style.height = `${H}px`;
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, W, H);
      ctx.imageSmoothingEnabled = false;
      drawBackground();
      drawHealthBars();

      // shadows + fighters with simple punch animation
      // screen shake decay
      shakeRef.current.ax *= 0.9; shakeRef.current.ay *= 0.9;
      ctx.save();
      ctx.translate(shakeRef.current.ax, shakeRef.current.ay);

      drawShadow(b.x);
      drawShadow(r.x);
      animRef.current.blue.t = Math.max(0, animRef.current.blue.t - dt);
      animRef.current.red.t = Math.max(0, animRef.current.red.t - dt);
      if (animRef.current.blue.t === 0) animRef.current.blue.pose = 'idle';
      if (animRef.current.red.t === 0) animRef.current.red.pose = 'idle';
      drawFighter(b, animRef.current.blue);
      drawFighter(r, animRef.current.red);

      ctx.restore();

      // cooldown text
      ctx.fillStyle = '#e5e7eb';
      ctx.font = '11px monospace';
      if (cooldown.blue > 0) ctx.fillText(`CD ${(cooldown.blue/1000).toFixed(1)}s`, 10, 46);
      if (cooldown.red > 0) ctx.fillText(`CD ${(cooldown.red/1000).toFixed(1)}s`, W - 90, 46);

      // show reach indicator
      ctx.fillStyle = 'rgba(255,255,255,0.03)';
      const reach = 70;
      ctx.fillRect(b.x - reach, b.y - 40, reach, 34);
      ctx.fillRect(r.x, r.y - 40, reach, 34);
      // help text
      ctx.fillStyle = 'rgba(203,213,225,0.8)';
      ctx.font = '11px monospace';
      ctx.fillText('Blue: A/D move, J/K/L/U/I attack  •  Red: ←/→ move, 1..5 attack  •  Move into yellow reach boxes', 12, H - 10);

      // dust from movement
      const emitDust = (x: number, y: number) => { dustRef.current.push({ x, y, t: performance.now() }); };
      if (lastPosRef.current.bx) {
        const dx = Math.abs(b.x - lastPosRef.current.bx);
        if (dx > 4) emitDust(b.x - 10 * Math.sign(b.dir), 220);
      }
      if (lastPosRef.current.rx) {
        const dx = Math.abs(r.x - lastPosRef.current.rx);
        if (dx > 4) emitDust(r.x - 10 * Math.sign(r.dir), 220);
      }
      lastPosRef.current = { bx: b.x, rx: r.x };
      // draw dust
      const dlife = 500; const dcol = 'rgba(255,255,255,0.15)';
      dustRef.current = dustRef.current.filter(d => performance.now() - d.t < dlife);
      for (const d of dustRef.current) {
        const p = (performance.now() - d.t) / dlife;
        ctx.globalAlpha = 1 - p;
        const grad = ctx.createLinearGradient(d.x - 6, d.y, d.x + 6, d.y);
        grad.addColorStop(0, 'rgba(255,255,255,0)');
        grad.addColorStop(0.5, dcol);
        grad.addColorStop(1, 'rgba(255,255,255,0)');
        ctx.fillStyle = grad;
        ctx.fillRect(d.x - 6 + p * 10, d.y - 2 - p * 6, 12, 2);
        ctx.globalAlpha = 1;
      }

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
        const nx = Math.max(40, Math.min(widthRef.current - 40, f.x + dx));
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
        emitLocal({ type: 'points', athlete: athlete === 1 ? 'athlete1' : 'athlete2', description: `+${point_type}` });
        if (athlete === 1) { setScore(s => ({ ...s, blue: s.blue + point_type })); setCooldown(c => ({ ...c, blue: 1200 })); }
        else { setScore(s => ({ ...s, red: s.red + point_type })); setCooldown(c => ({ ...c, red: 1200 })); }
        // animate + spark
        const pose: AnimState['pose'] = point_type === 1 ? 'punch' : point_type === 2 ? 'kick_body' : point_type === 3 ? 'kick_head' : 'spin';
        if (athlete === 1) { animRef.current.blue = { t: 300, pose }; sparksRef.current.push({ x: (b.x + r.x)/2, y: b.y - 26, t: performance.now(), color: '#60a5fa' }); }
        else { animRef.current.red = { t: 300, pose }; sparksRef.current.push({ x: (b.x + r.x)/2, y: r.y - 26, t: performance.now(), color: '#f87171' }); }
        if (pose === 'kick_head' || pose === 'spin') { shakeRef.current.ax += (Math.random() * 2 - 1) * 2; shakeRef.current.ay += (Math.random() * 2 - 1) * 1.5; }
        if (!mute) retroSound.playAttack(point_type, athlete === 1 ? 'blue' : 'red');
        // floater
        const fx = (b.x + r.x) / 2; const fy = ((b.y + r.y) / 2) - 24;
        floatersRef.current.push({ x: fx, y: fy, t: performance.now(), text: `+${point_type}` , color: athlete === 1 ? '#93c5fd' : '#fca5a5' });
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
        case 'KeyU': hit(1, 4); break; // spin body
        case 'KeyI': hit(1, 5); break; // spin head

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

        // Extras & fallbacks
        case 'KeyQ': sendManualEvent('warning', { athlete: 1 }); emitLocal({ type: 'warnings', athlete: 'athlete1', description: 'Warning' }); if (!mute) retroSound.playWarning(); break;
        case 'Slash': sendManualEvent('warning', { athlete: 2 }); emitLocal({ type: 'warnings', athlete: 'athlete2', description: 'Warning' }); if (!mute) retroSound.playWarning(); break;
        case 'KeyZ': sendManualEvent('hit_level', { athlete: 1, level: 25 }); emitLocal({ type: 'hit_level', athlete: 1, level: 25 }); if (!mute) retroSound.playHitLevel(25); break;
        case 'Period': sendManualEvent('hit_level', { athlete: 2, level: 25 }); emitLocal({ type: 'hit_level', athlete: 2, level: 25 }); if (!mute) retroSound.playHitLevel(25); break;
        case 'Space': hit(1, 1); break; // fallback punch blue
        case 'Numpad0': hit(2, 1); break; // fallback punch red
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [running, sendManualEvent, b, r, cooldown]);

  // Listen to browser PSS events for HUD flashes; ensure we also mirror current_scores updates visually
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
        // update local fallback so HUD shows something even without store
        if (typeof e.athlete1_score === 'number' && typeof e.athlete2_score === 'number') {
          setScore({ blue: e.athlete1_score, red: e.athlete2_score });
        }
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
        if (Math.abs(ax) > dz) setP(prev => ({ ...prev, x: Math.max(40, Math.min(widthRef.current - 40, prev.x + Math.sign(ax) * 6)), dir: ax > 0 ? 1 : -1 }));
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
            animRef.current.blue = { t: 250, pose: pt >= 3 ? 'kick' : 'punch' };
            if (!mute) retroSound.playAttack(pt, 'blue');
          } else {
            if (cooldown.red > 0) return;
            if (!inRange(r, b)) return;
            sendManualEvent('point', { athlete: 2, point_type: pt });
            animRef.current.red = { t: 250, pose: pt >= 3 ? 'kick' : 'punch' };
            if (!mute) retroSound.playAttack(pt, 'red');
          }
        };
        if (press(`p${player}-punch`, map.punch.index)) tryHit(1);
        if (press(`p${player}-body`, map.body.index)) tryHit(2);
        if (press(`p${player}-head`, map.head.index)) tryHit(3);
        if (press(`p${player}-tb`, map.tech_body.index)) tryHit(4);
        if (press(`p${player}-th`, map.tech_head.index)) tryHit(5);
        if (press(`p${player}-warn`, map.warning.index)) { sendManualEvent('warning', { athlete: player }); emitLocal({ type: 'warnings', athlete: player === 1 ? 'athlete1' : 'athlete2' }); if (!mute) retroSound.playWarning(); }
        if (press(`p${player}-hl`, map.hit_level.index)) { sendManualEvent('hit_level', { athlete: player, level: cfg.hitLevelValue }); emitLocal({ type: 'hit_level', athlete: player, level: cfg.hitLevelValue }); if (!mute) retroSound.playHitLevel(cfg.hitLevelValue); }
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
    <div className="space-y-3" ref={containerRef}>
      <div className="flex items-center justify-between">
        <div className="text-sm text-gray-300">Arcade Mode – Blue: A/D J K L U I | Red: ←/→ 1..5</div>
        <Button size="sm" variant={running ? 'secondary' : 'primary'} onClick={() => setRunning(v => !v)}>
          {running ? 'Pause' : 'Resume'}
        </Button>
      </div>
      <div className="flex items-center justify-end space-x-2">
        <label className="text-xs text-gray-400 flex items-center space-x-1">
          <input type="checkbox" checked={mute} onChange={(e) => { setMute(e.target.checked); retroSound.setMuted(e.target.checked); }} />
          <span>Mute SFX</span>
        </label>
        <label className="text-xs text-gray-400 flex items-center space-x-1">
          <input type="checkbox" checked={music} onChange={async (e) => { setMusic(e.target.checked); await retroSound.setMusicOn(e.target.checked); }} />
          <span>Music</span>
        </label>
      </div>
      <canvas ref={canvasRef} className="w-full border border-gray-700 bg-[#0d131a]" />
      <ArcadeBindingsPanel />
    </div>
  );
};

export default ArcadeModePanel;


