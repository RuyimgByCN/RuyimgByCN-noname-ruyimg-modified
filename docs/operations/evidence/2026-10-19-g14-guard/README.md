# G14 守卫：`phaseBegin` 被系统性挂到 `Phase::Judge`（**9 条**，2026-10-19）

> **可复现**：`python3 -B scripts/report_g14.py`（`--tsv` 输出本文的 `g14.tsv`）。
> **链路**：`filtering*.rs` 的 filter → `keys.rs` 的 `host:` 键 → 包数据 `skills[].mapping`
> → 上游 `skill.js` / `mode/*.js` 的 `trigger`。**8 条自动查不到，靠人工已核表补齐**（见脚本 `MANUAL`）。

## 1 全表（**14 条** `Phase::Judge, begin=true`）

| 判定 | 条数 | 说明 |
|---|---:|---|
| **G14（不符）** | **9** | **上游 `phaseBegin`（任意阶段开始）⇒ 原生 `Phase::Judge`（判定阶段）** |
| **不符（准备阶段）** | **2** | `swd_xiuluo` / `hlss_fengqi` —— **上游含 `phaseZhunbeiBegin`，而引擎 `Phase` 无 `Prepare`**（G18） |
| **其他（非阶段类）** | **3** | `baiyi`（`enable` 主动技）、`lexue`（无 trigger）、`huajin2`（`damageBegin`）—— **它们根本不是阶段触发** |

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
- **3 条非阶段类**应从 G14 讨论中剔除（**它们被挂到 `Judge` 本身是另一个问题**）。
