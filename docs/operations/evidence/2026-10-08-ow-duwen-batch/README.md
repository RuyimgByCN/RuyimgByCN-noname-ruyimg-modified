# U05 ow 纵切批五：毒瘟（黑百合）（规则 v122，2026-10-08）

ow 纵切批五：**毒瘟**（duwen，黑百合单技能）映射 `host:wudian_duwen`——
锁定技：造成伤害时若你手牌数与受伤害角色相等，伤害 +1（铁索传导为独
立事件照常判定）。commit_damage 钩直读双方手牌数（赤龙牙/刺猬同款）。
前置：[ow 纵切批四](../2026-10-08-ow-shouhu-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js duwen）

- `duwen`：trigger source damageBegin、forced、filter 手牌数相等 &&
  notLink、content `trigger.num++`。

## 改动（分开陈述）

- **代码已实现**：`WudianDuwen`（tag 114，权限型 Noop）——
  commit_damage 钩（护甲块前）：来源持 duwen 实例 && 来源手牌数==目标
  手牌数 → amount+1。mapping 接线（黑百合 duwen）。
- **登记差异**：无（纯数据钩语义等价）。
- **转换包**：cards.json 增 黑百合（generals）+ duwen skills 条目，三
  哈希重算。

## 验收口径

- **回归已加入**：`heibaihe_duwen_equal_hand_damage_bonus`——①双方各
  1 张（出杀后 1=1）伤害 +1（4→2）②手牌不等（2≠1）无加成（2→1）。
- **全量通过**：engine-native **435 passed / 0 failed / 1 ignored**
  （测试清单 436 项；注：此前各批 awk 汇总口径虚高至 439，本轮以逐二
  进制实测为准）；依赖包 **63/0**（content-runtime 33 + engine-ai 29
  + content-native-loader 1）。
- **golden**：版本 121→122 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（毒瘟不在标准场景，纯版本位移）。
- **剩余**：ow 其余 19 武将、xianjian 36 武将逐武将纵切；U07
  duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
