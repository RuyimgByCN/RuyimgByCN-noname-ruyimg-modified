#!/usr/bin/env node
// 包装：复用已验证的 LX.2 追踪脚本，写入当日证据目录。
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "../..");
const outDir = path.resolve(
	process.argv[2] || path.join(REPO, "docs/operations/evidence/2026-10-16-t9-lx-battle-trace")
);
const src = path.resolve(
	process.argv[3] || path.join(REPO, "../scratch/legacy-harness/liuli/extension/龙血玄黄/extension.js")
);
const tracer = path.join(REPO, "docs/operations/evidence/2026-10-08-e01-longxue-lx2/lx2_trace.js");
const r = spawnSync(process.execPath, [tracer, src, outDir], { stdio: "inherit" });
process.exit(r.status ?? 1);
