# 「目标平台」**构建级**验收（2026-10-19）

- 工作包：**NC08 / 八项第 6 项**；规则版本 v322（工作树）/ 存档格式 v105
- 原始输出：[raw.txt](raw.txt)（含 `rustc --version`、`rustup target list --installed`）
- 设备/工具链：Apple M3 Max / macOS 27.0.1 / **rustc 1.95.0**

## 1 结果

| 目标 | crate | 结果 | 说明 |
|---|---|---|---|
| **`aarch64-apple-ios`** | `engine-native` | ✅ **OK** | iOS 真机架构 |
| **`x86_64-apple-ios`** | `engine-native` | ✅ **OK** | iOS 模拟器架构 |
| **`wasm32-unknown-unknown`** | `engine-native` | ✅ **OK** | 浏览器/边缘 |
| **`wasm32-unknown-unknown`** | `engine-native-wasm` | ✅ **OK** | WASM 交付层 |
| **`wasm32-unknown-unknown`** | `content-native-loader` | ✅ **OK** | 扩展装载器可入 WASM |
| `aarch64-linux-android` | `engine-native` | ❌ **FAIL** | `failed to find tool "aarch64-linux-android-clang"`（**NDK 未安装**）|
| `x86_64-pc-windows-msvc` | `engine-native` | ❌ **FAIL** | `blake3` 构建脚本需 **MSVC C 工具链**（未安装）|

⇒ **5 项通过 / 2 项因缺 C 工具链失败**（**环境限制，非代码缺陷**）。

## 2 结论（**如实**）

| 判据 | 状态 |
|---|---|
| **构建级**跨平台（iOS 双架构 / WASM 三层） | ✅ **本文件** |
| **真机运行**（安装、启动、对局） | ❌ **无设备**，**未做** |
| 浏览器端实际运行（WebGPU/WebGL、帧率） | ❌ **未做** |
| Android / Windows 构建 | ❌ **缺 NDK / MSVC C 工具链**（`blake3` 走 C 实现，需要 clang-cl）|

⇒ 八项第 6 项由「**0 证据 / 无设备**」推进为「**构建级 5 项通过（iOS 双架构 + WASM 三层）**」，
**真机运行与浏览器运行仍未验收**。

## 3 复现

```bash
cd noname-next
cargo build --offline --lib -p engine-native --target aarch64-apple-ios
cargo build --offline --lib -p engine-native --target x86_64-apple-ios
cargo build --offline --lib -p engine-native --target wasm32-unknown-unknown
cargo build --offline --lib -p engine-native-wasm --target wasm32-unknown-unknown
cargo build --offline --lib -p content-native-loader --target wasm32-unknown-unknown
```

## 4 若要继续（**需要外部资源**）

1. **Android**：安装 NDK 并设置 `CC_aarch64-linux-android` / `AR_*`（或 `cargo-ndk`）；
2. **Windows**：安装 Visual Studio Build Tools（`cl.exe` / `clang-cl`）；
3. **真机**：需要 iOS/Android 设备与签名凭据；**本会话无设备** ⇒ 保持待办。
