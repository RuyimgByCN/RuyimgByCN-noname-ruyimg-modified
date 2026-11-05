import assert from "node:assert/strict";
import { test } from "node:test";
import {
  ACCEPTED_ORIGINAL_OWNER,
  BattleAudioOwner,
  MAX_BATTLE_AUDIO_QUEUE,
  blake3Hex,
} from "../src/engine/battleAudioOwner.ts";
import { EMPTY_CONTENT_LOCK_HASH } from "../src/engine/nativeMatchContent.ts";
import {
  configureLocalAppearanceCatalog,
  configureOnlineAppearanceCatalog,
} from "../src/engine/appearanceStore.ts";
import { presentationEnvelope, presentationView } from "./presentation-fixtures.mjs";

function createFakeAudioHarness() {
  const instances = [];
  const createAudio = () => {
    const audio = {
      src: "",
      volume: 1,
      muted: false,
      preload: "none",
      crossOrigin: null,
      paused: true,
      removedSrc: false,
      loadCalls: 0,
      playCalls: 0,
      pauseCalls: 0,
      nextPlayError: null,
      onended: null,
      onerror: null,
      onloadeddata: null,
      play() {
        this.playCalls += 1;
        this.paused = false;
        if (this.nextPlayError) {
          const err = this.nextPlayError;
          this.nextPlayError = null;
          return Promise.reject(err);
        }
        return Promise.resolve();
      },
      pause() {
        this.pauseCalls += 1;
        this.paused = true;
      },
      load() {
        this.loadCalls += 1;
      },
      removeAttribute(name) {
        if (name === "src") {
          this.src = "";
          this.removedSrc = true;
        }
      },
    };
    instances.push(audio);
    return audio;
  };
  return { instances, createAudio };
}

function makeOriginalVoice(index = 1, compat = false) {
  const ext = compat ? "m4a" : "mp3";
  const category = compat ? "compat" : "core";
  const file = compat ? `mugyu${index}.m4a` : `rende${index}.mp3`;
  const pack = compat ? "key" : "standard";
  const symbol = compat ? "key_tsumugi" : "liubei";
  return {
    role: {
      owner: { ...ACCEPTED_ORIGINAL_OWNER },
      kind: "general",
      local_id: { pack, raw_symbol: symbol },
    },
    form: null,
    assets: [
      {
        source: { ...ACCEPTED_ORIGINAL_OWNER },
        reference: {
          kind: "original",
          path: `audio-declared/${pack}/character/${symbol}/skill/rende/base/000${index}/${file}`,
          source_category: category,
          source_path: `audio/skill/${file}`,
        },
        digest_algorithm: "sha256",
        digest: String(index % 10).repeat(64),
        size: 1024 + index,
      },
    ],
  };
}

function makeBatchWithVoices(revision, count, compat = false) {
  return {
    revision,
    status: "complete",
    unavailable: [],
    occurrences: Array.from({ length: count }, (_, ordinal) => ({
      ordinal,
      event: {
        kind: "skill_activated",
        actor: 1,
        skill: {
          definition: {
            owner: { ...ACCEPTED_ORIGINAL_OWNER },
            kind: "skill",
            local_id: { pack: "standard", raw_symbol: "rende" },
          },
        },
        targets: [2],
        voice: makeOriginalVoice(ordinal + 1, compat),
      },
    })),
  };
}

function makeFrozenPackageFixture(version = "1.0.0", audioBytes = [0x49, 0x44, 0x33, 0x04, 0x00]) {
  const enc = new TextEncoder();
  const assetBytes = Uint8Array.from(audioBytes);
  const assetDigestHex = blake3Hex(assetBytes);
  const assetHashArray = Array.from({ length: 32 }, (_, i) =>
    Number.parseInt(assetDigestHex.slice(i * 2, i * 2 + 2), 16),
  );
  const logicHashHex = "11".repeat(32);
  const logicHashArray = Array(32).fill(0x11);
  const relPath = "assets/audio/skill_voice.mp3";
  const packageObj = {
    manifest: {
      namespace: "author",
      package_id: "voice_pack",
      version,
      logic_hash: logicHashArray,
      asset_paths: { "voice.skill.1": relPath },
      asset_hashes: { "voice.skill.1": assetHashArray },
    },
  };
  const packageJson = JSON.stringify(packageObj);
  const contentChunks = [
    enc.encode("noname-next/content-package/v1\0"),
    enc.encode(packageJson),
    new Uint8Array([0]),
    enc.encode(relPath),
    new Uint8Array([0]),
    assetBytes,
    new Uint8Array([0xff]),
  ];
  const total = contentChunks.reduce((s, c) => s + c.length, 0);
  const merged = new Uint8Array(total);
  let off = 0;
  for (const c of contentChunks) {
    merged.set(c, off);
    off += c.length;
  }
  const contentHashHex = blake3Hex(merged);
  const owner = {
    kind: "package",
    namespace: "author",
    package_id: "voice_pack",
    version,
    logic_hash: logicHashHex,
    content_hash: contentHashHex,
  };
  const frozen = {
    binding: {
      namespace: owner.namespace,
      package_id: owner.package_id,
      version: owner.version,
      logic_hash: owner.logic_hash,
      content_hash: owner.content_hash,
    },
    package_json: packageJson,
    assets: { [relPath]: Array.from(assetBytes) },
  };
  const batch = {
    revision: 3,
    status: "complete",
    unavailable: [],
    occurrences: [
      {
        ordinal: 0,
        event: {
          kind: "skill_activated",
          actor: 1,
          skill: {
            definition: {
              owner,
              kind: "skill",
              local_id: 101,
            },
          },
          targets: null,
          voice: {
            role: {
              owner,
              kind: "general",
              local_id: 201,
            },
            form: "shadow_form",
            assets: [
              {
                source: owner,
                reference: {
                  kind: "package",
                  namespace: "author",
                  package_id: "voice_pack",
                  asset_id: "voice.skill.1",
                },
                digest_algorithm: "blake3",
                digest: assetDigestHex,
                size: assetBytes.length,
              },
            ],
          },
        },
      },
    ],
  };
  return { frozen, batch, assetDigestHex, assetBytes };
}

test("blake3Hex matches Rust ContentLock empty seed and multi-chunk inputs", () => {
  const seed = new TextEncoder().encode("noname-next/content-lock/v1");
  assert.equal(blake3Hex(seed), EMPTY_CONTENT_LOCK_HASH);
  const large = new Uint8Array(2500);
  for (let i = 0; i < large.length; i++) large[i] = i & 0xff;
  const h1 = blake3Hex(large);
  const h2 = blake3Hex(large);
  assert.equal(h1.length, 64);
  assert.equal(h1, h2);
});

test("synchronous cue admission deduplicates same-revision batch, rejects conflicts, and ignores stale revisions", () => {
  const { instances, createAudio } = createFakeAudioHarness();
  const owner = new BattleAudioOwner({
    createAudio,
    originalBaseUrl: () => "http://127.0.0.1:7700/original-content",
  });

  owner.establishSilentBaseline(1);
  assert.equal(owner.currentRevision, 1);

  const batch2 = makeBatchWithVoices(2, 1);
  const env2 = { ...presentationEnvelope(batch2), revision: 2, view: presentationView(2) };
  const first = owner.admitEnvelope(env2);
  assert.equal(first.admitted, 1);
  assert.equal(first.duplicate, false);
  assert.equal(first.stale, false);
  assert.equal(instances.length, 1);

  // Duplicate WS broadcast with identical batch is a no-op
  const dup = owner.admitEnvelope(env2);
  assert.equal(dup.admitted, 0);
  assert.equal(dup.duplicate, true);
  assert.equal(instances.length, 1);

  // Same revision with altered batch content fails atomically
  const conflictingBatch = makeBatchWithVoices(2, 2);
  const conflictingEnv = { ...presentationEnvelope(conflictingBatch), revision: 2, view: presentationView(2) };
  assert.throws(() => owner.admitEnvelope(conflictingEnv), /conflict for same revision/);
  assert.equal(owner.currentRevision, 2);

  // Stale batch at revision 1 does not roll back baseline or replay cues
  const staleBatch = makeBatchWithVoices(1, 1);
  const staleEnv = { ...presentationEnvelope(staleBatch), revision: 1, view: presentationView(1) };
  const stale = owner.admitEnvelope(staleEnv);
  assert.equal(stale.admitted, 0);
  assert.equal(stale.stale, true);
  assert.equal(owner.currentRevision, 2);
  assert.equal(instances.length, 1);
});

test("queue caps at 8 pending items, drops oldest overflow without replay, and plays compat m4a", () => {
  const { instances, createAudio } = createFakeAudioHarness();
  const owner = new BattleAudioOwner({
    createAudio,
    originalBaseUrl: () => "http://127.0.0.1:7700/original-content",
  });

  // 11 cues in one batch: #0 plays immediately; #1..#10 (10 items) enter queue(cap 8) -> #1 and #2 dropped, #3..#10 kept
  const batch = makeBatchWithVoices(2, 11, true);
  const outcome = owner.admitBatch(batch);
  assert.equal(outcome.admitted, 11);
  assert.equal(owner.hasActiveAudio, true);
  assert.equal(owner.pendingQueueLength, MAX_BATTLE_AUDIO_QUEUE);
  assert.equal(owner.overflowDrops, 2);
  assert.match(instances[0].src, /mugyu1\.m4a$/);

  for (let expectedOrdinal = 3; expectedOrdinal <= 10; expectedOrdinal++) {
    const current = instances.at(-1);
    current.onended?.();
    const next = instances.at(-1);
    assert.match(next.src, new RegExp(`mugyu${expectedOrdinal + 1}\\.m4a$`));
  }
  instances.at(-1).onended?.();
  assert.equal(owner.hasActiveAudio, false);
  assert.equal(owner.pendingQueueLength, 0);
  assert.equal(instances.length, 9);
});

test("mute and volume 0 immediately stop active playback and discard queue without replaying on unmute", () => {
  const { instances, createAudio } = createFakeAudioHarness();
  const owner = new BattleAudioOwner({
    createAudio,
    originalBaseUrl: () => "http://127.0.0.1:7700/original-content",
  });

  owner.admitBatch(makeBatchWithVoices(2, 3));
  assert.equal(owner.hasActiveAudio, true);
  assert.equal(owner.pendingQueueLength, 2);

  owner.setMuted(true);
  assert.equal(owner.hasActiveAudio, false);
  assert.equal(owner.pendingQueueLength, 0);
  assert.equal(instances[0].pauseCalls, 1);
  assert.equal(instances[0].removedSrc, true);

  // Cues arriving while muted are consumed and not queued
  const mutedOutcome = owner.admitBatch(makeBatchWithVoices(3, 2));
  assert.equal(mutedOutcome.admitted, 2);
  assert.equal(owner.hasActiveAudio, false);
  assert.equal(owner.pendingQueueLength, 0);

  // Unmuting enables future cues only
  owner.setMuted(false);
  assert.equal(owner.hasActiveAudio, false);
  owner.admitBatch(makeBatchWithVoices(4, 2));
  assert.equal(owner.hasActiveAudio, true);
  assert.equal(owner.pendingQueueLength, 1);

  // Volume 0 also stops and discards queue
  owner.setVolume(0);
  assert.equal(owner.hasActiveAudio, false);
  assert.equal(owner.pendingQueueLength, 0);
});

test("NotAllowedError clears queue without autoplay retry and gesture unlock enables future cues only", async () => {
  const { instances, createAudio } = createFakeAudioHarness();
  const owner = new BattleAudioOwner({
    createAudio: () => {
      const a = createAudio();
      if (instances.length === 1) {
        const err = new Error("play() failed because the user didn't interact with the document first");
        err.name = "NotAllowedError";
        a.nextPlayError = err;
      }
      return a;
    },
    originalBaseUrl: () => "http://127.0.0.1:7700/original-content",
  });

  owner.admitBatch(makeBatchWithVoices(2, 3));
  await new Promise((r) => setTimeout(r, 0));
  assert.equal(owner.isAutoplayBlocked, true);
  assert.equal(owner.hasActiveAudio, false);
  assert.equal(owner.pendingQueueLength, 0);

  owner.unlockFromUserGesture();
  assert.equal(owner.isAutoplayBlocked, false);
  assert.equal(owner. isGestureUnlocked, true);
  assert.equal(owner.hasActiveAudio, false);
  assert.equal(instances.length, 1);

  // Future cues play normally after gesture unlock
  owner.admitBatch(makeBatchWithVoices(3, 1));
  await new Promise((r) => setTimeout(r, 0));
  assert.equal(owner.hasActiveAudio, true);
  assert.equal(instances.length, 2);
});

test("media errors are bounded per path, advance queue, and late callbacks from prior generation/reconnect are ignored", () => {
  const { instances, createAudio } = createFakeAudioHarness();
  const owner = new BattleAudioOwner({
    createAudio,
    originalBaseUrl: () => "http://127.0.0.1:7700/original-content",
  });

  owner.admitBatch(makeBatchWithVoices(2, 2));
  const firstAudio = instances[0];
  const firstPath = owner.playedCues[0].pathKey;
  firstAudio.onerror?.();
  assert.equal(owner.erroredPaths.has(firstPath), true);
  assert.equal(instances.length, 2);

  const secondAudio = instances[1];
  const staleEnded = secondAudio.onended;
  const staleLoaded = secondAudio.onloadeddata;

  // Reconnect suspends playback while keeping dedup until authoritative rejoin baseline
  owner.suspendPlaybackForReconnect();
  assert.equal(owner.hasActiveAudio, false);
  staleEnded?.();
  staleLoaded?.();
  assert.equal(owner.hasActiveAudio, false);

  // Duplicate of rev 2 during reconnect is still deduplicated
  const dupDuringReconnect = owner.admitBatch(makeBatchWithVoices(2, 2));
  assert.equal(dupDuringReconnect.duplicate, true);
  assert.equal(instances.length, 2);

  owner.establishRejoinBaseline(5);
  assert.equal(owner.currentRevision, 5);
  const staleAfterRejoin = owner.admitBatch(makeBatchWithVoices(4, 1));
  assert.equal(staleAfterRejoin.stale, true);

  const freshAfterRejoin = owner.admitBatch(makeBatchWithVoices(6, 1));
  assert.equal(freshAfterRejoin.admitted, 1);
  const thirdAudio = instances[2];
  thirdAudio.onloadeddata?.();
  assert.equal(owner.erroredPaths.has(owner.playedCues.at(-1).pathKey), false);

  // Reset generation on restore/unmount/source change retires callbacks
  const genBefore = owner.generation;
  owner.resetGeneration("restore", 10);
  assert.equal(owner.generation, genBefore + 1);
  assert.equal(owner.hasActiveAudio, false);
});

test("frozen SDK package audio verifies 5-field owner + BLAKE3 digest and resists store upgrade/tampering", () => {
  const { instances, createAudio } = createFakeAudioHarness();
  const owner = new BattleAudioOwner({
    createAudio,
    originalBaseUrl: () => "http://127.0.0.1:7700/original-content",
  });

  const { frozen, batch } = makeFrozenPackageFixture("1.0.0", [1, 2, 3, 4, 5, 6]);
  owner.setFrozenPackages([frozen]);
  const res = owner.admitBatch(batch);
  assert.equal(res.admitted, 1);
  assert.match(instances[0].src, /^data:audio\/mpeg;base64,/);
  assert.equal(owner.playedCues[0].form, "shadow_form");

  // Simulating an upgrade in external storage while keeping the match's pinned frozen package
  // does not retarget the occurrence; if a batch claims a different version/hash than pinned, it rejects!
  const upgraded = makeFrozenPackageFixture("2.0.0", [9, 9, 9, 9]);
  upgraded.batch.revision = 4;
  assert.throws(() => owner.admitBatch(upgraded.batch), /frozen package owner mismatch/);

  // Tampered frozen package asset bytes with unchanged binding/pins are rejected by setFrozenPackages
  const tamperedFrozen = structuredClone(frozen);
  tamperedFrozen.assets["assets/audio/skill_voice.mp3"][0] ^= 0xff;
  assert.throws(() => owner.setFrozenPackages([tamperedFrozen]), /content_hash 不匹配/);

  // Tampered original owner pin is rejected before playback
  const badOriginalBatch = makeBatchWithVoices(5, 1);
  badOriginalBatch.occurrences[0].event.voice.assets[0].source.catalog_sha256 = "f".repeat(64);
  assert.throws(() => owner.admitBatch(badOriginalBatch), /pin mismatch/);
});

test("source change via appearanceStore resets generation automatically", () => {
  const previousWindow = globalThis.window;
  globalThis.window = {
    location: { href: "http://127.0.0.1:5173/", protocol: "http:", hostname: "127.0.0.1" },
  };
  try {
    configureLocalAppearanceCatalog("http://127.0.0.1:7700");
    const { createAudio } = createFakeAudioHarness();
    const owner = new BattleAudioOwner({ createAudio });
    owner.resetGeneration("init", 1);
    const g1 = owner.generation;
    owner.admitEnvelope({ ...presentationEnvelope(makeBatchWithVoices(2, 1)), revision: 2, view: presentationView(2) });
    assert.equal(owner.generation, g1);

    configureOnlineAppearanceCatalog("ws://127.0.0.1:7701/ws");
    owner.admitEnvelope({ ...presentationEnvelope(makeBatchWithVoices(1, 1)), revision: 1, view: presentationView(1) });
    assert.equal(owner.generation, g1 + 1);
  } finally {
    globalThis.window = previousWindow;
  }
});
