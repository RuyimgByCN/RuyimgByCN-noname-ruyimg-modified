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
