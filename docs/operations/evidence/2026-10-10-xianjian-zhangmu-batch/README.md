# U05 xianjian 第三十九批切片（规则 v181，2026-10-10）

南宫煌（pal_nangonghuang）张目（zhangmu）单技闭环——**南宫煌升全闭
环**（响应询问改写以伤害前置直防等价接入，登记差异）。前置：
[双刃批](../2026-10-09-xianjian-xshuangren-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js zhangmu）

- `zhangmu`：chooseToRespondBegin/chooseToUseBegin direct usable:1——
  需要打出闪响应且有手牌闪时，可弃一张闪直接视为已响应
  （untrigger+responded=true，不出牌）。

## 改动（分开陈述）

- **状态**：`TurnState.zhangmu_used: bool`（每回合 1 次=上游 usable:1
  同口径，Judge begin 清）。存档 v35→**v36**。
- **宿主行为**：`XianjianZhangmu`（tag 205，DamageBefore forced）——杀
  伤害指向自己+有手牌闪+本回合未发动时：弃首张手牌闪+prevented 直写
  （杀不命中）。**登记差异**：①响应询问改写简化为伤害前置直防（上游
  untrigger+responded 语义在响应询问内，原生 DamageBefore 窗口直写——
  杀不命中等价）②弃哪张闪确定性取首张闪（上游玩家自选）。
- **cards.json**：南宫煌 zhangmu mapping null→host + rehash 两步；**版
  本** 180→181；a0 追加 205。
- **南宫煌升全闭环**。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`nangonghuang_zhangmu_flash_discard_
  prevent`（杀伤害前置弃闪直防体力不变+闪离手+回合位；同回合再杀杀限
  次拒绝；新回合张目再防+闪再弃）。
- **全量通过**：engine-native **514 passed / 0 failed / 1 ignored**
  （v180 基线 513 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 180→181 + zhangmu_used 入 canonical_hash——14 场景
  全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- 南宫煌升全闭环。xianjian 其余 2/36 将（柳梦璃 tianxian mod 系、慕容
  紫英 qianfang）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
