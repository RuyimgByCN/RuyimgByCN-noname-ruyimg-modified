# P2/N08 批③-b 第八把：雌雄双股剑（规则 v72，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第八把（收尾）：雌雄双股剑（♠2，射程 2，
杀指定异性目标后目标弃一张手牌或令使用者摸一张）。上游对照
`card/standard.js`（cixiong 条目：attackFrom −1 = 射程 2；特效
cixiong_skill）。前置：[第七把丈八蛇矛](../2026-10-07-n08-zhangba-3b7/README.md)。

## 本批落地

- **命名武器牌种 Cixiong（编码 102）**：equip1 武器、射程 2
  （named_weapon_range 登记 2）。
- **性别面**：`state.effective_gender(p)`——主将槽有武将取武将性别，
  无槽默认男（「性别有槽时筛选、无槽默认」惯例）。
- **特效接线（ShaResolve 出闪询问前）**：杀指定异性目标后 → 询问目标
  （`cixiong_discard_or_give`，Confirm=弃一张手牌 / Pass=令使用者摸一张，
  不可放弃）→ Confirm 弃目标手牌首张（**登记差异：上游目标自选**），
  Pass 来源摸 1；目标无手牌仅可令摸（直接生效不再询问）；本杀已问过
  标志（cixiong_done）防重问，随新杀重置。
- Sha 快照追加 `asking_cixiong`/`cixiong_done` 字段（穿齐 enc/dec/
  check/哈希/rebuild/ctor/a0 样本）。
- 声明表 122→**123 张**；`STANDARD_RULESET_VERSION` 71→**72**；
  **14 golden 重录**。

## 测试（`cixiong_opposite_gender_discard_or_draw`）

1. 弃牌路径：目标 1 号装配甄姬（女）+ 手牌 → 杀 → Confirm → 目标弃
   一张手牌、使用者未摸。
2. 同性不触发：2 号装配张飞（男）→ 杀 → 无雌雄询问、手牌保留
   （跨回合推进避开 ShaLimit）。

## 验收口径（分开陈述）

- **代码已实现**：雌雄牌种/声明表/effective_gender/异性弃牌或令摸语义
  （双路径 + 同性反例）。
- **回归已加入**：`cixiong_opposite_gender_discard_or_draw`。
- **golden 已重录**：14 场景全量（v72）。
- **全量通过**：engine-native **380/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0、loader 1/0；lib `-D warnings` 通过。

## 剩余

**批③-b 八把全部落地**（青釭/麒麟/寒冰/连弩/贯石斧/青龙/丈八/雌雄）；
模式对齐；主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
