import type { EngineEnvelope, FrozenPackage } from "../types/engine";
import type {
  BoundAudioAsset,
  FrozenOwnerKey,
  PublicPresentationBatch,
  PublicPresentationEvent,
  PublicVoice,
} from "../types/presentation";
import {
  appearanceState,
  currentAppearanceBaseUrl,
  currentAppearanceSourceKey,
} from "./appearanceStore";
import { originalResourceUrl } from "./originalResources";
import { validatePresentationResponse } from "./presentationMetadata";

export const ACCEPTED_ORIGINAL_OWNER: Extract<FrozenOwnerKey, { kind: "original" }> = {
  kind: "original",
  repository: "https://github.com/libnoname/noname.git",
  adopted_revision: "1a548c95acdf8ac1ef44d74473403eda2ac86cf0",
  catalog_sha256: "2c375927fec9387a2a78e58db7d49422091fb17427c082e220db5e18ac0779fe",
  asset_manifest_sha256: "7e5792aaefc8fbecd880581fbbea4dab5fed3c09d8a3ffe625fca65c4ffa5493",
};

export const MAX_BATTLE_AUDIO_QUEUE = 8;
const MAX_RETAINED_REVISIONS = 16;
const MAX_PATH_ERRORS = 64;

export interface AudioElementLike {
  src: string;
  volume: number;
  muted: boolean;
  preload?: string;
  crossOrigin?: string | null;
  onended: ((ev?: any) => any) | null;
  onerror: ((ev?: any) => any) | null;
  onloadeddata: ((ev?: any) => any) | null;
  play(): Promise<void>;
  pause(): void;
  load?(): void;
  removeAttribute?(name: string): void;
}

export interface QueuedAudioCue {
  cueKey: string;
  generation: number;
  revision: number;
  ordinal: number;
  eventKind: PublicPresentationEvent["kind"];
  actor: number;
  roleKey: string;
  form: string | null;
  url: string;
  pathKey: string;
  digest: string;
  digestAlgorithm: "sha256" | "blake3";
  size: number;
}

export interface AdmissionOutcome {
  admitted: number;
  duplicate: boolean;
  stale: boolean;
  cues: QueuedAudioCue[];
}

export interface BattleAudioOwnerOptions {
  createAudio?: () => AudioElementLike;
  originalOwnerPin?: Extract<FrozenOwnerKey, { kind: "original" }>;
  originalBaseUrl?: () => string;
  allowRemotePackageFallback?: () => boolean;
}

const BLAKE3_IV = new Uint32Array([
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
  0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]);
const BLAKE3_MSG_PERM = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8];
const CHUNK_START = 1;
const CHUNK_END = 2;
const PARENT = 4;
const ROOT = 8;

function rotr32(x: number, n: number): number {
  return ((x >>> n) | (x << (32 - n))) >>> 0;
}

function blake3G(s: Uint32Array, a: number, b: number, c: number, d: number, mx: number, my: number): void {
  s[a] = (s[a] + s[b] + mx) >>> 0;
  s[d] = rotr32(s[d] ^ s[a], 16);
  s[c] = (s[c] + s[d]) >>> 0;
  s[b] = rotr32(s[b] ^ s[c], 12);
  s[a] = (s[a] + s[b] + my) >>> 0;
  s[d] = rotr32(s[d] ^ s[a], 8);
  s[c] = (s[c] + s[d]) >>> 0;
  s[b] = rotr32(s[b] ^ s[c], 7);
}

function blake3Round(s: Uint32Array, m: Uint32Array): void {
  blake3G(s, 0, 4, 8, 12, m[0], m[1]);
  blake3G(s, 1, 5, 9, 13, m[2], m[3]);
  blake3G(s, 2, 6, 10, 14, m[4], m[5]);
  blake3G(s, 3, 7, 11, 15, m[6], m[7]);
  blake3G(s, 0, 5, 10, 15, m[8], m[9]);
  blake3G(s, 1, 6, 11, 12, m[10], m[11]);
  blake3G(s, 2, 7, 8, 13, m[12], m[13]);
  blake3G(s, 3, 4, 9, 14, m[14], m[15]);
}

function blake3Compress(
  cv: Uint32Array,
  blockWords: Uint32Array,
  counter: number,
  blockLen: number,
  flags: number,
): Uint32Array {
  const s = new Uint32Array(16);
  s.set(cv, 0);
  s.set(BLAKE3_IV.subarray(0, 4), 8);
  s[12] = counter >>> 0;
  s[13] = Math.floor(counter / 0x100000000) >>> 0;
  s[14] = blockLen >>> 0;
  s[15] = flags >>> 0;
  let m = new Uint32Array(blockWords);
  for (let r = 0; r < 7; r++) {
    blake3Round(s, m);
    if (r < 6) {
      const next = new Uint32Array(16);
      for (let i = 0; i < 16; i++) next[i] = m[BLAKE3_MSG_PERM[i]];
      m = next;
    }
  }
  const out = new Uint32Array(8);
  for (let i = 0; i < 8; i++) out[i] = (s[i] ^ s[i + 8]) >>> 0;
  return out;
}

function bytesToWords16(bytes: Uint8Array, offset: number, len: number): Uint32Array {
  const block = new Uint8Array(64);
  block.set(bytes.subarray(offset, offset + len));
  const words = new Uint32Array(16);
  const dv = new DataView(block.buffer);
  for (let i = 0; i < 16; i++) words[i] = dv.getUint32(i * 4, true);
  return words;
}

export function blake3Hex(input: Uint8Array): string {
  const numChunks = Math.max(1, Math.ceil(input.length / 1024));
  const cvStack: Uint32Array[] = [];

  function chunkOutput(chunkIdx: number): { cv: Uint32Array; words: Uint32Array; counter: number; blockLen: number; flags: number } {
    const start = chunkIdx * 1024;
    const chunkLen = Math.min(1024, Math.max(0, input.length - start));
    let cv: Uint32Array = new Uint32Array(BLAKE3_IV);
    const blocks = Math.max(1, Math.ceil(chunkLen / 64));
    for (let b = 0; b < blocks; b++) {
      const blockOffset = start + b * 64;
      const blockLen = chunkLen === 0 ? 0 : Math.min(64, start + chunkLen - blockOffset);
      const words = bytesToWords16(input, blockOffset, blockLen);
      const flags = (b === 0 ? CHUNK_START : 0) | (b === blocks - 1 ? CHUNK_END : 0);
      if (b === blocks - 1) {
        return { cv, words, counter: chunkIdx, blockLen, flags };
      }
      cv = blake3Compress(cv, words, chunkIdx, blockLen, flags);
    }
    return { cv, words: new Uint32Array(16), counter: chunkIdx, blockLen: 0, flags: CHUNK_START | CHUNK_END };
  }

  function parentOutput(left: Uint32Array, right: Uint32Array) {
    const words = new Uint32Array(16);
    words.set(left, 0);
    words.set(right, 8);
    return { cv: new Uint32Array(BLAKE3_IV), words, counter: 0, blockLen: 64, flags: PARENT };
  }

  let lastNode = chunkOutput(0);
  if (numChunks > 1) {
    for (let chunkIdx = 0; chunkIdx < numChunks; chunkIdx++) {
      const node = chunkOutput(chunkIdx);
      let cv = blake3Compress(node.cv, node.words, node.counter, node.blockLen, node.flags);
      let totalChunks = chunkIdx + 1;
      if (totalChunks < numChunks) {
        while ((totalChunks & 1) === 0) {
          const left = cvStack.pop()!;
          const p = parentOutput(left, cv);
          cv = blake3Compress(p.cv, p.words, p.counter, p.blockLen, p.flags);
          totalChunks >>>= 1;
        }
        cvStack.push(cv);
      } else {
        cvStack.push(cv);
      }
    }
    while (cvStack.length > 2) {
      const right = cvStack.pop()!;
      const left = cvStack.pop()!;
      const p = parentOutput(left, right);
      cvStack.push(blake3Compress(p.cv, p.words, p.counter, p.blockLen, p.flags));
    }
    const right = cvStack.pop()!;
    const left = cvStack.pop()!;
    lastNode = parentOutput(left, right);
  }

  const rootWords = blake3Compress(
    lastNode.cv,
    lastNode.words,
    lastNode.counter,
    lastNode.blockLen,
    lastNode.flags | ROOT,
  );
  const outBytes = new Uint8Array(32);
  const dv = new DataView(outBytes.buffer);
  for (let i = 0; i < 8; i++) dv.setUint32(i * 4, rootWords[i], true);
  return Array.from(outBytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

function computeFrozenContentHashHex(pkg: FrozenPackage): string {
  const enc = new TextEncoder();
  const chunks: Uint8Array[] = [
    enc.encode("noname-next/content-package/v1\0"),
    enc.encode(pkg.package_json),
    new Uint8Array([0]),
  ];
  const entries = Object.entries(pkg.assets).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
  for (const [relPath, bytes] of entries) {
    chunks.push(enc.encode(relPath), new Uint8Array([0]), Uint8Array.from(bytes), new Uint8Array([0xff]));
  }
  const total = chunks.reduce((sum, c) => sum + c.length, 0);
  const merged = new Uint8Array(total);
  let offset = 0;
  for (const c of chunks) {
    merged.set(c, offset);
    offset += c.length;
  }
  return blake3Hex(merged);
}

interface VerifiedFrozenEntry {
  frozen: FrozenPackage;
  assetPaths: Record<string, string>;
  assetHashes: Record<string, string>;
}

function audioMimeType(path: string): string {
  const lower = path.toLowerCase();
  if (lower.endsWith(".mp3")) return "audio/mpeg";
  if (lower.endsWith(".ogg") || lower.endsWith(".opus")) return "audio/ogg";
  if (lower.endsWith(".wav")) return "audio/wav";
  if (lower.endsWith(".m4a") || lower.endsWith(".mp4")) return "audio/mp4";
  if (lower.endsWith(".aac")) return "audio/aac";
  if (lower.endsWith(".flac")) return "audio/flac";
  return "application/octet-stream";
}

function bytesToBase64(bytes: Uint8Array): string {
  const nodeBuffer = (
    globalThis as { Buffer?: { from(data: Uint8Array): { toString(enc: string): string } } }
  ).Buffer;
  if (nodeBuffer !== undefined) {
    return nodeBuffer.from(bytes).toString("base64");
  }
  let binary = "";
  for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
  return btoa(binary);
}

function roleKeyFromVoice(voice: PublicVoice): string {
  const role = voice.role;
  if (role.owner.kind === "original" && typeof role.local_id === "object") {
    return `original:${role.local_id.pack}:${role.local_id.raw_symbol}`;
  }
  return `package:${
    role.owner.kind === "package"
      ? `${role.owner.namespace}/${role.owner.package_id}@${role.owner.version}`
      : "unknown"
  }#${String(role.local_id)}`;
}

function extractEventVoice(event: PublicPresentationEvent): { actor: number; voice: PublicVoice } | null {
  switch (event.kind) {
    case "card_used":
    case "card_responded":
    case "skill_activated":
    case "player_died":
      return event.voice ? { actor: event.actor, voice: event.voice } : null;
    default:
      return null;
  }
}

export class BattleAudioOwner {
  private sessionGeneration = 1;
  private latestRevision = 0;
  private baselineRevision = 0;
  private sourceKey = "";
  private readonly consumedCueKeys = new Set<string>();
  private readonly retainedBatches = new Map<number, string>();
  private readonly frozenBySlot = new Map<string, VerifiedFrozenEntry>();
  private readonly queue: QueuedAudioCue[] = [];
  private activeAudio: AudioElementLike | null = null;
  private activePlaySeq = 0;
  private activePathKey: string | null = null;
  private muted = false;
  private volume = 0.85;
  private gestureUnlocked = false;
  private autoplayBlocked = false;
  private droppedOverflowCount = 0;
  private lastDiagnostic: string | null = null;
  private readonly pathErrors = new Set<string>();
  private readonly playedHistory: QueuedAudioCue[] = [];

  private readonly createAudio: () => AudioElementLike;
  private readonly originalOwnerPin: Extract<FrozenOwnerKey, { kind: "original" }>;
  private readonly originalBaseUrl: () => string;
  private readonly allowRemotePackageFallback: () => boolean;

  constructor(options: BattleAudioOwnerOptions = {}) {
    this.createAudio =
      options.createAudio ??
      (() => {
        if (typeof Audio !== "undefined") return new Audio();
        throw new Error("HTMLAudioElement unavailable");
      });
    this.originalOwnerPin = options.originalOwnerPin ?? ACCEPTED_ORIGINAL_OWNER;
    this.originalBaseUrl =
      options.originalBaseUrl ??
      (() => {
        const base = currentAppearanceBaseUrl() ?? "http://127.0.0.1:7700";
        return `${base.replace(/\/+$/, "")}/original-content`;
      });
    this.allowRemotePackageFallback = options.allowRemotePackageFallback ?? (() => false);
  }

  get generation(): number {
    return this.sessionGeneration;
  }

  get currentRevision(): number {
    return this.latestRevision;
  }

  get isMuted(): boolean {
    return this.muted;
  }

  get currentVolume(): number {
    return this.volume;
  }

  get isAutoplayBlocked(): boolean {
    return this.autoplayBlocked;
  }

  get isGestureUnlocked(): boolean {
    return this.gestureUnlocked;
  }

  get pendingQueueLength(): number {
    return this.queue.length;
  }

  get hasActiveAudio(): boolean {
    return this.activeAudio !== null;
  }

  get overflowDrops(): number {
    return this.droppedOverflowCount;
  }

  get diagnostic(): string | null {
    return this.lastDiagnostic;
  }

  get erroredPaths(): ReadonlySet<string> {
    return this.pathErrors;
  }

  get playedCues(): readonly QueuedAudioCue[] {
    return this.playedHistory;
  }

  snapshot(): {
    sessionGeneration: number;
    lastCommittedRevision: number;
    baselineRevision: number;
    muted: boolean;
    volume: number;
    autoplayBlocked: boolean;
    gestureUnlocked: boolean;
    pendingQueueLength: number;
    hasActiveAudio: boolean;
    overflowDrops: number;
    diagnostic: string | null;
    erroredPathsCount: number;
    playedCuesCount: number;
  } {
    return {
      sessionGeneration: this.sessionGeneration,
      lastCommittedRevision: this.latestRevision,
      baselineRevision: this.baselineRevision,
      muted: this.muted,
      volume: this.volume,
      autoplayBlocked: this.autoplayBlocked,
      gestureUnlocked: this.gestureUnlocked,
      pendingQueueLength: this.queue.length,
      hasActiveAudio: this.activeAudio !== null,
      overflowDrops: this.droppedOverflowCount,
      diagnostic: this.lastDiagnostic,
      erroredPathsCount: this.pathErrors.size,
      playedCuesCount: this.playedHistory.length,
    };
  }

  setFrozenPackages(packages: readonly FrozenPackage[] | undefined): void {
    this.frozenBySlot.clear();
    if (!packages) return;
    for (const frozen of packages) {
      const slot = `${frozen.binding.namespace}/${frozen.binding.package_id}`;
      if (this.frozenBySlot.has(slot)) {
        throw new Error(`重复的冻结扩展包槽位: ${slot}`);
      }
      const computedContentHash = computeFrozenContentHashHex(frozen);
      if (computedContentHash !== frozen.binding.content_hash) {
        throw new Error(`冻结扩展包 content_hash 不匹配: ${slot}`);
      }
      let parsed: Record<string, any>;
      try {
        parsed = JSON.parse(frozen.package_json) as Record<string, any>;
      } catch {
        throw new Error(`冻结扩展包 JSON 解析失败: ${slot}`);
      }
      const manifest = parsed?.manifest;
      if (
        !manifest ||
        manifest.namespace !== frozen.binding.namespace ||
        manifest.package_id !== frozen.binding.package_id ||
        manifest.version !== frozen.binding.version ||
        !Array.isArray(manifest.logic_hash) ||
        manifest.logic_hash.map((b: number) => b.toString(16).padStart(2, "0")).join("") !==
          frozen.binding.logic_hash
      ) {
        throw new Error(`冻结扩展包 manifest 与 binding 不匹配: ${slot}`);
      }
      const assetPaths: Record<string, string> = {};
      for (const pathsObj of [parsed.asset_paths, manifest.asset_paths]) {
        if (pathsObj && typeof pathsObj === "object") {
          for (const [k, v] of Object.entries(pathsObj)) {
            if (typeof v === "string") assetPaths[k] = v;
          }
        }
      }
      const assetHashes: Record<string, string> = {};
      if (Array.isArray(parsed.asset_manifest?.entries)) {
        for (const entry of parsed.asset_manifest.entries) {
          if (
            entry &&
            typeof entry.asset_id === "string" &&
            Array.isArray(entry.source_hash) &&
            entry.source_hash.length === 32 &&
            entry.source_hash.every((n: unknown) => Number.isInteger(n) && (n as number) >= 0 && (n as number) <= 255)
          ) {
            assetHashes[entry.asset_id] = entry.source_hash
              .map((n: number) => n.toString(16).padStart(2, "0"))
              .join("");
          }
        }
      }
      if (manifest.asset_hashes && typeof manifest.asset_hashes === "object") {
        for (const [k, v] of Object.entries(manifest.asset_hashes)) {
          if (Array.isArray(v) && v.length === 32 && v.every((n) => Number.isInteger(n) && n >= 0 && n <= 255)) {
            assetHashes[k] = v.map((n: number) => n.toString(16).padStart(2, "0")).join("");
          }
        }
      }
      this.frozenBySlot.set(slot, { frozen, assetPaths, assetHashes });
    }
  }

  resetGeneration(reason = "reset", baselineRevision = 0): void {
    this.sessionGeneration += 1;
    this.latestRevision = baselineRevision;
    this.baselineRevision = baselineRevision;
    this.sourceKey = currentAppearanceSourceKey();
    this.consumedCueKeys.clear();
    this.retainedBatches.clear();
    this.autoplayBlocked = false;
    this.lastDiagnostic = reason ? `generation:${reason}` : null;
    this.stopAndClearPlayback();
  }

  establishSilentBaseline(revision: number): void {
    const nextSourceKey = currentAppearanceSourceKey();
    if (this.sourceKey && nextSourceKey && this.sourceKey !== nextSourceKey) {
      this.resetGeneration("source-changed", revision);
      return;
    }
    this.sourceKey = nextSourceKey;
    if (revision > this.latestRevision) {
      this.latestRevision = revision;
    }
    if (revision > this.baselineRevision) {
      this.baselineRevision = revision;
    }
  }

  suspendPlaybackForReconnect(): void {
    this.lastDiagnostic = "reconnect:suspended";
    this.stopAndClearPlayback();
  }

  establishRejoinBaseline(revision: number): void {
    this.stopAndClearPlayback();
    this.baselineRevision = revision;
    if (revision > this.latestRevision) {
      this.latestRevision = revision;
    }
    this.lastDiagnostic = `rejoin:baseline:${revision}`;
  }

  unlockFromUserGesture(): void {
    this.gestureUnlocked = true;
    this.autoplayBlocked = false;
  }

  setMuted(muted: boolean): void {
    this.muted = !!muted;
    if (this.muted) {
      this.stopAndClearPlayback();
    }
  }

  setVolume(volume: number): void {
    const clamped = Number.isFinite(volume) ? Math.max(0, Math.min(1, volume)) : 0;
    this.volume = clamped;
    if (clamped <= 0) {
      this.stopAndClearPlayback();
      return;
    }
    if (this.activeAudio) {
      this.activeAudio.volume = clamped;
    }
  }

  admitEnvelope(env: EngineEnvelope): AdmissionOutcome {
    validatePresentationResponse(env);
    const nextSourceKey = currentAppearanceSourceKey();
    if (this.sourceKey && nextSourceKey && this.sourceKey !== nextSourceKey) {
      this.resetGeneration("source-changed", 0);
    } else if (!this.sourceKey && nextSourceKey) {
      this.sourceKey = nextSourceKey;
    }
    if (!env.presentation) {
      this.establishSilentBaseline(env.revision);
      return { admitted: 0, duplicate: false, stale: false, cues: [] };
    }
    return this.admitBatch(env.presentation);
  }

  admitBatch(batch: PublicPresentationBatch): AdmissionOutcome {
    const canonical = JSON.stringify(batch);
    const existing = this.retainedBatches.get(batch.revision);
    if (existing !== undefined) {
      if (existing !== canonical) {
        throw new Error("native: public presentation metadata conflict for same revision");
      }
      return { admitted: 0, duplicate: true, stale: false, cues: [] };
    }
    if (batch.revision < this.latestRevision || batch.revision <= this.baselineRevision) {
      this.lastDiagnostic = `stale:${batch.revision}<=${Math.max(this.latestRevision, this.baselineRevision)}`;
      return { admitted: 0, duplicate: false, stale: true, cues: [] };
    }

    const resolvedPerOccurrence: Array<{ ordinal: number; cue: QueuedAudioCue | null }> = [];
    for (const occ of batch.occurrences) {
      const cueKey = `${this.sessionGeneration}:${batch.revision}:${occ.ordinal}`;
      if (this.consumedCueKeys.has(cueKey)) {
        continue;
      }
      const extracted = extractEventVoice(occ.event);
      if (!extracted) {
        resolvedPerOccurrence.push({ ordinal: occ.ordinal, cue: null });
        continue;
      }
      const selected = this.selectVerifiedCandidate(
        cueKey,
        batch.revision,
        occ.ordinal,
        occ.event.kind,
        extracted.actor,
        extracted.voice,
      );
      resolvedPerOccurrence.push({ ordinal: occ.ordinal, cue: selected });
    }

    this.latestRevision = batch.revision;
    this.retainedBatches.set(batch.revision, canonical);
    if (this.retainedBatches.size > MAX_RETAINED_REVISIONS) {
      const oldest = this.retainedBatches.keys().next().value;
      if (oldest !== undefined) this.retainedBatches.delete(oldest);
    }

    const admittedCues: QueuedAudioCue[] = [];
    for (const item of resolvedPerOccurrence) {
      const cueKey = `${this.sessionGeneration}:${batch.revision}:${item.ordinal}`;
      this.consumedCueKeys.add(cueKey);
      if (item.cue) {
        admittedCues.push(item.cue);
      }
    }

    if (!this.muted && this.volume > 0) {
      for (const cue of admittedCues) {
        this.enqueueCue(cue);
      }
    }

    return {
      admitted: admittedCues.length,
      duplicate: false,
      stale: false,
      cues: admittedCues,
    };
  }

  private selectVerifiedCandidate(
    cueKey: string,
    revision: number,
    ordinal: number,
    eventKind: PublicPresentationEvent["kind"],
    actor: number,
    voice: PublicVoice,
  ): QueuedAudioCue | null {
    const seenPayloads = new Set<string>();
    const roleKey = roleKeyFromVoice(voice);
    for (const asset of voice.assets) {
      const payloadId = `${asset.digest_algorithm}:${asset.digest}:${asset.size}`;
      if (seenPayloads.has(payloadId)) continue;
      seenPayloads.add(payloadId);

      const resolved = this.resolveBoundAsset(asset, voice.role);
      if (!resolved) {
        continue;
      }
      return {
        cueKey,
        generation: this.sessionGeneration,
        revision,
        ordinal,
        eventKind,
        actor,
        roleKey,
        form: voice.form,
        url: resolved.url,
        pathKey: resolved.pathKey,
        digest: asset.digest,
        digestAlgorithm: asset.digest_algorithm,
        size: asset.size,
      };
    }
    return null;
  }

  private resolveBoundAsset(
    asset: BoundAudioAsset,
    role: PublicVoice["role"],
  ): { url: string; pathKey: string } | null {
    if (asset.source.kind === "original" && asset.reference.kind === "original") {
      if (role.owner.kind !== "original") {
        this.recordPathError("original:owner-kind-mismatch");
        throw new Error("native: original audio asset attached to non-original role");
      }
      const pin = this.originalOwnerPin;
      if (
        asset.source.repository !== pin.repository ||
        asset.source.adopted_revision !== pin.adopted_revision ||
        asset.source.catalog_sha256 !== pin.catalog_sha256 ||
        asset.source.asset_manifest_sha256 !== pin.asset_manifest_sha256 ||
        role.owner.repository !== pin.repository ||
        role.owner.adopted_revision !== pin.adopted_revision ||
        role.owner.catalog_sha256 !== pin.catalog_sha256 ||
        role.owner.asset_manifest_sha256 !== pin.asset_manifest_sha256
      ) {
        this.recordPathError(`original:pin:${asset.reference.path}`);
        throw new Error("native: original audio asset pin mismatch");
      }
      const expectedBackingPath =
        asset.reference.source_category === "core"
          ? asset.reference.source_path
          : asset.reference.source_path.replace(/^audio\//, "audio-compat/");
      const refBaseName = asset.reference.path.split("/").at(-1);
      const backingBaseName = expectedBackingPath.split("/").at(-1);
      if (
        !refBaseName ||
        refBaseName !== backingBaseName ||
        !/\.(?:mp3|ogg|wav|m4a|aac|flac|opus|mp4)$/.test(asset.reference.path)
      ) {
        this.recordPathError(`original:backing:${asset.reference.path}`);
        throw new Error("native: original audio backing path mismatch");
      }
      const base = this.originalBaseUrl();
      const declaredUrl = originalResourceUrl(base, `assets/${asset.reference.path}`);
      const backingUrl = originalResourceUrl(base, `assets/${expectedBackingPath}`);
      if (!declaredUrl || !backingUrl) {
        this.recordPathError(`original:url:${asset.reference.path}`);
        throw new Error("native: original audio URL invalid");
      }
      return { url: declaredUrl, pathKey: asset.reference.path };
    }

    if (asset.source.kind === "package" && asset.reference.kind === "package") {
      const slot = `${asset.source.namespace}/${asset.source.package_id}`;
      const frozenEntry = this.frozenBySlot.get(slot);
      if (frozenEntry) {
        const b = frozenEntry.frozen.binding;
        if (
          b.version !== asset.source.version ||
          b.logic_hash !== asset.source.logic_hash ||
          b.content_hash !== asset.source.content_hash
        ) {
          this.recordPathError(`package:binding:${slot}/${asset.reference.asset_id}`);
          throw new Error(`native: frozen package owner mismatch for ${slot}`);
        }
        const relPath = frozenEntry.assetPaths[asset.reference.asset_id];
        const declaredDigest = frozenEntry.assetHashes[asset.reference.asset_id];
        if (!relPath || !declaredDigest || declaredDigest !== asset.digest) {
          this.recordPathError(`package:manifest:${slot}/${asset.reference.asset_id}`);
          throw new Error(`native: frozen package manifest asset mismatch for ${slot}/${asset.reference.asset_id}`);
        }
        const rawBytes = frozenEntry.frozen.assets[relPath];
        if (!rawBytes || rawBytes.length !== asset.size) {
          this.recordPathError(`package:size:${slot}/${asset.reference.asset_id}`);
          throw new Error(`native: frozen package asset size mismatch for ${slot}/${asset.reference.asset_id}`);
        }
        const actualDigest = blake3Hex(Uint8Array.from(rawBytes));
        if (actualDigest !== asset.digest) {
          this.recordPathError(`package:digest:${slot}/${asset.reference.asset_id}`);
          throw new Error(`native: frozen package asset digest mismatch for ${slot}/${asset.reference.asset_id}`);
        }
        const mime = audioMimeType(relPath);
        const url = `data:${mime};base64,${bytesToBase64(Uint8Array.from(rawBytes))}`;
        return {
          url,
          pathKey: `${slot}@${b.content_hash}:${asset.reference.asset_id}`,
        };
      }

      if (this.allowRemotePackageFallback()) {
        const base = currentAppearanceBaseUrl();
        if (!base) return null;
        const encodedSlot = [asset.reference.namespace, asset.reference.package_id]
          .map(encodeURIComponent)
          .join("/");
        const encodedAsset = asset.reference.asset_id
          .split("/")
          .map(encodeURIComponent)
          .join("/");
        const lockHash = appearanceState.contentLockHash || asset.source.content_hash;
        const url = `${base.replace(/\/+$/, "")}/package-assets/${encodedSlot}/${encodedAsset}?v=${encodeURIComponent(
          lockHash,
        )}`;
        return {
          url,
          pathKey: `${slot}@${asset.source.content_hash}:${asset.reference.asset_id}`,
        };
      }

      this.recordPathError(`package:missing:${slot}/${asset.reference.asset_id}`);
      return null;
    }

    return null;
  }

  private enqueueCue(cue: QueuedAudioCue): void {
    if (!this.activeAudio && this.queue.length === 0) {
      this.startCuePlayback(cue);
      return;
    }
    if (this.queue.length >= MAX_BATTLE_AUDIO_QUEUE) {
      const dropped = this.queue.shift();
      this.droppedOverflowCount += 1;
      this.lastDiagnostic = `queue-overflow:dropped:${dropped?.cueKey ?? "unknown"}`;
    }
    this.queue.push(cue);
  }

  private startCuePlayback(cue: QueuedAudioCue): void {
    const generation = this.sessionGeneration;
    const playSeq = ++this.activePlaySeq;
    let audio: AudioElementLike;
    try {
      audio = this.createAudio();
    } catch {
      this.recordPathError(cue.pathKey);
      this.advanceQueue(generation, playSeq);
      return;
    }
    this.activeAudio = audio;
    this.activePathKey = cue.pathKey;
    audio.preload = "auto";
    if (!cue.url.startsWith("data:")) {
      audio.crossOrigin = "anonymous";
    }
    audio.muted = this.muted;
    audio.volume = this.volume;
    audio.onloadeddata = () => {
      if (generation !== this.sessionGeneration || playSeq !== this.activePlaySeq) return;
      this.pathErrors.delete(cue.pathKey);
    };
    audio.onended = () => {
      if (generation !== this.sessionGeneration || playSeq !== this.activePlaySeq) return;
      this.cleanupActiveAudio();
      this.advanceQueue(generation, playSeq);
    };
    audio.onerror = () => {
      if (generation !== this.sessionGeneration || playSeq !== this.activePlaySeq) return;
      this.recordPathError(cue.pathKey);
      this.cleanupActiveAudio();
      this.advanceQueue(generation, playSeq);
    };
    audio.src = cue.url;
    this.playedHistory.push(cue);
    if (this.playedHistory.length > 64) {
      this.playedHistory.shift();
    }

    let playPromise: Promise<void>;
    try {
      playPromise = Promise.resolve(audio.play());
    } catch (err) {
      this.handlePlayRejection(err, cue.pathKey, generation, playSeq);
      return;
    }
    void playPromise.catch((err: unknown) => {
      this.handlePlayRejection(err, cue.pathKey, generation, playSeq);
    });
  }

  private handlePlayRejection(
    err: unknown,
    pathKey: string,
    generation: number,
    playSeq: number,
  ): void {
    if (generation !== this.sessionGeneration || playSeq !== this.activePlaySeq) return;
    const name = (err as { name?: string } | null)?.name ?? "";
    if (name === "NotAllowedError") {
      this.autoplayBlocked = true;
      this.lastDiagnostic = "autoplay-blocked";
      this.stopAndClearPlayback();
      return;
    }
    this.recordPathError(pathKey);
    this.cleanupActiveAudio();
    this.advanceQueue(generation, playSeq);
  }

  private advanceQueue(generation: number, playSeq: number): void {
    if (generation !== this.sessionGeneration || playSeq !== this.activePlaySeq) return;
    if (this.muted || this.volume <= 0) {
      this.queue.length = 0;
      return;
    }
    const next = this.queue.shift();
    if (next) {
      this.startCuePlayback(next);
    }
  }

  private cleanupActiveAudio(): void {
    const audio = this.activeAudio;
    this.activeAudio = null;
    this.activePathKey = null;
    if (!audio) return;
    audio.onended = null;
    audio.onerror = null;
    audio.onloadeddata = null;
    try {
      audio.pause();
      if (typeof audio.removeAttribute === "function") {
        audio.removeAttribute("src");
      } else {
        audio.src = "";
      }
      audio.load?.();
    } catch {
      /* ignore teardown errors */
    }
  }

  stopAndClearPlayback(): void {
    this.activePlaySeq += 1;
    this.queue.length = 0;
    this.cleanupActiveAudio();
  }

  private recordPathError(pathKey: string): void {
    if (this.pathErrors.has(pathKey)) return;
    if (this.pathErrors.size >= MAX_PATH_ERRORS) {
      const oldest = this.pathErrors.values().next().value;
      if (oldest !== undefined) this.pathErrors.delete(oldest);
    }
    this.pathErrors.add(pathKey);
    this.lastDiagnostic = `media-error:${pathKey}`;
  }
}
