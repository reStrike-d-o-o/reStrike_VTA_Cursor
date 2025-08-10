import React, { useEffect, useRef, useState } from 'react';
import { useSimulationStore } from '../../../stores/simulationStore';
import Button from '../../atoms/Button';

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
      raf = requestAnimationFrame(draw);
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, [b, r, running]);

  // Key bindings → PSS events
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!running) return;
      // Movement
      const move = (f: Fighter, dx: number) => {
        const nx = Math.max(40, Math.min(WIDTH - 40, f.x + dx));
        f.dir = dx > 0 ? 1 : -1;
        return { ...f, x: nx };
      };
      switch (e.code) {
        // Blue movement
        case 'KeyA': setB(prev => move(prev, -12)); break;
        case 'KeyD': setB(prev => move(prev, 12)); break;
        // Red movement
        case 'ArrowLeft': setR(prev => move(prev, -12)); break;
        case 'ArrowRight': setR(prev => move(prev, 12)); break;

        // Blue attacks
        case 'KeyJ': sendManualEvent('point', { athlete: 1, point_type: 1 }); break; // punch
        case 'KeyK': sendManualEvent('point', { athlete: 1, point_type: 2 }); break; // body kick
        case 'KeyL': sendManualEvent('point', { athlete: 1, point_type: 3 }); break; // head kick
        case 'KeyU': sendManualEvent('point', { athlete: 1, point_type: 4 }); break; // tech body
        case 'KeyI': sendManualEvent('point', { athlete: 1, point_type: 5 }); break; // tech head

        // Red attacks (numpad digits as alternative to top row)
        case 'Digit1':
        case 'Numpad1': sendManualEvent('point', { athlete: 2, point_type: 1 }); break;
        case 'Digit2':
        case 'Numpad2': sendManualEvent('point', { athlete: 2, point_type: 2 }); break;
        case 'Digit3':
        case 'Numpad3': sendManualEvent('point', { athlete: 2, point_type: 3 }); break;
        case 'Digit4':
        case 'Numpad4': sendManualEvent('point', { athlete: 2, point_type: 4 }); break;
        case 'Digit5':
        case 'Numpad5': sendManualEvent('point', { athlete: 2, point_type: 5 }); break;

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
    </div>
  );
};

export default ArcadeModePanel;


