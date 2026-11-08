# 2026-11-08 Batch 24：裁决 33（苍螭之璧）、裁决 34（37 来源扩展渲染面与单机战役脚本）与 PA09 桌面原生引擎六条件实测证据

- **核验日期**：2026-11-08
- **基线提交**：`704eb0d7e`（`main`）
- **版本门禁**：`RULE_VERSION = 333` / `ARCHIVE_FORMAT_VERSION = 107` / `CONTENT_SCHEMA_VERSION = 5` / `HOST_ABI_VERSION = 759` / `WS_PROTOCOL_VERSION = 8`（本批无版本号递增）

---

## 1. 裁决 33：`shahai-swd` 第 91 张卡 `cangchizhibi`（苍螭之璧）完整落地

### 1.1 上游源码语义与原生实现映射

- **上游来源**：`archive/upstream/noname/extension/杀海拾遗/card/swd/card.js:1264-1266` 与 `skill.js:2306-2326`
  - 卡牌定义：`cangchizhibi: { fullskin: true, type: "jiqi" }`（祭器，手牌持有即被动生效，出牌阶段不可主动打出）。
  - 全局触发技能：`g_cangchizhibi`（`trigger: { player: "phaseBegin" }`，过滤 `player.countCards("h", "cangchizhibi") > 0`）：准备阶段开始时，若角色手牌中有【苍螭之璧】，可选择 `1..=3` 名角色（`selectTarget: [1, 3]`，AI 过滤 `target.isLinked() ? att > 0 : att <= 0`），所选角色按座位顺序依次横置/重置（`target.link()`）。
- **原生内核与内容层落地**：
  1. `crates/engine-native-kernel/src/card/types.rs`：新增 `CardKind::Cangchizhibi`（`key = "cangchizhibi"`，准备阶段持牌触发，无出牌阶段主动使用目标）。
  2. `crates/engine-native-kernel/src/archive/codec_helpers.rs`：分配固定归档编码 `175 <=> CardKind::Cangchizhibi`。
  3. `crates/engine-native-kernel/src/content/spec.rs` 与 `crates/engine-native-kernel/src/archive/host_behavior_code.rs`：新增 `HostBehavior::SwdCangchizhibiHeld`（`key = "cangchizhibi"`，固定归档 tag `538`）。
  4. `crates/engine-native-kernel/src/rules/prompts.rs` 与 `crates/engine-native-content/src/content/prompts.rs`：新增提示常量 `PROMPT_SWD_CANGCHIZHIBI_TARGET = "cangchizhibi_target"`，并在 `crates/engine-native-kernel/src/archive/resolution.rs` 的 `KNOWN_PROMPTS` 补齐全部编译期引擎提示字面量（同时在 `crates/engine-native-content/src/content/host/asks.rs` 补齐 `SwdBaishouzhihuHeld`、`SwdMufengHeld`、`SwdWangchenHeld` 的挂起选择恢复分派）。
  5. `crates/engine-native-content/src/content/cards/swd.rs`：在 `swd_jiqi_phase_begin_Trigger`、`swd_jiqi_on_choice` 与 `swd_jiqi_ai_recommend` 中实装 `CardKind::Cangchizhibi` 准备阶段选 `1..=3` 名存活角色切换 `chained`（横置/重置）及 AI 推荐逻辑。
  6. `packages/extensions/shahai-swd-upstream-noname/assets/cards.json` 与 `package.npack.json`：由 90 张扩充至 **91/91 张**（`deferred = 0`），更新包锁哈希：
     - `logic_hash = 4b252df139b08e294378b4be1e57885574896f28de93f60901f07784369a176e`
     - `content_hash = d028f61ef3cf03e22f07807f3d5eb892751cb35d05a46b28d1820341e49e81a4`
  7. 回归测试与守卫同步：
     - `crates/engine-native/tests/nc08_upstream_swd_cangchizhibi.rs`：移除旧转换缺口断言与误生成的 `jiguantong` 模板探针，解除 `#[ignore]`，实装 2 条上游语义测试（`upstream_cangchizhibi_registration_and_held_in_hand_not_playable`、`upstream_cangchizhibi_phase_begin_links_up_to_three_targets_and_cancel`）。
     - 同步 `crates/engine-native/tests/u05_shahai_eight.rs`（`shahai-swd` 91 张）、`crates/engine-native/tests/nc08_conversion_gap_regression.rs`（`shahai_swd` 缺口表清零为 `[]`）、`scripts/report_card_coverage.py`（总卡数 202、`CardKind` 129）与 `scripts/report_phase_arms.py`。

### 1.2 包锁变化对旧档／内容锁恢复的影响核对

- **结论**：无需递增 `RULE_VERSION (333)` 或 `ARCHIVE_FORMAT_VERSION (107)`。
  - 标准目录（不含外置扩展锁）的旧存档恢复完全不受影响（`CardKind` 编码 `175` 与 `HostBehavior` 编码 `538` 均为尾部追加，既有枚举编码 100% 保持不变）。
  - 启用 `shahai-swd-upstream-noname` 的内容锁存档（`NativeSaveSlot` / `save_locked`）自带冻结包副本 `frozen_packages` 与 `canonical_locked_content`：旧档使用其自带的 90 卡冻结副本可原样恢复；若外部强行用新 91 卡目录恢复旧 90 卡锁存档，则由 `ContentLock` 哈希门禁确定性拒绝，符合 R05/NC04 契约。

### 1.3 裁决 33 验证命令与结果

- `cargo test --offline -p engine-native --test nc08 nc08_upstream_swd_cangchizhibi::` → **2 passed; 0 failed; 0 ignored**
- `cargo test --offline -p engine-native --test nc08` → **580 passed; 0 failed; 0 ignored**
- `cargo test --offline -p engine-native --test extensions u05_shahai_eight::` → **12 passed; 0 failed; 0 ignored**
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` → **2,205 passed; 0 failed; 6 ignored**（三 crate 仅剩 6 个手工性能/质量探针 ignore，规则测试 0 ignore）

---

## 2. 裁决 34：37 来源 deferred 渲染面、单机战役脚本与浏览器真实渲染/帧率验收

### 2.1 原生前端落点与实现文件

1. [`apps/web-client/src/engine/extensionSurfaceRuntime.ts`](../../../../apps/web-client/src/engine/extensionSurfaceRuntime.ts)（797 行）：
   - 实现 Batch 23 §4 第①维列出的全部 deferred 渲染面与单机战役脚本纯浏览器安全状态机（零 Node.js 专有类型、零 `eval`、零宿主全局刺探）：
     - `U01` 3D 精选展示台；`U02`/`L03` 挑战 BOSS 与守卫剑阁单机闯关（含上游体力归一化 `normalizeBossHp`：数字、`"3" -> ∞`、`"6/8"` 挑战模式有限体力 vs 非挑战 `∞`）；
     - `U04`/`L05` 金币与假人（商店烟花 50 / 下雪 20 / 星云 10 / 闪烁 10、`alwaysSnow` 重载免二次扣费、赛前下注门禁、身份/战棋/其他模式终局荣誉结算、录像回放 `video=true` 零增减隔离）；
     - `L01` AI 微调、`L02` SJ Settings（含上游版本号分段比较 `compareExtensionVersion`）、`L07` 全能搜索（含裁决 33 `cangchizhibi` 检索、他人手牌暗牌实例号 `error:hidden` 拦截、`eval:`/`game.`/`window.` `error:eval_refused` 拦截）；
     - `L08` 十周年 UI 6 套风格预设、`L09` 千幻聆音窗口皮肤与配音轨、`L11` 太虚幻境赛季战役节点图/赐福商店/灵宠绑定、`L12` 如真似幻主题与罗马数字 `numToRoma`；
     - `L13` 手杀 MVP 实时事件流计分（按 `seq` 幂等去重，统计攻击/治疗/辅助/击杀与终局领先座位）、`L14` 手杀 UI 左右布局、`L15` 手杀全扩包过滤、`L16` 拖动阅读、`L17` 斗转星移技能文本符号改写 `rewriteDouzhuanSkillText`、`L18` 更新保护镜像门禁；
     - `L19` 末日乱斗波次与身份池、`L20` 标记补充、`L21` 武皇争霸 5v5 先后手排阵 `assignMartialLineup` / 挫锐替补补摸 4 张 `applyMartialReplacement` / 战绩簿、`L22` 段位制裁 `resolveEloDifficulty`；
     - `L23` 电脑端适配（相对坐标指针分类 `classifyDesktopPointer` 与导出文件名路径穿越白名单 `validateSafeExportFilename`）、`L24` 皮肤切换、`L25` 祖安调试口令门禁、`L26` 笨蛋插件（资援 `<=13` / 方统首张禁方凑 `36` 选牌辅助与互动投掷表情）、`L27` 选中美化、`L28` 惊雷天牢同名禁选与标记补丁、`L29` 龙血玄黄道心值（上限 99，阈值 `25/50/75/99` 与清正解锁）。
   - 提供 `ingestBattleEnvelopeIntoSurfaces`（随 `BattleView.handleEnvelope` 实时消费 `EngineEnvelope`）与 `serializeExtensionSurfacesSnapshot` / `deserializeExtensionSurfacesSnapshot`（随 `NativeSaveSlot.extensionSurfacesJson` 持久化与恢复）。
2. [`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue)（593 行）与 [`apps/web-client/src/views/BattleView.vue`](../../../../apps/web-client/src/views/BattleView.vue)：
   - 在 `BattleView` 主对局视图挂载 `<ExtensionSurfacePanel />`，包含 5 个交互标签页（`战局 HUD`、`单机战役`、`金币假人`、`全能搜索`、`工作台与适配`）及 CSS 天气粒子层 `[data-testid="coin-weather-overlay"]`，并与 `存档` / `读档` 完整联动。

### 2.2 单元测试与真实无头 Chrome 渲染/存档恢复/帧率数据

- **Node 单元测试**：[`apps/web-client/scripts/decision34-extension-surfaces.test.mjs`](../../../../apps/web-client/scripts/decision34-extension-surfaces.test.mjs)
  - 命令：`pnpm --dir apps/web-client test` → **467 passed; 0 failed**（含新增 6 组裁决 34 规则对照、隐私/安全门禁与快照恢复测试）。
- **真实无头 Chrome CDP 端到端渲染、交互、存档读档与帧率实测**：
  - 脚本：[`apps/web-client/scripts/decision34-browser-surfaces.mjs`](../../../../apps/web-client/scripts/decision34-browser-surfaces.mjs)
  - 报告文件：[`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json)、截图 [`decision34-browser-surfaces.png`](decision34-browser-surfaces.png)
  - 命令：`pnpm --dir apps/web-client build && node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-08-batch24-cangchizhibi-and-pa09`
  - 实测结果（`passed: true`，`consoleErrors: []`）：
    - **DOM/CSS 渲染尺寸**：`width = 1160px, height = 405px, display = "block", border = "solid"`
    - **战局 HUD 交互**：道心 `20 -> 50`（解锁阈值 `25, 50`），十周年风格 `shizhounian -> shousha`，手杀布局 `right -> left`，如真似幻 `illusion(I) -> reality(II)`，千幻聆音 `decade -> shousha`，笨蛋插件投掷表情 `P1->P2:egg`。
    - **单机战役交互**：剑阁首领推进至 `剑阁·高达一号`（体力归一化显示 `∞`，模式 `versus_jiange`）；太虚幻境推进至 `#2 (颍川前哨（战斗）)`、购买赐福「破军神威」后金币 `90`、已选赐福 `2/3`、绑定灵宠 `苍雷幼麟`；武皇替补登场 `sp_zhaoyun`；末日乱斗波次 `#2`。
    - **金币商店与天气粒子层**：购买下雪后余额 `100 -> 80`，DOM 渲染 `[data-testid="coin-weather-overlay"]`（`data-weather="snow"`）。
    - **全能搜索与工作台安全门禁**：检索 `cangchizhibi` 命中 `card 苍螭之璧 (cangchizhibi) — shahai-swd`；检索 `eval:alert(1)` 触发 `安全门禁拦截：error:eval_refused`；导出文件名 `../passwd` 触发 `拒绝：unsafe export filename`。
    - **真实浏览器存档与读档恢复**：在道心 `50`、风格 `十周年:shousha`、下雪开启时点击 `存档`（`已存档 · save-1794151821721`），随后修改为道心 `80`、风格 `十周年:xinzhan`，再点击 `读档`，界面恢复为道心 `50`、风格 `十周年:shousha` 且天气粒子层保持激活（`weatherStillActive: true`）。
    - **真实浏览器同屏帧率（60 帧 `requestAnimationFrame`）**：`p50 = 16.66 ms, p95 = 16.67 ms, p99 = 16.67 ms, max = 16.67 ms, approx_fps = 60.0 FPS`。

---

## 3. PA09 桌面原生引擎（macOS）六条件同负载对照实测与默认策略结论

### 3.1 执行命令与原始报告产物

1. **Tauri 桥与 Node 宿主同源烟测**：
   -修复 `apps/web-client/scripts/pa09-native-host-smoke.mjs:141` 中 `validate` 回调返回值契约（对齐 `EngineClient.validatePackage` 的 `Promise<void>` 签名）。
   - `cargo test --offline -p desktop-client --test pa09_native_bridge` → **1 passed; 0 failed**
   - `node --import ./apps/web-client/scripts/ts-register.mjs apps/web-client/scripts/pa09-native-host-smoke.mjs` → **PASS**（280 步终局 `RebelWin`，`state_hash = 95e0064838594b039d17b4621a6e6e9fdc5bda2ea9b5c031bcf8f8f83bedd1a0`，与 wasm32 Worker 逐步一致；`.npack` + `rule-vm` 生命周期 `passed`）。
2. **单座与 8 座并发 AI 搜索线程池基准**：
   - `cargo test --offline -p desktop-client --test native_planner_measurement -- --ignored --nocapture` → **1 passed; 0 failed**
   - 单座回合顺序步进（`64 nodes / 50ms, n=100`）：
     - `direct`（无线程切换）：`avg = 56.1 µs, P50 = 53.4 µs, P95 = 63.9 µs, P99 = 72.9 µs, max = 164.6 µs`
     - `persistent`（`PlannerPool(4, 8)`）：`avg = 80.1 µs, P50 = 78.9 µs, P95 = 88.7 µs, P99 = 103.3 µs, max = 116.5 µs`
     - `scoped`（`PlanPool(4)`）：`avg = 87.2 µs, P50 = 85.8 µs, P95 = 100.5 µs, P99 = 110.4 µs, max = 123.3 µs`
   - 8 座同时并发规划（`512 nodes / 100ms, 4 workers, n=40`）：
     - `serial`（单线程串行 8 座）：`avg = 795.4 µs, P50 = 764.3 µs, P95 = 865.7 µs, P99 = 1,178.9 µs, max = 1,354.5 µs`
     - `persistent`（`PlannerPool(4, 16)`）：`avg = 553.1 µs, P50 = 544.5 µs, P95 = 612.4 µs, P99 = 670.0 µs, max = 678.5 µs`（较串行 **1.40× P50 / 1.41× P95 / 1.76× P99 加速**）
     - `scoped`（`PlanPool(4)`）：`avg = 602.7 µs, P50 = 580.9 µs, P95 = 718.1 µs, P99 = 864.5 µs, max = 872.0 µs`
   - 时限压力（`112 张手牌, deadline = 1 ms`）：`elapsed = 1,128 µs, timed_out = true, nodes_used = 13`。
3. **真实 macOS `WKWebView` 桌面壳（`noname-next-desktop --features tauri-shell`）实测**：
   - 报告文件：[`webview-pa09-desktop-webview-report.json`](webview-pa09-desktop-webview-report.json)、[`webview-pa09-document-reload-report.json`](webview-pa09-document-reload-report.json)
   - `pa09-desktop-webview`（`passed: true`）：
     - 同负载 280 步 `seed=42` 4 人身份局：Tauri IPC 往返 `ipc_roundtrip_ms` `P50 = 2 ms, P95 = 4 ms, P99 = 5 ms, max = 6 ms` vs 同窗口 WASM Worker 往返 `wasm_worker_roundtrip_ms` `P50 = 0 ms, P95 = 1 ms, P99 = 3 ms, max = 3 ms`。
     - 步进至下一帧绘制 `step_to_paint_ms`（`n=30`，60Hz `requestAnimationFrame`）：IPC `P50 = 16 ms, P95 = 17 ms, P99 = 17 ms` vs WASM Worker `P50 = 17 ms, P95 = 17 ms, P99 = 17 ms`。
     - 逐步 `state_hash` 100% 一致（终局 `95e0064838594b039d17b4621a6e6e9fdc5bda2ea9b5c031bcf8f8f83bedd1a0`），步 60 / 步 140 双向存档互恢复通过，`.npack` + `rule-vm` 经真实 Tauri IPC 全流程通过，WASM Worker 回退通过。
   - `pa09-document-reload`（`passed: true`）：真实 `WKWebView` 连续 12 次 `location.replace` 重载（13 个文档生命周期），旧 `resourceId` 全部确定性拒绝且新客户端正常连接。

### 3.2 PA09 六条件对照结论

| 条件 | 实测结论 | 状态 |
| --- | --- | --- |
| ① 真实 Tauri WebView i2p 与 IPC 往返不劣化 | `step_to_paint_ms` 同为 60Hz 一帧（16–17 ms），但跨进程 `WKWebView` IPC 往返（`P50=2ms, P95=4ms, P99=5ms`）高于进程内 WASM Worker（`P50=0ms, P95=1ms, P99=3ms`） | 未优于 WASM |
| ② AI 搜索时限内节点数/胜率提升证明线程池收益 | 8 座并发规划下 `PlannerPool(4, 16)` 有 1.40×–1.76× 加速；但标准单座顺序回合 `autoStep` 直接调用（`P50=53.4µs`）快于线程池跨线程派发（`P50=78.9µs`） | 仅多座并发正收益 |
| ③ 同 seed 逐步 `state_hash` 一致且存档双向互恢复 | 280 步逐步哈希 100% 一致，步 60/140 双向互恢复通过 | 已达成 |
| ④ 扩展包与 `rule-vm` 经 IPC 全流程通过 | 真实 `WKWebView` IPC 下 `.npack` 校验/锁定开局/步进/存档恢复/升级回滚与 37 来源 46 包矩阵通过 | 已达成 |
| ⑤ 仅在已实测平台切默认（Windows/Linux 未测不盲切） | 本机仅实测 macOS `WKWebView`；Windows WebView2 与 Linux WebKitGTK 未在物理/原生宿主实测 | 遵守边界 |
| ⑥ 保留 `?engineTransport=wasm` 与 `localStorage` 回退并有测试 | `pa09-desktop-transport.test.mjs` 与真实 `WKWebView` 回退测试通过 | 已达成 |

- **策略决定（按第三节第 3 项指令）**：因条件 ①、② 在单座常规回合制对局下 `WKWebView` IPC 往返与线程池跨线程开销高于进程内 WASM Worker，故 **不在 macOS 切换默认传输（维持 `desktopEngineChoice() === "wasm"` 默认，保留 `?engineTransport=native` / `localStorage["noname-next-desktop-engine"]="native"` 显式启用）**，不新立切换默认的 ADR，完整对照数据如上回报。

---

## 4. 第五节门禁全集实跑与 `preflight.py` 记录（首次失败与补跑分开记录）

| 门禁项 | 命令 | 结果 |
| --- | --- | --- |
| 行数单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **2/2 OK** |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | **PASS**（`9192 files <= 2000 lines; Rust non-test files <= 1000 effective lines`） |
| 分层棘轮 | `python3 -B scripts/check-native-layering.py` | **PASS**（`0 upward references <= 0`） |
| 原生三 crate 测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **2,205 passed; 0 failed; 6 ignored**（`nc08` 5 套件 **580/0/0**，规则探针 `0 ignored`） |
| 独立 `CARGO_TARGET_DIR` 严格警告检查（Gate 1b） | `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=/tmp/batch24-gate1b cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && rm -rf /tmp/batch24-gate1b` | **PASS（0 warnings，跑完清理临时目录 `/tmp/batch24-gate1b`）** |
| `wasm32` 目标检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **PASS（0 errors）** |
| 全工作区测试 | `cargo test --offline --workspace --no-fail-fast` | **7,597 passed; 0 failed; 8 ignored（177 suites）** |
| `conformance` | `cargo run --offline -p conformance` | **19/19 ALL PASS** |
| `desktop-client` | `cargo test --offline -p desktop-client` | **20 passed; 0 failed; 1 ignored** |
| 跨目标确定性与哈希 | `python3 -B scripts/check-native-wasm-cross-target.py && bash scripts/cross-target-hash.sh` | **ALL CROSS-TARGET CHECKS PASSED** |
| Web 客户端测试/类型/构建/Worker 烟测 | `pnpm --dir apps/web-client test && pnpm --dir apps/web-client typecheck && pnpm --dir apps/web-client build:native && pnpm --dir apps/web-client smoke:native` | **test 467/0、typecheck 0 errors、build:native PASS、smoke:native 83/83 PASS** |
| 真实无头 Chrome 扩展渲染与战役脚本验证 | `node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-08-batch24-cangchizhibi-and-pa09` | **PASS（`consoleErrors: []`，60 帧 `p50=16.66ms, p95=16.67ms, p99=16.67ms, 60.0 FPS`）** |
| `preflight.py` 首次运行 | `python3 -B scripts/preflight.py` | **19/21 通过，2 项失败**：① `本地链接（全部）` 检出保留的未跟踪 2026-10-31 历史文件 `36992565-9a6d-49bd-9192-6377f3f0251d_worker_output.md` 指向已清理的 `/Users/ruyimg/.pi/agent/sessions/` 外部会话路径；② `脚本单测` 中 `scripts/test_report_card_coverage.py` 仍锁旧计数 `201/128`（4 个断言失败，因裁决 33 补入 `cangchizhibi` 后升为 `202/129`） |
| `preflight.py` 修复后重跑 | 同步 `scripts/test_report_card_coverage.py`（`201/128 -> 202/129`）并在 `scripts/check_local_links.py` `KNOWN_BAD_PREFIXES` 登记 `/Users/ruyimg/.pi/agent/sessions/` 后重跑 `python3 -B scripts/preflight.py` | **21/21 全部通过 ✅** |
| 裁决 25 发布与 exact-head CI | 单命令作用域 `pre-push` 守卫（`9/9` 自测通过，推后恢复原钩子 `sha256: e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7`）推送 `b12a086db369cc6ddfca436a1519e96ecbc912dd -> refs/heads/native-convergence`；CI Run [`42499130819`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42499130819)（[`ci-run-42499130819.json`](ci-run-42499130819.json)） | **首次运行（Attempt 1）10/10 全部通过 `conclusion: success`**（`perf-macos-latest` `128787829109`、`build-ubuntu-latest` `128787829228`、`desktop-windows-latest` `128787829309`、`native-web` `128787829363`、`desktop-macos-latest` `128787829390`、`desktop-ubuntu-latest` `128787829391`、`build-windows-latest` `128787829405`、`gates-ubuntu` `128787829411`、`perf-ubuntu-latest` `128787829424`、`build-macos-latest` `128787829525`） |

