class RetroSound {
  private static instance: RetroSound;
  private ctx: AudioContext | null = null;
  private muted = false;
  private musicOn = false;
  private musicGain: GainNode | null = null;
  private musicNodes: { bass?: OscillatorNode; arp?: OscillatorNode; noise?: AudioBufferSourceNode; kickInt?: number } = {};

  static getInstance(): RetroSound {
    if (!RetroSound.instance) RetroSound.instance = new RetroSound();
    return RetroSound.instance;
  }

  isMuted() { return this.muted; }
  setMuted(m: boolean) { this.muted = m; }
  isMusicOn() { return this.musicOn; }
  async setMusicOn(v: boolean) {
    this.musicOn = v;
    if (v) await this.startMusic(); else this.stopMusic();
  }

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

  // Different SFX per attack type
  async playAttack(pointType: number, side: 'blue' | 'red') {
    switch (pointType) {
      case 1: // punch
        await this.playPunch(side); break;
      case 2: // body kick
        await this.playBodyKick(side); break;
      case 3: // head kick
        await this.playHeadKick(side); break;
      case 4: // tech body
        await this.playTechBody(side); break;
      case 5: // tech head
        await this.playTechHead(side); break;
      default:
        await this.playHit(side);
    }
  }

  private async playPunch(side: 'blue' | 'red') {
    if (this.muted) return;
    await this.resume();
    if (!this.ctx) return;
    const now = this.now();
    const osc = this.ctx.createOscillator();
    const g = this.ctx.createGain();
    osc.type = 'square';
    const f0 = side === 'blue' ? 240 : 210;
    osc.frequency.setValueAtTime(f0, now);
    osc.frequency.exponentialRampToValueAtTime(f0 * 0.7, now + 0.08);
    g.gain.setValueAtTime(0.22, now);
    g.gain.exponentialRampToValueAtTime(0.001, now + 0.1);
    osc.connect(g); g.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.11);
  }

  private async playBodyKick(side: 'blue' | 'red') {
    if (this.muted) return; await this.resume(); if (!this.ctx) return;
    const now = this.now();
    const noise = this.createNoise(140);
    const bp = this.ctx.createBiquadFilter(); bp.type = 'bandpass'; bp.frequency.value = 900; bp.Q.value = 0.6;
    const g = this.ctx.createGain(); g.gain.setValueAtTime(0.22, now); g.gain.exponentialRampToValueAtTime(0.001, now + 0.16);
    if (noise) { noise.connect(bp); bp.connect(g); g.connect(this.ctx.destination); noise.start(); noise.stop(now + 0.16); }
  }

  private async playHeadKick(side: 'blue' | 'red') {
    if (this.muted) return; await this.resume(); if (!this.ctx) return;
    const now = this.now();
    const osc = this.ctx.createOscillator(); osc.type = 'triangle';
    const g = this.ctx.createGain();
    const f0 = side === 'blue' ? 440 : 400;
    osc.frequency.setValueAtTime(f0, now);
    osc.frequency.exponentialRampToValueAtTime(f0 * 1.6, now + 0.12);
    g.gain.setValueAtTime(0.18, now); g.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
    osc.connect(g); g.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.15);
  }

  private async playTechBody(side: 'blue' | 'red') {
    if (this.muted) return; await this.resume(); if (!this.ctx) return;
    const now = this.now();
    const osc = this.ctx.createOscillator(); osc.type = 'sawtooth';
    const g = this.ctx.createGain();
    const f0 = side === 'blue' ? 300 : 270;
    osc.frequency.setValueAtTime(f0, now);
    osc.frequency.linearRampToValueAtTime(f0 * 0.5, now + 0.18);
    g.gain.setValueAtTime(0.22, now); g.gain.exponentialRampToValueAtTime(0.001, now + 0.2);
    osc.connect(g); g.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.21);
  }

  private async playTechHead(side: 'blue' | 'red') {
    if (this.muted) return; await this.resume(); if (!this.ctx) return;
    const now = this.now();
    const osc = this.ctx.createOscillator(); osc.type = 'square';
    const g = this.ctx.createGain();
    const f0 = side === 'blue' ? 520 : 480;
    osc.frequency.setValueAtTime(f0, now);
    osc.frequency.exponentialRampToValueAtTime(f0 * 1.3, now + 0.14);
    g.gain.setValueAtTime(0.2, now); g.gain.exponentialRampToValueAtTime(0.001, now + 0.18);
    osc.connect(g); g.connect(this.ctx.destination);
    osc.start(now); osc.stop(now + 0.19);
  }

  // Lightweight background music loop (bass + arp + noise hats)
  private async startMusic() {
    await this.resume(); if (!this.ctx) return;
    const ctx = this.ctx;
    if (!this.musicGain) { this.musicGain = ctx.createGain(); this.musicGain.gain.value = 0.12; this.musicGain.connect(ctx.destination); }
    // Bassline
    const bass = ctx.createOscillator(); bass.type = 'square';
    const bassGain = ctx.createGain(); bassGain.gain.value = 0.18;
    bass.connect(bassGain); bassGain.connect(this.musicGain);
    const t0 = ctx.currentTime;
    const seq = [55, 55, 82.41, 55, 65.41, 55, 82.41, 55]; // G1.. patterns
    for (let i = 0; i < seq.length; i++) {
      const t = t0 + i * 0.25;
      bass.frequency.setValueAtTime(seq[i], t);
    }
    bass.start(t0);
    bass.stop(t0 + 2.1);
    this.musicNodes.bass = bass;
    // Arp
    const arp = ctx.createOscillator(); arp.type = 'triangle';
    const arpGain = ctx.createGain(); arpGain.gain.value = 0.08;
    arp.connect(arpGain); arpGain.connect(this.musicGain);
    const notes = [440, 554.37, 659.25, 739.99, 880, 659.25, 554.37, 440]; // more action
    for (let i = 0; i < 16; i++) {
      const t = t0 + i * 0.125;
      arp.frequency.setValueAtTime(notes[i % notes.length], t);
    }
    arp.start(t0);
    arp.stop(t0 + 2.0);
    this.musicNodes.arp = arp;
    // Hats via short noise bursts
    const kickInt = window.setInterval(() => {
      const n = this.createNoise(22); if (!n || !this.musicGain) return;
      const hp = ctx.createBiquadFilter(); hp.type = 'highpass'; hp.frequency.value = 6000;
      const g = ctx.createGain(); g.gain.value = 0.09;
      n.connect(hp); hp.connect(g); g.connect(this.musicGain);
      n.start(); n.stop(ctx.currentTime + 0.025);
    }, 110);
    this.musicNodes.kickInt = kickInt;
    // Auto-loop every 2 seconds
    window.setTimeout(() => { if (this.musicOn) this.startMusic(); }, 2000);
  }

  private stopMusic() {
    try { if (this.musicNodes.bass) this.musicNodes.bass.stop(); } catch {}
    try { if (this.musicNodes.arp) this.musicNodes.arp.stop(); } catch {}
    if (this.musicNodes.kickInt) window.clearInterval(this.musicNodes.kickInt);
    this.musicNodes = {};
  }
}

const retroSound = RetroSound.getInstance();
export default retroSound;


