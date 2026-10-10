# U05 xianjian 第四十批切片（规则 v182，2026-10-10）

慕容紫英（pal_murongziying）千方（qianfang）单技闭环——**慕容紫英升全
闭环**（xuanning 获得路径仍登记差异；虚拟 AOE 原语第三次消费）。前置：
[张目批](../2026-10-10-xianjian-zhangmu-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js qianfang/qianfang_draw）

- `qianfang`：player phaseBegin direct——xuanning>0 且 he+xuanning≥3：
  num=3-xuanning；targets=其他角色（万箭可指定）；num>0 →
  chooseToDiscard(num, he)（可取消）→ xuanning=0 → useCard wanjian 对
  targets；num≤0 → chooseBool 确认。
- `qianfang_draw`（subSkill）：source damageEnd forced——被万箭伤到的
  敌人每名紫英摸 1。

## 改动（分开陈述）

- **宿主行为**：`XianjianQianfang`（tag 206，PhaseBegin[Judge] direct
  机内取消：Act 发问弃 num=3-玄砚 张 he（min=max=num，可取消=放弃；
  num=0 时直接 AOE）→ SelectCard 弃置+清零+虚拟万箭 AOE（
  use_virtual_card ArcheryAttack 全其他存活=**原语第三次消费**）→被
  伤敌人数紫英摸（qianfang_draw 直连=**登记差异**：上游 damageEnd 逐
  目标即时，原生结算后统一摸））——六表+ALL 205→206。无新状态字段
  （xuanning_mark 复用 v179 基建）。
- **cards.json**：慕容紫英 qianfang mapping null→host + rehash 两步；
  **版本** 181→182；a0 追加 206。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`murongziying_qianfang_discard_and_
  aoe`（注入玄砚 1→下回合 Judge begin 询问弃 2→清零+虚拟万箭 1/2 号
  各-1+紫英摸 2）。
- **全量通过**：engine-native **515 passed / 0 failed / 1 ignored**
  （v181 基线 514 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 181→182 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 慕容紫英升全闭环（xuanning 获得路径仍登记差异）。xianjian 其余
  2/36 将（南宫煌 zhangmu 已于 v181 闭环、柳梦璃 tianxian mod 系暂
  缓）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
