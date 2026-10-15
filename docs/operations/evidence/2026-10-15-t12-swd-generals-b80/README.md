# T12 swd 武将面批㊟：司空宇

- 日期：2026-10-15
- 范围：一将 partial 转正——**67/79 名**（上批 66 将+1）；新增 HostBehavior tag 367
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（sliufeng 逐段核验）

## 一将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd司空宇 (wu hp4, 男) | 疾风·司空宇面 sliufeng（linyun 两牌当杀+hutian 护天 defer） | SwdSliufeng(tag 367)=锁定 marker：engine 杀距离校验臂新增豁免——目标 hp≤自己 hp 时无视距离（上游 mod targetInRange 同口径） |

## 设计要点

- 杀距离校验臂豁免第四查询（zfs/baolei/feiren/zhuyue 后追加 sliufeng）：按目标 hp 门控（区别于全目标豁免的 feiren）。

## 验证

- **定向回归** `swd_generals_b80_sliufeng`：p2（hp2≤3 对座）入范围受 1；反例 p3（hp4>3 对座）出界拒。
- **全量**：engine-native 792 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **67/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
