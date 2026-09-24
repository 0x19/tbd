"use client";

// The microphone, once, for every playground that listens: opened on
// request, read every frame, closed on stop or leaving. Nothing here
// records, uploads or keeps a sample; the frame callback gets the window
// and forgets it.

import { useCallback, useEffect, useRef, useState } from "react";

import { rms } from "@/lib/music/pitch";

export type MicStatus = "idle" | "starting" | "listening" | "blocked" | "unsupported";
export type Input = { id: string; label: string };
export type Frame = (buf: Float32Array, sampleRate: number, analyser: AnalyserNode) => void;

export type Mic = {
  status: MicStatus;
  /** The current window's level, 0..1 of full scale. */
  level: number;
  inputs: Input[];
  inputId: string;
  start: (deviceId?: string) => Promise<void>;
  stop: () => void;
  /** The audio context, made on first use, for playing sounds too. */
  context: () => AudioContext;
};

export function useMic(onFrame: Frame, fftSize = 4096): Mic {
  const [status, setStatus] = useState<MicStatus>("idle");
  const [level, setLevel] = useState(0);
  const [inputs, setInputs] = useState<Input[]>([]);
  const [inputId, setInputId] = useState("");
  const ctx = useRef<AudioContext | null>(null);
  const stream = useRef<MediaStream | null>(null);
  const analyser = useRef<AnalyserNode | null>(null);
  const frame = useRef(0);
  const callback = useRef<Frame>(onFrame);
  useEffect(() => {
    callback.current = onFrame;
  }, [onFrame]);

  const context = useCallback((): AudioContext => {
    if (!ctx.current) ctx.current = new AudioContext();
    return ctx.current;
  }, []);

  const stop = useCallback(() => {
    cancelAnimationFrame(frame.current);
    stream.current?.getTracks().forEach((t) => t.stop());
    stream.current = null;
    analyser.current = null;
    setLevel(0);
    setStatus("idle");
  }, []);

  useEffect(
    () => () => {
      stop();
      void ctx.current?.close();
      ctx.current = null;
    },
    [stop],
  );

  const tick = useCallback(() => {
    const node = analyser.current;
    const audio = ctx.current;
    if (!node || !audio) return;
    const buf = new Float32Array(node.fftSize);
    node.getFloatTimeDomainData(buf);
    setLevel(rms(buf));
    callback.current(buf, audio.sampleRate, node);
    frame.current = requestAnimationFrame(tick);
  }, []);

  const start = useCallback(
    async (deviceId = inputId) => {
      if (
        typeof navigator === "undefined" ||
        !navigator.mediaDevices?.getUserMedia ||
        !("AudioContext" in window)
      ) {
        setStatus("unsupported");
        return;
      }
      stop();
      setStatus("starting");
      try {
        const media = await navigator.mediaDevices.getUserMedia({
          audio: {
            ...(deviceId ? { deviceId: { exact: deviceId } } : {}),
            echoCancellation: false,
            noiseSuppression: false,
            autoGainControl: false,
          },
        });
        const devices = await navigator.mediaDevices.enumerateDevices();
        setInputs(
          devices
            .filter((d) => d.kind === "audioinput")
            .map((d, i) => ({ id: d.deviceId, label: d.label || `Microphone ${i + 1}` })),
        );
        setInputId(media.getAudioTracks()[0]?.getSettings().deviceId ?? deviceId);
        const audio = context();
        await audio.resume();
        const node = audio.createAnalyser();
        node.fftSize = fftSize;
        node.smoothingTimeConstant = 0;
        audio.createMediaStreamSource(media).connect(node);
        stream.current = media;
        analyser.current = node;
        setStatus("listening");
        frame.current = requestAnimationFrame(tick);
      } catch {
        setStatus("blocked");
      }
    },
    [context, fftSize, inputId, stop, tick],
  );

  return { status, level, inputs, inputId, start, stop, context };
}
