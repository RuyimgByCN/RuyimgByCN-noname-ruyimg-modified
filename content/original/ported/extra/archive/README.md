# extra 包遗留技能存档

本目录存放已被更高保真实现取代、退出编译池（rebuild-pack 的 .skill glob
不递归本目录）的历史遗留文件。保留仅供追溯，不进入 npack。

## longhun1-4.skill（2026-08-28 第一百三十九批移入）

迁移早期的单臂 view_as 遗留（红桃→桃/方片→杀/黑桃→无懈/梅花→闪各一臂）。
已被同目录 `longhun.skill`（组技能，六臂：tao/sha-fire 双窗口 + wuxie/shan，
全部 `owner_hp_min_one_cards` = max(1,hp) 动态张数，hand_equip_expansion 区）
完整覆盖且超越（单臂文件缺 response 窗与动态张数）。武将面神赵云实际挂载
`relonghun`（up_to 2 same_suit + 火属性 + 双牌加成，对应上游 group 子技能
relonghun_num/relonghun_discard），亦为高保真。
原近似注记「花色条件未建模」失实：四文件体内均已带花色过滤。

## dawu3/kuangfeng3/qixing2/g_drlt_zhiti/g_nzry_jieying/duorui_clear.skill（2026-08-28 第一百四十批移入）

全部属 sync 脚本 `foldedIntoParentSkills` 表登记的被取代遗留（无武将挂载、
无跨包授予、无测试引用，catalog 以 covered_by 指向父体）：
- dawu3/kuangfeng3：上游「标记+addTempSkill 执行件」架构拆分；本库由
  qixing.skill 的 dawu2/kuangfeng2（damage_taken_cancel/火伤+1 modifier）
  直挂目标等价建模，二者 draw-1 假体为早期占位。
- qixing2：摸牌阶段摸 1 占位；被 qixing 的 resolve_qixing(self,0) 交换臂
  取代（同事件双触发冗余）。
- g_drlt_zhiti：无条件手牌上限 -1 占位；父体 drlt_zhiti（drlt_duorui.skill）
  已有条件版 `global_hand_limit add -queried_player_wounded_in_owner_attack_range_other`。
- g_nzry_jieying：上游 1.11.5 已删除该全局助手（sync 表仅存注释），父体
  nzry_jieying（nzry_longnu.skill）为权威实现；本文件为孤儿。
- duorui_clear：上游为 sourceSkill 清理语义（无规则行为）；本文件误带
  optional draw-1 假体，被 drlt_duorui/drlt_duorui1 取代。
