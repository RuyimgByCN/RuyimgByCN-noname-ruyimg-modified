# T12 swd 武将面批 22：玉鳞/凝霜/工神三技三将部分转正（2026-10-15）

## 承载

三技 HostBehavior（tag 275/276/277，键数冻结表 274→**277**，HostStage 复用
Act/SelectCard 两段——无新 stage 无存档 bump）：

- **玉鳞 yulin**（兰茵）：受伤前置窗口（可选确认）弃 1 张装备牌（手/装备
  区，equip 区走 revoke_equip_skills+to_discard idiom）→
  `current_damage_mut().prevented = true` 免此伤害。取消=不免伤。登记差异：
  上游 direct 无确认直接进弃牌询问，原生可选触发门+选牌窗两段等价。
- **凝霜 ningshuang**（芝茵）：成为**黑色牌**目标（可选确认）弃 1 张黑色
  牌（手/装备区），对使用者阶梯——已翻面→失 1 体力（LoseHp emit）；已横
  置→翻面（TurnOvered emit）；否则→横置（Linked emit）**且仅此支自己摸
  1**。上游 skill.js 代码实为顺序双 if（filter 排除「横置且翻面」组合），
  info 文案未含失体力支——按实现转正并登记。触发上游 `useCardToBegin`，
  原生 CardUsed 帧等价接入（同批 21 墨剑口径）。
- **工神 gongshen**（黄雷）：其他角色使用基本/锦囊牌且有目标（可选确认）
  弃 1 张装备牌令其失效——`cancelled = true` 写回（强固守卫同款）**+牌从
  table 归还使用者手牌**（上游 trigger.cancel 同口径；筑城旧注释「杀已由
  守卫归还」在现行代码无归还步骤，本批按上游语义补齐并实测）。异类种
  （地/机关）不计入基本/锦囊（is_basic_trick_or_equip 排除集复用）。

## 武将归属（translate.js 核验）

兰茵 swd_lanyin（wei/female/hp3：星点/玉鳞/落梅）、芝茵 swd_zhiyin
（wei/female/hp3：雪煌/凝霜/竹语）、黄雷 swd_huanglei（qun/male/hp3：
机略/工神/改造）——均 1/3 或 1/2 部分转正，cards.json generals 11→**14**，
npack 双哈希重算。

## 过程勘误（三处接线/布阵修正）

1. 三技 Act 臂首版漏 `self.stage = HostStage::SelectCard` 状态推进——应答
   后机器停回 Act 重开询问，弃牌从未执行（免伤反因 prevented 默认 false
   假绿）——由「zhuge 已弃」断言抓出。
2. **debug_set_deck_top_by 只把匹配牌压到牌堆底**（并返回 id），
   debug_place 从堆前取——两者不衔接，布色不保证（标准牌堆黑 tao 不存在
   直接踩空）——批 22 新 helper `place_sha_color`：按谓词定位实体（返回
   id）后 `deck.retain + hand.push` 直接入手，确定性布色。
3. ③ 段撤销凝霜的快照块漏带杀限重置（②d 的杀已消耗）→ 主线出杀被拒无
   窗口——批 21 教训 95 的变体（重置必须与所授技撤销同块迁移）。

## 测试（swd_generals_b22_yulin_ningshuang_gongshen，反例 10 项）

| 技 | 正例 | 反例 |
|---|---|---|
| 玉鳞 | 手牌装备弃置免伤 | 取消→照常伤；无装备牌→无窗；装备区装备牌弃置免伤 |
| 凝霜 | 黑杀目标→弃黑牌横置+摸 1 | 已横置→翻面不摸；横置且翻面→无窗；红杀→无窗 |
| 工神 | 弃装备令杀失效（p2 免伤+杀回 p1 手牌） | 取消→照常伤；无装备牌→无窗；无目标牌（桃）→无窗 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **33/33**（批 21 32/33 基础上 +1） |
| engine-native 全量 | **752/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | 首跑拦未用变量 zg4（已改 `_zg4`）→ 复跑 exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**15/79 名、15/241 技转正**（九整将格局不变；三将均部分：
  兰茵 1/3、芝茵 1/3、黄雷 1/3）。
- 工神取消归还牌为筑城强固守卫现行代码缺失面的对照样本（该 ow 批行为本
  批不动，留待 ow 复核批对照）。
- 批 23 候选：剩余 64 名继续按将勘察，逐技回读上游 skill.js/translate.js
  核验后再接线（批 21/22 两轮证明转写记忆不可直接采信）。
