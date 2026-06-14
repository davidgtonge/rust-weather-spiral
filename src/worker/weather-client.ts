import { Decoder, encode } from "cbor-x";
import type { EngineUpdate } from "@dtonge/engine-shell";
import type {
  AppEvent,
  EffectCommand,
  ViewModel,
  ViewModelPatch,
  WorkerInput,
  WorkerOutput,
} from "../protocol/types";
import type { PresentationSidecar } from "./drain-presentation-effects";
import type { FrameTimings, WorkerTimings } from "./timings";

const metaDecoder = new Decoder({ copyBuffers: false, useRecords: false });

type OutboundMessage = {
  bytes: ArrayBuffer;
  presentation?: PresentationSidecar;
  timings: WorkerTimings;
};

export type WeatherUpdate = EngineUpdate<ViewModel, ViewModelPatch, EffectCommand> & {
  presentation?: PresentationSidecar | null;
  timings?: FrameTimings;
  debug?: WorkerOutput;
};

type Job = {
  input: WorkerInput;
  sentAt: number;
  label: string;
  resolve: (u: WeatherUpdate) => void;
  reject: (e: Error) => void;
};

export type WeatherWorkerClient = {
  init: (input: WorkerInput) => Promise<WeatherUpdate>;
  dispatch: (input: WorkerInput) => Promise<WeatherUpdate>;
  dispose: () => void;
};

function encodeWorkerInput(input: WorkerInput): Uint8Array {
  return encode(input) as Uint8Array;
}

function decodeWorkerOutput(bytes: ArrayBuffer): WorkerOutput {
  return metaDecoder.decode(new Uint8Array(bytes)) as WorkerOutput;
}

function inputLabel(input: WorkerInput): string {
  if (input.kind === "init") return "init";
  return input.event.type;
}

function parseUpdate(msg: OutboundMessage, sentAt: number): WeatherUpdate {
  const decodeStart = performance.now();
  const output = decodeWorkerOutput(msg.bytes);
  const decodeMs = performance.now() - decodeStart;
  const roundTripMs = performance.now() - sentAt;

  if (output.kind === "error") {
    throw new Error(output.message);
  }

  const timings: FrameTimings = {
    ...msg.timings,
    decodeMs,
    roundTripMs,
  };

  const presentation = msg.presentation ?? null;

  if (output.kind === "initialized") {
    return {
      viewModel: output.viewModel,
      patches: [],
      effects: output.effects,
      diagnostics: [],
      presentation,
      timings,
      debug: output,
    };
  }

  return {
    patches: output.patches,
    effects: output.effects,
    diagnostics: output.diagnostics,
    presentation,
    timings,
    debug: output,
  };
}

export function createWeatherWorkerClient(
  onTimings?: (label: string, timings: FrameTimings, output: WorkerOutput) => void,
): WeatherWorkerClient {
  const worker = new Worker(new URL("./app-worker.ts", import.meta.url), { type: "module" });
  const jobs: Job[] = [];
  let busy = false;

  worker.onmessage = (event: MessageEvent<OutboundMessage>) => {
    const job = jobs.shift();
    if (!job) {
      busy = false;
      return;
    }

    try {
      const update = parseUpdate(event.data, job.sentAt);
      onTimings?.(job.label, update.timings!, update.debug!);
      job.resolve(update);
    } catch (err) {
      job.reject(err instanceof Error ? err : new Error(String(err)));
    }

    busy = false;
    pump();
  };

  worker.onerror = (err) => {
    jobs.forEach((j) => j.reject(new Error(String(err.message))));
    jobs.length = 0;
    busy = false;
  };

  function pump(): void {
    if (busy || jobs.length === 0) return;
    const job = jobs[0]!;
    busy = true;
    const bytes = encodeWorkerInput(job.input);
    const buffer = bytes.buffer.slice(
      bytes.byteOffset,
      bytes.byteOffset + bytes.byteLength,
    ) as ArrayBuffer;
    worker.postMessage({ bytes: buffer }, [buffer]);
  }

  function enqueue(input: WorkerInput): Promise<WeatherUpdate> {
    const label = inputLabel(input);
    return new Promise((resolve, reject) => {
      jobs.push({ input, sentAt: performance.now(), label, resolve, reject });
      pump();
    });
  }

  return {
    init: enqueue,
    dispatch: enqueue,
    dispose: () => worker.terminate(),
  };
}

export function weatherInitInput(weatherBundle: Uint8Array): WorkerInput {
  return { kind: "init", weatherBundle };
}

export function weatherEventInput(event: AppEvent): WorkerInput {
  return { kind: "event", event };
}

export function weatherRequestFrameEvent(): AppEvent {
  return { type: "requestFrame" };
}
