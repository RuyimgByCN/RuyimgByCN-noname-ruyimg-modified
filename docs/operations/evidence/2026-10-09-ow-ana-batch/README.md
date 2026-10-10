# U05 ow 纵切批十四：安娜支援（规则 v131，2026-10-09）

ow 纵切批十四：**安娜**（ow_ana，wei 3 女）——支援（zhiyuan，伤害前
置窗口触发）单技闭环；**mianzhen 暂缓**（目标侧 chooseToRespond 响应
询问+mianzhen2 永久子技）+ **aqianghua 暂缓**（给牌+护甲+aqianghua2
子技链）。上游 25 将已闭环第 13 将。前置：
[ow 纵切批十三](../2026-10-09-ow-falaozhiying-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js zhiyuan）

- trigger source damageBefore（check 询问式）：filter event.num>0；
  content：trigger.cancel()（伤害取消）+ trigger.player.recover(num)
  （受伤者回复等量体力）。

## 改动（分开陈述）

- **代码已实现**：`WudianZhiyuan`（tag 128，EventKind::DamageBefore 前
  置窗口触发、optional:true=上游 check 询问式同口径；触发路径自动包
  OptionalGate 确认门）——filter（来源=持有者 && 未防止 && 量>0）→
  确认后 Act：`current_damage_mut().prevented = true`（碎盾 v? 直写先
  例）+受伤者回复等量体力（满血饱和=既有 recover 路径）。
  **DamageBefore 前置窗口首个 ow 消费者**（窗口基建自伊籍/碎盾等标准
  批已就绪）。
- **登记差异**：无新增（取消+回复一一对应；回复在满血时无效果同上游
  recover 饱和）。
- **登记暂缓**：mianzhen（响应询问+子技）、aqianghua（子技链）。
- **转换包**：cards.json 增 安娜（wei 3 女，zhiyuan mapping+双暂缓
  reason）+ npack 资产哈希重算。无新状态字段——存档 v9 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`ana_zhiyuan_cancel_damage_and_
  equal_recover`（确认门 observer=持有者/确认→伤害防止体力不变+受伤
  者回复 1（2→3）/取消（空选择）→伤害照常（3→2）/无残留挂起）。
- **全量通过**：engine-native **452 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v130 基线 451 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 130→131 入 canonical_hash——14 场景全 mismatch
  （纯版本位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：OptionalGate 确认门的取消=空 selection（Pass 非
  options 成员——v126 同款教训第三次命中）；测试探测补丁半途改写导致
  确认被应答两次（answer_open 与手动提交并存）——删重复行后恢复。

## 剩余

- ow 其余 12 武将逐武将纵切；xianjian 36 武将；虚拟 useCard 原语
  （huoyu/youzhan 同源）、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
