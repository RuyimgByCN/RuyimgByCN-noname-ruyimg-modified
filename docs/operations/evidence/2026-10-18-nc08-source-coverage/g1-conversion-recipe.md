# G1 内容转换**配方**（2026-10-19，从真实转换包反推并核对）

- 用途：把 **B 类**（待重构）来源从「包内 0 卡 0 将」推进到「可加载 + 可对照」
- 样例包：`packages/extensions/huanle-upstream-noname`（**U06 欢乐卡牌**，已转换、八项已验收）

## 1 目录结构（实测）

```
packages/extensions/<pkg-name>/
  manifest.json          { "schema_version": 1, "enabled": true, "package": "package.npack.json" }
  package.npack.json     包描述（由 manifest 指向）
  assets/cards.json      内容数据（见下）
```

## 2 `assets/cards.json` 结构（实测样例，逐字段）

```json
{
  "source": "noname",
  "upstream_path": "apps/core/extension/欢乐卡牌/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "huanle-upstream-noname", "version": 1,
            "description": "G1 欢乐卡牌", "card_prefix": "hnl_" },
  "cards": [ { "name": "kuwu", "kind": "weapon", "range": 2,
               "equip_skills": ["huanle_kuwu"], "suit": "spade", "rank": 6 } ],
  "skills": [ { "name": "huanle_kuwu", "mapping": "host:sha_damage_force_discard" },
              { "name": "huanle_jiuwei", "mapping": "host:phase_end_recover_or_draw" } ]
}
```

## 3 转换步骤（可机械执行的部分）

1. **建目录**：`packages/extensions/<pkg>/`，写 `manifest.json`（照抄上表字段）；
2. **登记上游**：`source` / `upstream_path` / `upstream_commit`（取归档快照的提交号）；
3. **抽内容**：从上游 JS 抽出武将（`lib.character`）、卡牌（`lib.card`）、技能（`lib.skill`）
   → 写入 `generals[]` / `cards[]` / `skills[]`；
4. **映射技能**：每条技能写 `mapping`（形如 `host:<key>` 或规则 VM 表达式）——
   **前置条件**：该 `host:<key>` **必须已存在于** `crates/engine-native/src/vocabulary/host/keys.rs`
   （`HostBehavior` 枚举 + `as_key()`），否则该技能**无法**在原生侧生效；
5. **验收**：包可发现 + 加载 + 行为对照（第⑦层样本）→ 才计入对应指标。

## 4 **关键前置**（决定 B 类能否批量推进）

| 前置 | 现状 | 影响 |
|---|---|---|
| 宿主行为键（`host:<key>`）已实现 | 已有一批（`vocabulary/host/keys.rs`） | **键不存在**的技能需先补实现（属 **A 类**工作） |
| 包发现/加载链路 | 已有（41 个包目录） | B 类可直接复用 |
| 上游可读 | 除 **L12/L29**（`jsjiami`）外均可读 | L12/L29 只能**弱对照** |

→ **结论**：B 类的**机械部分**（1–3 步）可批量做；**卡点在步骤 4**——凡上游技能在
`vocabulary/host/keys.rs` 中**没有对应键**，就必须先做实现，**不能**靠「写进 cards.json」蒙混过关。

## 4b 步骤 4 的自动化尝试：**同名比对不可用**（2026-10-19）

实测 5 个 B 类来源：上游技能名与原生 `host:<key>` **命名空间不同** → 同名命中**全部为 0**
（详见 [README §1g](README.md) 与 [host-key-inventory.txt](host-key-inventory.txt)）。
→ 步骤 4 **必须逐条语义映射**；同时得到规模数据（U08≈172 键、L20≈151、L27≈45、L10≈37、L19≈16）。

## 5 待办

- [ ] 对 14 个 B 类来源，逐个产出「技能 → 宿主键」**存在性清单**（决定各自是纯转换还是转换+实现）
- [ ] 该清单产出前，**不**声称任何 B 类来源「可批量转换」
