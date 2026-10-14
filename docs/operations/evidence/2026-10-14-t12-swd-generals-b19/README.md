# T12 swd 武将面批 19：五技五将——伤害线批量转正（2026-10-14）

## 承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 承剑 chengjian（轩辕剑侍，optional 确认） | 其他角色受到杀伤害后来源摸 1 | tag 266：global Damage 触发 + optional 确认门 + 来源摸 1 |
| 缚斧 jufu（程咬金，forced） | 杀伤害 +1（需武器） | tag 267：DamageBefore 帧直写 num+1（乱剑先例）+ 武器栏 filter |
| 花镜 huajing（秦叔宝，frequent 自动） | 锦囊牌伤害后回 1 摸 1 | tag 268：Damage 触发 + 锦囊判定（三类减基本/装备）+ 回 1（已受伤门）摸 1 |
| 护盾 hudun（子达书，forced） | 造成伤害后自己无护甲则 +1 | tag 269：Damage 触发 + change_hujia |
| 破星 poxing（张高，forced） | 对体力高于自己的目标伤害 +1 | tag 270：Damage 触发 + 目标 hp>持有者 filter + 帧直写 amount+1 |

## 过程要点

- **filter 臂遗漏由回归抓漏**：首版五触点重放遗漏 SwdPoxing filter 臂——
  `_ => false` 兜底吞机（破星不触发，hp 只扣 1）——grep 五技 matches 检查
  补齐后转绿。接线清单：枚举/tag/key/from_tag/ALL/trigger_meta/**filter**/
  提取/step 九项逐一核验。
- 承剑断言语义：来源（出杀者）摸 1 与出杀 -1 净持平。
- 花镜回 1 有已受伤门（满血饱和）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_swd **30/30** |
| engine-native 全量 | **748/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**7/79 名、7/241 技转正**（韩罗/造之仁君式整将 0——本批五将
  全部部分转正；cp：韩罗/习嫣/轩辕剑侍/程咬金/秦叔宝/子达书/张高各 1-2 技）。
- 批 20 候选： continuation（缚斧已含伤害直写；其余 trigger 型按将勘察）。
