# 2026-10-10 T5：全工作区测试与 CI 其余门禁实跑

任务来源：[进度主稿 T5](../../../architecture/native-progress.md)。上次有留存日志的全工作区实跑为 9-23（v59）；本批在 git `e1ffc3fd`（含同日 T2/T3/T7/T10/T14/T6① 全部改动）实跑五项。

## 结果总览

| 命令 | 结果 | 日志 |
|---|---|---|
| `cargo test --workspace --no-fail-fast` | **5098 通过 / 0 失败 / 2 ignore**（exit 0；旧线 engine-core 与原生线合计，含 2 个 ignore 为写轮眼探针与旧线既有 ignore） | [cargo-test-workspace.log](cargo-test-workspace.log) |
| `cargo run --offline -p conformance` | **ALL PASS**（exit 0） | [conformance.log](conformance.log) |
| `bash scripts/cross-target-hash.sh` | **ALL CROSS-TARGET CHECKS PASSED**（exit 0；wasm32 构建与宿主哈希一致。日志含一处 engine-wasm 测试代码的编译警告提示，非本批范围） | [cross-target-hash.log](cross-target-hash.log) |
| `node scripts/sync-original-content.mjs --verify-only` | **通过**（exit 0）：legacy-source 366 文件全复制、portraits 2703 文件全链接 | [sync-verify.log](sync-verify.log) |
| `pnpm --dir apps/web-client build` | **成功**（exit 0；engine_wasm 7,746.05 kB / gzip 2,106.98 kB，构建 776ms） | [web-client-build.log](web-client-build.log) |

## 口径

- 全工作区合计通过数包含旧线（engine-core/engine-ai/engine-wasm）与原生线（engine-native）及装载/运行时各 crate；两线分开的历史口径见进度主稿快照。
- 原生线单独结果以同日 [T10 证据](../2026-10-10-t10-jiguantong-v193/engine-native-tests.log)为准（527/0/1）。
- 磁盘：开工余量约 207 GiB；`target/` 构建后约 60 GiB（共享目录，未清理）。

## 边界

- 本批只验证，无代码改动；失败项为零，无需另立任务。
- 联机与目标设备实机运行不在本批范围（进度主稿口径）。
