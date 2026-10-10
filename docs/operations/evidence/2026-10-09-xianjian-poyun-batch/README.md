# U05 xianjian 第三十七批切片（规则 v179，2026-10-09）

慕容紫英（pal_murongziying，wei 4 男）破云（poyun）单技闭环——**玄砚
计数基建**（获得路径上游缺失=调试/宿主注入登记差异）；qianfang AOE 已
通暂缓、xuanning 获得路径缺失登记。前置：
[姜云凡批](../2026-10-09-xianjian-liefeng-batch/README.md)。

## 来源勘误（v178 批登记更正）

xuanning 并非离场区缺口族：为**计数标记**且上游 skill.js 无获得路径
（仅 poyun/qianfang 消费）——慕容紫英移出离场区族，xuanning 获得路
径缺失单独登记。

## 上游语义核验（源码为准，…/character/xianjian/skill.js poyun/qianfang）

- `poyun`：source damageEnd direct——storage.xuanning>0 且受伤目标 he
  有牌 → discardPlayerCard(target, he, [1,2])（被弃者自选）→
  xuanning--。
- `qianfang`：phaseBegin direct——xuanning 计数+he≥3-num 时弃 num 张
  →（AOE 万箭式效果）；AOE 虚拟用牌已通但 xuanning 获得路径缺失=暂
  缓。

## 改动（分开陈述）

- **状态**：`PlayerState.xuanning_mark: u32`（玄砚计数——**登记差异**：
  获得路径上游缺失，原生仅支持调试/宿主注入）。存档 v34→**v35**。
- **宿主行为**：`XianjianPoyun`（tag 202，source Damage forced direct：
  filter 玄砚>0+受伤目标 he 有牌→Act 问被弃者自选 he 1-2 张（min1
  max2 必选）→弃牌（hand/equips 分流 revoke）→计数 -1）——六表+ALL
  201→202。
- **登记暂缓/差异**：qianfang 暂缓（AOE 已通但计数获得路径缺失）；
  xuanning 获得路径缺失（登记差异）。
- **cards.json**：慕容紫英行（wei 4 男）+ rehash 两步；**版本** 178→
  179；a0 追加 202。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`murongziying_poyun_discard_and_
  decrement`（注入计数 2→伤害→被弃者自选弃 2→计数-1→再伤弃 1 归
  零→计数 0 不触发）。
- **全量通过**：engine-native **512 passed / 0 failed / 1 ignored**
  （v178 基线 511 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 178→179 + xuanning_mark 入 canonical_hash——14 场
  景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- 慕容紫英半闭环（poyun 闭环/qianfang+xuanning 获得路径暂缓）。
  xianjian 其余 4/36 将按缺口族；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next
根）。
