# U05 xianjian 第四十四批切片（规则 v186，2026-10-10）

居十方（pal_jushifang）**整将入包**（34→35 将，缺 1：长卿）+械磐
（xiepan）单技闭环——御甲/偃师机关牌种缺口暂缓登记=居十方半闭环。
实体发现基建第二次消费。前置：
[影锋批](../2026-10-10-xianjian-yingfeng-batch/README.md)。

## 未入包 2 将选型依据

- **居十方**（械磐+御甲+偃师）：械磐=CardsDiscarded 触发+实体发现
  （基建直用，最小）；御甲/偃师=「机关牌」种不在目录（dujiang 同
  族缺口）。**本批选居十方**。
- **长卿**（乱剑+天符）：乱剑=双杀 viewAs（丈八同构）+多目标+距离+
  50% 增伤可单切；天符=闪置武将牌上+延迟雷伤（挂附标记）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xiepan）

- `xiepan`：player loseEnd direct——filter=本次失去含手牌基本牌
  （original==="h" && type basic）&& 手牌已无基本牌 && 手牌非空；
  content=chooseToDiscard("h")（可取消）→弃一张→
  gain(createCard(随机装备牌名))（凭空造牌）。

## 改动（分开陈述）

- **宿主行为**：`XianjianXiepan`（tag 210，CardsDiscarded forced 机
  内取消）——filter（自己弃牌含基本牌+手牌已无基本牌+手牌非空）→
  Act 发问弃一张手牌（可取消=放弃）→ SelectCard 收应答：弃置+实体
  发现随机装备 1 张入手（牌堆+弃堆装备实体池 RNG，不足无）。**登记
  差异**：①弃置路径触发（上游 loseEnd 覆盖一切失去路径=双刃同款收
  窄）②弃牌来源不分手牌/装备（CardsDiscarded 无 original 标记）③
  获得=实体池发现替代 createCard 凭空造牌（不足无）。无新状态字段。
- **cards.json**：居十方整将条目（shu/3 血/male；械磐 host、御甲/
  偃师 null+理由登记）+ rehash 两步；**版本** 185→186；a0 追加
  210。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`jushifang_xiepan_discard_and_
  discover_equip`（压上限 1+手牌[sha,wuzhong]→弃牌阶段弃 sha→械磐
  挂起弃 wuzhong→手牌=1 张装备实体+两牌入弃堆）。
- **全量通过**：engine-native **519 passed / 0 failed / 1 ignored**
  （v185 基线 518 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 185→186 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 居十方半闭环（御甲/偃师=机关牌种缺口）。未入包 1 将（长卿乱剑/
  天符可单切）；机关牌族（御甲/偃师/dujiang）需 createCard 实体造
  牌基建先行；长期项不变（zhenfa 五阵法、整包验收 0/37 等）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
