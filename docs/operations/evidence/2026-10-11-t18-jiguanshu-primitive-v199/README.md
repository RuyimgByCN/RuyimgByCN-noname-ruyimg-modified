# 2026-10-11 T18：机关术×2 转正——CardMod 受控合成原语（规则 v199，存档 v45）

任务来源：[进度主稿 T18](../../../architecture/native-progress.md)（领取锁 `d0e7893c`；落地提交 `2ce5d763`）。设计决策与备选方案否决理由见 [ADR 0780](../../../adr/0780-card-instance-mod-primitive.md)（已采纳）。

## 语义

- 上游 swd 机关术（`lingjianduanzao`，`swd_jiguanshu1` ♣2 / `swd_jiguanshu2` ♦2，kind jiguan）：运行时按名字组合合成/改造卡牌实体（数值合并升级、`lib.card[name]` 动态定义）。
- 原生建模：**CardMod 伴随表** `PlayerState.card_mods: BTreeMap<u32,u32>`（装备实体→附着灵剑实体，灵剑随机入表）；目录保持只读（不运行时改定义、不 intern 新名）。
- **登记差异**：①名称合成/数值升级（getValue 封顶 10）以「附着登记」近似——点数升级当前无原生消费点；②jiqi/yuchan 池暂缓（机关术全部附着灵剑）；③改造不随卡牌转移（换装/获得离去即失效，表随拥有者存档）。

## 版本与验证

| 项 | 结果 |
|---|---|
| 规则版本 | 198→**199**；14 golden 双探针后重录（位移唯一来源=版本字段） |
| 存档格式 | 44→**45**（PlayerState 尾部 len+(u32,u32) 序列；v44 字节被版本门禁拒绝） |
| 回归 | `g1_shahai_swd.rs::swd_jiguanshu_attaches_card_mods_and_roundtrips`：施放后 CardMod 恰一条且附着实体为灵剑；CardMod 随存档字节往返一致；无装备不可用 |
| 全量 | `cargo test --offline -p engine-native --no-fail-fast` **545 通过 / 0 失败 / 1 ignore**（[完整日志](engine-native-tests.log)，合计核验 2026-10-11） |
| 收口 | 机关牌池 8 种 17 张全数转正（无暂缓），swd 包 45 张；规则基准 v199 条目 2026-10-11 补录 |

## 边界

包数据变化仅 swd cards.json（机关术×2 实体 14 行）与 npack 重打包；CardMod 数值消费点如后续机制需要（升级数值参与比较/伤害），需另立任务接入消费端。
