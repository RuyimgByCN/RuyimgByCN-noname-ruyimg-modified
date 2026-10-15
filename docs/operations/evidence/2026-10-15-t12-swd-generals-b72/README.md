# T12 swd 武将面批㉜：恒爱/Miles/康那里士/赵云

- 日期：2026-10-15
- 范围：四将 partial 转正——**49/79 名**（上批 45 将+4）；新增 HostBehavior tag 349-350
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（hlingbo/miles_xueyi/xuying/longdan 逐段核验）

## 四将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd恒爱 (shu hp3, 女) | 凌波·恒爱面 hlingbo（funiao/ningxian defer） | SwdHlingbo(tag 349)：双事件（CardResponded primary + CardUsed extra）闪结算 frequent 摸 2 |
| swdMiles (qun hp4, 男) | 血裔·Miles 面 miles_xueyi（aojian/mohua2 defer） | SwdMilesXueyi(tag 350)：DamageBefore prevented 写回+**无条件** LoseHp 1（空手同失去——与虚影条件面对照） |
| swd康那里士 (qun hp1, 男) | 虚影 xuying（**def 复用零代码**——淳于衡同技；busi/yinguo defer） | cards.json 映射即生效；空手=完全免伤对照面 |
| swd赵云 (shu hp4, 男) | 龙胆 longdan（**builtin 白名单复用**；pozhen/tanlin defer） | 龙胆已在白名单——零代码；黑杀响应候选+闪避验证 |

## 设计要点

- **血裔 vs 虚影对照**：同为「伤害取消+失去 1」家族——血裔无条件失去（空手同失）、虚影有手才失去（空手完全免伤）——两语义在批内互为对照测试面。
- 凌波·恒爱面双事件 meta event=CardResponded + extra CardUsed（浴尘/挟雷同型，frequent=非 optional）。

## 验证

- **定向回归** `swd_generals_b72_hlingbo_miles_xueyi`：凌波闪响应摸 2（出 1 摸 2）；血裔空手同失去（3→2）；虚影复用空手完全免伤（与血裔对照）；龙胆黑杀入响应候选+闪避。
- **全量**：engine-native 784 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0（陈旧布阵块清理后复跑绿）。

## 结论

- **49/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
