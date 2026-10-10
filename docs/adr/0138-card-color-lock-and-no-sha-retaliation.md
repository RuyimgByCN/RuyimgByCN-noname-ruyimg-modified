# ADR 0138：技能实例颜色锁与阶段末无杀反击

## 决策

Rule VM ABI 48 增加两个类型化效果和一个通用 modifier：

```text
choose_discard_matching_equip_color_add_skill(player, target, skill_def)
use_virtual_sha_from_card_color_lock_source_if_no_sha(player)
card_action_cancel_for_skill_color_mask
```

第一个效果让发动者可取消地选择一张手牌或装备区的非基本牌；该牌的有效红黑颜色必须与发动者
当前至少一张装备牌相同。选择恢复时重新校验技能实例、阶段、双方存活、牌的所有权、区域、逻辑
类别和装备颜色，再先弃置成本、后授予目标技能，伪造或过期选择不会产生部分结果。

颜色锁技能使用带魔数和版本的实例状态保存来源 `PlayerId` 与两位红黑掩码。重复授予同一定义时
合并颜色掩码，并用最后一次发动者替换来源；不会叠加多个辅助技能实例。modifier 仅按实体材料的
有效颜色过滤主动使用、响应打出和濒死救援，颜色无关的无实体牌响应不受影响。

阶段结束时，第二个效果只在目标本出牌阶段没有使用【杀】且记录来源仍存活时生效。来源对目标
使用一张无花色虚拟【杀】，复用普通目标、响应、伤害与结算管线，但忽略距离和普通【杀】次数；
辅助技能在该次结算后统一移除。核心不识别「司敌」、曹真或任何具体技能定义 ID。

旧 JavaScript 只用于确认可取消成本、颜色累计、最后来源覆盖和 `canUse(..., false)` 的语义，不进入
编译产物或运行时。

## 版本

- Rule VM ABI：48
- Rule IR：59
- Engine semantics：134
