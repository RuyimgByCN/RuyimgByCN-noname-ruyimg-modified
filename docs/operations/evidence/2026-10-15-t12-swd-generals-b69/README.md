# T12 swd 武将面批㉙：玉澧/喀玛

- 日期：2026-10-15
- 范围：两将转正（玉澧 partial+喀玛整将）——**39/79 名**（上批 37 将+2）；新增 HostBehavior tag 341-342
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（lingxin/yueren/shangshi 逐段核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd玉澧 (wu hp3, 女) | 灵心 lingxin（天香非原生 defer） | SwdLingxin(tag 341)：PhaseEnd frequent——牌堆顶三张（pop×3 含洗弃堆防御）：红桃入手、其余弃置 |
| swd喀玛 (qun hp3, 女) | 渔人 yueren+伤逝 shangshi（**builtin 白名单 7→8**，伤逝在冻结键表） | SwdYueren(tag 342)=marker：sha 机器 !asked 门内联臂——确定性判定：黑→弃目标 he 随机一张（RNG 同上游 discardPlayerCard forced）；红→摸 1（**登记差异：上游收回此杀实体**——实体回流与在结机器持有冲突改摸 1）；每杀一次=!asked 门 |

## 登记差异

1. **渔人红面**：上游收回此杀实体进手牌——原生 sha 结算机器持有实体、回流会破坏守恒（手牌+桌面双区），改摸 1 等价补偿（登记）。
2. **渔人判定**：内联确定性口径（同铁骑，无鬼道改判窗）。
3. 玉澧天香：原生标准目录无天香（冻结键表无）——defer。

## 验证

- **定向回归** `swd_generals_b69_lingxin_yueren`：灵心压顶三张（红桃×2 入手+黑♠弃置+净增 2）；渔人黑判→目标 he 随机弃 1+杀照常结算；红判→摸 1（出 1 摸 1 净不变）+杀照常结算。
- **全量**：engine-native 781 passed / 0 failed（无哈希面改动——golden 零扰动）。
- **Gate 1b**：exit 0。

## 测试布阵勘误（入记忆）

- **压顶序**：debug 压顶三张按「最后压=先取」pop 序排列期望。
- **净手牌断言**：出牌+摸牌对冲时以净变化断言（出 1 摸 1=不变），勿按摸得断言。

## 结论

- **39/79 名**（upstream 79 将 183 技）；整将累计二十五（+1：喀玛）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
