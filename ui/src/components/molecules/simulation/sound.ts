class RetroSound {
  private static instance: RetroSound;
  private ctx: AudioContext | null = null;
  private muted = false;

  static getInstance(): RetroSound {
    if (!RetroSound.instance) RetroSound.instance = new RetroSound();
    return RetroSound.instance;
  }

  isMuted() { return this.muted; }
  setMuted(m: boolean) { this.muted = m; }

  async resume() {
    try {
      if (!this.ctx) this.ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      if (this.ctx && this.ctx.state !== 'running') await this.ctx.resume();
    } catch {}
  }

  private now() { return this.ctx ? this.ctx.currentTime : 0; }

  private createNoise(durationMs: number): AudioBufferSourceNode | null {
    if (!this.ctx) return null;
    const bufferSize = Math.floor((durationMs / 1000) * this.ctx.sampleRate);
    const buffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < bufferSize; i++) data[i] = Math.random() * 2 - 1;
    const src = this.ctx.createBufferSource();
    src.buffer = buffer;
    return src;
  }

  async playHit(side: 'blue' | 'red' = 'blue') {
    if (this.muted) return;
    await this.resume();
    if (!this.ctx) return;
    const now = this.now();
    // noise burst + thump osc
    const noise = this.createNoise(90);
    const lp = this.ctx.createBiquadFilter();
    lp.type = 'lowpass';
    lp.frequency.value = 1500;
    const gn = this.ctx.createGain();
    gn.gain.setValueAtTime(0.25, now);
    gn.gain.exponentialRampToValueAtTime(0.001, now + 0.12);
    if (noise) { noise.connect(lp); lp.connect(gn); gn.connect(this.ctx.destination); noise.start(); noise.stop(now + 0.15); }

    const osc = this.ctx.createOscillator();
    const go = this.ctx.createGain();
    osc.type = 'square';
    osc.frequency.setValueAtTime(side === 'blue' ? 220 : 200, now);
    osc.frequency.exponentialRampToValueAtTime(side === 'blue' ? 110 : 100, now + 0.12);
    go.gain.setValueAtTime(0.20, now);
    go.gain.exponentialRampToValueAtTime(0.001, now + 0.12);
    osc.connect(go); go.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.13);
  }

  async playWarning() {
    if (this.muted) return;
    await this.resume();
    if (!this.ctx) return;
    const now = this.now();
    const osc = this.ctx.createOscillator();
    osc.type = 'sawtooth';
    const g = this.ctx.createGain();
    osc.frequency.setValueAtTime(580, now);
    osc.frequency.linearRampToValueAtTime(420, now + 0.25);
    g.gain.setValueAtTime(0.12, now);
    g.gain.exponentialRampToValueAtTime(0.001, now + 0.28);
    osc.connect(g); g.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.3);
  }

  async playHitLevel(level: number) {
    if (this.muted) return;
    await this.resume();
    if (!this.ctx) return;
    const now = this.now();
    const osc = this.ctx.createOscillator();
    const g = this.ctx.createGain();
    osc.type = 'triangle';
    const base = 220 + (level || 0) * 3;
    osc.frequency.setValueAtTime(base, now);
    osc.frequency.exponentialRampToValueAtTime(base * 1.4, now + 0.14);
    g.gain.setValueAtTime(0.15, now);
    g.gain.exponentialRampToValueAtTime(0.001, now + 0.18);
    osc.connect(g); g.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.2);
  }
}

const retroSound = RetroSound.getInstance();
export default retroSound;


