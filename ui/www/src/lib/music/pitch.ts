// The arithmetic of a tuner, with no audio in it: a window of samples goes
// in, a frequency comes out, and a frequency becomes a note and the cents
// it is off by. Pure functions, so the page is the only thing that touches
// the microphone.

/** What a window of samples was found to hold. */
export type Pitch = {
  /** Hz. */
  freq: number;
  /** 0..1: how periodic the window was; below ~0.8 it is noise or a chord. */
  clarity: number;
};

/**
 * McLeod's pitch method: the normalised square difference of the window
 * against itself, the first peak after the first dip that reaches most of
 * the way to the strongest, refined between samples with a parabola. It
 * holds the fundamental where plain autocorrelation jumps to an overtone,
 * which is what a low E does through a phone microphone.
 */
export function detectPitch(
  buf: Float32Array,
  sampleRate: number,
  floor = 0.0015,
  minHz = 60,
  maxHz = 1200,
): Pitch | null {
  const n = buf.length;
  // Too quiet to mean anything: the room, not a string. A laptop microphone
  // hears a guitar across a desk at a few thousandths of full scale.
  if (rms(buf) < floor) return null;

  const maxTau = Math.min(n - 1, Math.floor(sampleRate / minHz));
  const minTau = Math.max(2, Math.floor(sampleRate / maxHz));
  // nsdf[tau] = 2 * sum(x[i] x[i+tau]) / sum(x[i]^2 + x[i+tau]^2)
  const nsdf = new Float32Array(maxTau + 1);
  for (let tau = minTau; tau <= maxTau; tau += 1) {
    let acf = 0;
    let norm = 0;
    for (let i = 0; i + tau < n; i += 1) {
      const a = buf[i]!;
      const b = buf[i + tau]!;
      acf += a * b;
      norm += a * a + b * b;
    }
    nsdf[tau] = norm > 0 ? (2 * acf) / norm : 0;
  }

  // Key maxima: one per positive run, after the first crossing below zero.
  let tau = minTau;
  while (tau <= maxTau && nsdf[tau]! > 0) tau += 1;
  const peaks: { tau: number; value: number }[] = [];
  while (tau <= maxTau) {
    while (tau <= maxTau && nsdf[tau]! <= 0) tau += 1;
    let best = -1;
    let bestValue = 0;
    while (tau <= maxTau && nsdf[tau]! > 0) {
      if (nsdf[tau]! > bestValue) {
        bestValue = nsdf[tau]!;
        best = tau;
      }
      tau += 1;
    }
    if (best > 0) peaks.push({ tau: best, value: bestValue });
  }
  if (peaks.length === 0) return null;
  const strongest = Math.max(...peaks.map((p) => p.value));
  const chosen = peaks.find((p) => p.value >= 0.9 * strongest);
  if (!chosen || chosen.value < 0.5) return null;

  // Refine the peak between samples with the parabola through its neighbours.
  let period = chosen.tau;
  const left = nsdf[chosen.tau - 1] ?? chosen.value;
  const right = nsdf[chosen.tau + 1] ?? chosen.value;
  const denom = 2 * (2 * chosen.value - left - right);
  if (denom !== 0) period += (right - left) / denom;
  const freq = sampleRate / period;
  if (!Number.isFinite(freq) || freq < minHz || freq > maxHz) return null;
  return { freq, clarity: Math.min(1, chosen.value) };
}

/** The window's level, 0..1 of full scale. */
export function rms(buf: Float32Array): number {
  let power = 0;
  for (let i = 0; i < buf.length; i += 1) power += buf[i]! * buf[i]!;
  return buf.length ? Math.sqrt(power / buf.length) : 0;
}

const NAMES = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"] as const;

/** Hz of a MIDI note number against the reference for A4 (MIDI 69). */
export function freqOfMidi(midi: number, a4 = 440): number {
  return a4 * 2 ** ((midi - 69) / 12);
}

/** The note nearest a frequency, and how far off it is. */
export function noteOf(
  freq: number,
  a4 = 440,
): { midi: number; name: string; octave: number; cents: number } {
  const exact = 69 + 12 * Math.log2(freq / a4);
  const midi = Math.round(exact);
  return {
    midi,
    name: NAMES[((midi % 12) + 12) % 12]!,
    octave: Math.floor(midi / 12) - 1,
    cents: Math.round((exact - midi) * 100),
  };
}

/** Cents from a frequency to a target. */
export function centsOff(freq: number, target: number): number {
  return Math.round(1200 * Math.log2(freq / target));
}

export type Tuning = { id: string; name: string; note: string; strings: number[] };

/** Strings low to high, as MIDI numbers. */
export const TUNINGS: Tuning[] = [
  { id: "standard", name: "Standard", note: "E A D G B E", strings: [40, 45, 50, 55, 59, 64] },
  { id: "drop-d", name: "Drop D", note: "D A D G B E", strings: [38, 45, 50, 55, 59, 64] },
  { id: "half-down", name: "Half step down", note: "E♭ A♭ D♭ G♭ B♭ E♭", strings: [39, 44, 49, 54, 58, 63] },
  { id: "dadgad", name: "DADGAD", note: "D A D G A D", strings: [38, 45, 50, 55, 57, 62] },
  { id: "open-g", name: "Open G", note: "D G D G B D", strings: [38, 43, 50, 55, 59, 62] },
];

/** The string of a tuning nearest to a frequency. */
export function nearestString(freq: number, tuning: Tuning, a4 = 440): number {
  let best = 0;
  let bestDistance = Infinity;
  tuning.strings.forEach((midi, i) => {
    const d = Math.abs(centsOff(freq, freqOfMidi(midi, a4)));
    if (d < bestDistance) {
      bestDistance = d;
      best = i;
    }
  });
  return best;
}

/** The middle value of the last few readings: a needle that does not shake. */
export function median(values: number[]): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[mid]! : (sorted[mid - 1]! + sorted[mid]!) / 2;
}
