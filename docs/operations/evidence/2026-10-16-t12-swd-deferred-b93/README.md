# T12 swd deferred 消解批 93：孵鸟/凝弦/妖童

- 日期：2026-10-16
- 范围：三技 deferred 消解（tag 395-397）——**121/184 技条目已映射**（deferred 65→62）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（funiao/ningxian/yaotong 逐段核验）

## 三技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 孵鸟 funiao（恒爱） | 出牌阶段限一次将一张手牌交给一名其他角色，摸 1 并观看其手牌 | SwdFuniao(tag 395)=ActivateSkill usable:1：SelectCard→TargetChoice→交牌（CardsGiven）+摸 1（观看=展示面登记差异，原生无隐藏信息） |
| 凝弦 ningxian（恒爱） | 受伤后可弃任意张黑色牌并选等量其他角色各受 1 伤 | SwdNingxian(tag 396)=Damage optional：确认→SelectCards 黑 he（1..N）→SelectPlayers 等量（min=max=N）→逐个 1 伤 |
| 妖童 yaotong（齐） | 手牌奇数→任意手牌当杀或闪；偶数→当无懈 | SwdYaotong(tag 397)=marker：viewAs 链臂——手牌数%2==1 时任意手牌→杀（**闪响应面/无懈偶数面登记 defer**——响应向 viewAs 为独立引擎面） |

## 登记差异

1. **孵鸟**：观看手牌为展示动作，原生无隐藏手牌信息面——略去（登记）。
2. **凝弦**：目标数=弃牌数硬绑定（min=max），上游 selectTarget 动态联动同口径；目标去重由 SelectPlayers 语义承载。
3. **妖童**：仅使用向杀转化；响应向（当闪打出）与偶数向（当无懈）需响应窗 viewAs 基建——**登记 defer**（与 jqimou「使用牌续段」同族登记）。

## 验证

- **定向回归** `swd_deferred_b93_funiao_ningxian_yaotong`：孵鸟交牌 p1+1+自己净不变（2-1+1=2）；凝弦弃 2 黑→p2/p3 各 1 伤+p0 受杀 1；妖童奇数手牌闪当杀命中+偶数反例拒。
- **全量**：engine-native 810 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **121/184 映射，deferred 65→62**。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
