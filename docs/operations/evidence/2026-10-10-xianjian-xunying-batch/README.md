# U05 xianjian 第四十一批切片（规则 v183，2026-10-10）

姜云凡（pal_jiangyunfan）迅影（xunying）单技闭环——**姜云凡升全闭
环**（疾风+天剑 v178、迅影 v183；炎杀 yanzhan viewAs 仍登记差异）。
真实牌自动使用第二例。前置：
[千方批](../2026-10-10-xianjian-qianfang-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xunying）

- `xunying`：player shaAfter direct——filter=`canUse("sha", target)
  && hasSha() && target.isIn()`；content=chooseToUse 再杀同一目标
  （第 4 参 -1）+酒状态处理（removeSkill jiu→未用还原）。
- 技能文："每当你使用【杀】对一名目标结算完毕后，你可以继续对目标
  使用【杀】。"——每次再杀结算后 shaAfter 再问，链式直至放弃/无杀。

## 改动（分开陈述）

- **宿主行为**：`XianjianXunying`（tag 207，CardUsed AfterTriggers
  [shaAfter] direct）——filter（自己使用杀+目标存活+computed_distance
  ≤attack_range+有手牌杀）成立时自动取手牌首张杀对同一目标 emit
  CardUsed 再杀（真实牌 commit 自取，机器不预取=凤略同款）；链式再
  杀终止于无杀/目标亡/超距。**登记差异**：①上游 chooseToUse 玩家自
  选可放弃，原生自动取首张杀直到无杀②上游 canUse 简化为距离口径
  ③酒状态保留不迁移④再杀不经 PlayCard 限次校验（上游 chooseToUse
  第 4 参 -1 同口径）。无新状态字段。
- **cards.json**：姜云凡 xunying mapping null→host + rehash 两步；
  **版本** 182→183；a0 追加 207。
- **工程修正**：机器工厂 `machine()` 迅影臂初稿注入 (None, None)，
  机器步 `self.target` 为 None 直接 Done——回归暴露后改为从
  CardUsed 事件取首目标（本批唯一语义断点，定向测试红→绿）。
- **包数据计数核验修正**：xianjian 上游 36 将已入包 **33** 将（余
  **3**：长卿/居十方/越今朝；慕长兰=暮菖兰、罗招嫣=洛昭言、月琦=
  越祈、姜城=姜承、仙卿=闲卿 为包内改名非缺失）；generals 技能映射
  host **68**/null **24**。台账此前「34 将/44 闭环+1 半闭环/余
  2/36」不可核验口径已替换（cards.json 为准）。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`jiangyunfan_xunying_auto_second_
  sha`（杀 1 命中后迅影自动再出杀 2 对同目标，6→4 两段伤害；手牌两
  杀均离手=链终止于无杀）。
- **全量通过**：engine-native **516 passed / 0 failed / 1 ignored**
  （v182 基线 515 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 182→183 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 姜云凡升全闭环。xianjian 其余未闭环将为柳梦璃（tianxian mod 系
  登记差异简化待评估）及带登记暂缓技诸将（姜城 yanzhan viewAs、夏
  侯瑾轩 danqing、苏媚 dujiang、仙卿 xiaoyue、罗招嫣 tanhua、慕容
  紫英 xuanning 获得路径等，逐技状态见台账批次链）；未入包 3 将
  （长卿/居十方/越今朝）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
