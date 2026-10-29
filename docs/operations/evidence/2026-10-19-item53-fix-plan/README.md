# 第 53 项修复方案：趁火打劫 `damageEnd` 窗口（2026-10-19）

- 工作包：**NC08 / 修复清单第 53 项**；规则版本 v322（工作树）/ 存档格式 v105
- 状态：**方案完成，实施受阻**（见 §4）

## 1 根因（已确证）

| 面 | 事实 | 证据 |
|---|---|---|
| **触发器** | `HostBehavior::ChenhuodajieCard` 已声明（`vocabulary/host/keys.rs:327` → `"host:chenhuodajie_card"`） | `vocabulary/host/keys.rs:327`、`vocabulary/host/behavior_list.rs:19` |
| **过滤** | `filter_behavior`（`filtering.rs:473`）**委托** `filter_common`；后者对它是 **`=> false`** | `filtering.rs:473`、`filtering/common.rs:548` |
| **挂载** | `packages/extensions/shahai-yunchou-upstream-noname/assets/cards.json` 的 **`skills` 为 `null`** ⇒ **无任何宿主技声明** | 包数据 |
| **行为面** | 他人受伤（hp 4→3 已确认）后**无窗口**；**受伤者本人持有亦无窗口** | 样本 152 + 387 轮对照 |

⇒ **两处都不可缺**：①过滤器恒 `false`；②包数据**未声明**该宿主技。

## 2 上游依据（逐字）

- `.../yunchou/skill.js` 的 `g_chenhuodajie`：**`trigger: { global: "damageEnd" }`**；
- `.../yunchou/card.js` 的 `chenhuodajie`：`type: "trick"`、`filterTarget: true`、`global: "g_chenhuodajie"`；
  `content` 为 `if (target.countCards("he")) player.gainPlayerCard("he", target, true)`。

## 3 修复步骤（**精确到文件与位置**）

| # | 文件 | 改动 | 说明 |
|---|---|---|---|
| 1 | `crates/engine-native-content/src/content/host/filtering/common.rs`（`ChenhuodajieCard` 臂） | 把 `=> false` 改为 **`host:damageEnd` 语义**：`EventData::Damage{ begin:false, .. }` 且 **持有者存活** | 上游为 `{ global: "damageEnd" }` ⇒ **任意角色**受伤结束 |
| 2 | `packages/extensions/shahai-yunchou-upstream-noname/assets/cards.json` | 新增 `skills: [{"name": "chenhuodajie_window", "mapping": "host:chenhuodajie_card"}]` | 当前为 `null`；**无声明则行为永不实例化** |
| 3 | `crates/engine-native-content/src/content/host/step/*.rs`（`ChenhuodajieCard` 实现臂） | 补**目标选择**：窗口内需选**受伤者**为 `he` 来源 | 上游 `filterTarget: true` + `gainPlayerCard(target)` |

**注意**：步骤 1 的文件属**未跟踪目录**（并行拆分）⇒ **实施受阻**（见 §4）；
步骤 2 属**已跟踪**包数据，**单独实施不完整**（仍不会触发）⇒ **与步骤 1 同步提交**。

## 4 阻塞说明（**如实**）

`crates/engine-native-content/src/content/host/**` 在当前工作区**未跟踪**（并行拆分未提交）⇒
按本会话一贯纪律，**不在未跟踪文件上改动**；因此第 53 项**保持待办**。

**解除条件**：拆分提交落地后，按 §3 三步实施，并用样本 152 的 `#[ignore]` 用例验收。

## 5 验收

样本 152（`nc08_upstream_yunchou_chenhuodajie_window.rs`）的 `#[ignore]` 用例应：
**他人受伤后**打开窗口 → 使用趁火打劫 → **获得受伤者一张 `he` 牌**（1 号 `he` 2 → 1、0 号手牌数不变）。

## 6 同类影响（**参考**）

`filter_common` 的其他 `=> false` 项**不应**照此批量放开：
本会话已定性「**`=> false` 块整体为良性占位，真正的漏接仅此 1 项**」
（见 [触发面清单](../2026-10-19-host-behavior-false-block/README.md)）。
