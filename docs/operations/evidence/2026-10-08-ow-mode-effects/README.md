# U05 ow 纵切批八：堡垒模式效果钩（规则 v125，2026-10-08）

ow 纵切批八：补做 v124 批登记待做的**模式效果钩**——哨卫模式（
baolei_mode==1）：①出杀无次数限制 ②杀无视距离 ③目标侧不可闪避（目
标持技+哨卫模式时闪选项过滤）+ 源侧直中（ShaResolve 仁王盾后哨卫直
中，跳过闪询问与防具）；消 v124 的「模式效果钩登记待做」。前置：
[ow 纵切批七](../2026-10-08-ow-baolei-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js bshaowei_sha subSkill）

- mod targetInRange（杀）true / cardUsable Infinity（无次数）/
  trigger shaBegin directHit（不可闪避+无视防具）——哨卫模式双向生效
  （player+target 双 trigger）。

## 改动（分开陈述）

- **代码已实现**：PlayCard Sha 门控——哨卫模式（来源持 bshaowei &&
  baolei_mode==1）①杀次数豁免（与 paoxiao/has_zhuge 并列）②距离豁免
  （baolei_range_free 与 zfs_free 并列）；ShaResolve——③哨卫源侧直中
  （仁王盾后 emit_sha_damage 复用，跳过闪询问/防具/雌雄等——上游
  directHit 同口径）④目标侧闪选项过滤（目标持技+哨卫模式 → shans 过
  滤为空 → 无闪可出）。消 v124「模式效果钩登记待做」。
- **登记差异**：directHit 同时无视防具（八卦/仁王盾等不判定）——与上
  游 directHit 同口径；雌雄双股剑等非闪避分支不受影响。
- **转换包**：cards.json 堡垒 general 补 zhencha_draw mapping（侦察摸
  回子技注册），三哈希重算。

## 验收口径

- **回归已加入**：`baolei_mode_effects_on_sha`——哨卫源侧直中（1 号有
  闪不掉血，4→3）→ 1 号无哨卫模式时正常可闪/无闪则掉血（3→2）。
- **全量通过**：engine-native **438 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 124→125 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（哨卫不在标准场景，纯版本位移）。
- **剩余**：侦察摸回触发已接线（v125 filter CardUsed Sha+mode2）——
  效果回归并入批九；ow 其余 19 武将、xianjian 36 武将逐武将纵切；
  U07 duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
