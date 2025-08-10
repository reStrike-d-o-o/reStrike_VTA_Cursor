class RetroSound {
  private static instance: RetroSound;
  private ctx: AudioContext | null = null;
  private muted = false;
  private musicOn = false;
  private musicGain: GainNode | null = null;
  private musicNodes: { bass?: OscillatorNode; arp?: OscillatorNode } = {};
  private timers: number[] = [];

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

  // New action background music: 132 BPM, layered kick/snare/hats + bass + arp
  private async startMusic() {
    await this.resume(); if (!this.ctx) return;
    const ctx = this.ctx;
    if (!this.musicGain) { this.musicGain = ctx.createGain(); this.musicGain.gain.value = 0.18; this.musicGain.connect(ctx.destination); }

    const bpm = 132;
    const stepDur = 60 / bpm / 4; // 16th note
    let step = 0;

    const scale = [49, 55, 58.27, 65.41, 73.42, 82.41]; // G2, A2, Bb2, C3, D3, E3
    const bassSeq = [0, 0, 3, 0, 4, 0, 5, 0];
    const arpNotes = [392, 494, 587, 659, 784, 659, 587, 494]; // G4-B4-D5-E5-G5-...

    const playKick = (t: number) => {
      const o = ctx.createOscillator(); o.type = 'sine';
      const g = ctx.createGain();
      o.frequency.setValueAtTime(120, t);
      o.frequency.exponentialRampToValueAtTime(55, t + 0.12);
      g.gain.setValueAtTime(0.6, t);
      g.gain.exponentialRampToValueAtTime(0.001, t + 0.13);
      o.connect(g); g.connect(this.musicGain!);
      o.start(t); o.stop(t + 0.14);
    };

    const playSnare = (t: number) => {
      const n = this.createNoise(120); if (!n) return;
      const bp = ctx.createBiquadFilter(); bp.type = 'bandpass'; bp.frequency.value = 1800; bp.Q.value = 0.6;
      const g = ctx.createGain(); g.gain.setValueAtTime(0.22, t); g.gain.exponentialRampToValueAtTime(0.001, t + 0.12);
      n.connect(bp); bp.connect(g); g.connect(this.musicGain!);
      n.start(t); n.stop(t + 0.12);
    };

    const playHat = (t: number) => {
      const n = this.createNoise(40); if (!n) return;
      const hp = ctx.createBiquadFilter(); hp.type = 'highpass'; hp.frequency.value = 7000;
      const g = ctx.createGain(); g.gain.setValueAtTime(0.08, t); g.gain.exponentialRampToValueAtTime(0.001, t + 0.04);
      n.connect(hp); hp.connect(g); g.connect(this.musicGain!);
      n.start(t); n.stop(t + 0.04);
    };

    const playBass = (t: number, noteIdx: number) => {
      const freq = scale[noteIdx % scale.length];
      const o = ctx.createOscillator(); o.type = 'square';
      const g = ctx.createGain(); g.gain.value = 0.12;
      o.frequency.setValueAtTime(freq, t);
      o.connect(g); g.connect(this.musicGain!);
      o.start(t); o.stop(t + stepDur * 3);
    };

    const playArp = (t: number, idx: number) => {
      const o = ctx.createOscillator(); o.type = 'triangle';
      const g = ctx.createGain(); g.gain.value = 0.07;
      o.frequency.setValueAtTime(arpNotes[idx % arpNotes.length], t);
      o.connect(g); g.connect(this.musicGain!);
      o.start(t); o.stop(t + stepDur * 2);
    };

    const interval = window.setInterval(() => {
      if (!this.ctx || !this.musicOn) return;
      const now = this.ctx.currentTime + 0.01;
      // Drums
      if (step % 4 === 0) playKick(now); // beats 1 and 3
      if (step % 8 === 4) playSnare(now); // beats 2 and 4
      playHat(now);
      // Bassline on quarter notes
      if (step % 4 === 0) playBass(now, bassSeq[(step / 4) % bassSeq.length]);
      // Arp on 8th notes
      if (step % 2 === 0) playArp(now, step / 2);
      step = (step + 1) % 16;
    }, stepDur * 1000);
    this.timers.push(interval);
  }

  private stopMusic() {
    try { if (this.musicNodes.bass) this.musicNodes.bass.stop(); } catch {}
    try { if (this.musicNodes.arp) this.musicNodes.arp.stop(); } catch {}
    this.musicNodes = {};
    this.timers.forEach((id) => window.clearInterval(id));
    this.timers = [];
  }
}

const retroSound = RetroSound.getInstance();
export default retroSound;


