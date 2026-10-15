# T12 swd 武将面批 83：久悠/临月

- 日期：2026-10-15
- 范围：两将转正——**73/79 名**（上批 71 将+2）；新增 HostBehavior tag 373–377
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（lexue/zhenjiu/lmazui 逐段核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd久悠 (wei hp3, 男) | 乐学 lexue | **整将**：SwdLexue(tag 373)=PhaseBegin forced——任一角色回合开始：自己回合授 {制衡/集智} 之一、他人回合授 {遗计/急救/鬼道/反馈} 之一（`ctx.add_skill_until_turn_end`，回合收束 `expire_temporary_skills` 移除）；登记差异：上游池含缔盟/驱虎（原生未实现），回合内池取已实现子集；上游 phaseBegin/phaseEnd 双池收敛为回合开始单窗 |
| swd临月 (wei hp3, 男) | 枕流 zhenjiu + 麻醉 lmazui | **整将**：SwdZhenjiu(tag 374)=ActivateSkill usable:1——SelectCard 红色手牌 → TargetChoice 其他未挂附角色 → off_court 挂附+授予 SwdZhenjiu2(tag 375)；触发面=其准备阶段（Judge begin）→红牌入手+回复 1+自清。SwdLmazui(tag 376) 同型黑牌挂附 → SwdLmazui2(tag 377) 触发面=其造成伤害前置（source 面）→黑牌返还+伤害 -1+自清 |

## 设计要点

- **内部触发面经 `skills` 数组注册**（cards.json 顶层 `skills` 段——`bridge_host_skills` 通道；先例：机关牌伴随技）。授技与实例插入沿用 SKILL_STATE 模式（天符挂附同族）。
- **麻醉触发为 source 面**（上游 `trigger: {source: "damageBegin"}`——首版误写 target 面由测试抓漏修正）。
- 登记差异：枕流/麻醉取牌按花色过滤（若 off_court 有他技挂附可能误取——目标挂附时校验 `off_court.is_empty()` 收窄）；麻醉返还为伤前（上游 damageEnd 归还）。
- 乐学 RNG 走 `core.rng.gen_range`（规则随机流，参与哈希）。

## 验证

- **定向回归** `swd_generals_b83_lexue_zhenjiu_lmazui`：
  - 枕流正例：红牌挂附 p1（off_court 断言）→推进至 p1 准备阶段→回 1（2→3）+红牌入手（off_court 清）。
  - 麻醉正例：黑牌挂附 p2→推进至 p2 出杀 p1→伤害 -1 不扣血（hp 不变）+黑牌伤前返还。
  - 乐学正例：回合循环后授予之一（在池/外池断言）。
- **包装载**：73 将入册（cards.json 71→73）。
- **全量**：engine-native **795 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。

## 结论

- **73/79 名**（两整将）。
- 四项用户决策维持待用户裁决清单登记。
