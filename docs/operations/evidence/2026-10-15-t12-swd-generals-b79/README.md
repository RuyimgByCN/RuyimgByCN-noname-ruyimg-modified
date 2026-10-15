# T12 swd 武将面批㊞：张烈/轩辕剑仙

- 日期：2026-10-15
- 范围：两将 partial 转正——**66/79 名**（上批 64 将+2）；新增 HostBehavior tag 366
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（poxiao 三面/huajian 复用核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd张烈 (qun hp4, 男) | 破晓 poxiao（huajin 授技面 defer） | SwdPoxiao(tag 366)=锁定 marker **三查询点**：①viewAs 链装备牌→杀（万钧同型）②computed_distance minus 链无武器 -1（疾风同位）③max_targets 链无武器 +1（连舞同位） |
| swd轩辕剑仙 (qun hp4, 男) | 花间 huajian（**def 复用零代码**——批㉟已实现；pozhou 计数/xuanyuan 召剑 defer） | cards.json 映射即生效 |

## 设计要点

- **engine.rs 编辑脚本断言失败会整体回滚**：多编辑脚本中间 assert 失败时已匹配的编辑不落盘（本次 viewAs+max_targets 两臂回滚，第二次脚本重做）；长脚本改用增量 if-check 模式。
- 破晓无武器攻击距离 -1 与杀目标上限 +1、有武器仅 viewAs 面——三面独立查询点互不门控。

## 验证

- **定向回归** `swd_generals_b79_poxiao`：破晓装备牌当杀（有武器下 viewAs 面独立）+装备消耗；无武器距离 -1 对座 p2 入范围受 1（新回合防杀限）；花间 def 复用虚拟杀。
- **全量**：engine-native 791 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **66/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
