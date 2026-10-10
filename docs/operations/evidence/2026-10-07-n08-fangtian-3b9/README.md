# P2/N08 批③-b 第九把：方天画戟（规则 v73，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第九把（收官）：方天画戟（♦5，射程 4，
该杀为最后手牌时可指定至多三个目标）。上游对照 `card/standard.js`
（fangtian 条目：attackFrom −3 = 射程 4；fangtian_skill = selectTarget
mod：全部手牌被选为该杀卡牌时目标上限 +2，content 为空——多目标由杀
卡自身结算逐目标处理）。前置：
[批③-b⑨ 立项评估](../2026-10-07-n08-fangtian-eval/README.md)。

## 本批落地

- **命名武器牌种 Fangtian（编码 103）**：equip1 武器、射程 4
  （named_weapon_range 登记 4）。
- **PlayCard 门控放宽**：该杀为来源最后手牌（hand.len()==1）且装备
  方天时，目标 1..=3 个（逐目标独立查 距离/存活/空城，BTreeSet 去重；
  非方天或非最后手牌保持单目标）。
- **ShaResolveMachine 目标队列化**：新增 `extra_targets` 队列（首目标
  即期结算，其余入队）；当前目标结算完毕（伤害/闪避终态）→
  `advance_target`：逐目标状态全复位（防具判定/无双双闪/铁骑/流离/
  护驾/各武器询问标志）并重开该目标的出闪询问。各武器特效
  （仁王盾/铁骑/八卦/贯石斧/麒麟弓/寒冰/青龙）在每目标上完整重走。
- Sha 快照追加 `extra_targets` 字段（穿齐 enc/dec/check/哈希/rebuild/
  ctor/a0 样本/engine 测试构造）。
- 声明表 123→**124 张**（♦5）；`STANDARD_RULESET_VERSION` 72→**73**；
  **14 golden 重录**。

## 测试（`fangtian_multi_target_sha`）

1. 非最后手牌：留一张非杀手牌 → 三目标被拒（IllegalTargets）。
2. 方天三目标：清手牌至仅杀 → 三目标齐伤（各 4→3，逐目标出闪询问
   排空）。
3. 两目标变体：跨回合后再次注入杀 → 两目标合法。

## 验收口径（分开陈述）

- **代码已实现**：方天牌种/声明表/多目标门控/目标队列化结算。
- **回归已加入**：`fangtian_multi_target_sha`。
- **golden 已重录**：14 场景全量（v73）。
- **全量通过**：engine-native **382/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0；lib `-D warnings` 通过。

## 剩余

**批③-b 九把全部落地**（青釭/麒麟/寒冰/连弩/贯石斧/青龙/丈八/雌雄/
方天）。P2/N08 剩余：逐技能语义对照与逐张花色对齐、模式入口扩充；
主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
