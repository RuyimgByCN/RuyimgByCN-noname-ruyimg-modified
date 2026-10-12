# 2026-10-11 T11 群雄乱入甄姬片：踏水/凌波/娇孝/芳华（规则 v262）＋甄姬条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续）。上游依据：`boss_zhenji`（甄|宓/shen/hp4/female/skills `[tashui, lingbo, jiaoxia, fanghua]`，boss.js:1494）；`lingbo`@8358 / `jiaoxia`@8397 / `fanghua`@9977 / `tashui`@10003（boss.js）。

## 建模

- **凌波**（触发 `CardResponded`+extra `CardUsed`+一步机 `LingboMachine`）：打出/使用【闪】后摸 2（frequent 无询问；filter=打出实体牌种为闪/使用按声明牌种 viewAs 闪）。
- **踏水**（触发 `CardResponded`+extra `CardUsed`+一步机 `TashuiMachine`）：使用/打出黑色牌后，座位序首个其他存活角色翻面（经 `TurnOverBefore` 走翻面提交=虚失同款通道；chooseTarget AI 态度面→首个确定性近似登记）。
- **娇孝**（查询型 Noop）：commit_card_used 内联臂——成为**红色牌**（实体花色 Heart/Diamond）目标后摸 1；闪电自置（targets 空隐式自目标）等价分支；桃园/五谷/无中为隐式全员 targets 空→不触发（登记差异：上游 useCardToTargeted 逐目标发事件）。
- **芳华**（查询型 Noop）：End begin 臂——存在翻面角色时全部翻面者各失去 1 体力。
- **甄姬条目**：`BOSS_GENERALS` 尾追加（def 索引 40，hp4，female）`[踏水, 凌波, 娇孝, 芳华]`；势力 shen→群目录近似。

## 引擎工具 bug 修复（顺带）

`debug_move_named_card_to_hand` 手牌回收分支 **push 两次**（函数体内一次+尾部统一 push 一次）——同 id 双份实体，区域守恒炸（`存在同一实体牌出现在多个牌区`）。修复=删函数体内 push（尾部统一 push 保留）。历史测试未触发该分支（同 kind 牌恰在牌库/弃牌堆），故此前全量无感。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_zhenji_lingbo_jiaoxia_draws` | ①凌波：0 出杀打 1（持闪）→闪打出后摸 2+免伤 ②娇孝正例：甄姬自置红闪电（♥12）→摸 1 ③娇孝负例：黑杀指定→伤害 1 不摸（踏水翻面联动）④守恒+往返 |
| 新回归 `boss_zhenji_tashui_flip_and_fanghua` | ①踏水：黑杀→座位序首个（0 号）翻面 ②芳华：甄姬 End→翻面者失 1（不翻回、他人不动）③往返 |
| 版本递增探针 | 普通批口径：262→261 回退复跑 golden **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 行级程序化重录 2/2 绿 |
| 全量 | **616 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 261→**262**；存档不变 v60 |

## 坑位实录（本批五坑）

- **`debug_set_deck_top_by` 语义陷阱**：实现是 push 到**牌堆尾**（供判定摸牌 pop「后推先抽」），与 `debug_place` 的**头部扫描**不衔接——「set_deck_top+place 取同牌」组合失效（拿到自然序头部牌）。
- **`debug_move_named_card_to_hand` 双 push bug**（已修，见上）。
- **`debug_place` 手牌回收分支只查不删**：deck/discard 耗尽后从手牌「回收」实为复制——重影实体。绕开=测试内 `state_core_mut` 精确搬移（helper `place_sha_of_color`：先快照 catalog 花色/牌种表再 remove+push）。
- **place 只按 kind 匹配**：`shandian_safe`（♥12）与 `shandian`（♠1）同 kind——place("shandian_safe") 拿到黑闪；按名取须用 `debug_move_named_card_to_hand`。
- **play 者须为回合玩家**：布阵进非回合玩家手再 play=CardNotInHand——红闪电自置段须先 advance_to_play(1)。
- **娇孝臂结构**：holders 先按 targets 过滤则自置分支（targets 空）死路——先取全部持有者再分支判目标面。

## 遗留登记（本批新增）

- 娇孝：桃园/五谷/无中隐式全员 targets 空不触发（上游逐目标事件）；凌波 viewAs 闪按声明牌种 ✓；踏水翻面目标=座位序首个（AI 态度面不落）。
- 下批候选：群雄乱入诸葛亮（hp=Infinity 需 maxHp 建模勘验）、庞统（天狱/奇物/涅槃/浴火）、周瑜（火神/红莲?/仙音）、十殿阎罗族首勘、吕布变身系。
