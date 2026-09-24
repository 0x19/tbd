// Playing the synthesised strings through a context: one note, a strum, a
// pair of notes one after the other or together. Used by whichever
// playground needs a sound; nothing here touches the microphone.

import { freqOfMidi } from "./pitch";
import { pluck } from "./pluck";

/** One plucked note at `at` seconds on the context's clock (now by default). */
export function playNote(
  ctx: AudioContext,
  midi: number,
  at = ctx.currentTime,
  gain = 1,
  seconds = 2.6,
): void {
  const samples = pluck(ctx.sampleRate, freqOfMidi(midi), seconds);
  const buffer = ctx.createBuffer(1, samples.length, ctx.sampleRate);
  buffer.copyToChannel(samples, 0);
  const source = ctx.createBufferSource();
  source.buffer = buffer;
  const g = ctx.createGain();
  g.gain.value = gain;
  source.connect(g).connect(ctx.destination);
  source.start(Math.max(at, ctx.currentTime));
}

/** The notes low to high, a little apart, as a pick would. */
export function strum(ctx: AudioContext, midis: readonly number[], spread = 0.045): void {
  const t0 = ctx.currentTime + 0.02;
  const gain = 1 / Math.sqrt(Math.max(1, midis.length));
  midis.forEach((m, i) => playNote(ctx, m, t0 + i * spread, gain));
}

/** Two notes: one after the other, or together. */
export function pair(ctx: AudioContext, a: number, b: number, together: boolean, gap = 0.7): void {
  const t0 = ctx.currentTime + 0.02;
  playNote(ctx, a, t0, together ? 0.7 : 1);
  playNote(ctx, b, together ? t0 : t0 + gap, together ? 0.7 : 1);
}
