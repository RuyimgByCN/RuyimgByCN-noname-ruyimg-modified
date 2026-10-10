# U05 swd 第四批：赤龙牙（规则 v88，2026-10-08）

swd 子包第四批转换：赤龙牙（♦3，equip1 武器射程 2，锁定技——你造成的
火属性伤害 +1）。前置：[swd 第三批](../2026-10-08-u05-swd-batch3/README.md)。

## 上游语义核验（源码为准）

- `card/swd/card.js` `chilongya`：equip1，`distance: { attackFrom: -1 }`
  （射程 2），`skills: ["chilongya"]`。
- `skill.js` `chilongya`：equipSkill、`trigger: { source: "damageBegin" }`、
  forced、filter `hasNature("fire") && notLink()`、content `trigger.num++`。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Chilongya`（编码 107：archive enc/dec、a0
  variant/ALL 同步；`EquipSlot::Weapon` + `named_weapon_range Some(2)`）；
  commit_damage 火伤增幅——来源存活且装备赤龙牙时 `amount += 1`（Fire
  only；铁索传导为独立 Damage 事件照常 +1，对应上游 damageBegin 每事件
  `num++` 语义）。
- **转换包**：cards.json 增 ♦3 赤龙牙 + 哈希重算（转换包现 9 张）。

## 验收口径

- **回归已加入**：`swd_chilongya_amplifies_fire_damage`——火伤 1→实际 2
  （+1）、非火 1 点照常、射程 2。
- **全量通过**：engine-native **403 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 87→88 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无赤龙牙，纯版本位移）。
- **剩余**：swd 其余 13+ 卡种逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
