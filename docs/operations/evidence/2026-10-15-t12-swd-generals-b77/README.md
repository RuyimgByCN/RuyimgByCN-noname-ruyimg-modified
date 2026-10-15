# T12 swd 武将面批㊲：慧彦/秦壬

- 日期：2026-10-15
- 范围：两将 partial 转正——**63/79 名**（上批 61 将+2）；新增 HostBehavior tag 362-364
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（hwendao/lingfeng/tianhuo 逐段核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd慧彦 (qun hp4, 男) | 横刀·慧彦面 hwendao+凌锋 lingfeng（hxunzhi limited defer） | SwdHwendao(tag 362)：双事件回合外用牌/响应结算确认门——当前回合角色二选一窗（弃同花色 he 牌或被随机获得一张，RNG 登记差异上游自选 gainPlayerCard）；SwdLingfeng(tag 363)：Phase{Play,end} 直发窗口——出牌数门 cards_used_play_phase ≥ min(3,hp) → TargetChoice 攻击距离内→Confirm 窗（Confirm=1 伤，取消=自获 1 护甲） |
| swd秦壬 (wei hp3, 女) | 天火·秦壬面 tianhuo（huanyin 判定改写/xuanzhou defer） | SwdTianhuo(tag 364)=ActivateSkill usable:1：座位序判定区有牌的其他角色各——弃判定牌全部+受等量火伤（nosource 登记差异：原生 Damage 需 source 改记 owner） |

## 设计要点

- **新行为必须加 filter 臂**：filter match 有 `_ => false` 兜底——漏加 filter 臂不编译报错、运行时永不触发（凌锋首版漏加实证）；新行为检查单含 filter 臂一项。
- 凌锋出牌数门=cards_used_play_phase（上游 player.countUsed() 出牌阶段计数同口径登记）。
- 横刀花色锚=响应牌花色（respondAfter 载荷 card），同花色候选=当前回合角色 he 合集过滤。

## 验证

- **定向回归** `swd_generals_b77_hwendao_lingfeng_tianhuo`：横刀确认→同花色候选（动态花色匹配勘误——闪全 diamond，锚取响应牌花色）→p1 弃同花色牌+其余保留；凌锋出牌数达标→目标窗→取消分支自获 1 护甲+无额外伤害；天火判定牌全弃+受等量火伤（p1 hp 基线重置勘误）。
- **全量**：engine-native 789 passed / 0 failed（golden 零扰动；冻结键数 361 漏更由 a0 拦截后按实测修正）。
- **Gate 1b**：exit 0。

## 结论

- **63/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
