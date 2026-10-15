# T12 swd 武将面批㉛：子巧/莉莲/吉挺

- 日期：2026-10-15
- 范围：三将转正（子巧整将+莉莲/吉挺 partial）——**45/79 名**（上批 42 将+3）；新增 HostBehavior tag 345-348
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（guaili/fuyan/wuxie/guanhu 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd子巧 (shu hp3, 女) | 乖戾 guaili+敷衍 fuyan（**整将**） | SwdGuaili(tag 345)=双事件同机分流（DamageBefore extra_events[Damage]）：BeforeTriggers 面杀伤害 +1（非铁索 from_chain 过滤，缚斧直写同款）；AfterTriggers 面弃两张手牌（不足弃全部）——**按帧 stage 字段区分 Before/After（kind 同为 Damage 不可辨）**；SwdFuyan(tag 348)=受伤后确认令一名无护甲角色 +1 护甲（change_hujia） |
| swd莉莲 (qun hp3, 女) | 无邪 swd_wuxie（青城判定循环 defer） | SwdWuxie(tag 346)=marker：engine Le/ShanDian 校验臂——延时锦囊指定他人拒（基础路由本拒自置=乐/兵整体不可用，闪电自置不受限登记） |
| swd吉挺 (wei hp4, 女) | 观虎 guanhu（灵视多面 defer） | SwdGuanhu(tag 347)：确认门→SelectCards 自选目标 he 合集至多 2 张弃置（上游 discardPlayerCard [1,num] 可选面保留） |

## 设计要点

- **Before/After 同载荷同 kind——帧 stage 字段区分**：Damage 帧在 BeforeTriggers/AfterTriggers 两窗口各跑一次机器（extra_events 双事件），stage 枚举是唯一区分点（frame.kind 恒为 Damage）。
- 无邪挂 engine Le/ShanDian 校验臂（判官/无心同段）；skill 名=swd_wuxie（上游键，避免与标准「无懈」歧义）。

## 验证

- **定向回归** `swd_generals_b71_guaili_fuyan_wuxie_guanhu`：乖戾杀 +1（4→2）+弃两张；敷衍确认→p0（无护甲候选）+1 护甲；无邪延时对他人拒+自置亦拒；观虎确认→手牌+装备各弃一张+杀照常结算。
- **全量**：engine-native 783 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 测试布阵勘误（入记忆）

- **双事件同机分流用 frame.stage**：Before/After 面载荷与 kind 全同，frame.stage（BeforeTriggers vs AfterTriggers）是唯一判别点。
- **持技跨段污染**：乖戾（+1 持续技）跨段未撤→④段杀致 2 伤——持续型技测试段间撤销同 viewAs 先例。
- **乐/兵基础路由拒自置**：无邪「只能自置」在原生=乐/兵整体不可用（闪电自置不受限）。

## 结论

- **45/79 名**（upstream 79 将 183 技）；整将累计二十六（+1：子巧）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
