# NC08 第⑦层样本 41：来源 `longxue`（龙血玄黄 / D5）—— **弱对照**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**，但**证据强度 = 弱对照**（**不可与其它样本同等记账**）
- **来源包**：`packages/extensions/longxue-upstream-liuli`（1 将 / 2 技映射）

## 1 **证据强度声明（必读）**

上游 `noname-liuli/noname/extension/龙血玄黄/extension.js` **被混淆**（`jsjiami.com.v6`）——
**连技能说明文本也不可读**（`mbqianlong_info` 在整目录内**无明文命中**，见
[obfuscation-proof.txt](obfuscation-proof.txt)）→ **无法**从上游源码逐字引用语义。

本样本的对照依据是：**原生注释中记录的上游键名与语义**（`content/host/behavior.rs:1279–1285`、
`state.rs:256–262`）+ **包数据映射**。→ 这**只能**证明「原生实现与其**自述**的上游口径一致、
映射齐备」，**不能**证明与上游行为等价。**台账/进度按此强度记账。**

## 2 验收结果（`nc08_upstream_longxue.rs`，3 项）

| # | 依据 | 判据 | 结果 |
|---|---|---|---|
| — | 上游混淆事实 | `extension.js` 含 `jsjiami` 且**无** `mbqianlong_info` 明文 → 固定「弱对照」前提 | 通过 |
| — | 包映射 | 1 将 / 2 技（`mbqianlong` + `mbweitong`），均映射 `host:` 行为键 | 通过 |
| — | 原生注释所载口径 | 开局 **20 点**（魏势力持卫统者 **60 点**）、**上限 99**、阈值 **25/50/75/99** 解锁清正/酒诗/放逐/绝进；卫统 +60 | 通过 |

**原生实现引用**：`content/host/behavior.rs:1279`（`MbQianlong`/`MbWeitong`）、
`state.rs:259`（`mbqianlong_marks`）、`content/host/constants.rs:804`（键名）。

## 3 解除弱对照的前置（待办）

1. **无头轨迹采集**（台账 **D5** 项既定路径）：在 Node 中驱动归档 JS 得到**运行期行为序列**；
2. 以该序列替换本样本的对照依据 → 升级为**上游行为级**对照；
3. 在此之前，该来源**不应**计入「上游规则等价」的通过数。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **无头轨迹采集**（D5）→ 升级为行为级对照
- [ ] 混淆来源是否还有其它（`jsjiami` 全库扫描）——建议一次性盘清
- [ ] 样本 42+：其余来源（当前 **23/37**，其中 **1 个为弱对照**）
