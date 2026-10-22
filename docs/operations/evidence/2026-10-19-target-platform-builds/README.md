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
