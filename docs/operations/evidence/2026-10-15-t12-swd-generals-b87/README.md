# T12 swd 武将面批 87：尉迟嫣红（幻形）

- 日期：2026-10-15
- 范围：一将 partial 转正——**76/79 名**（上批 75 将+1）；新增 HostBehavior tag 383–384
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（huanxing/huanxing2 逐段核验）

## 一将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd尉迟嫣红 (shu hp3, 女) | 幻形 huanxing（惑乱 meihuo defer） | SwdHuanxing(tag 383)=PhaseBegin(Judge) optional——SelectCard 弃一张 he → TargetChoice 男性其他存活 → 复制其将技能（非自己已持有）为技能实例+置在位旗标；SwdHuanxing2(tag 384)=Damage forced——受伤后移除全部授予实例+清旗标 |

## 设计要点

- **在位旗标复用内部状态技能实例**（`SWD_HUANXING_STATE_KEY` 注册于 cards.json skills 数组——免存档字段批，已启用技能随快照/内容锁自然往返）。
- 授予集移除按「其他玩家将技能名全表−自己本将技能」扫描（登记差异：不追踪具体授予集，同名他将技能亦会被移除的边界已收窄到本将排除）。
- 目标候选按 `effective_gender == Male`（上游 filterTarget `target.sex !== "male" → false`）。
- **惑乱 meihuo 登记结构性缺口**：触发面依赖「装备区牌离场且装备区为空且非主动使用」事件——原生当前无装备离场事件（Equipped 仅同栏替换面），与 Dismantle 机直删装备区路径不构事件，挂后续基建批。

## 验证

- **定向回归** `swd_generals_b87_huanxing`：全链（推进至准备阶段确认窗→弃牌→男性候选断言→选中 p1 后 `debug_has_skill` 断言复制技能实例在位）。
- **包装载**：76 将入册（cards.json 75→76）。
- **全量**：engine-native **799 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **76/79 名**（剩余 3 将：车芸/端蒙/姜子牙）。
- 四项用户决策维持待用户裁决清单登记。
