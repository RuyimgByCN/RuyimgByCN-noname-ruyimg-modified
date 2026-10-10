# U05 xianjian 第四十二批切片（规则 v184，2026-10-10）

柳梦璃（pal_liumengli）天弦（tianxian）单技闭环——**柳梦璃升全闭
环**（runxin v166、xjzhimeng 系、天弦 v184）。锁定旗标行为首例（铁
掠同款：效果内联、无触发机器）。前置：
[迅影批](../2026-10-10-xianjian-xunying-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js tianxian）

- 技能文："锁定技，你的【杀】无视距离且可指定任意多个目标，目标须进
  行一次判定，若结果为黑色则取消之。"
- `mod.targetInRange`：杀无视距离；`mod.selectTarget`：range[1]=∞；
  `useCardToBefore` forced priority 5.5：每目标判定，黑→
  trigger.cancel()（取消对该目标的结算，不进入闪询问）。

## 改动（分开陈述）

- **宿主行为**：`XianjianTianxian`（tag 208，锁定旗标=trigger_meta
  None/filter 恒否/step Done——效果三件内联）：
  ①杀无视距离：engine PlayCard `sha_range_free` 追加天弦臂（飞刃/
  飞弹同款，首个与额外目标距离检查共用）；
  ②目标数无限：engine `max_targets` 天弦臂（usize::MAX，上游
  selectTarget range[1]=∞ 同口径——sha 有限上限必满足其 !==-1 条
  件）；
  ③逐目标判定黑取消：杀机器 `!asked` 臂顶（每目标窗口位=asked 逐目
  标复位）判定，黑→advance_target 跳过该目标（不问闪不受伤）。**登
  记差异**：铁骑同款内联确定性判定（上游 judge 事件含鬼道/天妒改判
  窗，原生无）。无新状态字段、无新机器字段。
- **cards.json**：柳梦璃 tianxian mapping null→host + rehash 两步；
  **版本** 183→184；a0 追加 208。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`liumengli_tianxian_range_targets_
  and_judge_cancel`（5 人局：无技距离 2 单/双目标双拒绝→授技后距离
  2 双目标合法；判定序 2 号红=正常结算 4→3[闪 Pass]、3 号黑=取消
  不问闪；杀离手）。
- **全量通过**：engine-native **517 passed / 0 failed / 1 ignored**
  （v183 基线 516 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 183→184 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 柳梦璃升全闭环——xianjian 36 将中带登记暂缓技诸将（姜城 yanzhan
  viewAs、夏侯瑾轩 danqing、苏媚 dujiang、仙卿 xiaoyue、罗招嫣
  tanhua、慕容紫英 xuanning 获得路径、姜云凡炎杀授技）为差异登记态
  非批次缺口；未入包 3 将（长卿/居十方/越今朝）；长期项不变
  （zhenfa 五阵法、整包验收 0/37 等）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（12 文件，相对 noname-next
根）。
