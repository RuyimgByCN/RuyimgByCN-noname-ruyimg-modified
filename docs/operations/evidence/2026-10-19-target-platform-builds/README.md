# 目标平台**构建级**验收（2026-10-19，本轮实测）

> **口径**：本文只证明「`engine-native` / `engine-native-wasm` 能在该 target 上**编译通过**」。
> **不证明**：真机运行、内存/性能特征、平台 I/O、UI、存档跨设备兼容。
> 八项里的「目标平台」要求的是**该平台上的真实运行验收**——本文**只覆盖其中「构建级」这一层**。

## 工具链

```text
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
rustc 1.95.0 (59807616e 2026-04-14)
```

## 结果

| target | crate | 结果 | 备注 |
|---|---|---|---|
| `aarch64-apple-ios` | `engine-native` | ✅ 通过（26.4s）| iOS 真机架构 |
| `aarch64-apple-ios-sim` | `engine-native` | ✅ 通过（24.8s）| iOS 模拟器架构 |
| `x86_64-linux-android` | `engine-native` | ✅ 通过（25.8s）| Android x86_64 |
| `armv7-linux-androideabi` | `engine-native` | ✅ 通过（25.0s）| Android armv7 |
| `i686-linux-android` | `engine-native` | ✅ 通过（25.5s）| Android x86 |
| `wasm32-unknown-unknown` | `engine-native-wasm` | ✅ 通过（47.7s）| **17 个 warning**（见下）|
| `aarch64-linux-android` | `engine-native` | ❌ **失败** | `cc-rs: failed to find tool "aarch64-linux-android-clang"` ⇒ **缺 Android NDK** |

### 命令

```bash
cargo build --offline -p engine-native --target <target>
cargo build --offline -p engine-native-wasm --target wasm32-unknown-unknown
```

## 唯一的失败：`aarch64-linux-android`

```
error occurred in cc-rs: failed to find tool "aarch64-linux-android-clang": No such file or directory
```

⇒ **缺的是设备/工具链**（Android NDK 的 clang），不是代码问题：
同一 crate 在 `x86_64`/`armv7`/`i686` 三个 Android target 上都编译通过，
说明是 `aarch64` 那条目标链上有一个需要 C 工具链的依赖（`cc` 构建脚本）。

**要补齐**：安装 Android NDK 并设置 `CC_aarch64-linux-android` / `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER`。

## 仍需真机/模拟器验收（未做）

- 三平台（iOS / Android / WASM）的**真机或模拟器运行**：启动、建局、完整对局、存档往返；
- 跨设备存档兼容（同一 `ARCHIVE_FORMAT_VERSION` 在另一平台恢复）；
- 性能与内存特征（与桌面差异）。

⇒ 「目标平台」这一项**仍为待办**：构建级 7 个 target 已有证据，运行级 **0**。

---

## WASM 目标：**运行级**证据（2026-10-19，本轮实测，Node 宿主）

### 命令

```bash
wasm-pack build crates/engine-native-wasm --target nodejs --release --no-opt --out-dir /tmp/wasm-pkg
PKG=/tmp/wasm-pkg node scripts/native_wasm_smoke.mjs
```

### 结果：**11/11 全部 PASS**

```text
PASS build | ruleset=v322 players=4
PASS view_own_hand | own=4
PASS view_hidden_hand | {"hidden":true}
PASS view_deck_count_only | deck=108
PASS view_per_seat
PASS auto_game_progress | steps=235 finished=true answers=39
PASS auto_suspended_none_or_resumable | suspended=0
PASS restore_view_equal | mid-save restore per-view equal
PASS restore_matches_same_seed | same-seed 60 steps equal restored view
PASS dual_instance_determinism | hash_equal=true
PASS choice_visibility_own_and_other | own=true other=true
{"total":11,"failed":0,...}
```

### 这证明了什么（按八项的口径）

| 维度 | 是否覆盖 |
|---|---|
| WASM **构建**（release + wasm-bindgen）| ✅ |
| WASM **运行**：建局 / 视图隔离 / 自动对局（235 步）/ 应答（39 次）| ✅ |
| WASM **存档往返** + 同 seed 重放一致 | ✅ |
| WASM **双实例确定性**（state hash 相等）| ✅ |
| **浏览器**内运行（DOM/WebWorker/网络）| ❌ 未做（Node 宿主，非浏览器）|
| iOS/Android **真机**运行 | ❌ 未做（无设备）|

⇒ **「目标平台」这一项从「构建级」推进到「WASM 运行级」**，
但**仍缺浏览器与真机** ⇒ **八项完成数仍为 0**。

---

## **浏览器内运行**证据（2026-10-19，本轮实测，headless Chrome）

### 复现步骤

```bash
wasm-pack build crates/engine-native-wasm --target web --release --no-opt --out-dir /tmp/wasm-web
cp /tmp/wasm-web/* /tmp/wasm-browser/          # 见下方最小页面
python3 -m http.server 8899 -d /tmp/wasm-browser &
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --headless=new --disable-gpu --no-sandbox --user-data-dir=/tmp/chrome-prof \
  --virtual-time-budget=15000 --dump-dom http://127.0.0.1:8899/index.html
```

### 最小页面（`index.html`，31 行）

```html
<!doctype html><html><head><meta charset="utf-8"><title>wasm-boot</title></head>
<body><pre id="out">booting</pre>
<script type="module">
const log = (k, v) => { document.getElementById("out").textContent += "\n" + k + "=" + v; };
try {
  const mod = await import("./engine_native_wasm.js");
  await mod.default("./engine_native_wasm_bg.wasm");
  const e = new mod.NativeEngine(4, 4242n);
  log("boot", "ok");
  log("view0", e.view_json(0).slice(0, 24));
  let steps = 0;
  while (!e.outcome_done && steps < 300) { if (!e.auto_step()) break; steps++; }
  log("steps", steps);
  log("RESULT", "PASS");
} catch (err) { document.getElementById("out").textContent += "\nRESULT=FAIL " + err; }
</script></body></html>
```

### 实测输出（headless Chrome）

```text
boot=ok
view0={"deckCount":108,"discar
steps=300
RESULT=PASS
```

### 这证明什么

| 维度 | 覆盖 |
|---|---|
| **浏览器**内加载 wasm（ES module + `default(wasm)` 初始化）| ✅ |
| 浏览器内**建局**（`new NativeEngine(4, 4242n)`）| ✅ |
| 浏览器内**座位视图**（`view_json(0)`）| ✅ |
| 浏览器内**自动对局**（`auto_step` 跑满 300 步上限，无异常）| ✅ |

### 仍未做

- 真实 UI 客户端（`apps/web-client`）+ WebWorker 线程 + 网络层；
- Firefox / Safari 的交叉验证（本轮只跑 Chrome）；
- iOS/Android 真机。

⇒ 「目标平台」现已覆盖：**构建 6/7 target + Node 运行 11/11 + 浏览器运行 4/4**，
但**真机与真实 UI 客户端仍未做** ⇒ **八项完成数仍为 0**。

### 交叉浏览器：**未完成**（本轮试过，环境不支持）

尝试用 headless Firefox 截图取证：

```bash
"/Applications/Firefox.app/Contents/MacOS/firefox" --headless   --screenshot /tmp/ff-shot.png --window-size=900,700 http://127.0.0.1:8899/firefox.html
```

⇒ 进程启动（打印 `*** You are running in headless mode.`）但**没有产出截图文件**，
且该版本 Firefox 不提供 Chrome 那样的 `--dump-dom`。
⇒ **Safari / Firefox 的交叉验证仍为待办**，本轮只证明 **Chrome** 可运行。
