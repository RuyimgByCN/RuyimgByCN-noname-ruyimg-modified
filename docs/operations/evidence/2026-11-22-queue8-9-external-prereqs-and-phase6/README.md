# Queue Item 8 & 9 Prep：外部前置条件仓内闭环、Spine 4.0 用户指引与第六阶段切换范围（2026-11-22）

- **日期与工作基准**：`2026-11-22`，工作树 `.wt-conv-q1-u01`，分支 `swarm/conv-q8-q9-prereqs`（基于 `main` HEAD `a47f8b0e110cb217f54cb4f40f111a223dab649a`）
- **冻结版本与锁保持零变更**：`RULES_VERSION = 333`、`NGSA = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`、`14` 项标准 golden 与 `u01-dddanzhi-pending-v107.ngsa` 均未改动；未触碰 `docs/architecture/native-progress.md`、`docs/migration/g0/inventory_status.json` 与 `docs/migration/upstream-extension-reconstruction.md`。
- **范围定位**：完成 [原生收敛执行提示词](../../native-convergence-agent-prompt.md) 第四节第 8 项（`PA08`/`PA11`/`NC12` 外部前提重探、仓内可验模拟器/跨平台编译与桌面打包校验）与第 6/9 项及第三节第 3 条 ①–⑤ 的精准对齐与待用户裁决清单。**严格区隔口径**：仓内跨平台编译检查、模拟器/无头触控采样与未签名桌面二进制构建不能替代物理真机温控与三平台商业签名公证。

---

## 一、A：`PA08`/`PA11`/`NC12` 真机与三平台签名/公证用户操作清单

### 1.1 实时本机设备、SDK 与签名凭据重探（2026-11-22T03:59:23Z）

原始输出见 `docs/operations/evidence-raw/queue8-9/01-hardware-signing-probe.log`（`sha256: 0ed0c1c2902ed37cb3bcad2eee5c96a8c26bc21ce50ccd11ec84b1c632bdedae`）：

| 探测项 | 实际命令 | 2026-11-22 实测输出 | 当前可用状态 |
| --- | --- | --- | --- |
| 宿主机与 Xcode | `sw_vers`、`xcodebuild -version` | macOS `27.2.1` (`26B131`, arm64)，`Xcode 27.2` (`27B52`) | 已就绪 |
| Android USB 物理真机 | `/Users/ruyimg/Library/Android/sdk/platform-tools/adb devices -l` | `List of devices attached`（**0** 台在线物理机） | **阻塞：无物理真机** |
| Android SDK / AVD | `ls platforms build-tools ndk`、`emulator -list-avds` | `android-34`/`36`、`build-tools 35.0.0/36.0.0`、`ndk/29.0.14206865`、`openjdk 25.0.4`、`gradle`、4 台 AVD（`codex35` 等） | 仓内/模拟器可用 |
| iOS 物理真机 | `xcrun devicectl list devices` | `Ruyimg音` (`00008140-0002149826F3001C`, `iPhone17,2`) 为 **`unavailable`** | **阻塞：物理 iPhone 离线** |
| iOS 模拟器 | `xcrun simctl list runtimes`、`xcrun simctl list devices available` | `iOS 27.0 (24A434)` + `iOS 27.1 (24A94425)`，共 14 台可用模拟器（均 `Shutdown`） | 模拟器可用 |
| macOS / iOS 代码签名 | `security find-identity -v -p codesigning` | **`0 valid identities found`** | **阻塞：无可用签名证书** |
| CI/打包签名环境变量 | 读取预设名存在性（不输出值） | `TAURI_SIGNING_PRIVATE_KEY`、`APPLE_*`、`ANDROID_KEYSTORE_PATH` 均为 `UNSET`（仅 `ANDROID_HOME=SET`） | **阻塞：签名/公证凭据未配置** |
| 编译工具链 | `rustc`/`wasm-pack`/`node`/`pnpm`/`tauri` | `rustc 1.95.0`（含 `aarch64-apple-ios-sim`、`aarch64-linux-android`、`wasm32-unknown-unknown`、`x86_64-pc-windows-msvc` 等 12 目标）、`wasm-pack 0.15.0`、`node v26.4.0`、`pnpm 10.34.3`、`tauri-cli 2.11.4` | 仓内全齐 |

### 1.2 仓内已完成验证与实查脚本/工程缺口

1. **桌面与移动端跨目标编译 + 宿主单测（全数通过）**：
   - `cargo test --offline -p desktop-client --all-targets`（`04-desktop-client-tests.log`）：**20 passed; 0 failed; 1 ignored**（覆盖 9 项 `mobile::tests::*`、包冻结与签名安装、`NativeEngineHost` IPC 单写序/37 来源 46 包锁定建局/恢复；`native_planner_measurement` 墙钟耗时基准保持 `#[ignore]`）。
   - 跨目标离线检查（`05-desktop-cross-target-checks.log`、`09-desktop-strict-and-wfe-host-smoke.log`）：
     - `cargo check --offline -p desktop-client --features tauri-shell --bin noname-next-desktop`：**exit 0**；`RUSTFLAGS="-D warnings" cargo check --offline -p desktop-client --features tauri-shell --all-targets`：**exit 0（零警告）**；
     - `cargo build --offline -p desktop-client --features tauri-shell --bin noname-next-desktop`（`08-web-build-mobile-touch-and-desktop-bin.log`）：产出 `78M` 未签名桌面二进制（exit 0）；
     - `cargo check --offline -p engine-native-wasm --target wasm32-unknown-unknown`：**exit 0**；
     - `cargo check --offline -p desktop-client --lib --target aarch64-apple-ios-sim`：**exit 0**；
     - `cargo check --offline -p desktop-client --lib --target aarch64-linux-android`：**exit 0**；
     - `tauri info`（`apps/desktop-client`）：**exit 0**（`tauri 2.11.5`、`wry 0.55.1`、`frontendDist: ../web-client/dist`）。
2. **前端打包、移动触控模拟与外部采样脚本修复**：
   - `wasm-pack build crates/engine-native-wasm --target web --release --no-opt` + `pnpm --dir apps/web-client test` + `smoke:native`（`06-web-native-node-smoke-and-test.log`）：**527/527 passed（0 failed）**、`smoke:native` **83/83 passed（failed: 0）**；含 [`build-pipeline.test.mjs`](../../../../apps/web-client/scripts/build-pipeline.test.mjs) 10/10（校核 [tauri.conf.json](../../../../apps/desktop-client/tauri.conf.json) WiX `zh-CN` 与 [tauri.linux.conf.json](../../../../apps/desktop-client/tauri.linux.conf.json) `.deb`/`.rpm` 模板）。
   - `pnpm --dir apps/web-client run build`（`08-...log`）：`vue-tsc --noEmit && vite build` 通过（`1.94s`，生成 `dist/assets/engine_native_wasm_bg-*.wasm` `27,866.90 kB` 与仍被导入的旧 `dist/assets/engine_wasm_bg-*.wasm` `15,514.71 kB`）。
   - 无头 Chrome 移动视口触摸回归（[`nc12-mobile-touch.mjs`](../../../../apps/web-client/scripts/nc12-mobile-touch.mjs)，`mobile-touch-report.json`）：`viewport={"w":390,"h":844,"touch":true}` 下触摸 `本地开局` 成功进入 `/battle?players=4&seed=42`，`interactive: true`、`console_errors: []`（仅为 CDP 模拟，不替代物理真机）。
   - **修复仓内真机采样脚本硬编码漏洞**：实查发现 [2026-11-11 外部前提说明](../2026-11-11-external-prerequisites/README.md) 曾误写未实现的 `--device android` / `--device ios --real-device` 参数，且 [`web-frame-external.mjs`](../../../../apps/web-client/scripts/web-frame-external.mjs#L170-L186) 原固定绑定 `127.0.0.1`，会阻断物理 iPhone 经 Wi-Fi 局域网连接 Mac 采样端。本次已在该脚本新增可选 `--host <ip>`（默认仍为 `127.0.0.1`，向后兼容），并以 `--host 0.0.0.0 --driver click --steps 20` 实跑 Chrome 验证通过（`10-wfe-host-click-smoke.log`：`BOUND_0_0_0_0_PORT=62523`、`report received (6714 bytes)`、`long_tasks=0`、`frame_ms p99=16.8`）。
3. **如实登记仓内工程现状缺口（移动端原生壳工程尚未初始化）**：
   - [`apps/desktop-client/Cargo.toml`](../../../../apps/desktop-client/Cargo.toml)、[`apps/desktop-client/src/lib.rs`](../../../../apps/desktop-client/src/lib.rs#L354) 与 `icons/{android,ios}/` 已包含移动抽象与图标，**但 `apps/desktop-client/gen/android`（`tauri android init`）、`gen/apple`（`tauri ios init`）及 `capacitor.config.*` 当前均未在仓内生成**。因此：`PA08`/`NC12` 移动端**浏览器/WebView 实测**在设备接入后可立即可用 `web-frame-external.mjs` 开跑；若需交付**独立安装 `.apk`/`.ipa`**，取得签名凭据后还需执行 `tauri android init` / `tauri ios init`。

### 1.3 用户需提供的外部硬件、凭据与接入后立即执行的命令

> 本节延续并修正 [2026-11-11 外部前提说明](../2026-11-11-external-prerequisites/README.md) 中的实际命令参数口径。

| 编号与阶段 | 用户需提供的物理输入 / 凭据 | 接入后仓内/本机执行命令与验收阈值 |
| --- | --- | --- |
| **PA08 / NC12 Android 真机** | 1 台及以上 Android 真机（建议含中低端 2GB–4GB RAM Android 8–12 设备），打开「开发者选项 → USB 调试」并通过数据线信任当前 Mac（可选：Release `.jks` keystore 密码/别名以构建签名 `.apk`） | 1. 确认设备：`adb devices -l` 显示 `device`。<br>2. 构建前端后，另终端执行 `adb reverse tcp:8123 tcp:8123`。<br>3. 启动 ≥30 分钟长局采样（Chrome/WebView 打开 `http://127.0.0.1:8123/`）：<br>`node --import ./apps/web-client/scripts/ts-register.mjs apps/web-client/scripts/web-frame-external.mjs --dist apps/web-client/dist --out docs/operations/evidence-raw/pa08-android-real --host 127.0.0.1 --port 8123 --driver auto --loop 1 --duration 1800000 --timeout 1900000 --app none`<br>4. 若打包原生 `.apk`：先 `pnpm --dir apps/web-client exec tauri android init`，配妥签名环境变量后执行 `pnpm --dir apps/web-client exec tauri android build`。<br>**阈值**：帧间隔达标、无热降频失控/内存泄漏，记录设备型号、System WebView 版本、电池温度曲线与 `summary.json`。 |
| **PA08 / NC12 iOS 真机** | 将 `Ruyimg音`（或目标 iPhone/iPad，iOS 16+）接入当前 Mac、解锁信任并开启「开发者模式」（`xcrun devicectl list devices` 非 `unavailable`）；若打包原生 `.ipa`，另需 Apple Development/Distribution 证书与 Provisioning Profile | 1. **切勿传 `--coop`**（见 [`web-frame-external.mjs`](../../../../apps/web-client/scripts/web-frame-external.mjs#L113-L116)，Safari 开启 `crossOriginIsolated` 会导致 Worker 往返异常）。<br>2. Mac 同局域网启动采样服务（或用 Xcode Instruments 挂接 Safari/WebKit）：<br>`node --import ./apps/web-client/scripts/ts-register.mjs apps/web-client/scripts/web-frame-external.mjs --dist apps/web-client/dist --out docs/operations/evidence-raw/pa08-ios-real --host 0.0.0.0 --port 8123 --driver auto --loop 1 --duration 1800000 --timeout 1900000 --app none`<br>3. 手机 Safari 打开 `http://<Mac局域网IP>:8123/` 完成 30 分钟运行；在 Xcode Instruments 采集内存与发热（要求 `< 512 MB`）。<br>4. 原生 `.ipa` 需在配置 Apple 团队与证书后执行 `tauri ios init` 与 `tauri ios build`。 |
| **PA11 GPU 牌桌（ADR 0785 草案）** | 上述 Android/iOS 真机 ≥30 分钟温控/帧率/内存实测样本，以及您对是否投入原型的最终拍板（参考 [ADR 0785](../../../adr/0785-wgpu-table-prototype.md) 与 [PA08-6 模拟器基线](../2026-10-26-pa08-6-mobile/README.md)） | 若真机实测与 DOM 预算一样无掉帧/热节流，按 ADR 0785 保持 DOM 方案不立项 wgpu；仅在真机出现性能硬瓶颈或确认立项特写剧场特效时，按 ADR 0785 同设备/同负载三指标（帧间隔、点击到出帧、功耗）立项原型。 |
| **NC12 三平台签名 / 公证** | 1. **Tauri 更新私钥**：`TAURI_SIGNING_PRIVATE_KEY`（及 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`）填入本地与 GitHub Secrets（对应 [ci.yml](../../../../.github/workflows/ci.yml#L174-L176)）；<br>2. **macOS**：`Developer ID Application` `.p12` 证书与密码，及 Apple 公证凭据（`APPLE_API_KEY`/`APPLE_API_ISSUER` 或 `APPLE_ID`/`APPLE_PASSWORD`/`APPLE_TEAM_ID`）；<br>3. **Windows**：EV/Authenticode 代码签名 `.pfx` 证书与密码；<br>4. **Linux**：GPG 签名私钥（用于 `.deb`/`.rpm` 仓库签名）。 | **macOS 签名与公证**（导入钥匙串确保 `security find-identity -v -p codesigning` 非空）：<br>`pnpm --dir apps/web-client run desktop:build`<br>`codesign --force --options runtime --deep --sign "Developer ID Application: <名称> (<TeamID>)" target/release/bundle/macos/无名杀\ Next.app`<br>`ditto -c -k --keepParent target/release/bundle/macos/无名杀\ Next.app /tmp/noname-next-mac.zip`<br>`xcrun notarytool submit /tmp/noname-next-mac.zip --keychain-profile "<profile>" --wait && xcrun stapler staple target/release/bundle/macos/无名杀\ Next.app && spctl --assess -vv target/release/bundle/macos/无名杀\ Next.app`<br>**Windows**：`cargo tauri build` 产出 `.msi`/NSIS 后使用 `signtool sign /fd SHA256 ...`；**Linux**：在 `ubuntu-latest` 产出 `.deb`/`.rpm` 后执行 GPG/dpkg-sig 验证并确认 CI `desktop-*` 作业全绿。 |

---

## 二、B：Spine 4.0 用户本机自有动皮对照指引（裁决 35 / 37 / 38）

### 2.1 当前代码基线与净室声明（对照 HEAD `a47f8b0e1`）

- 遵守裁决 35 与 38：仓内**零官方 Spine 运行库**、**零商业皮肤/骨骼/贴图资产**；纯自研 WebGL / Canvas2D 净室播放器按 Spine 4.0 公开规范编写。
- **当前 HEAD `a47f8b0e1` 实测基线（诚实计数）**：[`spine40-cleanroom.test.mjs`](../../../../apps/web-client/scripts/spine40-cleanroom.test.mjs#L39-L494) 共 **`11/11` 通过**（见 `03-web-node-tests.log` / `06-web-native-node-smoke-and-test.log`；含 4.0.x 版本门禁与 3.5/3.7/3.8/4.1/4.2 回退、JSON 与标准 `.skel` 二进制有符号 `varint`/`hash`/`nonessential=false`/`LinkedMesh` 编解码、5 种 `TransformMode`、2-bone IK/Transform/Path 约束、6 类附件与碰撞、三次贝塞尔与多轨 `replace`/`add`、Atlas/PMA WebGL 批次、着色器与纹理上传、凹多边形耳切裁剪）；另在 `batch6-ui-closure.test.mjs`、`batch7-ui-closure.test.mjs`、`decision34-extension-surfaces.test.mjs`、`decade-configs-surfaces.test.mjs` 中通过零 Spine 商业运行库净室守卫。详见 [2026-11-09 净室报告](../2026-11-09-spine40-cleanroom/README.md) 与 [本地导入详细手册](../2026-11-11-tabletop-and-spine40/spine40-user-guide.md)。注：若合入队列中有另行扩充的 `Spine-Tests` 分支，需待其正式合入后再核计新测试数，不提前混入当前基线。
- **二进制 `.skel` 限制声明**：[`Spine40SkinImportBox.vue`](../../../../apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue#L160) 明确标示 **`.skel` 解码处于「未经真实资源验证」状态**（仅经自研编码器往返校验），对照优先使用 **`.json`** 导出；若 `.skel` 实测异常，记录报错与导出工具版本即可。

### 2.2 用户本机三件套导入与旧版无名杀同场景并排操作步骤

1. **准备合法自有 Spine 4.0 动皮三件套**：
   - `<skin>.json`（优先；确认文本含 `"skeleton": { "spine": "4.0.xx" }`）或 `<skin>.skel`；
   - 配套 `<skin>.atlas`（或 `.atlas.txt`）；配套 `<skin>.png`（可选 `.webp`/`.jpg`）。
   - 同时可另备 1 份非 4.0（如 `3.8` 或 `4.1`）骨骼以验证版本拦截与静态立绘兜底回退（裁决 35）。
2. **构建并打开原生牌桌**：
   ```bash
   pnpm --dir apps/web-client run build:native
   pnpm --dir apps/web-client dev
   ```
   浏览器打开 `http://localhost:5173/battle?players=4&seed=42&mode=identity`（或直接运行桌面端），将浏览器视口设为 **`1440×1080`**。
3. **打开导入入口（已实查组件定位）**：
   - 路径 1（牌桌顶栏快捷按钮，[`BattleBoard.vue`](../../../../apps/web-client/src/components/battle/BattleBoard.vue#L1290-L1306)）：点击 **`动皮`**（`btn-skinswitch-quick-edit`）或 **`Spine`**（`btn-skinswitch-quick-preview`）弹出 [`SkinSwitchModal.vue`](../../../../apps/web-client/src/components/battle/modals/SkinSwitchModal.vue#L125-L242)，切换至页签 **`Spine 4.0 自备导入`**（`btn-skinswitch-tab-spine40`），点击 **`玩家动皮导入 (saveSkinParams)`**（`btn-spine40-toggle-import`）；
   - 路径 2（牌桌扩展面板，[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue#L557-L594)）：展开「主题皮肤与镜头切换」，在内置 `Spine40SkinImportBox` 中点击 **`玩家动皮导入 (saveSkinParams)`**。
4. **填写参数并依次选入本地文件**：
   - 在 [`Spine40SkinImportBox.vue`](../../../../apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue#L256-L306) 中填入武将 ID、皮肤名、版本 `4.0`、动作（如 `daiji` / `gongji`）；
   - 依次上传 **`骨骼 (.json/.skel)`**、**`图集 (.atlas)`**、**`贴图 (.png)`**；
   - **正例期望**：画布 `spine-skeletal-canvas` 与 `spine-webgl-canvas` 实时绘制动画，状态显示 `ready-spine40 — SPINE_40_LOADED:4.0.xx`，`WebGL批次` 非 `0/0△`；
   - **负例期望**：导入 3.5/3.7/3.8/4.1/4.2 骨骼或清空资产后，显示 `static-portrait-fallback` 且无页面报错、可安全回退静态立绘。
5. **并排截图对比与反馈清单**（[对照维表](../2026-11-11-tabletop-and-spine40/spine40-user-guide.md#L67-L76)）：
   - 在旧版无名杀（十周年 UI / 皮肤切换）加载同一自备动皮、同一武将与动作、`1440×1080` 同偏移/缩放截图，对比 **骨骼关节、IK、网格权重、`Clipping` 凹遮罩、PMA 半透明 `Additive` 刀光、多轨切换混合** 6 项；
   - 请将：**① 并排截图（存 `docs/operations/evidence-raw/` 不入库商业素材本身）**、**② 骨骼导出版本号（如 `4.0.64`）**、**③ `.json` 与 `.skel` 实测加载状态及控制台日志** 提供给我们闭环修复。

---

## 三、C：第六阶段（Phase 6）精确切换范围提案（待用户确认）

依据 [原生收敛执行方案 §NC12](../../../architecture/native-convergence-plan.md#nc12) 与 [执行提示词 第三节第 7 条](../../native-convergence-agent-prompt.md#L37)，在第三节第 1–6 条全部成立并取得您确认后，按如下经源码逐项核查的清单执行**原生切换与旧执行入口退场**：

### 3.1 拟退场 / 删除（Retire）

| 分类 | 精确仓库路径与行号 | 当前仍残留在主线的原因与切换动作 |
| --- | --- | --- |
| **旧规则/AI/WASM 核心包** | `crates/engine-core/`、`crates/engine-ai/`、`crates/engine-wasm/`；[`Cargo.toml`](../../../../Cargo.toml#L7-L65) 第 7、17、19、40、64、65 行（另第 11–12 行重复列出了 `"crates/engine-native-wasm"`，切默认成员时一并清理） | 确认无调用方依赖后从 workspace 移除并删除 3 个旧 crate；同时将 `Cargo.toml` `default-members` 从旧 `engine-core` / `engine-cli` 切换为 `engine-native*`、`content-native-loader`、`engine-native-wasm`。 |
| **Web 客户端旧 Worker 与回退开关** | [`apps/web-client/src/worker/engine.worker.ts`](../../../../apps/web-client/src/worker/engine.worker.ts#L7-L20)、`src/wasm-pkg/`、`public/pkg/`、[`scripts/build-wasm.mjs`](../../../../apps/web-client/scripts/build-wasm.mjs)、[`package.json`](../../../../apps/web-client/package.json#L9)、[`vite.config.ts`](../../../../apps/web-client/vite.config.ts#L32)、`src/vite-env.d.ts:5`、[`src/engine/client.ts`](../../../../apps/web-client/src/engine/client.ts#L30-L78)（`LegacyEngineWorker` / `VITE_ENGINE_BACKEND=legacy`）、[`src/views/HomeView.vue`](../../../../apps/web-client/src/views/HomeView.vue#L87)、[`scripts/build-desktop.mjs`](../../../../apps/web-client/scripts/build-desktop.mjs#L16)、[`.github/workflows/ci.yml`](../../../../.github/workflows/ci.yml#L125) | 仅因前端仍在静态导入 `LegacyEngineWorker`，每次 `vite build` 仍须编译旧 `engine-wasm`（**多出约 15.5 MB 未压缩 / 3.45 MB gzip 产物**）。Phase 6 确认后移除 `LegacyEngineWorker`、`VITE_ENGINE_BACKEND` 分支与 `build:wasm` 构建步骤，更新 `nc04-worker-failure.test.mjs:52` 与 `build-pipeline.test.mjs:119-126`。 |
| **服务端旧线房间执行路径** | [`apps/game-server/src/room.rs`](../../../../apps/game-server/src/room.rs#L8-L99)（及 `L543` `engine_ai`）、[`lobby.rs`](../../../../apps/game-server/src/lobby.rs#L5-L29)、[`persist.rs`](../../../../apps/game-server/src/persist.rs#L10-L114)（及 `L378-L463`）、`apps/game-server/src/main.rs` 中的 `engine_core::Engine` / `engine_ai` 模板与房间分支、[`config.rs`](../../../../apps/game-server/src/config.rs#L32-L58) | 当前 [`ServerConfig::default().native_rooms`](../../../../apps/game-server/src/config.rs#L58) **仍默认为 `false`**（走旧 `engine_core` 房间）。Phase 6 确认后将线上权威房间默认并唯一切为 `room_native_actor`（`engine-native`），退场 `room.rs` 与 `engine_template: engine_core::Engine`，并从 `apps/game-server/Cargo.toml` 移除 `engine-core` / `engine-ai`。**前置解耦边界**：[`room_native_actor.rs`](../../../../apps/game-server/src/room_native_actor.rs#L14-L15) 及下表多套原生服务端测试当前仍从 `room.rs` / `persist.rs` 导入 `JoinReply` / `RoomHandle` / `RoomMsg` / `RoomPersist`（而 `persist.rs:10,378-463` 绑定 `engine_core::{Snapshot, Engine}`）；须先将上述消息与持久化类型从 `room.rs` / `persist.rs` 抽离并剥离 `engine_core` 导入，才可安全删除 `room.rs` 与旧 `persist` 恢复路径。 |
| **旧线哈希与校验脚本** | [`scripts/wasm-hash.mjs`](../../../../scripts/wasm-hash.mjs)、[`scripts/cross-target-hash.sh`](../../../../scripts/cross-target-hash.sh)、[`scripts/o4_build_bundle.sh`](../../../../scripts/o4_build_bundle.sh) | 这 3 个脚本目前仍绑定旧 `engine-cli` / `engine-wasm` / `wasm-pkg`。确认后由纯原生跨端校验（`check-native-wasm-cross-target.py` + `engine-native` CLI）替代，并在 `.github/workflows/ci.yml` Gate 3 同步替换。 |

### 3.2 切换前须先迁至 `engine-native`（Migrate / Rewrite）

| 现存消费者 | 精确文件 | 迁移内容 |
| --- | --- | --- |
| **`crates/conformance`**（CI Gate 2） | [`crates/conformance/Cargo.toml`](../../../../crates/conformance/Cargo.toml#L10)、[`src/lib.rs`](../../../../crates/conformance/src/lib.rs#L23) | 当前仍绑定 `engine-core` 向量；需迁至 `engine-native` 标准 14 golden + `content-native-loader` 一致性向量后才可删 `engine-core`。 |
| **`apps/engine-cli`** | [`apps/engine-cli/Cargo.toml`](../../../../apps/engine-cli/Cargo.toml#L14)、`src/main.rs` | 将无头诊断 CLI（`hash`/`manifest`/`skill-vm`/回放检查）改绑 `engine-native` 与 `content-native-loader`。 |
| **`crates/devtools`** | [`crates/devtools/Cargo.toml`](../../../../crates/devtools/Cargo.toml#L10)、`src/{timeline,query_viewer,leak_harness,state_diff,bisector,minimizer,package_runner}.rs` | 将规则时间线、差分、二分、泄密探针等开发者工具接口改绑 `engine-native`。 |
| **`apps/benchmark-runner`**（CI `perf`） | [`apps/benchmark-runner/Cargo.toml`](../../../../apps/benchmark-runner/Cargo.toml#L14-L15)、[`src/main.rs`](../../../../apps/benchmark-runner/src/main.rs#L19-L20) | 将规则/AI 压测场景从 `engine_core` + `engine_ai` 切至 `engine_native` + `BoundedPlanner`（保留 `device_tiers.rs` 预算结构）。 |
| **`fuzz/`（libFuzzer 目标）** | [`fuzz/Cargo.toml`](../../../../fuzz/Cargo.toml#L16)、[`fuzz/Cargo.lock`](../../../../fuzz/Cargo.lock)、[`fuzz_snapshot_decode.rs`](../../../../fuzz/fuzz_targets/fuzz_snapshot_decode.rs#L8)、[`fuzz_replay_decode.rs`](../../../../fuzz/fuzz_targets/fuzz_replay_decode.rs#L8) | 直接依赖 `engine-core`（`Snapshot::from_bytes` / `ReplayLog::from_bytes`）；删除 `crates/engine-core` 前须将两个 fuzz target 迁至 `engine-native` 解码入口并移除 `fuzz/Cargo.toml:16` 的 `engine-core` 依赖（同步更新 `fuzz/README.md` 与 `crates/engine-native/tests/fuzz_corpus_replay.rs:12` 说明）。 |
| **`apps/game-server` 集成测试** | [`nc03_nc09_product_api_and_timeout_matrix.rs`](../../../../apps/game-server/tests/nc03_nc09_product_api_and_timeout_matrix.rs#L9-L31)；`{tournament_wal_signing,room_flow,multi_room_stress,fault_injection,m8_rolling_upgrade,long_run_bot,long_run_soak,long_run_memory}.rs`；`nc09_dual_backend.rs`、`nc08_cross_line.rs`、`nc08_command_by_command.rs`；原生 `#[path]` 挂载组 `{adr0787_timeout,nc09_native_actor,nc09_native_durability,nc09_native_room_session,nc02_nc05_extension_lifecycle_matrix}.rs` | ① 将 `nc03_nc09_product_api_and_timeout_matrix.rs:24-25` 的 `use engine_core::rule_vm::...` 改为直接依赖 `rule_vm`；② 将 `tournament_wal_signing` / `fault_injection` / `m8_rolling_upgrade` 及旧 `room::spawn_room*` 测试（`room_flow` / `multi_room_stress` / `long_run_bot` / `long_run_soak` / `long_run_memory`）迁往 `room_native_actor` / `engine-native`；③ 双线退场后退休 `nc09_dual_backend.rs`、`nc08_cross_line.rs`、`nc08_command_by_command.rs`；④ 解耦 `RoomMsg` / `RoomHandle` / `JoinReply` / `RoomPersist` 后，同步更新上述 5 套原生测试及 `nc03_nc09` 的 `#[path]`/`use` 模块路径。 |
| **`scripts/websocket-smoke.mjs`** | [`scripts/websocket-smoke.mjs`](../../../../scripts/websocket-smoke.mjs#L38-L58) | 脚本硬编码过时的 `network_protocol: 4` / `content_schema: 3` / `rule_vm_abi: 3`（当前 `WS=8` / `schema=5` / `ABI=759`），迁移更新至 `WS=8` 或由 Rust `nc09_native_actor` 等原生测试替代。 |
| **`apps/desktop-client/README.md`** | [`apps/desktop-client/README.md`](../../../../apps/desktop-client/README.md#L15) | [`apps/desktop-client/Cargo.toml`](../../../../apps/desktop-client/Cargo.toml#L26) **已完全切至 `engine-native-wasm`**（零 `engine-core` 代码依赖），仅 README 结构图仍误写 `engine-core`，需同步文档口径（连同过期历史参考 [phase6-web-client.md](../../../architecture/phase6-web-client.md) 横幅提示）。 |

### 3.3 明确保留（Keep — 依裁决 36 / 38 与 NC12）

| 保留模块 / 工具 | 精确文件 | 保留原因 |
| --- | --- | --- |
| **桌面 `WASM Worker` 默认与 `?desktopEngine=native` 原型入口（裁决 36）** | [`apps/web-client/src/engine/desktopEngineTransport.ts`](../../../../apps/web-client/src/engine/desktopEngineTransport.ts#L24-L27)、[`SettingsView.vue`](../../../../apps/web-client/src/views/SettingsView.vue#L89-L99)、[`apps/desktop-client/src/native_engine_host.rs`](../../../../apps/desktop-client/src/native_engine_host.rs) | 裁决 36 已通过“桌面默认 WASM Worker、保留 `?desktopEngine=native` / `SettingsView` 原型切换”关闭 `PA09`；Phase 6 仅退休旧 `engine-wasm`，不撤掉该双原生传输切换入口。 |
| **迁移分析工具链** | [`crates/legacy-analyzer/Cargo.toml`](../../../../crates/legacy-analyzer/Cargo.toml#L23)、[`apps/extension-studio/Cargo.toml`](../../../../apps/extension-studio/Cargo.toml#L13) | 依 `NC12` 保留第三方 JS 迁移分析工具 `legacy-analyze` 与 `extension-studio`；仅需移除 `crates/legacy-analyzer` 的 `[dev-dependencies]` `engine-core`（并将 `tests/full_rewrite_pipeline.rs` 迁往 `engine-native`）。 |
| **共享协议/VM/资产与历史存档夹具** | `crates/{engine-api,rule-api,rule-vm,content-schema,content-compiler,content-runtime,content-native-loader,engine-protocol,diagnostics,resource-*,accessibility-schema,licensing-schema}`、`packages/`、`archive/upstream/`、`docs/adr/`、历史证据与 `old-native-saves.json` / `u01-dddanzhi-pending-v107.ngsa` | 保持纯原生基础设施、向后兼容性回归夹具、上游只读对照快照与历史 ADR/证据完整性。 |

---

## 四、D：其余第三节第 3 条待用户裁决汇总指针

| 条款 | 涉及项与精确边界 | 现状与现有裁决约束 | 需用户做出的选择与后续动作 |
| --- | --- | --- | --- |
| **3①（需升版本/改锁边界）** | **`L28:th_zhengjing`**（[`l28_batch_a_ext.rs`](../../../../crates/engine-native-content/src/content/host/step/l28_batch_a_ext.rs#L298-L397)、[Batch 1.2](../2026-11-20-batch1-2-l28/README.md#L23)） | `count=0` 与任意非空子集置经已修；上游切水果实时 DOM 小游戏与斗地主概率早停依**裁决 40**抽象为 `ChoiceSpec` `0..=5` 计数（保持 `简化近似`）。 | 确认是否接受裁决 40 抽象升为等价或保持已登记差异；若要求逐帧小游戏/外部输入事件透传内核，需另批新 `ChoiceSpec`/ABI（升 `RULE_VM_ABI_VERSION` / `NGSA`）。 |
| **3①（需升版本/改锁边界）** | **`U01:dddanzhi`**（[`u01_batch_d_asks.rs`](../../../../crates/engine-native-content/src/content/host/step/u01_batch_d_asks.rs#L383-L501)、[规则基准 §U01-3](../../../architecture/native-rules-baseline.md#L19-L21)、[Queue 0 证据](../2026-11-13-queue0-gates/README.md#L13-L15)） | 黑色末牌历史与目标独占免疫已修；现为保护冻结夹具 `u01-dddanzhi-pending-v107.ngsa` 与版本锁，仍保留 `Prepare` 触发与 `(Normal, 1, false)` 元数据（上游为真回合开始及 `(Normal, 0, true)`）。 | 已在后续 `U01-3` 收口其余 20 项并明确保持 `dddanzhi` 为 `简化近似` 以护 `v107` 挂起档，若切 `phaseBegin` 等价须提交版本边界申请（是否允许更新该待定夹具或升 `NGSA 107`）。另 `U01` 其余批次、`L11` 与 `U05` 后续若遇到确定超界技能将按同格式单列。 |
| **3①（需升版本/确定性墙钟边界）** | **`U05:shixu` & `U05:hs_nozdormu`**（[`hooks.rs`](../../../../crates/engine-native-content/src/content/host/step/u05_b6b_hearth/hooks.rs#L97-L301)、[`wal.rs`](../../../../apps/game-server/src/room_native/wal.rs#L204-L248)、[规则基准 Batch S1](../../../architecture/native-rules-baseline.md)） | `hs_nozdormu` 静态属性一致且 `shixu` 非时钟逻辑（含 `locked: true` 仅受缠怨封印不受铁骑封印）已等价闭环；仅上游 `get.time() - time > 8000` 真钟毫秒测定违背确定性回放且 `EndPhase`/`StoredNativeAction` 不含耗时字段（目前以 `PhaseEnded.elapsed_ms` 确定性参数代理，保持 `简化近似`）。 | **选项 A（不升版本号，维持确定性 `elapsed_ms` 代理并确认接受为已登记边界）**；或 **选项 B（要求联机实时计时录入 WAL，授权升级 `WS_PROTOCOL_VERSION` `8->9`、`STANDARD_RULESET_VERSION` `333->334` 及〔若独立字段〕`ARCHIVE_FORMAT_VERSION` `107->108`）**。 |
| **3①（Decision 40 实时 DOM 小游戏）** | **`U05:old_longfan`**（[U05-杀海拾遗](../../../migration/g0/U05-杀海拾遗.md)） | 上游为 200ms `setInterval` 实时 DOM 4 位虎口机交互；在冻结 `WS=8` / `ABI=759` 下保持 `简化近似`。 | 确认维持确定性回合制抽象为已登记差异，或另案设计跨版本前端实时输入回传协议。 |
| **3②（资产取得边界）** | **超出裁决 38 的商用美术/动皮/配音资产**（[裁决 38](../2026-10-28-user-decisions/README.md)） | 当前严格遵守裁决 38：零商业二进制资产入库，只保留配置 Schema 与玩家本地导入回退。 | 若需改变资产分发或联网获取方式，请明确授权来源与版权策略；否则维持裁决 38 不变。 |
| **3③（Git 历史体量）** | **2026-11-14 以前已入库约 `560 MB` 证据历史**（[提示词 §2](../../native-convergence-agent-prompt.md#L25)） | 严格遵守“不改写历史、不强推”；2026-11-14 之后增量严格受控于 `check_evidence_budget.py`（单文件 `<= 200 KB`、单批 `<= 2 MB`）。 | 确认保持完整历史不变，或在全仓合入收口并离线备份后统一执行一次历史瘦身与远端基线重置。 |
| **3④ / 3⑤** | **真机 / 签名公证 / Spine 4.0 用户对照 / 第六阶段切换范围确认** | 见本文 **第一节（A）**、**第二节（B）** 与 **第三节（C）**。 | 提交对应外部输入及范围确认后，立即接续实机验收、Spine 视觉对照修复与 Phase 6 清理合入。 |

---

## 五、本次原始命令日志与 SHA-256 索引（`docs/operations/evidence-raw/queue8-9/`）

| 产物文件 | 内容摘要 | SHA-256 |
| --- | --- | --- |
| `01-hardware-signing-probe.log` | 本机 Android `adb`、iOS `devicectl`/`simctl`、`security codesigning`、工具链与环境变量重探 | `0ed0c1c2902ed37cb3bcad2eee5c96a8c26bc21ce50ccd11ec84b1c632bdedae` |
| `02-preflight.log` | 基准工作树 `python3 -B scripts/preflight.py` 全 22 项初始基线扫描（18 通过；基准既有的 4 项文档链接/引行预存失败：`native-progress.md` 3 处、`l27-beautify-review` 2 处越界） | `9504454d2fab195f1fb748648155705e77433d3189aeeecabb7d9fee8e82649f` |
| `03-web-node-tests.log` | 无 `wasm-pkg`/`native-pkg` 下独立 `spine40-cleanroom.test.mjs`（`11/11`）及 `typecheck` 预探 | `589fb1906a3c953e23659f4b60cf9a82903906251b9bbf960551c787d97f7747` |
| `04-desktop-client-tests.log` | `cargo test --offline -p desktop-client --all-targets`（`20 passed; 0 failed; 1 ignored`） | `8ad538a8d6214d0e8b9108511691464acd6227e81e526a3f16af05f41dc1a6dc` |
| `05-desktop-cross-target-checks.log` | `tauri-shell`、`wasm32`、`aarch64-apple-ios-sim`、`aarch64-linux-android` 离线 `cargo check` 全过（exit 0）与 `tauri info` | `dc048840ebf69464fc4b9a8376eac1db1d2b7d2363f6e55f2bd54235ce0d1e55` |
| `06-web-native-node-smoke-and-test.log` | `wasm-pack --target web` 构建 `native-pkg` 后：`pnpm test` **527/527**（含 Spine 4.0 **11/11**、build-pipeline **10/10**）、`smoke:native` **83/83** | `5f7c3c16aa3aeee2683568e65819e0c10c96a0a70edce43570d8db4217fbea86` |
| `07-original-content-gate4.log` | CI Gate 4 原生资产 AST/序单测 `test_original_*.mjs` **17/17 通过**（本地未打 CI 补丁的 `sync-original-content --verify-only` 预期失败记录留存） | `d9a00f79311b48f9a670f81b0366b8076c866587006353a9c8b71fc6ce901942` |
| `08-web-build-mobile-touch-and-desktop-bin.log` | 完整 `vue-tsc --noEmit && vite build`、`nc12-mobile-touch.mjs` 移动触控与 `noname-next-desktop` 二进制构建（exit 0） | `aa76326307ff90c7b0440264e6c683917c89a1526e552e651953e0b0b092a710` |
| `mobile-touch-report.json` | 无头 Chrome `390×844` 移动触控报告（`interactive: true`, `console_errors: []`） | `4a2d740306754134a5707ed928568b3179d0359dca3f0742c46bd440b43f8e0c` |
| `09-desktop-strict-and-wfe-host-smoke.log` | `RUSTFLAGS="-D warnings" cargo check --offline -p desktop-client --features tauri-shell --all-targets` 零警告通过（exit 0） | `1386d3f4f34411bab7aa46c44f95116a24fb2bbe9767016d0dac77826ca200f3` |
| `10-wfe-host-click-smoke.log` | `web-frame-external.mjs --host 0.0.0.0 --driver click --steps 20` 外部采信全链路验证通过（exit 0） | `5761f2b6549d760aebfa734a571431bc56257b12cdbf2d701dc25de6cc1812aa` |
| `web-frame-external-host-smoke/summary.json` | 对应采样统计（`frame_ms p99=16.8`, `long_tasks=0`, `memory_mb=9`）及 `.csv` 原始流 | `549b95ed0e8cc048111e17ce5df1acae2bfb0d6a77e6b1dc3f3d89cb296d4068` |
