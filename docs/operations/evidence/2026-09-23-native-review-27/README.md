# 2026-09-23 第 27 轮：趁火打劫 + 索链甲原语（规则版本 38）

按运筹帷幄包数据登记的 reason 再增补两个宿主原语并扩展切片至 **14/22**：
**趁火打劫**（`ChenhuodajieCard`：获得目标一张手牌，锦囊无懈窗口包装；
登记差异：上游为目标任选可见牌，原生取首张确定性获得）与**索链甲**
（`CardKind::Suolianjia` 防具 + `link_on_equip` 横置钩子：装备时横置
计入铁索、离场时重置；装备技伤害修改部分登记暂缓）。
`STANDARD_RULESET_VERSION` **37 → 38**（`CardKind::Chenhuodajie`/`Suolianjia`、
`CardDef.link_on_equip`；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **302 passed / 0 failed**（第 26 轮 300 + 趁火打劫/索链甲 2 项）。全工作区 4,868 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v38 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 回归（g1_shahai_yunchou 新增 2 项 + 切片计数更新）

- `chenhuodajie_gains_target_hand_card`：获得目标首张手牌 + 无懈窗口直结算。
- `suolianjia_links_on_equip_resets_on_leave`：装备横置 + 横置状态随快照恢复。

## 边界（如实登记）

- 运筹帷幄切片 **14/22**（暂缓 8 张：洪水 1[自-target 延时锦囊判定原语]、
  偷梁换柱 2[换牌双选]、草船借箭 2[多步+代用杀]、隔岸观火 2[多目标拼点链]、
  舌战群儒 2[舌战体系]）；整包验收仍 **0/37**。
- 索链甲装备技的伤害修改部分（damageBefore 滤 unequip）暂缓；本批仅交付
  横置/重置生命周期。
