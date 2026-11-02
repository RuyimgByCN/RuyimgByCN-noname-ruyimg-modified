# C331 合併前閘門總表（2026-11-02 核對；樹＝私有 worktree，331／106／WS7）

| 閘門 | 命令 | 結果 | 證據 |
| --- | --- | --- | --- |
| 三 crate 單元 | `cargo test -p engine-native-kernel -p engine-native-content -p engine-native` | 1726/0/75（本輪之前、原始碼未變） | 04-final-workspace.log 內含 |
| 全量 workspace | `cargo test --offline --workspace --no-fail-fast` | **168 二進位／7137 passed／1 failed（golden 12）／77 ignored** | 04-final-workspace.log |
| strict（5 crate、`-D warnings`、all-targets） | `RUSTFLAGS="-D warnings" cargo check …` | exit 0 | 02-strict.log |
| wasm32 檢查 | `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0 | 本輪之前 |
| WASM 重建 | `pnpm build:native` | exit 0；4,286,486 B、sha256 `de42b8bb…` | 07-build-native-wasm.log |
| native↔wasm 跨標的 | `python3 -B scripts/check-native-wasm-cross-target.py` | `native_wasm_match: true` | 08-cross-target.log |
| conformance | `cargo run -p conformance` | ALL PASS | 13-conformance.log |
| 前端測試 | `pnpm test` | 187/0 | 05-frontend-tests.log |
| 前端 typecheck | `pnpm typecheck` | exit 0 | 06-typecheck.log |
| 前端 build | `pnpm build` | exit 0（含 4,286.49 kB WASM） | 12-frontend-build.log |
| Worker smoke | `pnpm smoke:native` | **82／0** | 09-worker-smoke.log |
| preflight | `python3 -B scripts/preflight.py` | 19/21（2 項：文件行號已修；腳本單測 3 項＝2 佈局假象＋1 玄影） | 14-preflight.log |
| 腳本單測 | `python3 -B -m unittest discover -s scripts` | 192 項、3 失敗（見上） | 14-preflight.log |
| golden 12 場 | `replay_scenarios_match_golden` | 1 失敗（11 business-equal＋1 語意待裁決） | 10-golden-dispositions.md／15-…patch |
| 行數門禁 | `python3 -B scripts/check-source-lines.py` + `unittest … test_check_source_lines.py` | **exit 0**：9052 檔 ≤ 2000 行、Rust 非測試 ≤ 1000 有效行；單測 2/2 | 19-source-lines.log |
| Tauri IPC（desktop-client） | `cargo test -p desktop-client` | **12 + 6 passed、0 failed**（另有 2 個空套件 1 ignored） | 20-tauri-ipc.log |
| 瀏覽器 7 案例 | — | **未跑**（需 331／106 專屬夾具與 oracle） | — |

## 合併前仍需你裁決／補做

1. golden 11 場重錄（提案已備，未套用）；`chain_fire_propagation` 需語意裁決。
2. 玄影 parent-group grant（方案 B）——目前 `=> false` 是臨時阻擋，arms 87≠88 會讓 `test_report_event_arms` 紅。
3. 文件行號同步（本輪已修 `phase-timing-extension.md` 兩處；其餘 5 處越界是私有樹 archive 符號連結造成的假陽性，正常 checkout 應為 0）。
4. registry 262/290 實際推進（清單見 `17-c331-remaining-28.md`）；規則 332／R05 仍排在 331 之後。
5. 瀏覽器案例與 legacy WASM 重建（舊線凍結，需你同意）。
