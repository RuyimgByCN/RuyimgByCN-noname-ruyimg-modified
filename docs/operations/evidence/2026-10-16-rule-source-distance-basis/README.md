# 2026-10-16 规则来源明确：采用玩法版本与距离语义基准（覆盖清单两行收口）

核验日期：2026-10-16。**本批只登记规则来源与语义基准，不改代码、不改规则/存档版本、
不重录 golden**（规则 v290 / 存档 v77 不变）。

## 1. 为什么有这一批

覆盖清单行为差异表两行的处置都停在「规则来源/玩法版本未明确」：

| 原行 | 差异 | 原处置 |
|---|---|---|
| 距离下限 1 | 采用存活环、±马与下限 1；已有独立参照模型全对全比对 | 参照模型证明项目声明，**采用玩法版本仍须明确** |
| 座距只计存活 | 已有死座压缩与 next_alive/seat_order_from 模型对照 | **不外推为所有距离技能和官方版本完全等价** |

两条都不是「实现有问题」，而是**没有点名参照版本**，因此既无法判对错，也无法决定
后续差异登记以谁为准。本批按 AGENTS.md 第 5 条（固定规则来源、逐条对照）收口。

## 2. 采用玩法版本（项目声明）

**采用版本 = 本地 `archive/upstream/noname`（上游 v1.11.7，本次核对 HEAD `df2682e6`）的
standard 规则与标准包**；Liuli 侧差异、扩展自带规则另见[两仓台账](../../../migration/upstream-extension-reconstruction.md)。
旧线（`engine-core`）不是规则来源，只作为迁移与对照对象（其 golden 只证明旧线自身）。
未采用其它官方玩法版本；换版本须另立批次并重跑对照。

## 3. 距离语义核对（上游源码 → 原生实现）

源码摘录见 [upstream-distance-source.txt](upstream-distance-source.txt)。

| 语义点 | 上游实现 | 原生实现 | 判定 |
|---|---|---|---|
| 座距只计存活 | `get.distance` 沿 `nextSeat` 走圈，仅 `isAlive() && !isOut() && !undist && !isMin` 计数；`length` 由存活人数推出 | 存活环 + `next_alive`/`seat_order_from` 压缩死座 | 一致 |
| 下限 1 | 环形取最小 `n = Math.min(n, length - n)` 后 `return Math.max(1, n)` | 下限 1（同款） | 一致 |
| 防御马（的卢/绝影/爪黄飞电） | 装备 `distance: { globalTo: 1 }` → 加**他人到自己**的距离 | +马加他人到持有者距离 | 一致 |
| 进攻马（赤兔/大宛/紫骍） | 装备 `distance: { globalFrom: -1 }` → 减**自己到他人**的距离 | -马减持有者到他人距离 | 一致 |
| 攻击范围 | `method == "attack"`：`m += attackTo`，诸葛连弩 `attackFrom: -1` 等 | 武器射程覆盖基础 1，替换即时生效 | 一致（`attack_range` 行） |
| 自我距离 | `from == to` → 0；非在场对象 → `Infinity` | 同 | 一致 |
| 距离修正顺序 | `checkMod globalFrom/globalTo` + 装备叠加后统一 `Math.max(1, n)` | 修饰链后取下限 | 一致 |

既有独立证据：`distance_range_reference_differential`（独立参照模型全对全比对，
[ADR 0723](../../../adr/0723-native-n06-distance-differential-reward-ordering.md)）、
`equip_distance_matrix`、`equip_horses_weapon_range_and_replacement`、
`equip_distance_skips_dead_players`。

## 4. 收口方式与边界

- 覆盖清单行为差异表两行**移除**，改为在第 2 节声明的采用版本下按一致处理；表内保留
  一句勘正说明与本节链接。
- **不声称**「所有距离技能与官方玩法完全等价」：本批只固定参照版本与距离/座距/下限/±马
  四点的语义归属；其余距离类技能（马术、飞刃、凶兽、妖兽、疾风、平恤等扩展/招数）仍
  逐技能按各自批次证据验收。
- `undist`/`isMin` 等上游附加过滤在原生线无对应建模（未有内容触发），登记为未建模而非
  差异；一旦有内容需要，按新批次立项。

## 5. 影响

文档改动：`docs/native-coverage-manifest.md`（行为差异表两行移除 + 勘正说明）、
`docs/architecture/native-rules-baseline.md`（「当前声明与验证边界」增距离行 +
「本地上游对齐的新增核查」前声明采用版本）。无代码/版本/golden 变化。
