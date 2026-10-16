# T12 swd deferred 消解批 99：化金/明镜

- 日期：2026-10-16
- 范围：两技 deferred 消解（tag 408-410）——**133/184 技条目已映射**（deferred 51→49）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（huajin/huajin2/huajin3、minjing/guangshatianyi 逐段核验）

## 两技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 化金 huajin（张烈） | 出牌阶段限一次弃一张牌，令你造成的伤害 +1 直到你的下一回合开始 | SwdHuajin(tag 408)=ActivateSkill usable:1：SelectCard 弃 1 he → 授予 `swd_huajin2`；SwdHuajin2(tag 409)=DamageBefore 来源侧 amount+1，PhaseBegin（准备=Judge begin）自清 |
| 明镜 minjing（妮可） | 无防具时视为装备光纱天衣——即将受伤 1/3 概率伤害 -1 | SwdMinjing(tag 410)=DamageBefore 目标侧；filter=无防具；效果步 `rng%3==0` 则 amount-1（登记差异：上游在 filter 掷点，原生 filter 只读） |

## 验证

- **定向回归** `swd_deferred_b99_huajin_minjing`：化金弃代价+伤害 4→2+限一次反例+下回合准备自清；明镜 60 次内至少一次减至 0。
- **全量**：engine-native 816 passed / 0 failed / 1 ignored（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。

## 结论

- **133/184 映射，deferred 51→49**。
