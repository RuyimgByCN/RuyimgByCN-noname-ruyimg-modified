# T12 swd 卡牌面窗口机批（b62）：神秘果/鼠槌/萨登荆环/昊天塔

- 日期：2026-10-15
- 范围：swd 卡牌面四件窗口机行为面收口——**卡牌面 92 行全量转正**（对账面闭环；余仅「包内多余 13 张处置」数据决策）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/card/swd/{card.js,skill.js,translate.js,list.js}`（g_shenmiguo skill.js:250-315 / shuchui skill.js:366-409 / sadengjinhuan skill.js:415-438 / haotianta skill.js:（equipSkill judgeBefore）逐段核验）

## 四件语义与接线

| 卡 | 上游 | 原生接线 |
|---|---|---|
| 神秘果 ×2（♠2/♥1，basic） | 出牌阶段用基本/普通锦囊结算后，chooseToUse 神秘果→createCard 复制品再结算；shenmiguo2 临时技=限一次+循环守卫 | HostBehavior::SwdShenmiguoWindow(tag 315，CardUsed after+optional 确认门)；jiqi InitHook 授全员+hand_has_kind 门控；TurnState.shenmiguo_used=限一次与循环守卫共用；载体 CardUsed 发射+Shenmiguo commit 臂弃置 |
| 鼠槌（equip5） | 出牌阶段限一次，指定攻击范围角色，依次至多 3 张杀，任一杀 damageEnd（shuchui2 charlotte）→终局摸 1 | SwdShuchuiUse(tag 316，ActivateSkill 通道 usable:1)；凤略 Yield 循环同型（每次重扫手中首张杀）；伤害检测=TurnState.shuchui_armed/hit+commit_damage 钩；equip_skills=["shuchui"] 真实装备通道 |
| 萨登荆环（equip5） | 杀被闪（shaMiss）可选判定，红→目标额外打出一张闪，未打出→shaHit 改写 | ShaResolveMachine 内联窗（asking_sadeng/asking_sadeng_shan 两旗）：青龙→贯石斧→荆环确定性次序；判定=内联确定性口径（同铁骑）；命中改写走寒冰/麒麟窗（sadeng_continue_to_damage） |
| 昊天塔（equip5） | 任意角色判定前（global judgeBefore，锁定 direct），持有者观看牌堆顶 2 强选 1 为判定结果（trigger.cancel 绕过改判） | JudgePhaseMachine 判定翻牌点插窗（JudgementProgress +haotian_holder/haotian_other）：强制选一（min=1 allow_cancel=false）、次候选弃置、rewriters/tian_du 队列置空跳过 |

## 登记差异（全部入规则基准语境）

1. **神秘果**：上游凭空复制同花色点数新实体（原件先入弃堆），原生原体自弃堆上桌面复用——再结算窗口内弃堆少一张观测差；view-as 转化用牌（use_as≠实体牌种）、五谷（Remained 面未实现）、机关异类、延时锦囊不进再结算白名单；嵌套用牌（parent 链）不区分——每阶段旗封顶。
2. **鼠槌**：伤害检测=commit_damage 钩（source=回合主+杀牌种+非 prevented）——上游 shuchui2 charlotte 同口径；不限击数提升面（activate 门 `!sha_used_play_phase` 保守收窄，连弩持有者登记差异）；凤略循环同型=Yield 跨帧，机器直发不经 PlayCard 校验。
3. **萨登荆环**：判定无鬼道/天妒改判窗（内联确定性，同铁骑口径）；三窗并发次序青龙→贯石斧→荆环（上游同点触发技按目标序）。
4. **昊天塔**：改判/天妒窗随插窗跳过（上游 trigger.cancel 取消判定事件=结果触发技一并绕过，登记差异同语境）；多持有者仅座序首位开窗（上游逐人触发时序未定义）。

## 字段批与验证

- **字段批**：TurnState +3（shenmiguo_used/shuchui_armed/shuchui_hit）+Sha 快照 +2（asking_sadeng/asking_sadeng_shan）+JudgementProgress +2（haotian_holder/haotian_other）——**存档 v74→v75、规则 v287→v288、golden 14 场全量重录**。
- **版本回退探针**：v287+去三旗哈希复跑 → 14/14 与旧 golden 逐字节一致（`/tmp/b62_probe_pairs.txt` 对照 `/tmp/b62_old_pairs.txt`，git HEAD 提取）——位移唯一来源=版本字段+新字段。
- **定向回归**：`swd_window_machines_b62`（神秘果原杀+再结算 2 伤/旗守卫反例[第二张果不消耗]/载体簿记；鼠槌 2 杀 2 伤+终局摸 1+限一次反例+equip_skills 实例授予；荆环红判拒打额外闪→命中/黑判→被闪维持反例；昊天塔强制选一+次候选弃置）+既有 54 项全绿。
- **全量**：engine-native 774 passed / 0 failed（发牌敏感修复：b32 神秘果入池级联——全域摘除两果+反例③改直用惰性断言；87→89 张计数四处同步）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0（gate1b-all-targets.log）。

## 过程勘误（入记忆）

- **荆环臂误嵌贯石斧块**：插臂点选在 `if has_guanshi { ... }` 内层——has_guanshi=false 时整块跳过（DBG 实证 setup 从未执行）；窗口臂插入须核对块层级。
- **active_ask 铁律再证**：鼠槌首版单 stage+target.is_none() 门做 ask/consume 复用——应答后 resume 重新命中 ask 分支（无限新窗）；挂起前 stage 必须前移到消费阶段。
- **use_as 语义勘误**：PlayCard 按牌种路由对每张牌声明 `use_as: Some(结算牌种)`（非 view-as 专属）——「未转化」判据=`use_as.map(|k| k==实体牌种).unwrap_or(true)`。
- **神秘果 Restage 弃堆摘除后必须 table.push**（首版三区全空→take_hand_card panic）。
- drive-by：删除木甲人 yangpijuan 恒真过滤器（`!= Tao || true`，clippy logic_bug 拦截；语义不变，全量复跑零扰动）；Boss 狱炎 never_loop（预存，不在 Gate 1b 口径内）未动。
- 发牌敏感第七变体：包加卡（神秘果 2 张）改变发牌序+实体发现池——b32 五谷窄池摸走神秘果→窗口级联；全域摘除+惰性断言修复。

## 结论

- swd 卡牌面 92 行对账：**92/92 行为面全量转正**（神秘果 2 张入包、鼠槌/荆环/昊天塔行为面完成）；包 87→89 张。
- 挂账余项：包内多余 13 张处置（数据决策）；玉蝉换形态 mutation/煅造装备面（批⑬ 挂账不变）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
