# U05 xianjian 第三十一批切片（规则 v173，2026-10-09）

**moveCard 基建落地**（台账待建排序④）+ 月琦离光（liguang）单技闭环
——月琦升全闭环（tianwu 登记不转）。前置：
[照妖批](../2026-10-09-xianjian-zhaoyao-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js liguang）

- `liguang`：player phaseEnd direct——弃一张手牌（可取消=放弃）+
  moveCard(true)（移动场上的一张牌：装备移至空栏角色[revoke+grant 技
  能]；判定区牌移至无同名牌角色）。

## 改动（分开陈述）

- **moveCard 基建首例（host 步内确定性扫描）**：装备→首个空同栏目的
  地（revoke_equip_skills+grant_equip_skills 技能随迁+Equipped 日志）；
  判定区→首个无同名牌目的地（CardsGiven 日志）。**登记差异**：上游
  moveCard(true)=AI 自动选取，原生=确定性首张可移牌+首个合法目的地
  （座位序）。
- **宿主行为**：`XianjianLiguang`（tag 194，PhaseEnd direct 机内取消：
  首臂发问弃牌（可取消=放弃）→TargetChoice 弃牌+确定性移动）——六表
  +ALL 193→194。无新状态字段。
- **cards.json**：月琦 liguang mapping null→host + rehash 两步；**版
  本** 172→173；a0 追加 194。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`yueqi_liguang_discard_and_move_
  equip`（弃手牌→武器 1→0 号（首合法目的地）迁移+revoke/grant+弃牌
  入弃堆；第二循环取消=放弃）。
- **全量通过**：engine-native **504 passed / 0 failed / 1 ignored**
  （v172 基线 503 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 172→173 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①确定性首目的地=座位序首个空栏（0 号优先于
  2 号）②离光询问在 End 阶段末帧（第 3 次 NP）。

## 剩余

- 月琦升全闭环（tianwu 登记不转）。xianjian 其余 9/36 将（响应链
  zhangmu/longxi、mod 系 tianxian/liefeng、createCard、Recover 前置
  窗口、暗牌等按缺口族）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
