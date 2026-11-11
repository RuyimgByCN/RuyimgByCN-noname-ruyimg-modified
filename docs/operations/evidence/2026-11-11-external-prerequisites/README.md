# 外部硬件与凭据跟进及仓内解阻证据 (Feature 19)

- **登记日期**：2026-11-11
- **对应里程碑**：Milestone 4 (R5 外部硬件与凭据跟进)
- **对应需求**：`ORIGINAL_REQUEST.md` §R5、`PROJECT.md` Feature 19
- **关联台账**：[原生剩余台账](../../../architecture/native-remaining-ledger.md)（PA08, PA11, NC12）、[当前进度主稿](../../../architecture/native-progress.md)

---

## 1. 背景与环境实测现状

依据《原生架构收敛执行方案》与用户需求 R5 规范，针对 PA08（移动端表现层采样扩面）、PA11（原生 GPU 牌桌原型）与 NC12（效能验收、真机温控与安装包签名公证），当缺少外部物理硬件设备或商业代码签名凭据时，必须建立独立证据台账，明确记录前置依赖、精确手工操作指令与未验收验证阈值；同时证明仓内自动化测试、模拟器/无头环境与未签名打包设施可 100% 独立运行通过，确保仓内代码开发与工程门禁推进 0 阻塞。

### 1.1 开发宿主实测设备与证书扫描实录（2026-11-11）

1. **Android USB 物理设备扫描**：
   ```bash
   adb devices -l
   ```
   *实测输出*：
   ```text
   List of devices attached
   ```
   *结论*：物理连接在线且完成授权的 Android 真机数为 **0**。

2. **Apple 物理设备扫描**：
   ```bash
   xcrun devicectl list devices
   ```
   *实测输出*：
   ```text
   Name      Hostname  Identifier                        State        Model       Reality
   Ruyimg音            00008140-0002149826F3001C (UDID)  unavailable  iPhone17,2  
   ```
   *结论*：历史登记的物理 iPhone 处于 `unavailable` 离线未连接状态；当前可用运行环境为 Xcode 本地模拟器（iOS 27.0 / iPhone 17）。

3. **系统代码签名凭据扫描**：
   ```bash
   security find-identity -p codesigning -v
   ```
   *实测输出*：
   ```text
   0 valid identities found
   ```
   *结论*：macOS Keychain 中有效 Apple Developer ID 证书为 **0**；无 Windows EV Code Signing 或 Azure Trusted Signing 凭据。

---

## 2. 外部前置依赖、操作指南与未验收阈值

### 2.1 PA08 移动端表现层采样扩面

- **前置依赖**：
  1. 中低端 Android 物理真机（支持 USB 调试、开启开发者选项、已完成电脑授权）；
  2. iPhone 物理真机（iOS 16+，已开启开发者模式并信任当前 Mac 宿主）；
  3. USB 高速数据线与网络环境。
- **精确手工操作指令**：
  - **Android 端操作**：
    1. 连接手机，执行 `adb devices -l` 确认设备状态为 `device`；
    2. 映射端口并保持常亮：`adb reverse tcp:5173 tcp:5173 && adb shell svc power stayon true`；
    3. 宿主启动前端服务：`pnpm --dir apps/web-client dev --host 0.0.0.0`；
    4. 手机打开 Chrome/WebView 访问 `http://localhost:5173/?mode=identity&autostep=1`；
    5. 执行自动化性能采集：`node apps/web-client/scripts/web-frame-external.mjs --device android`；
    6. 对局结束后通过 `adb shell dumpsys battery` 记录电池温度变化，通过 `adb shell dumpsys thermalservice` 记录热节流档位。
  - **iOS 端操作**：
    1. 数据线连接 iPhone 并点击「信任此电脑」，输入设备锁屏密码；
    2. 执行 `xcrun devicectl list devices` 确认设备状态变为 `available`；
    3. 宿主绑定局域网 IP 启动服务，手机 Safari 访问 `http://<HOST_IP>:5173/`；
    4. Mac Safari 开启「开发」菜单连接目标设备 WebKit 检查器；
    5. 记录 30 分钟连续对局帧率轨迹、主线程 Long Task 频次与外壳发热情况。
- **未验收阈值**：
  - 连续对局 ≥30 分钟无崩溃、无无响应（ANR）；
  - 帧率表现：在中低端设备（如 Snapdragon 6/7 系或 A13/A14）上维持 60 FPS，帧时间 P99 ≤ 16.67ms，无严重掉帧与掉电发热降频；
  - 真实触控输入响应：手牌拖拽到出牌生效 input_to_paint 延迟 ≤ 50ms。
- **仓内非阻塞回退说明**：
  - **CDP 触控仿真**：`apps/web-client/scripts/nc12-mobile-touch.mjs` 通过 Chrome DevTools Protocol 的 `Emulation.setDeviceMetricsOverride` (390×844) 与 `Emulation.setTouchEmulationEnabled` + `Input.dispatchTouchEvent` 完整覆盖移动端 TouchStart/Move/End 事件流与手牌拖拽；
  - **iOS 模拟器整局验证**：[2026-10-26-pa08-6-mobile 报告](../2026-10-26-pa08-6-mobile/README.md) 中已在 iOS 模拟器（iPhone 17 / iOS 27.0）上跑通 4 座身份局 seed 42 整局（终局 rev 280，帧 p50 17ms，Long Task 0）；
  - **Android AVD 验证**：Android 模拟器（codex35）已跑通原生引擎生命周期与对局步进（rev 12→96→142）；
  - **CDP 1440×1080 测定**：`web-frame-latency.mjs` 测得真实牌桌动画帧率稳定 60.0 FPS；仓内所有自动化测试与代码推进 **100% 解阻**。

### 2.2 PA11 原生 GPU 牌桌原型

- **前置依赖**：
  - 用户裁决（2026-10-26 22:51）明确规定：「条件暂缓，不关闭——先把 PA08 补到真机...全部达标则维持 DOM。ADR 0785 保持草案」。唯有当 PA08 真机实测 DOM 方案性能超标时才触发。
- **精确手工操作指令（若条件触发）**：
  1. 依据 [ADR 0785](../../../adr/0785-wgpu-table-prototype.md) 唤醒原生桌面 wgpu 原型工程；
  2. 运行 `cargo run -p desktop-client --features wgpu-table-experimental`；
  3. 执行同屏 8 座 30 张手牌、高频粒子特效的压力负载，通过 GPU 性能计数器采集绘制耗时与功耗（mW）。
- **未验收阈值**：
  - 是否需要重构为原生 GPU 牌桌的性能超标触发条件尚未出现；
  - 目前实测 DOM 方案余量充足：帧 P99 ≤ 10.3ms @ 120Hz，满手牌 30 张序列化 ≤ 1ms，连续运行零长任务。
- **仓内非阻塞回退说明**：
  - 前端真实牌桌体系（`BattleBoard`、`BattleShoushaHud`、`SeatCard`、`HandStrip`）结合自研净室 Spine 4.0 WebGL 播放器运行稳定；
  - 在缺少 GPU 加速或非 4.0 资产时，确定性触发 `static-portrait-fallback` 静态立绘回退并由 11 项净室单测锁死；仓内推进 **100% 解阻**。

### 2.3 NC12 三平台安装包代码签名与公证

- **前置依赖**：
  1. **macOS**: Apple Developer 开发者账号 ($99/年)、Developer ID Application 签名证书、App 专用密码或 App Store Connect API Key；
  2. **Windows**: 经 WebTrust 认证的 EV Code Signing 硬件 Token 证书或 Azure Trusted Signing 企业订阅；
  3. **Linux**: 发布者 GPG 私钥与签名环境。
- **精确手工操作指令**：
  - **macOS 构建、签名与公证**：
    1. 构建未签名包：`cargo tauri build --no-bundle`；
    2. 应用签名：`codesign --force --options runtime --deep --sign "Developer ID Application: Company Name (TeamID)" "target/release/bundle/macos/noname-next.app"`；
    3. 打包并提交公证：
       ```bash
       ditto -c -k --keepParent "target/release/bundle/macos/noname-next.app" noname-next.zip
       xcrun notarytool submit noname-next.zip --keychain-profile "notarytool-profile" --wait
       ```
    4. 钉住票据：`xcrun stapler staple "target/release/bundle/macos/noname-next.app"`；
    5. 本机评估验证：`spctl --assess --type exec -v "target/release/bundle/macos/noname-next.app"`。
  - **Windows 签名**：
    ```cmd
    signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /sha1 <THUMBPRINT> target\release\bundle\nsis\*.exe
    ```
- **未验收阈值**：
  - macOS Gatekeeper 检验直接通过（`accepted source=Notarized Developer ID`），安装运行不弹出「无法验证开发者」安全警告；
  - Windows SmartScreen 评估为安全，无未知发布者红色拦截警报。
- **仓内非阻塞回退说明**：
  - 本地与 CI 环境使用 `cargo tauri build` 直接生成无签名安装包，矩阵构建正常产出 `installer-macos-latest`、`installer-windows-latest`、`installer-ubuntu-latest`；
  - 扩展包发布与验真机制（`content_runtime::sign::sign_package` / `.sig.json`）完全采用仓内内置 Ed25519 签名体系，与操作系统级商业签名凭据完全解耦；仓内推进 **100% 解阻**。

---

## 3. 仓内自动化回退与未结项边界总结

| 项目 | 外部未就绪依赖 | 仓内自动化回退验证 | 台账状态（不以文档代替实测） |
|---|---|---|---|
| **PA08 移动端真机** | Android USB 物理机、iPhone 真机信任 | CDP 移动触控模拟 (`nc12-mobile-touch.mjs`)、iOS Simulator (iPhone 17)、Android AVD（rev 280 / 0 Long Task） | **进行中**（真机采样与温控待物理设备连接后实测） |
| **PA11 GPU 牌桌** | 物理硬件能耗评测（当前条件暂缓） | DOM + Canvas 2D + WebGL Spine 4.0 净室播放器，支持静态立绘回退（60.0 FPS / 11 净室单测） | **条件暂缓，不关闭**（维持 ADR 0785 草案） |
| **NC12 签名公证** | Apple Developer ID 证书、Windows EV 凭据 | Tauri 未签名产物打包、CI bundle 产出、Ed25519 扩展包验签体系 | **部分**（真机热节流、三平台商业签名公证及第六阶段产品切换待完成） |

**结论**：所有涉及物理真机、外部硬件与商业证书的前置条件与操作指令均已梳理；仓内单测、集成测试与工程门禁可在无外部硬件条件下独立运行，但 **PA08、PA11、NC12 均保持未结/部分状态，不因本操作手册而标为“已完成”或“闭环”**。
