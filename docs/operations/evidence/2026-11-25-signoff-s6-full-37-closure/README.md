# 2026-11-25 Batch S6：全量 37 来源 / 46 正式包仓内可验八项签署全表收口与待用户/外部前置清单

- **前批精确 CI 核验**：Batch S5 提交 `3e68b62187f05ec594522a00c12fc77f4220aa8c` 的精确提交 CI Run [`45636201258`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/45636201258) Attempts 1–4 如实记入 [ci-runs.md](../ci-runs.md)：Attempt 4 **`10/10 success`**（`job_failure_count=0`，`logs-first.zip` `sha256: 670e23152bf007935960029af3051937d52041008e4681d4c2fe05f3063a5a8b`）。（注：Batch S5 证据文档曾沿用历史草稿内 `L08..L24` 单源细分数，[inventory_status.json](../../../migration/g0/inventory_status.json) 精确口径为 `L08=406, L09=102, L10=17, L11=936, L12=6, L24=46`，合仍 `1,513 / 0 / 0`；本收口文档与全表以精确核验数字为准。）
- **版本与门禁不变量**：`STANDARD_RULESET_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`、`CONTENT_SCHEMA_VERSION = 5`、`RULE_VM_ABI_VERSION = 759`、`WS_PROTOCOL_VERSION = 8`、`u01-dddanzhi-pending-v107.ngsa` 与全部 `14` 条 golden 哈希零改动；`inventory_status.json` 保持 **`4,419 / 5 / 0`**（99.89%，缺失 `0`）。

---

## 一、37 来源 / 46 正式包仓内可验签署收口总览

依据 [up37_eight_dim_signoff.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff.rs)（`wave1..wave5_6` 全 37 源 / 46 正式包，实测 **`48 passed; 0 failed`**）及各波次独立差异复核（`195a4733`、`6c2971da`、`2a8e7b2e`、`c323960a`、`15a3bb8a`、`1cf6b3e7`），全表已落入 [37 来源八项签署总表](../2026-10-19-source-signoff-32/README.md) 与 [两仓扩展台账](../../../migration/upstream-extension-reconstruction.md)：

| 分层 | 来源列表 | 包数 | 仓内完成维度 | 仍标「部分」的维度与精确原因 | 整包 8/8 满签计入 |
| --- | --- | ---: | --- | --- | --- |
| **无 Spine 100% 等价（28 源）** | `U02..U04, U06..U08, L01..L07, L13..L23, L25..L27, L29` | 29 | **`7/8 ✅`**（①②③④⑤⑦⑧） | 第 ⑥ 维（四端仓内编译/桌面构建已过，仅缺 Android/iOS 物理真机实测与三平台商业签名公证） | `0`（待 ⑥） |
| **含 Spine 4.0 100% 等价（6 源）** | `L08 (406/0/0), L09 (102/0/0), L10 (17/0/0), L11 (936/0/0), L12 (6/0/0), L24 (46/0/0)` | 6 | **`6/8 ✅`**（①②③④⑤⑦） | 第 ⑥ 维（同上）＋第 ⑧ 维（Web 净室静态替代外部 Spine 4.0，待用户本机自有动皮画面对照） | `0`（待 ⑥⑧） |
| **Decision 40 冻结边界（3 源）** | `U01 (94/1/0, 98.95%)`, `L28 (55/1/0, 98.21%)`, `U05 (1,618/3/0, 99.81%)` | 11 | **`5/8 ✅`**（②③④⑤⑦） | 第 ①、⑥、⑧ 维（各源精确冻结边界如下；另 `U01` 不含 Spine 仅用 CSS 3D，`L28` 兼含 Spine 4.0 对照） | `0`（待 ①⑥⑧） |
| **合计（Batch S6）** | **37 来源（`Wave 1–6` 全覆盖）** | **46 / 46** | **37/37 仓内分层收口** | 整包八项满签严格保持 **`0/37`**（不得在真机、Spine 对照与用户拍板前虚标满签） | **`0 / 37`** |

---

## 二、待用户裁决与外部实测前置汇总（接续 Queue §4 第 8–9 项）

详细执行命令与脚本见 [Queue 8–9 外部前提与切换清单](../2026-11-22-queue8-9-external-prereqs-and-phase6/README.md)；全部 5 项冻结边界及外部依赖集中如下：

1. **5 项 Decision 40 冻结边界升版本/口径裁决（`4,419 / 5 / 0`）**：
   - **`U01:dddanzhi`**：是否维持 `NGSA 107` 挂起夹具 `u01-dddanzhi-pending-v107.ngsa`（`Prepare` + `(Normal, 1, false)`）为已登记边界，或授权升版本改 `phaseBegin` 等价（见 [规则基准 §U01-3](../../../architecture/native-rules-baseline.md#L19-L21)）。
   - **`L28:th_zhengjing`**：确认接受裁决 40 `ChoiceSpec`（`0..=5` 抽象＋低风险约束）为等价或已登记差异，或另批 ABI/规则升版接入逐帧实时小游戏。
   - **`U05:old_longfan`**：确认维持固定选数 `ChoiceSpec` 替代上游 `200ms setInterval` 实时 DOM 连点为已登记差异，或另案设计实时输入回传协议。
   - **`U05:shixu` & `U05:hs_nozdormu`**（S1 复核结论）：non-clock `is_chanyuan_sealed`（`locked: true`）已在 S1 修齐；仅余墙钟 `get.time() - time > 8000` 边界。请择一：**选项 A**（不升 `WS 8`/`Ruleset 333`/`NGSA 107`，接受 `PhaseEnded.elapsed_ms` 确定性代理为已登记差异）；或 **选项 B**（授权升 `WS 8->9`、`Ruleset 333->334` 及〔若独立字段〕`NGSA 107->108` 以将联机真钟计时录入 WAL）。
2. **7 个 Spine 4.0 来源（`L08, L09, L10, L11, L12, L24, L28`）本机对照（裁决 42）**：按 [外部前提 §二](../2026-11-22-queue8-9-external-prereqs-and-phase6/README.md#L58-L89) 使用用户本机自有 `.json/.skel + .atlas + .png` 三件套对比旧版并回传截图；确认后可解除第 ⑧ 维限制（其中前 6 源转 `7/8 ✅`）。
3. **Android/iOS 物理真机（`PA08`/`PA11`/`NC12`）与三平台商业签名/公证**：接入物理设备跑满 `web-frame-external.mjs` ≥30 分钟采样并配置 Apple/Windows/Linux 仓库签名凭据，解除全部 37 源第 ⑥ 维限制（无 Spine 28 源即可升为整包 `8/8 ✅`）。
4. **第六阶段（Phase 6）权威切换范围确认**：确认 [外部前提 §三](../2026-11-22-queue8-9-external-prereqs-and-phase6/README.md#L92-L125) 所列 `engine-core`/`engine-ai`/`engine-wasm` 退场及 `conformance`/`benchmark-runner`/`fuzz`/`room_native_actor` 解耦迁移顺序后执行切换。
