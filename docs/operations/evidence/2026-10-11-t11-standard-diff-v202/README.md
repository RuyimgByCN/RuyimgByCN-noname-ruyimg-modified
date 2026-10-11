# 2026-10-11 T11 标准差异消除批：倾国/救援/结姻/急救（规则 v202）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `de3da554`）。上游依据：`archive/upstream/noname/apps/core/character/standard/skill.js`（qingguo:898/jiuyuan:1622/jieyin:1997/jijiu:2082）。

## 语义修正（对齐上游，登记差异撤销）

| 技能 | 原生改前 | 上游语义 | 修正 |
|---|---|---|---|
| 倾国（甄姬） | 装备区黑色牌当闪（kernel+snap 双处收集） | `filterCard: black` + `position: "hs"`（黑**手牌**） | 收集改 hand（排除已入列的常规闪/龙胆杀避免重复选项） |
| 救援（孙权主公技） | 仅「用桃者≠濒死者」 | `zhuSkill` + `event.player.group === "wu"` + 非本人 | 加 `is_lord(dying)`（原型局恒真/身份局查主公）+ `effective_faction(responder)==Wu`（护驾魏同口径） |
| 结姻（孙尚香） | 目标=装备区有牌的男性（登记「已婚=装备区有牌」简化） | `filterTarget`：male + `hp<maxHp` + 非自己（**无**已婚/装备条件） | activate 存在性门+TargetAsk 候选同改「其他受伤男性」；`slots_is_male` 辅助拆出供 QueryCtx 侧复用 |
| 急救（华佗） | 红牌当桃无回合窗口 | `viewAsFilter: player !== _status.currentPhase` | 加 `turn.current != responder`（限持有者回合外） |

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test native_kernel` | **204/0**（含重写的 `n08_qingguo_qicai_jiuyuan`[黑手牌入选项+装备反例；救援吴势力用桃 2 血+非吴对照 1 血]、`n08_jijiu_red_card_as_tao`[新增 (3) 自己回合内不可用→不开本人询问直接死亡]、`n08_jieyin_heals_both`[受伤目标继续可选中]） |
| 版本回退探针（202→201 复跑 `replay_scenarios`） | 2/2 全绿（证明 14 golden 位移唯一来源=版本字段；四技能修改分支均不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 `replay_scenarios` 2/2 绿（重录依据注释已入文件头） |
| `cargo test --offline -p engine-native --no-fail-fast` | **549 通过 / 0 失败 / 1 ignore**（exit 0；2026-10-11 本机） |
| `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib`（独立 target 目录） | exit 0 |
| 存档格式 | 不变（v46；本批无状态新增，仅行为条件收紧） |

## 剩余登记（基准「内容差异与后续验收」块更新后）

- 护驾：代出闪仍为弃置+计数而非完整 respond 通道（v29 重构后剩余项）。
- 反间：花色选择与条件伤害承载差异（以看牌选牌近似）。
