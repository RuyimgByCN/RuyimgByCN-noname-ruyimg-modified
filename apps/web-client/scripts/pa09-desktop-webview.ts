// Actual WKWebView acceptance entry, bundled with Vite and served by the external evidence runner.
import { invoke } from "@tauri-apps/api/core";
import { EngineClient } from "../src/engine/client";
import { activateExtension, deactivateExtension, importExtensionPackageValidated, readExtensionStore, setMatchFrozen, uninstallExtension } from "../src/engine/extensionStore";
import { prepareNativeMatchContent, restoreNativeMatchContent } from "../src/engine/nativeMatchContent";
import type { EngineEnvelope } from "../src/types/engine";
import v1 from "./fixtures/vm-trigger/package.npack.json";
import chars1 from "./fixtures/vm-trigger/character.json";
import v2 from "./fixtures/vm-trigger-v2/package.npack.json";
import chars2 from "./fixtures/vm-trigger-v2/character.json";

function check(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function equal(actual: unknown, expected: unknown, message: string) {
  const canonical = (value: unknown): string => JSON.stringify(value, (_key, item) => {
    if (item && typeof item === "object" && !Array.isArray(item)) return Object.fromEntries(Object.entries(item).sort(([a], [b]) => a.localeCompare(b)));
    return item;
  });
  check(canonical(actual) === canonical(expected), message);
}
async function rejects(action: () => Promise<unknown>, message: string) {
  let failed = false;
  try { await action(); } catch { failed = true; }
  check(failed, message);
}
const battle = new EngineClient();
const validator = new EngineClient();
const wasm = new EngineClient();
const storage = new Map<string, string>();
Object.defineProperty(window, "localStorage", { value: {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => storage.set(key, String(value)),
  removeItem: (key: string) => storage.delete(key),
} });

async function run() {
  history.replaceState(null, "", "/?engineTransport=native");
  await battle.connect();
  check(battle.getTransportKind() === "desktop-native", "battle did not select actual IPC");
  let env = await battle.start(4, 42, 0, undefined, "identity");
  const savedBeforeValidator = await battle.saveHex();
  await validator.connect();
  equal(await battle.saveHex(), savedBeforeValidator, "validator init reset battle resource");
  history.replaceState(null, "", "/?engineTransport=wasm");
  await wasm.connect();
  check(wasm.getTransportKind() === "wasm-worker", "explicit Wasm fallback did not select Worker");
  history.replaceState(null, "", "/?engineTransport=native");
  equal(env, await wasm.start(4, 42, 0, undefined, "identity"), "initial IPC/Worker envelopes differ");
  let steps = 0;
  for (; steps < 2000 && !env.finished; steps++) {
    env = await battle.autoStep();
    equal(env, await wasm.autoStep(), `IPC/Worker trajectory diverged at ${steps}`);
    if (steps === 30 || steps === 70) {
      const hex = await battle.saveHex();
      equal(hex, await wasm.saveHex(), "native/Wasm archive bytes differ");
      equal(await battle.restoreHex(await wasm.saveHex()), await wasm.restoreHex(hex), "cross-restore differed");
    }
  }
  check(env.finished, "identity workload did not reach its actual terminal outcome");
  const terminal = { steps, hash: env.hash_hex, outcome: env.view.outcome };
  const deep = (json: string) => validator.validatePackage(json);
  await importExtensionPackageValidated(JSON.stringify({ ...v1, characters: chars1 }), deep);
  const key = "official/vm-fixture";
  activateExtension(key);
  const frozen = prepareNativeMatchContent();
  await battle.start(4, 42, 0, frozen.packsJson);
  const candidates = await battle.generalCandidates();
  check(candidates.candidates.some((row) => row.name === "vm_fixture"), "installed VM general missing from actual IPC candidates");
  env = (await battle.chooseGeneral(0, "vm_fixture")).envelope;
  async function phaseEnd(delta: number) {
    for (let step = 0; step < 80; step++) {
      const before: EngineEnvelope = env;
      env = before.choice
        ? await battle.resolveChoice(before.choice.observer, before.choice.choice_id,
            (before.choice.options ?? []).slice(0, Math.max(1, before.choice.min_count ?? 0)).map((option) => option.token))
        : await battle.endPhase(before.view.current_player);
      if (before.view.phase === "Discard" && env.view.phase === "End") {
        equal(env.view.seats[0].hand_count - before.view.seats[0].hand_count, delta, "actual IPC VM effect did not use expected package version");
        return;
      }
    }
    throw new Error("VM phase end was not reached");
  }
  await phaseEnd(1);
  const save = { hex: await battle.saveHex(), ...frozen };
  setMatchFrozen(true);
  await rejects(() => importExtensionPackageValidated(JSON.stringify({ ...v2, characters: chars2 }), deep), "frozen install was accepted");
  setMatchFrozen(false);
  await importExtensionPackageValidated(JSON.stringify({ ...v2, characters: chars2 }), deep);
  const upgraded = prepareNativeMatchContent();
  check(upgraded.lockSeed !== frozen.lockSeed, "upgrade did not change content lock");
  equal(await battle.observer(), env.view, "upgrade mutated the live match");
  deactivateExtension(key);
  check(prepareNativeMatchContent().packsJson === undefined, "disabled package remained active");
  activateExtension(key);
  equal(prepareNativeMatchContent().lockSeed, upgraded.lockSeed, "enable lost upgraded version");
  deactivateExtension(key);
  uninstallExtension(key);
  equal(readExtensionStore().installed.length, 0, "uninstall did not remove installed slot");
  const original = restoreNativeMatchContent(save);
  const restored = await battle.restoreHex(save.hex, original.packsJson);
  equal(restored.hash_hex, env.hash_hex, "frozen old package restore failed after uninstall");
  const beforeRejectedRestore = await battle.saveHex();
  await rejects(() => battle.restoreHex(save.hex, upgraded.packsJson), "wrong-version archive restore was accepted");
  equal(await battle.saveHex(), beforeRejectedRestore, "rejected restore destroyed live archive");
  await battle.start(4, 42, 0, upgraded.packsJson);
  env = (await battle.chooseGeneral(0, "vm_fixture")).envelope;
  await phaseEnd(2);
  await battle.start(4, 42, 0);
  env = (await battle.chooseGeneral(0, "黄盖")).envelope;
  await battle.endPhase(0);
  env = await battle.endPhase(0);
  const active = env.legal_actions.find((action) => typeof action === "object" && "ActivateSkill" in action);
  check(active && typeof active === "object" && "ActivateSkill" in active, "active skill candidate missing");
  env = await battle.activateSkill(0, active.ActivateSkill.skill, [], []);
  const play = env.legal_actions.find((action) => typeof action === "object" && "PlayCard" in action);
  check(play && typeof play === "object" && "PlayCard" in play, "playable card candidate missing");
  env = await battle.playCard(0, play.PlayCard.card, play.PlayCard.targets);
  check((await battle.status()).revision === env.revision, "status revision differs");
  await rejects(() => battle.useSkillCard(0, 1, 0, [], 0), "unsupported request falsely succeeded");
  check(battle.getTransportKind() === "desktop-native", "unsupported request silently switched transport");
  const resourceId = await invoke<number>("native_engine_open");
  await invoke("native_engine_close", { resourceId });
  await rejects(() => invoke("native_engine_close", { resourceId }), "duplicate resource close was uncontrolled");
  await rejects(() => invoke("native_engine_request", { resourceId, request: { id: 1, type: "init" } }), "closed resource accepted a request");
  return { passed: true, terminal, vm_lifecycle: "actual IPC passed", isolation: "validator init preserves battle", fallback: "actual Wasm Worker passed", resources: "close/duplicate-close/closed-request passed" };
}

try {
  const result = await run();
  document.body.textContent = JSON.stringify(result);
  await fetch("/report", { method: "POST", body: JSON.stringify(result) });
} catch (error) {
  const result = { passed: false, error: String(error), stack: error instanceof Error ? error.stack : null };
  document.body.textContent = JSON.stringify(result);
  await fetch("/report", { method: "POST", body: JSON.stringify(result) });
} finally {
  battle.dispose(); validator.dispose(); wasm.dispose();
}
