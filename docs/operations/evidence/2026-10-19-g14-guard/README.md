# G14 守卫：`phaseBegin` 被系统性挂到 `Phase::Judge`（**9 条**，2026-10-19）

> **可复现**：`python3 -B scripts/report_g14.py`（`--tsv` 输出本文的 `g14.tsv`）。
> **链路**：`filtering*.rs` 的 filter → `keys.rs` 的 `host:` 键 → 包数据 `skills[].mapping`
> → 上游 `skill.js` / `mode/*.js` 的 `trigger`。**8 条自动查不到，靠人工已核表补齐**（见脚本 `MANUAL`）。

## 1 全表（**14 条** `Phase::Judge, begin=true`）

| 判定 | 条数 | 说明 |
|---|---:|---|
| **G14（不符）** | **9** | **上游 `phaseBegin`（任意阶段开始）⇒ 原生 `Phase::Judge`（判定阶段）** |
| **不符（准备阶段）** | **2** | `swd_xiuluo` / `hlss_fengqi` —— **上游含 `phaseZhunbeiBegin`，而引擎 `Phase` 无 `Prepare`**（G18） |
| **不符（凭空新增触发）** | **2** | `baiyi`（上游 `enable` 主动技、**无 trigger**）、`lexue`（上游 **35 B 无 trigger**）—— **原生却挂了判定阶段触发** ❗ |
| **不符（非阶段类）** | **1** | `huajin2` —— 上游 `damageBegin`；原生 filter **两个分支**：`Damage(source==owner)` ✅ 正确 + `Phase::Judge` 是**自加的「自清」分支** ❗ |

## 2 G14 的 9 条

```
hyunshen2  daixing2  zhenjiu2  huanxing  qianfang
diyguhuo   yougeng   xuelunyang  geju
```

**注意**：本会话早前记为 **8 条**，本脚本重算为 **9 条** —— 差的一条是 **`hyunshen2`**
（早前列表里被 `lexue`/`huajin2` 两个「非阶段类」占位挤掉）。

## 3 修法（不改 `event.rs`）

**照 `SwdJinlin2` 的现成写法**（`filtering/swd.rs:73`）：

```rust
EventData::Phase { player, begin: true, .. } if *player == owner   // `..` 省略 phase ⇒ 任意阶段开始
```

⇒ **9 个 filter 各改一个 pattern**，**不动 `event.rs` / `kernel/**`**。
**范本已核**：上游 `jinlin2` 的 `trigger` 也是 `phaseBegin`（`SwdJinlin2` 做对了）。

## 4 边界

- **本表只证明「映射不符」**，**不证明「修完就等价」** —— 修完仍需逐技能行为验收；
- **2 条准备阶段（G18）不在 G14 内** —— 它们要扩 `Phase` 枚举，属**设计**而非**对齐**；
- **2 条「凭空新增触发」（`baiyi`/`lexue`）** 与 **1 条「多挂一个分支」（`huajin2`）** 是**另一类问题**：
  **不是映射错，而是多了一个上游没有的触发** —— 修法是**删掉该分支**，**不是改 pattern** ⚠️

## 5 修法分类（决定了它属于哪一批）

| 类 | 条数 | 修法 | 批次 |
|---|---:|---|---|
| **`phaseBegin` ⇒ `Judge`（G14）** | **9** | **改 pattern**（照 `SwdJinlin2`）| **批 1（对齐）** ✅ |
| **准备阶段（G18）** | **2** | **扩 `Phase` 枚举** | **批 3（设计）** ⚠️ |
| **凭空新增触发** | **2** | **删掉该 filter 分支** | **批 1（对齐）** ✅ |
| **多挂一个分支（`huajin2`）** | **1** | **删掉 `Phase::Judge` 分支** | **批 1（对齐）** ✅ |

⇒ **批 1 从「9 行」变成「12 处」**（**9 改 pattern + 3 删分支**），**仍是「对齐」类** ✅

---

## 6 **全 52 条的机器对照**（第 649 轮，`all52.tsv`）

用同一链路把**全部 52 处**都回查上游，判定分布：

| 判定 | 条数 | 说明 |
|---|---:|---|
| **相符（待逐条确认）** | **21** | 上游 `phaseEnd`/`phaseUseBegin`/`phaseDrawBegin` 等与原生一致 |
| **未定位** | **13** | `Gwent*` 3（不可达）、`gujian_food_*` 3（卡牌宿主）、`Xiefang`/`Xiongzi`/`Miedao`/`zaibian`/`PhaseEndRecoverOrDraw`/`GujianFoodTanhuadong` 等 |
| **G14（`phaseBegin` ⇒ `Judge`）** | **9** | 见前文 |
| **G15（时长/档位不符）** | **4** | `fuchou2`（`phaseAfter`）、`rumeng`/`fenglue`（`phaseUseBefore`）、`reyingzi`（`phaseDrawBegin2`）|
| **G18（准备阶段）** | **2** | `swd_xiuluo`、`hlss_fengqi` |
| **凭空新增触发** | **2** | `baiyi`、`lexue` |
| **非阶段类** | **1** | `huajin2`（`damageBegin`）|
| **合计** | **52** | 4+2+21+13+9+2+1 = 52 ✅ |

### 分组对照

| Phase 组 | 条数 | 有上游 | 未定位 |
|---|---:|---:|---:|
| **End** | 21 | **16** | 4（+1 卡牌类）|
| **Judge** | 14 | **12** | 2 |
| **Play** | 8 | 5 | 3 |
| **Draw** | 7 | 3 | 4 |
| **Discard** | 2 | 0 | 2 |

### 与手工审计的关系

- **手工审计（`registry.txt`）记「已核 46 / 不适用 5 / 未核 1」**；
- **脚本记「有结论 36 / 未定位 13」**；
- **两者不矛盾**：**「不适用」在脚本里落入「未定位」**（**卡牌宿主与不可达都没有上游 `trigger` 可查**）⇒
  **脚本的「未定位 13」≈ 手工的「不适用 5 + 不可达 3 + 其他 5」** ✅
- **但脚本多认了 `fuchou2` 的 `phaseAfter`** ⇒ **属 G15**（**手工此前归「需判定」** ✓，**现确认**）

---

## 7 **根因：注释写「准备阶段」，代码写 `Phase::Judge`**（第 650 轮，读臂体后发现）

`filtering/*.rs` 里有一批**臂前注释明确写「自己准备阶段开始」**，但**代码用的是 `Phase::Judge`**：

| 文件:行 | 注释 | 行为 | 代码 | 上游 |
|---|---|---|---|---|
| `common.rs:216` | **修罗：自己准备阶段开始** | `swd_xiuluo` | `Phase::Judge, begin=true` | **`phaseZhunbeiBegin`** |
| `swd.rs:24` | **魂身·收束：自己准备阶段开始** | `hyunshen2` | 同上 | **`phaseBegin`** |
| `swd.rs:34` | **待行·清甲：自己准备阶段开始** | `daixing2` | 同上 | **`phaseBegin`** |
| `swd.rs:673` | **枕流触发面：自己准备阶段开始** | `zhenjiu2` | 同上 | **`phaseBegin`** |
| `swd.rs:699` | **幻形：自己准备阶段开始** | `huanxing` | 同上 | **`phaseBegin`** |

（`grep -c 准备阶段` 在 `filtering*.rs` 里共 **19 处**）

### 这说明什么

**作者把「准备阶段」实现成了「判定阶段」** ——
很可能因为 **`Phase` 枚举里没有 `Prepare`**（G18）⇒ **只好借用相邻的 `Judge`**。

### 但修法**不变**，且更清楚

| 情形 | 上游 | 修法 |
|---|---|---|
| **上游是 `phaseBegin`（任意阶段）** | 9 条中的 7 条 | **改成 `..`（任意阶段）** ✅ —— **注释也要改成「任意阶段开始」** ⚠️ |
| **上游是 `phaseZhunbeiBegin`（准备阶段）** | `swd_xiuluo` 1 条 | **必须扩 `Phase` 加 `Prepare`**（G18）⚠️ |

⇒ **G14 的「照 `SwdJinlin2` 改 `..`」仍然正确** ✅
⇒ **但修复时必须同时改注释**，否则下一个读者会以为「这里应该是准备阶段」而改回去 ⚠️

### 反过来看 G18

**`swd_xiuluo` 的注释是「准备阶段」、上游也是「准备阶段」，代码却是 `Judge`** ——
**它证明「引 `Judge`」是「没有 `Prepare`」的妥协**，**不是笔误** ✅
