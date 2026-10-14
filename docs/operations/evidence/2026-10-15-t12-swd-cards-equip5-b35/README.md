# T12 swd 卡牌面补切批⑤：equip5 两宝物——昆仑镜/东皇钟（2026-10-15）

## 承载

2 个新 CardKind（tag 147/148，equip5 宝物栏；包 57→**59 张**）：

- **昆仑镜 kunlunjingc**（tag 294，spade13）：出牌限一次三段机——亮牌堆
  顶 3 张选 1 → 以 1 张手牌替换（弃置）→ 获得选中牌。登记差异：上游替
  换牌入特殊区（ui.special）、其余亮牌插回牌堆顶——原生替换牌直接弃置、
  其余亮牌留牌堆原位（净效果等价）。
- **东皇钟 donghuangzhong**（tag 295，diamond13）：自己任一阶段结束
  （direct forced，蝶舞口径）弃 1 张红牌+选目标 → 从牌堆**发现**一张延时
  锦囊实体（Le/Bing/ShanDian 且目标判定区未有同 kind）置入目标判定区。
  登记差异：上游 createCard 凭空造+随机 kind → 原生实体发现+首张确定性。
- 触发机 stage 起步坑（第 6 犯变体）：触发机默认 stage=Act，而东皇钟臂
  从 SelectCard 开始——首步 answer() panic——machine() stage 臂补
  SelectCard（原-stage 先例同位）。
- golden 不变（行为门控）、无版本 bump。

## 测试（swd_cards_equip5_b35_kunlunjingc_donghuangzhong）

| 面 | 断言 |
|---|---|
| 昆仑镜正例 | 亮牌面含牌堆首张 → 替换手牌弃 → 选中牌入手 |
| 昆仑镜反例 | 本回合限一次 |
| 东皇钟正例 | 阶段末弃红牌+选 p2 → p2 判定区置入延时锦囊实体（Le/Bing/ShanDian 之一） |

测试驱动 ② 段用通用窗循环推进整回合（各段末触发窗+判定区置入检查）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **44/44** |
| engine-native 全量 | **763/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | 拦冗余 clone（已修）→ exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 59/92 行；equip5 余 7 kind（书锤/散灯金环/昊天塔/玉如意/空洞印/龟
  灵之桃/四非珍面——书锤多杀循环/玉如意摸牌改写/散灯金环判定改写为中复
  杂，空洞印死亡取消/女娲石同族可承接）。
- 批⑥ 候选：空洞印（dieBefore 死亡取消，death_cancelled 先例）或锦囊族。
