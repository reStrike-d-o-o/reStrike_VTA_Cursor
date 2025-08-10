import React, { useEffect, useRef, useState } from 'react';
import { useSimulationStore } from '../../../stores/simulationStore';
import Button from '../../atoms/Button';
import ArcadeBindingsPanel from './ArcadeBindingsPanel';

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

  // Draw loop
  useEffect(() => {
    const ctx = canvasRef.current?.getContext('2d');
    if (!ctx) return;
    let raf = 0;
    const draw = () => {
      if (!running) return;
      ctx.clearRect(0, 0, WIDTH, HEIGHT);
      // bg
      ctx.fillStyle = '#0e1621';
      ctx.fillRect(0, 0, WIDTH, HEIGHT);
      // ground
      ctx.fillStyle = '#152232';
      ctx.fillRect(0, 220, WIDTH, 40);
      // fighters
      const drawFighter = (f: Fighter) => {
        ctx.fillStyle = f.color === 'blue' ? '#3b82f6' : '#ef4444';
        ctx.fillRect(f.x - 20, f.y - 40, 40, 40);
        // face line
        ctx.strokeStyle = '#e5e7eb';
        ctx.beginPath();
        ctx.moveTo(f.x, f.y - 20);
        ctx.lineTo(f.x + (f.dir === 1 ? 12 : -12), f.y - 20);
        ctx.stroke();
      };
      drawFighter(b);
      drawFighter(r);
      // HUD
      ctx.fillStyle = '#e5e7eb';
      ctx.font = '12px monospace';
      ctx.fillText(`Blue ${score.blue}`, 10, 14);
      ctx.fillText(`${score.red} Red`, WIDTH - 80, 14);
      if (cooldown.blue > 0) ctx.fillText(`B CD:${cooldown.blue}`, 10, 30);
      if (cooldown.red > 0) ctx.fillText(`R CD:${cooldown.red}`, WIDTH - 90, 30);
      raf = requestAnimationFrame(draw);
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, [b, r, running]);

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


