# T12 swd 武将面批㉗：小黄龙/虎魄/司徒强/暗卡

- 日期：2026-10-15
- 范围：四将转正（皆 partial）——**34/79 名**（上批 30 将+4）；新增 HostBehavior tag 333-337
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（yeying/guiying/fengze/lingyue/anlianying 逐段核验）

## 四将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd小黄龙 (wei hp3, 男) | 夜影 yeying（聚悉 deferred） | SwdYeying(tag 333)=marker：viewAs 链臂黑牌→乾坤镖（单目标；目标+距离 1 邻座各弃一张——机器已有 JiguanQiankunCard） |
| swd虎魄 (wu hp3, 男) | 归影 guiying（遁行 deferred） | SwdGuiying(tag 334)=marker：viewAs 链臂黑牌→偷梁换柱（单目标换牌机 TouliangSwap 已有） |
| swd司徒强 (shu hp3, 女) | 凤泽 fengze+玲月 lingyue（锦鳞 limited defer） | SwdFengze(tag 335)=marker：viewAs 链臂黑牌→桃园结义（TaoyuanMachine 已有）；SwdLingyue(tag 336)=marker：sha 机器 !asked 门内联臂——目标弃 he 首张（确定性登记差异上游自选） |
| swd暗卡 (qun hp3, 男) | 暗莲映 anlianying（颂词 NOT FOUND/subskill defer） | SwdAnlianying(tag 337)：CardsDiscarded+自己手牌空→frequent 摸 2（登记差异：上游 loseEnd 全丢失面≈弃置面） |

## 设计要点

- **三个 viewAs marker 共用链臂模式**（万钧/火扑批㉖同型）：夜影/归影/凤泽黑色牌分别转乾坤镖/偷梁换柱/桃园结义——三种目标机全部已有，零新机器。
- 玲月=杀机器内联确定性臂（雌雄/天弦位置），零快照字段。
- **无 TurnState/快照/哈希改动——golden 零扰动全绿。**

## 验证

- **定向回归** `swd_generals_b67_yeying_guiying_fengze_lingyue_anlianying`：夜影乾坤镖弃牌（p1 唯一手牌被弃）；归影偷梁换柱换牌语义（v2 换入己方+own 入目标+v2b 保留）；凤泽桃园结义双目标回血；玲月目标弃 he 首张+杀照常结算；暗莲映拆桥弃空→摸 2。
- **全量**：engine-native 779 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 测试布阵勘误（入记忆）

- **viewAs 同持链序**：测试段间未撤前段技→链序先命中（yeying 在 guiying 前→黑牌全转乾坤镖）——段间 state_core_mut().skills.remove 撤销。
- **viewAs 测试原料**：viewAs 链外层守卫跳过 Sha/Le——黑牌原料须取黑锦囊（顺手牵羊）；机关/异类 kind 会走专臂（bc2 曾取到机关牌路由错机）。
- **乾坤镖语义**：目标+邻座各弃一张（非伤害）——he 空者跳过。
- **偷梁换柱语义**：单目标换牌（双方须有手牌）非弃牌。
- **弃牌阶段无法清空手牌**：弃牌数=手牌-hp 恒留 hp 张——「手牌空」面测试用过河拆桥路径。

## 结论

- **34/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
