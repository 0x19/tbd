// A plucked string, synthesised: Karplus-Strong. A burst of noise is fed
// into a delay loop one period long with a gentle low-pass in it, so every
// pass around the loop rounds the sound off the way a real string loses its
// high harmonics first. The delay is fractional, so the pitch is exact and
// not the nearest whole sample. Pure: samples in, samples out.

/**
 * `seconds` of a string at `freq` Hz, as samples at `sampleRate`, peaking
 * near 0.8 of full scale and dying away on its own.
 */
export function pluck(sampleRate: number, freq: number, seconds = 2.6): Float32Array<ArrayBuffer> {
  const n = Math.floor(sampleRate * seconds);
  const out = new Float32Array(new ArrayBuffer(n * 4));
  // The loop averages two neighbouring samples, which delays by half a
  // sample on its own; the read point is set so the whole loop is one period.
  const period = sampleRate / freq;
  const delay = period - 0.5;
  // A lower string keeps its harmonics longer: the loss per pass is smaller.
  const loss = freq < 120 ? 0.9975 : freq < 250 ? 0.996 : 0.994;

  // The excitation: one period of noise with its mean removed and a light
  // low-pass, which is roughly a pick some way from the bridge.
  const burst = Math.ceil(period);
  let mean = 0;
  for (let i = 0; i < burst; i += 1) {
    out[i] = Math.random() * 2 - 1;
    mean += out[i]!;
  }
  mean /= burst;
  let prev = 0;
  for (let i = 0; i < burst; i += 1) {
    const centred = out[i]! - mean;
    out[i] = 0.6 * centred + 0.4 * prev;
    prev = centred;
  }

  const at = (t: number): number => {
    if (t < 0) return 0;
    const i = Math.floor(t);
    const frac = t - i;
    const a = out[i] ?? 0;
    const b = out[i + 1] ?? 0;
    return a + (b - a) * frac;
  };
  for (let i = burst; i < n; i += 1) {
    out[i] = loss * 0.5 * (at(i - delay) + at(i - delay - 1));
  }

  // A short fade at the very end, so a still-ringing low string does not click.
  const tail = Math.min(n, Math.floor(sampleRate * 0.15));
  for (let i = n - tail; i < n; i += 1) {
    out[i]! *= (n - i) / tail;
  }
  // Normalise to a comfortable level.
  let peak = 0;
  for (let i = 0; i < n; i += 1) peak = Math.max(peak, Math.abs(out[i]!));
  if (peak > 0) {
    const g = 0.8 / peak;
    for (let i = 0; i < n; i += 1) out[i]! *= g;
  }
  return out;
}
