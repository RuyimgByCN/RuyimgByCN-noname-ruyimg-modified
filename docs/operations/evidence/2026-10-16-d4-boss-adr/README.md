# 2026-10-16 D4 Boss 关卡流程子系统 ADR 证据

**核验日期**：2026-10-16　**性质**：设计/文档批（无引擎代码、无规则版本递增、无 cargo 全量）

## 产物

| 项 | 路径 |
|---|---|
| ADR | [docs/adr/0783-boss-stage-flow-subsystem.md](../../../adr/0783-boss-stage-flow-subsystem.md) |
| INDEX 登记 | [docs/adr/INDEX.md](../../../adr/INDEX.md)（0783 行） |
| 用户裁决 | [2026-10-16-d4-d8-decisions](../2026-10-16-d4-d8-decisions/README.md) D4 行 |
| 待做映射 | [2026-10-16-todo-register](../2026-10-16-todo-register/README.md) · `T11-BOSS-STAGE-SUBSYSTEM` 及子项 |

本批**未改** `native-progress.md` 正文（并行 worker）；未改 `crates/**`、cards.json、npack。

## 上游勘点摘要（只读）

权威路径：`archive/upstream/noname/apps/core/mode/boss.js`（约 11480 行）。对照副本：`archive/upstream/noname-liuli/noname/mode/boss.js`（同族符号，行号可能漂移——实现以 noname 路径为准）。

### API / 模式态

| 符号 | 约行 | 备注 |
|---|---|---|
| `addBossFellow` | 1680 | 包装 `game.addFellow` |
| `changeBoss` | 1688 | 换 Boss 本体（可带第二参换侧座） |
| `_status.shidianyanluo_level` | 2512+、3376+、3400+ | 十殿关卡等级 0..3 |
| `_status.shidianyanluo_mengpo` | 3418+、3542+ | 孟婆已乱入 |
| `_status.shidianyanluo_mengpodie` | 4134、3488 | 孟婆已死（地藏王分支条件） |

### 还人族

| 符号 | 约行 | 备注 |
|---|---|---|
| `boss_qinguangwang` 条目 | 1295 | skills 含 `boss_newhuanren` |
| `boss_newhuanren` | 3404 | global→`boss_newhuanren2`；开局孟婆概率乱入；禁将表 |
| `boss_newhuanren2` | 3448 | die 后选下一殿；地藏王/映姬特殊候选 |
| 禁将字面名单 | 3436 | `luxun`/`re_luxun`/`zhangchunhua`/`zuoci`/…/`kongrong` |
| 关卡说明文案 | 11377–11379 | 四关 + 孟婆/地藏王/映姬规则 |

### 隐藏链八技

| 技能 | 约行 | 链向（dieAfter `changeBoss`） |
|---|---|---|
| `boss_qingmu2` / `*x` | 6253 / 6269 | → `boss_mushengoumang` |
| `boss_qingmu3` / `*x` | 6342 / 6362 | → `boss_taihao` + 侧座 |
| `boss_chiyan2` / `*x` | 6071 / 6087 | → `boss_huoshenzhurong` + 焰灵 fellow |
| `boss_chiyan3` / `*x` | 6160 / 6180 | → `boss_yandi` + 侧座 |
| `boss_xuanlin2` / `*x` | 6435 / 6451 | → `boss_shuishengonggong` |
| `boss_xuanlin3` / `*x` | 6523 / 6553 | → `boss_zhuanxu` |
| `boss_baimang2` / `*x` | 6633 / 6649 | → `boss_jinshenrushou` |
| `boss_baimang3` / `*x` | 6722 / 6742 | → `boss_shaohao` |

共性：`dieBegin` hide + `dieAfter` changeSeat/changeBoss/补给（详见[隐藏链处置](../2026-10-11-t11-boss-hidechain-disposition/README.md)）。

### 翼轮次 / bianshen

| 符号 | 约行 | 备注 |
|---|---|---|
| `boss_baiyi` | 5325 | 轮次硬绑三子技 |
| `boss_chiyi` | 7105 | round∈{3,5,7} 编排 |
| `boss_qingyi` | 7323 | 青翼（太昊尾技） |
| `boss_bianshen3` | 8747 | → 无常链 |
| `boss_bianshen4` | 8779 | 死亡 `changeBoss` 夜叉/罗刹池 |
| `boss_bianshen` | 9248 | 开局 init 壳（不入 D4 落包） |
| `boss_nbianshen` | 8580 | 年兽形态轮换（已有 v249 承载先例） |

### 边界对照（不进 D4 实现）

| 符号 | 约行 | 处置 |
|---|---|---|
| `boss_shikieiki` / `boss_yingzhong` | 1416 / 2846 | 映种：外部将池——维持排除；还人候选可达性另登记 |
| 诸葛亮/`ylyg*` | （offline 龙血包，非本文件主定义） | **D5** |
| 庞统涅槃/浴火 | 外部包技 | **非 D4**（见 ADR §5） |

## 既有原生证据（设计输入）

- [隐藏链整族处置](../2026-10-11-t11-boss-hidechain-disposition/README.md)
- [bianshen 三形态勘验](../2026-10-11-t11-boss-bianshen-survey/README.md)
- [赤翼/白翼处置（星夏批）](../2026-10-11-t11-boss-xingxia-v242/README.md)
- [秦广王·还人登记不落](../2026-10-14-t11-boss-qinguangwang-v276/README.md)
- [mode_boss 收口评估](../2026-10-11-t11-modeboss-closeout/README.md)

## 建议回填 T11 的文案草稿（供协调者合并进 native-progress.md）

> **D4（2026-10-16）已裁决**：建原生 Boss 关卡流程子系统并逐技落包。设计见 [ADR 0783](../../../adr/0783-boss-stage-flow-subsystem.md)；证据 [2026-10-16-d4-boss-adr](../2026-10-16-d4-boss-adr/README.md)。原「关卡流程面登记不落」项（还人、隐藏链八技、青/赤/白翼、bianshen 死亡换 Boss）转为待实现，分批 B0 骨架→B1 隐藏链→B2 bianshen4→B3 还人→B4 翼编排。映种排除与诸葛亮/`ylyg*`/庞统外部技边界不变（后二者挂 D5）。**实现与规则版本递增尚未开始**——本句仅记录裁决与 ADR，不计入完成度。

规则基准对应段可将「登记不落」改为「待实现（ADR 0783；实现前语义仍不可达）」并链本目录——由基准/进度 worker 择机回填，避免与并行编辑冲突。

## 建议首批实现技能（B1，B0 之后）

1. `boss_qingmu2` / `boss_qingmu2x`
2. `boss_qingmu3` / `boss_qingmu3x`
3. 其余隐藏链六技参数化同构落包

## 本批验证

| 项 | 结果 |
|---|---|
| 引擎 / 包数据改动 | 无 |
| 规则 / 存档 / golden | 不递增、不重录 |
| cargo 全量 | 未跑（文档批） |
