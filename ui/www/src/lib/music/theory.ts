// Notes, intervals, chords and a fretboard, as arithmetic on MIDI numbers.
// Pure, so every playground names the same things the same way.

export const SHARP = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"] as const;
export const FLAT = ["C", "D♭", "D", "E♭", "E", "F", "G♭", "G", "A♭", "A", "B♭", "B"] as const;

/** Pitch class 0..11 of a MIDI number. */
export function pc(midi: number): number {
  return ((midi % 12) + 12) % 12;
}

/** The note's name, sharps by default. */
export function name(midi: number, flats = false): string {
  return (flats ? FLAT : SHARP)[pc(midi)]!;
}

/** Scientific octave: MIDI 60 is C4. */
export function octave(midi: number): number {
  return Math.floor(midi / 12) - 1;
}

/** Standard tuning, low E to high E, as MIDI numbers. */
export const STANDARD = [40, 45, 50, 55, 59, 64] as const;

/** The note at a fret of a string (0 = low E) in a tuning. */
export function noteAt(string: number, fret: number, tuning: readonly number[] = STANDARD): number {
  return tuning[string]! + fret;
}

export type Interval = { semis: number; short: string; name: string };

export const INTERVALS: readonly Interval[] = [
  { semis: 0, short: "P1", name: "unison" },
  { semis: 1, short: "m2", name: "minor second" },
  { semis: 2, short: "M2", name: "major second" },
  { semis: 3, short: "m3", name: "minor third" },
  { semis: 4, short: "M3", name: "major third" },
  { semis: 5, short: "P4", name: "perfect fourth" },
  { semis: 6, short: "TT", name: "tritone" },
  { semis: 7, short: "P5", name: "perfect fifth" },
  { semis: 8, short: "m6", name: "minor sixth" },
  { semis: 9, short: "M6", name: "major sixth" },
  { semis: 10, short: "m7", name: "minor seventh" },
  { semis: 11, short: "M7", name: "major seventh" },
  { semis: 12, short: "P8", name: "octave" },
];

export type ChordType = { id: string; symbol: string; name: string; tones: readonly number[] };

/** Chord formulas as semitones above the root; the ninths sit an octave up. */
export const CHORDS: readonly ChordType[] = [
  { id: "maj", symbol: "", name: "major", tones: [0, 4, 7] },
  { id: "min", symbol: "m", name: "minor", tones: [0, 3, 7] },
  { id: "dim", symbol: "dim", name: "diminished", tones: [0, 3, 6] },
  { id: "aug", symbol: "aug", name: "augmented", tones: [0, 4, 8] },
  { id: "sus2", symbol: "sus2", name: "suspended second", tones: [0, 2, 7] },
  { id: "sus4", symbol: "sus4", name: "suspended fourth", tones: [0, 5, 7] },
  { id: "5", symbol: "5", name: "power chord", tones: [0, 7] },
  { id: "7", symbol: "7", name: "dominant seventh", tones: [0, 4, 7, 10] },
  { id: "maj7", symbol: "maj7", name: "major seventh", tones: [0, 4, 7, 11] },
  { id: "m7", symbol: "m7", name: "minor seventh", tones: [0, 3, 7, 10] },
  { id: "dim7", symbol: "dim7", name: "diminished seventh", tones: [0, 3, 6, 9] },
  { id: "m7b5", symbol: "m7♭5", name: "half-diminished", tones: [0, 3, 6, 10] },
  { id: "6", symbol: "6", name: "major sixth", tones: [0, 4, 7, 9] },
  { id: "m6", symbol: "m6", name: "minor sixth", tones: [0, 3, 7, 9] },
  { id: "7sus4", symbol: "7sus4", name: "seventh suspended fourth", tones: [0, 5, 7, 10] },
  { id: "add9", symbol: "add9", name: "added ninth", tones: [0, 4, 7, 14] },
  { id: "9", symbol: "9", name: "dominant ninth", tones: [0, 4, 7, 10, 14] },
  { id: "maj9", symbol: "maj9", name: "major ninth", tones: [0, 4, 7, 11, 14] },
  { id: "m9", symbol: "m9", name: "minor ninth", tones: [0, 3, 7, 10, 14] },
];

export type ChordName = { root: number; chord: ChordType; symbol: string; bass?: number };

/**
 * Every chord the pitch classes spell, most likely first: the one whose root
 * is in the bass, then the one with the fewest tones. `bass` is the lowest
 * sounding pitch class, so an inversion is named with its slash.
 */
export function nameChord(midis: readonly number[], flats = false): ChordName[] {
  if (midis.length === 0) return [];
  const classes = new Set(midis.map(pc));
  const bass = pc(Math.min(...midis));
  const found: ChordName[] = [];
  for (let root = 0; root < 12; root += 1) {
    for (const chord of CHORDS) {
      const tones = new Set(chord.tones.map((t) => pc(root + t)));
      if (tones.size !== classes.size || ![...classes].every((c) => tones.has(c))) continue;
      const slash = bass === root ? "" : `/${name(bass, flats)}`;
      found.push({
        root,
        chord,
        symbol: `${name(root, flats)}${chord.symbol}${slash}`,
        ...(bass === root ? {} : { bass }),
      });
    }
  }
  return found.sort(
    (a, b) =>
      Number(b.root === bass) - Number(a.root === bass) || a.chord.tones.length - b.chord.tones.length,
  );
}

const ALIASES: Record<string, string> = {
  "": "maj",
  maj: "maj",
  M: "maj",
  major: "maj",
  m: "min",
  min: "min",
  "-": "min",
  minor: "min",
  dim: "dim",
  "°": "dim",
  o: "dim",
  aug: "aug",
  "+": "aug",
  sus2: "sus2",
  sus4: "sus4",
  sus: "sus4",
  "5": "5",
  "7": "7",
  dom7: "7",
  maj7: "maj7",
  M7: "maj7",
  Δ7: "maj7",
  Δ: "maj7",
  m7: "m7",
  min7: "m7",
  "-7": "m7",
  dim7: "dim7",
  "°7": "dim7",
  m7b5: "m7b5",
  "m7♭5": "m7b5",
  ø: "m7b5",
  ø7: "m7b5",
  "6": "6",
  m6: "m6",
  "7sus4": "7sus4",
  "7sus": "7sus4",
  add9: "add9",
  "9": "9",
  maj9: "maj9",
  M9: "maj9",
  m9: "m9",
  min9: "m9",
};

/** "F#m7", "Bbmaj7", "C", "G/B" → root pitch class, chord type and any slash bass. */
export function parseChord(text: string): { root: number; chord: ChordType; bass?: number } | null {
  const m = /^\s*([A-Ga-g])([#♯bB♭]?)\s*([^/\s]*)\s*(?:\/\s*([A-Ga-g])([#♯bB♭]?))?\s*$/.exec(text);
  if (!m) return null;
  const letter = (l: string, acc: string) => {
    const base = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 }[l.toUpperCase()]!;
    const shift = acc === "#" || acc === "♯" ? 1 : acc === "b" || acc === "B" || acc === "♭" ? -1 : 0;
    return pc(base + shift);
  };
  const root = letter(m[1]!, m[2] ?? "");
  const id = ALIASES[m[3] ?? ""];
  const chord = CHORDS.find((c) => c.id === id);
  if (!chord) return null;
  const bass = m[4] ? letter(m[4], m[5] ?? "") : undefined;
  return bass === undefined || bass === root ? { root, chord } : { root, chord, bass };
}

/** A way to play a chord: one fret per string, low to high, `null` muted. */
export type Voicing = { frets: (number | null)[]; base: number };

/**
 * Playable shapes for a chord in a tuning: every chord tone present, the
 * root in the bass, muted strings only below the lowest played one, and
 * the fretted notes within `span` frets of each other. Lowest on the neck
 * first, then the fewest fingers.
 */
export function voicings(
  root: number,
  chord: ChordType,
  tuning: readonly number[] = STANDARD,
  maxFret = 12,
  span = 3,
): Voicing[] {
  const tones = new Set(chord.tones.map((t) => pc(root + t)));
  const out: Voicing[] = [];
  const seen = new Set<string>();
  for (let base = 0; base + span <= maxFret; base += 1) {
    const options = tuning.map((open) => {
      const frets: (number | null)[] = [null];
      if (tones.has(pc(open))) frets.push(0);
      for (let f = Math.max(1, base); f <= base + span; f += 1) if (tones.has(pc(open + f))) frets.push(f);
      return frets;
    });
    const walk = (string: number, chosen: (number | null)[]) => {
      if (string === tuning.length) {
        const sounding = chosen.map((f, i) => (f === null ? null : tuning[i]! + f));
        const first = sounding.findIndex((n) => n !== null);
        if (first < 0 || sounding.slice(first).some((n) => n === null)) return;
        const notes = sounding.filter((n): n is number => n !== null);
        if (notes.length < Math.min(4, tuning.length) || notes.length < chord.tones.length) return;
        if (pc(notes[0]!) !== root) return;
        const classes = new Set(notes.map(pc));
        if (![...tones].every((t) => classes.has(t))) return;
        const fretted = chosen.filter((f): f is number => f !== null && f > 0);
        if (fretted.length && Math.max(...fretted) - Math.min(...fretted) > span) return;
        const key = chosen.map((f) => (f === null ? "x" : f)).join(",");
        if (seen.has(key)) return;
        seen.add(key);
        out.push({ frets: chosen, base: fretted.length ? Math.min(...fretted) : 0 });
        return;
      }
      for (const f of options[string]!) walk(string + 1, [...chosen, f]);
    };
    walk(0, []);
  }
  return out.sort((a, b) => a.base - b.base || fingers(a) - fingers(b)).slice(0, 12);
}

function fingers(v: Voicing): number {
  return v.frets.filter((f) => f !== null && f > 0).length;
}
