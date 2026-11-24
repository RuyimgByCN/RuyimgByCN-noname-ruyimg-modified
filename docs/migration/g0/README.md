# G0 来源清单与裁决 34 逐项上游台账（37 来源）

## 2026-11-24 Batch P3-E1（L11 VM Fix A +18→663/936，全局 4,146/278/0）更新

- **核验与升级**：基线 `c229e560e725f62d5dee59a4cab5bdc0a56fea8e`（Batch `P3-C1a` CI Run `45378827474` Attempt 1 `10/10 success`）；合入 `swarm/p3-l11-vm-fix-a`（`08e3c7f0f3980c3301855c7e0249d0ee82265069`，`verifier-l11` `#272 APPROVE`），将 `HostBehavior::L11VmFixA` 重编号至 **`1392`**（保留 `L11OwnNpc` `1389`、`L11CrossNpcB` `1390` 与 `L11CrossNpcA` `1391`，`ALL=705`），闭环 `31` 项 `L11_TAIXU_VM_FIX_A_SKILLS` + `xiangle -> U01DddBatchD`（剥离过期 VM），回填 `d34_l11_mods_damage.rs` 至 `HEAD` 语义并迁移 `l11_gz_*` 至 `vm_aux.flags` 修复 `card_in_multiple_zones`，将 `#272` `17` 名及叠加上游 `P3-C1a` 解锁之 `txhj_zhurong` 共 **`18`** 名无其余阻塞 `L11:generals:*` 升为 `已复核等价`（`L11` **`663 / 273 / 0`**；保留 `txhj_moxiaoqiao`/`txhj_rexiaoqiao`/`txhj_huzhen` 等 `102` 名阻塞武将、`171` 模式节点及 `5` 项冻结项）。全局现 **`4,146 / 278 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [L11-太虚幻境.md](L11-太虚幻境.md)、[Batch P3-E1 证据摘要](../../operations/evidence/2026-11-24-p3-e1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-24 Batch P3-C1a（L11 跨包 NPC Part A +47→645/936，全局 4,128/296/0）更新

- **核验与升级**：基线 `12ba5f8c7fce6abc3f2911a2f9e61785f1b89724`（Batch `P3-C1b` CI Run `45363047982` Attempt 1 `10/10 success`）；合入 `swarm/p3-l11-npc-a`（`5a0511c24b81021bc6a332d9513688fb6aae98ea`，`verifier-u05-a` `#266 APPROVE`），将 `HostBehavior::L11CrossNpcA` 重编号至 **`1391`**（保留 `L11OwnNpc` `1389` 与 `L11CrossNpcB` `1390`，`ALL=704`）闭环 `80` 项 Part A 跨包 NPC 技 + `olhuoji`/`olkanpo`/`rexuanfeng` 与 `bazhen`/`reyingzi` 派生绑定并完成 R2/R3 修复，将 `#284` 核准之 **`47`** 名无其余阻塞 `L11:generals:*`（`31` `PART_A_ONLY` + `16` clean `both_ab`）升为 `已复核等价`（`L11` **`645 / 291 / 0`**；保留 `txhj_rexiaoqiao`/`txhj_huzhen` 等 `120` 名阻塞武将、`171` 模式节点及 `5` 项冻结项）。全局现 **`4,128 / 296 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [L11-太虚幻境.md](L11-太虚幻境.md)、[Batch P3-C1a 证据摘要](../../operations/evidence/2026-11-24-p3-c1a/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-24 Batch P3-C1b（L11 跨包 NPC Part B +20→598/936，全局 4,081/343/0）更新

- **核验与升级**：基线 `48ac34db771adc7c4d744485f8160506b321fde5`（Batch `P3-B` CI Run `45350449191` Attempt 1 `10/10 success`）；合入 `swarm/p3-l11-npc-b`（`e8244d6ed..f63f4c8523715c154e11b467d557e1b7e2d97e44`，`verifier-u05-b` `#246 APPROVE`），将 `HostBehavior::L11CrossNpcB` 重编号至 **`1390`**（保留 `L11OwnNpc` `1389`，`ALL=703`）闭环 `94` 项 `L11_NPC_B_SKILLS` 与 Round 2 `9` 项修复，将 `#246` 批准之 **`20`** 名无其余阻塞 `L11:generals:*` 升为 `已复核等价`（`L11` **`598 / 338 / 0`**；保留 `167` 名阻塞武将、`171` 模式节点及 `5` 项冻结项）。全局现 **`4,081 / 343 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [L11-太虚幻境.md](L11-太虚幻境.md)、[Batch P3-C1b 证据摘要](../../operations/evidence/2026-11-24-p3-c1b/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-24 Batch P3-B（L26 7/0/0、U02 17/0/0、L11 Tier-1 +24→578/936，全局 4,061/363/0）更新

- **核验与升级**：基线 `f08a704c3ef69a3ae56cbf5aff2d7711a2aca626`（Batch `P3-A` CI Run `45340220955` Attempt 1 `10/10 success`）；合入 `swarm/p3-l26-u02-l11-tier1`（`b5a83dd6d..0c03410f8d7daabe8bfe58b04aedf88a00a8905b`，`verifier-l11` `#238 APPROVE`），将 `#238` 批准之 **`26`** 项（`L26:configs:Fool_liegong`、`U02:modes:boss_play_arenaReady`、`L11:skills:{gx_taijifuchen, qilin_skill}` 与 `22` 名无阻塞 `L11` 武将）升为 `已复核等价`（`L26` **`7 / 0 / 0`**、`U02` **`17 / 0 / 0`**、`L11` **`578 / 358 / 0`**，`U05` 保持 `1,618 / 3 / 0`；保留 `40` 名受 `x164` / 共享缺陷阻塞之 `L11` 武将、`171` 模式节点及冻结项）。全局现 **`4,061 / 363 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [L11-太虚幻境.md](L11-太虚幻境.md)、[L26-笨蛋插件.md](L26-笨蛋插件.md)、[U02-boss.md](U02-boss.md)、[Batch P3-B 证据摘要](../../operations/evidence/2026-11-24-p3-b-tier1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-24 Batch P3-A（U05 杀海拾遗非冻结 31 项收口→1,618/1,621，全局 4,035/389/0）更新

- **核验与升级**：基线 `e8244d6ed8f14ad499a5bdf82949bafb67b354b7`（Batch `Q4-remaining` CI Run `45303299654` Attempt 1 `10/10 success`）；合入 `swarm/p3-u05-closure`（`e8b71bfc4..7d574912ecf29d8ef158b34388c7827271fd0c72`，`verifier-u05-b` `#231 APPROVE`），将 `U05` 剩余全部 **`31`** 项非冻结 `简化近似` 条目升为 `已复核等价`，严格保留 `shixu`、`hs_nozdormu`（墙钟边界）与 `old_longfan`（Decision 40 200ms `setInterval` 4 位 DOM 虎口机）共 `3` 项为 `简化近似`。`U05` 现 **`1,618 / 3 / 0`**，全局 **`4,035 / 389 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch P3-A 证据摘要](../../operations/evidence/2026-11-24-p3-a-u05-closure/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-24 Batch Q4-remaining（L03/L04/L15/L17/L21/L26/U02/U03/U04 +97→全局 4,004/420/0，Q4 收口）更新

- **核验与升级**：基线 `8fa7589963bbbe724dc89e656ffcdf6b1afdd8ea`（Batch `U05-7d` CI Run `45288232922` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-q4-remaining`（`586b51cdc..0e06ca047480eefdf6b8c86bb307f3702395079e`，`verifier-l11` `#114 APPROVE`），经主线三路合并与 `UI-7`（已闭环 7 项规则耦合/UI 配置）去重后，将剩余 **`97`** 项升为 `已复核等价`（`L03` +8→`8/0/0`、`L04` +21→`21/0/0`、`L15` +11→`14/0/0`、`L17` +1→`11/0/0`、`L21` +7→`9/0/0`、`L26` +1→`6/1/0`、`U02` +16→`16/1/0`、`U03` +30→`30/0/0`、`U04` +2→`7/0/0`），严格保留 `L26:configs:Fool_liegong` 与 `U02:modes:boss_play_arenaReady` 为 `简化近似`。全局现 **`4,004 / 420 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [Batch Q4-remaining 证据摘要](../../operations/evidence/2026-11-24-q4-remaining/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-24 Batch U05-7d（U05 杀海拾遗 SWD 角色技能 Part 2 与 79 武将 +181→1,587/1,621，全局 3,907/517/0）更新

- **核验与升级**：基线 `8a8d202c756f6e3eb77731aeb78da009d579dbad`（Batch `U05-7c` CI Run `45270007227` Attempt 3 `10/10 success`）；三路合入 `swarm/conv-u05-7d-swd-skills2`（`a47f8b0e1..ae303c561215c980baa8a1754a6cf299442c6e2e`，`verifier-u05-b` `#83 APPROVE`），完成 `step/swd_u05_7d.rs`/`activation/swd_u05_7d.rs` 拆分、`SwdXuanying::group_children` 与 `inject_pre_general_companions` 协同、`txhj_jiedao` 防回归、`play_card_targets` 扩展路径、`>= 40` VM 下界与 `test_report_event_arms.py`（`88->90`）校准后，将 U05 杀海拾遗 SWD 角色技能 Part 2 获批 `110` 项 `U05:skills:*` + `71` 名 `swd_*` 武将共 **`181`** 项升为 `已复核等价`（严格保留 `anlianying`/`jqimou`/`xingdian`/`lanzhi`/`yueren` 与 `8` 名 SWD 武将共 `13` 项为 `简化近似`）。`U05` 现 **`1,587 / 34 / 0`**，全局 **`3,907 / 517 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-7d 证据摘要](../../operations/evidence/2026-11-24-u05-7d-swd-skills2/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-7c（U05 杀海拾遗 SWD 角色技能 Part 1 +113→1,406/1,621，全局 3,726/698/0）更新

- **核验与升级**：基线 `12d5dca34bd499aac00eb3063054727ddea5b769`（Batch U05-7b CI Run `45251837050` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-u05-7c-swd-skills1`（`41b7e4ded..96c66dfe5a0e7fab5be8f983d6df8f992a9d5314`，`verifier-u05-b` `#64 APPROVE`），完成 `combat_rules_sha.rs` 拆分、`SwdXuanying::group_children` 移除 `xuanying_damage`、`play_rules.rs` `mianzhen2` `IllegalTargets` / `huanxia_yaotong_play_fallback` 协同与 `u05_b7c` 夹具 `declared_kinds`/`declared_slots` 清理后，将 U05 杀海拾遗 SWD 角色技能 Part 1 获批 **`113`** 项 `U05:skills:*` 升为 `已复核等价`（严格保留 `pojian` 与 `xuanyuan_ai` 2 项为 `简化近似`）。`U05` 现 **`1,406 / 215 / 0`**，全局 **`3,726 / 698 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-7c 证据摘要](../../operations/evidence/2026-11-23-u05-7c-swd-skills1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-7b（U05 杀海拾遗 SWD 卡牌 Part 2 +104→1,293/1,621，全局 3,613/811/0）更新

- **核验与升级**：基线 `61ca3573d3a0aff5d0e189b703b3afed2f898d8e`（Batch U05-7a CI Run `45233301007` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-u05-7b-swd-cards2`（`2099ce1a4..e7ff9868d41813f6d8bb667e79fd345014e04713`，`verifier-u05-b` `#86 APPROVE`），完成 `shahai_attach` 包名/串联修复、`host_on_outer_settled_all`、`yuchan_swap` `Stage::Done` 延迟与 `Caoyao` 自救、`validate_tail_ext_targets`、`is_tao_like_core`/`Tianxianjiu` 仅自救与 `sadengjinhuan_*` snapshot 推导后，将 U05 杀海拾遗 SWD 卡牌 Part 2 获批 `40` 张卡牌 + `64` 项伴生/灵剑/锻造技能共 **`104`** 项升为 `已复核等价`（严格保留 `yiluan`/`old_longfan`/`longfan`/`touzhi` 4 项为 `简化近似`）。`U05` 现 **`1,293 / 328 / 0`**，全局 **`3,613 / 811 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-7b 证据摘要](../../operations/evidence/2026-11-23-u05-7b-swd-cards2/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-7a（U05 杀海拾遗 SWD 卡牌 Part 1 +106→1,189/1,621，全局 3,509/915/0）更新

- **核验与升级**：基线 `fb4a15c6b1fe0c6072c58e2be4445021a76449d0`（Batch U05-4 CI Run `45207312135` Attempt 2 `10/10 success`）；三路合入 `swarm/conv-u05-7a-swd-cards1`（`c727d3922..d94c0f0d5d100351ff44ea3fd095bec41a7c206e`，`verifier-u05-b` `#56 APPROVE`），完成 `swd_u05_7a.rs` 拆分、`standard.rs` `U05SwdHufuViewAs` 全球技白名单、SWD `common.rs` `declared_kinds` 清理与门禁校准后，将 U05 杀海拾遗 SWD 卡牌 Part 1 获批 `39` 张卡牌 + `67` 项伴生/全局/锻造技能共 **`106`** 项全数升为 `已复核等价`（`0` 保留）。`U05` 现 **`1,189 / 432 / 0`**，全局 **`3,509 / 915 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-7a 证据摘要](../../operations/evidence/2026-11-23-u05-7a-swd-cards1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-4（U05 杀海拾遗 `xianjian` 武将/技能 +156→1,083/1,621，全局 3,403/1,021/0）更新

- **核验与升级**：基线 `362b01f2fc94c4a74b165041e94212e95a506195`（Batch U05-5b CI Run `45167390472` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-u05-4-xianjian`（`505ed0704..8e14ff93ae948c37135b3a7e0982e1ce1e0a43d8`，`verifier-u05-a` `#109 APPROVE`），完成 `turn_handoff.rs` 拆分、`standard.rs` `tuoqiao` 伴生绑定 `gj_food_shihuifen_ban` 与 `off_deck`/`feibiao` 测试夹具对齐后，将 U05 杀海拾遗 `xianjian` 获批 `36` 武将 + `120` 角色/衍生/全局技能共 **`156`** 项全数升为 `已复核等价`（`0` 保留）。`U05` 现 **`1,083 / 538 / 0`**，全局 **`3,403 / 1,021 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-4 证据摘要](../../operations/evidence/2026-11-23-u05-4-xianjian/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-5b（U05 杀海拾遗 `gwent` 武将/技能 +121→927/1,621，全局 3,247/1,177/0）更新

- **核验与升级**：基线 `2150f8511556c8f6b90275e6b140388bc5cf3196`（Batch U05-5a CI Run `45154024465` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-u05-5b-gwent-chars`（`4a145b5f7..0ec2caac8`，`verifier-u05-a` `#116 APPROVE`），完成 `has_active_equip_kind` 协同、`host_skill_bindings_std.rs` 拆分及 `round3.rs` `gj_food_molicha` 授予时序修复后，将 U05 杀海拾遗 `gwent` 获批 `52` 武将 + `69` 角色/衍生/全局技能共 **`121`** 项全数升为 `已复核等价`（`0` 保留，仅 `U05:configs:gwent` 未动）。`U05` 现 **`927 / 694 / 0`**，全局 **`3,247 / 1,177 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-5b 证据摘要](../../operations/evidence/2026-11-23-u05-5b-gwent-chars/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-5a（U05 杀海拾遗 `gwent` 卡牌/技能 +80→806/1,621，全局 3,126/1,298/0）更新

- **核验与升级**：基线 `d6d411439fc2f763ceeeef359e10372f702dd0b0`（Batch U05-6b CI Run `45138296268` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-u05-5a-gwent-cards`（`b7ac33cb6..278b0ac8a`，`verifier-u05-a` `#96 APPROVE`），完成 `has_active_equip_kind` 协同修正与 `play_card_targets_gwent.rs`/`extension_choices_dispatch.rs` 拆分后，将 U05 杀海拾遗 `gwent` 获批 `69` 卡牌 + `11` 卡牌伴生技能共 **`80`** 项全数升为 `已复核等价`（`0` 保留，`/character/gwent/` `121` 项留待 `U05-5b`）。`U05` 现 **`806 / 815 / 0`**，全局 **`3,126 / 1,298 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-5a 证据摘要](../../operations/evidence/2026-11-23-u05-5a-gwent-cards/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-6b（U05 杀海拾遗 `hearth` Part 2 +208→726/1,621，全局 3,046/1,378/0）更新

- **核验与升级**：基线 `cc0401e8ec93c64098fc989e385ad2a30800ed86`（Batch U05-6a CI Run `45126700137` Attempt 1 `10/10 success`）；三路合入 `swarm/conv-u05-6b-hearth-part2`（`e350ea568..92e1b4cac`，`verifier-u05-b` `#45 APPROVE`）并完成主线 `virtual_diaobing_use` / `stone_chenshui` 兼容修正及 300 VM 下界校准后，将 U05 杀海拾遗 `hearth` Part 2 获批 `105` 技能 + `103` 武将共 `208` 项升为 `已复核等价`，严格保留 `U05:skills:shixu`（墙钟确定性边界）与 `7` 名 `hs_*` 武将（`hs_alleria`/`hs_jaina`/`hs_kcthun`/`hs_nozdormu`/`hs_tgolem`/`hs_wuther`/`hs_xsylvanas`）为 `简化近似`。`U05` 现 **`726 / 895 / 0`**，全局 **`3,046 / 1,378 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-6b 证据摘要](../../operations/evidence/2026-11-23-u05-6b-hearth-part2/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-6a（U05 杀海拾遗 `hearth` +167→518/1,621，全局 2,838/1,586/0）更新

- **核验与升级**：基线 `a2e340f868c62b161a73b1f21907233b7eda4234`（Batch U05-3 CI Run `45116138360` Attempt 1 `10/10 success`）；合入 `swarm/conv-u05-6a-hearth`（`9703e2188`，`verifier-u05-b` `#27 APPROVE`）并完成 `known_prompts_u05_6a.rs`、`play_card_targets_hearth.rs` 拆分、`step_stone_chenshui` `detached_cards` 离场梦境牌适配及 `decision34_round4_u05_shahai.rs` 下界校准后，将 U05 杀海拾遗 `hearth` 子包全部 `57` 卡牌、`110` 技能共 `167` 项全数升为 `已复核等价`（`0` 项保留 `简化近似`）。`U05` 现 **`518 / 1,103 / 0`**，全局 **`2,838 / 1,586 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-6a 证据摘要](../../operations/evidence/2026-11-23-u05-6a-hearth/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-3（U05 杀海拾遗 `ow` +127→351/1,621，全局 2,671/1,753/0）更新

- **核验与升级**：基线 `35bc3def969bf9c4efa1a9c1806c19be11513a01`（Batch U05-2 CI Run `45107904173` Attempt 1 `10/10 success`）；合入 `swarm/conv-u05-3-ow`（`53ed4b1fa`，`verifier-u05-a` `#113 APPROVE`）并完成 `turn_reset.rs` 拆分、无色虚拟 AOE `aoe.rs` 指向校准与 500 VM 下界校准后，将 U05 杀海拾遗 `ow` 子包全部 `24` 武将、`103` 技能共 `127` 项全数升为 `已复核等价`（`0` 项保留 `简化近似`）。`U05` 现 **`351 / 1,270 / 0`**，全局 **`2,671 / 1,753 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-3 证据摘要](../../operations/evidence/2026-11-23-batch3-9-u05-3-ow/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-2（U05 杀海拾遗古剑奇谭 +136→224/1,621，全局 2,544/1,880/0）更新

- **核验与升级**：基线 `211c32d1adcbd8f1810f9896266a60bffd0d800f`（Batch U05-1 CI Run `45097178310` Attempt 1 `10/10 success`）；合入 `swarm/conv-u05-2-gujian`（`921a8f499`，`verifier-u05-a` `#95 APPROVE`）并完成 `host/skill_defs.rs` 拆分、`declared_kinds` 清理与 600 VM 下界校准后，将 U05 杀海拾遗 `gujian` 子包全部 `36` 卡牌、`18` 武将、`82` 技能共 `136` 项全数升为 `已复核等价`（`0` 项保留 `简化近似`）。`U05` 现 **`224 / 1,397 / 0`**，全局 **`2,544 / 1,880 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-2 证据摘要](../../operations/evidence/2026-11-23-batch3-8-u05-2-gujian/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch U05-1（U05 杀海拾遗 +87→88/1,621，全局 2,408/2,016/0）更新

- **核验与升级**：基线 `5de733e5e9209320500a9064a5d73f7e7970b537`；合入 `swarm/conv-u05-1`（`b4fd79501`，`verifier-l11` `#49 APPROVE`）及主线冲突/回归修复后，将 U05 杀海拾遗 `33` 卡牌、`2` 配置（`rand`/`wuxing`）、`12` 武将、`40` 技能共 `87` 项升为 `已复核等价`；保留 Batch UI-7 `tip` 为 `已复核等价`，严格保留 `qixingzhen`、`diy_caiwenji`、`stone_chenshui` 与 4 项子包开关（`mtg`/`zhenfa`/`gwent`/`yunchou`）为 `简化近似`。`U05` 现 **`88 / 1,533 / 0`**，全局 **`2,408 / 2,016 / 0`**（缺失 `0`，整包 8 维签字仍 **`0/37`**，`333/107/5/759/8` 与 14 条 golden 哈希不变）。详见 [U05-杀海拾遗.md](U05-杀海拾遗.md)、[Batch U05-1 证据摘要](../../operations/evidence/2026-11-23-batch3-7-u05-1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch L11-6b（L11 太虚幻境 206 模式节点/效果包与净集 +35→554/936）更新（待提交／发布）

基线 HEAD `6ecddea03c8c2d5317da85780609de16f8306e70`（Batch L11-6a 已发布，CI Run `45074648377` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-l11` 独立复核（#100 **`APPROVE`**，含 Round 4 与三轮 Headless Chrome CDP）及主线全量门禁复测，清单按 #100 净集划分更新为 **已复核等价 2,321／简化近似 2,103／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `519 / 417 / 0` 升为 **554 / 382 / 0**（+35 项无外部阻塞依赖节点：`2` `extension_servant.js`＋`18` `event_universal.js`＋`15` `extension_season.js`，其余 `171` 项〔含 `action` Spine 元数据〕维持 `简化近似`；整包八项仍 `0/37`）。详见 [Batch L11-6b 证据摘要](../../operations/evidence/2026-11-23-batch3-6b-l11-6b/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch L11-6a（L11 太虚幻境 320 武将档案与净集 +111→519/936）更新（待提交／发布）

基线 HEAD `b27a0968b13355a35292c767a3d3635401f5d31d`（Batch L11-5 已发布，CI Run `45065560795` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-l11` 独立复核（#57 **`APPROVE`**，含 Round 3 与两轮 Headless Chrome CDP）及主线全量门禁复测，清单按 #57 净集划分更新为 **已复核等价 2,286／简化近似 2,138／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `408 / 528 / 0` 升为 **519 / 417 / 0**（+111 名纯已等价／无定义技能武将，其余 209 名依赖待闭合技能之武将维持 `简化近似`；整包八项仍 `0/37`）。详见 [Batch L11-6a 证据摘要](../../operations/evidence/2026-11-23-batch3-6a-l11-6a/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch L11-5（L11 太虚幻境 15 `L11SystemHooks`＋6 `L11Weapons`＋21 `L11GeneralBossC`＋1 `L11BossLingfeng`＋5 L28/L27 共 +48→408/936）更新（待提交／发布）

基线 HEAD `6b8c847c6e5acbc878fd3fa4a333f53c5ed9298a`（Batch L11-4 已发布，CI Run `45056028805` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-l11` 独立复核（#41 **`APPROVE`**，含 Round 2 校验与 Headless Chrome CDP）及主线全量门禁复测，清单更新为 **已复核等价 2,175／简化近似 2,249／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `360 / 576 / 0` 升为 **408 / 528 / 0**（+48：15 项 `L11SystemHooks`、6 项 `L11Weapons`、21 项 `L11GeneralBossC`、1 项 `L11BossLingfeng` 与 5 项 L28/L27 复用防具/三工；整包八项仍 `0/37`）。详见 [Batch L11-5 证据摘要](../../operations/evidence/2026-11-23-batch3-5-l11-5/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-23 Batch L11-4（L11 太虚幻境 33 `L11BuffsC`＋37 `L11BuffsD`＋10 衍生技共 +80→360/936）更新（待提交／发布）

基线 HEAD `0f514af299519ba8325711136b49e447df2608d2`（Batch U01-3 已发布，CI Run `45048457551` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-l11` 独立复核（#22 **`APPROVE`**，含 Round 3 inline judge/yield 校验与 Headless Chrome CDP）及主线全量门禁复测，清单更新为 **已复核等价 2,127／简化近似 2,297／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `280 / 656 / 0` 升为 **360 / 576 / 0**（+80：33 项 `L11BuffsC`、37 项 `L11BuffsD` 与 10 项 `L11BuffHelpersB`；整包八项仍 `0/37`）。详见 [Batch L11-4 证据摘要](../../operations/evidence/2026-11-23-batch3-4-l11-4/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch U01-3（U01 3D精选 20 项→94/95；`dddanzhi` 保留冻结锁边界）更新（待提交／发布）

基线 HEAD `820aaddd64450d03c4cf66fdac808480872778ca`（Batch L11-3 已发布，CI Run `45022962559` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules` 首轮（#187，含 Chrome CDP）与 `verifier-rules-2` 次轮独立复核（#205 **`APPROVE`**）及主线全量门禁复测，清单更新为 **已复核等价 2,047／简化近似 2,377／缺失 0／上游未定义 0**（合计 4,424）：`U01 3D精选` 由 `74 / 21 / 0` 升为 **94 / 1 / 0**（+20，`dddxiaheng`..`dddbailei`；`dddanzhi` 按 Section 3 Item 3① 保持 `简化近似` 护冻结归档 `u01-dddanzhi-pending-v107.ngsa`；整包八项仍 `0/37`）。详见 [Batch U01-3 证据摘要](../../operations/evidence/2026-11-22-batch1-6d-u01-3/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch L11-3（L11 太虚幻境 40 祝福＋38 触发＋10 衍生技共 +88→280/936）更新（待提交／发布）

基线 HEAD `9c29b98996558302cadebec39c59ba8787cab86b`（Batch L01-Rules 已发布，CI Run `45017015012` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 首轮审查（#188，含 Chrome CDP）与 `verifier-ui` 次轮独立复核（#201 **`APPROVE`**）及主线全量门禁复测，清单更新为 **已复核等价 2,027／简化近似 2,397／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `192 / 744 / 0` 升为 **280 / 656 / 0**（+88：40 项 `buff_txhj_*`、38 项 `_buff_txhj_*` 触发与 10 项衍生辅助技 `txhj_kapaikeshi`..`txhj_huangtiannizhuan`；整包八项仍 `0/37`）。详见 [Batch L11-3 证据摘要](../../operations/evidence/2026-11-22-batch3-3-l11-3/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch L01-Rules（L01 AI优化 69 技能＋1 卡牌 `yiyi`＋27 规则配置共 +97→133/133）更新（待提交／发布）

基线 HEAD `042f5b811927476b1bf056bc73e462194eee34ce`（Batch L11-2 已发布，CI Run `45008789520` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 首轮（#180，含 Chrome CDP）与 `verifier-ui` 第三轮独立审查（#196 **`APPROVE`**）及主线全量门禁复测，清单更新为 **已复核等价 1,939／简化近似 2,485／缺失 0／上游未定义 0**（合计 4,424）：`L01 AI优化` 由 `36 / 97 / 0` 升为 **133 / 0 / 0**（+97：69 技能、`yiyi`、27 `ruleCoupled` 配置，达 100% 单项等价；整包八项仍 `0/37`）。详见 [Batch L01-Rules 证据摘要](../../operations/evidence/2026-11-22-batch4-1-l01-rules/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch L11-2（L11 太虚幻境 4 侍从钩子＋26 侍从技 B＋53 武将技 D/E/F 共 +83→192/936）更新（待提交／发布）

基线 HEAD `5bcd6b619ebfbe9bd5b414eeb5bbd7d2d1341443`（Batch Q2-L25 已发布，CI Run `45001901211` Attempt 1 `10/10 success`）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 独立审查（#174 **`APPROVE`**，含 16 场景 Chrome CDP 实跑探针与 Round 1 #1-#15 复测）与主线全量门禁复测，清单更新为 **已复核等价 1,842／简化近似 2,582／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `109 / 827 / 0` 升为 **192 / 744 / 0**（+83：`_slEnterGame`..`_slHurt`、`txhj_dundunSkill1`..`txhj_aHaoSkill3`、`oldfuman`..`txhj_oldhuxiao`，整包八项仍 `0/37`）。详见 [Batch L11-2 证据摘要](../../operations/evidence/2026-11-22-batch3-2-l11-2/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch Q2-L25（L25 祖安武将 终 18 项→257/257；Queue Item 2 收官）更新（待提交／发布）

基线 HEAD `89406fe50c5d33c349b4456b05a0b7724c439a5e`（Batch U01-2 已发布，CI Run `44995724474` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules` 独立审查（#167 **`APPROVE`**，含 Chrome CDP 实跑探针）与主线全量门禁复测，清单更新为 **已复核等价 1,759／简化近似 2,665／缺失 0／上游未定义 0**（合计 4,424）：`L25 祖安武将` 由 `239 / 18 / 0` 升为 **257 / 0 / 0**（+18：`ssr_jinfan`/`ssr_luoyi`/`ssr_lieren`、4 `ui_hooks`、11 `skills`，达 100% 单项等价，Queue Item 2 收官；整包八项仍 `0/37`）。详见 [Batch Q2-L25 证据摘要](../../operations/evidence/2026-11-22-batch2-4-l25/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch U01-2（U01 3D精选 第二批 21 项→74/95）更新（待提交／发布）

基线 HEAD `9842e555d9fc336ffb5c37a09d0b815da1a0ae32`（Batch Spine-Tests 已发布，CI Run `44990992457` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules` 独立审查（#157 **`APPROVE`**，含 4 轮 Chrome CDP 实跑探针）与主线全量门禁复测，清单更新为 **已复核等价 1,741／简化近似 2,683／缺失 0／上游未定义 0**（合计 4,424）：`U01 3D精选` 由 `53 / 42 / 0` 升为 **74 / 21 / 0**（+21，`dddzhengjun`..`dddfenji`）。详见 [Batch U01-2 证据摘要](../../operations/evidence/2026-11-22-batch1-6c-u01-2/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch L11-1（L11 太虚幻境 32 卡牌＋首批 71 技能共 +103→109/936）更新（待提交／发布）

基线 HEAD `a47f8b0e110cb217f54cb4f40f111a223dab649a`（Batch UI-7 已发布，CI Run `44981761423` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 独立审查（#135 **`APPROVE`**，含 10 项 Chrome CDP 实跑探针）与主线全量门禁复测，清单更新为 **已复核等价 1,720／简化近似 2,704／缺失 0／上游未定义 0**（合计 4,424）：`L11 太虚幻境` 由 `6 / 930 / 0` 升为 **109 / 827 / 0**（+103，32 张卡牌与首批 71 项技能 `txhj_daiyuan`..`cxyPoLu`，整包八项仍 `0/37`）。详见 [Batch L11-1 证据摘要](../../operations/evidence/2026-11-21-batch3-1-l11-1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch UI-7（16 来源 121 项纯 UI 收口；50 项 `ruleCoupled` 留待规则线）更新（待提交／发布）

基线 HEAD `f6cd6acb9228d8ca2ab490d61c26f1d856f6c502`（Batch Q2-L20+L22 已发布，CI Run `44976347997` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-ui` 独立审查（#126 **`APPROVE`**）与主线全 9 套 Chrome CDP 及全量门禁复测，清单更新为 **已复核等价 1,617／简化近似 2,807／缺失 0／上游未定义 0**（合计 4,424）：按 #126 升级 **121 项**纯 UI 条目（`L02` **5/0/0**、`L05` **4/0/0**、`L07` **13/0/0**、`L12` **6/0/0**、`L13` **10/0/0**、`L16` **3/0/0**、`L18` **13/0/0**、`L23` **1/0/0** 均达 100% 单项等价；`L01` **36/97/0**、`L11` **6/930/0**、`L15` **3/11/0**、`L17` **10/1/0**、`L21` **2/7/0**、`L26` **5/2/0**、`U04` **5/2/0**、`U05` **1/1620/0**），严格保留 `50` 项 `ruleCoupled: true` 配置为 `简化近似`（整包八项仍 `0/37`）。详见 [Batch UI-7 证据摘要](../../operations/evidence/2026-11-21-ui-batch7-closure/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch Q2-L20+L22（L20 终 3 技→95/95 ＋ L22 终 2 技→8/8）更新（待提交／发布）

基线 HEAD `41b7e4dedeada725ecd08b3e4b3400285bd44470`（Batch UI-6 已发布，CI Run `44969821746` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 独立审查（#115 **`APPROVE`**，含 6 项 Chrome CDP 实跑探针）与主线全量门禁复测，清单更新为 **已复核等价 1,496／简化近似 2,928／缺失 0／上游未定义 0**（合计 4,424）：`L20 标记补充` 由 `92 / 3 / 0` 升为 **95 / 0 / 0**（+3，`mibei`/`fz_liegong`/`wengua`，另重验 `tspowei`/`kuanshi`/`kuanshi2`/`wengua5`）；`L22 段位制裁` 由 `6 / 2 / 0` 升为 **8 / 0 / 0**（+2，`_eloDraw`/`_timeKiller`），双源达 100% 单项等价（整包八项仍 `0/37`）。详见 [Batch Q2-L20+L22 证据摘要](../../operations/evidence/2026-11-21-batch2-3-l20-l22/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch UI-6（L20/L09/L24/L27/L29/L10 UI 收口与 L25/L28 共 +158）更新（待提交／发布）

基线 HEAD `920f4fac0c5fa4b84da906f453e90e1d9f459483`（Batch U01-1 已发布，CI Run `44963498759` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-ui` 独立审查（#109 **`APPROVE`**）与主线 8 套 Chrome CDP 及全量门禁复测，清单更新为 **已复核等价 1,491／简化近似 2,933／缺失 0／上游未定义 0**（合计 4,424）：`L09 千幻聆音` +44→**102/0/0**、`L24 皮肤切换` +46→**46/0/0**、`L27 选将美化` +1→**54/0/0**、`L29 龙血玄黄` +4→**9/0/0**、`L10 史诗卡牌` +6→**17/0/0**、`L20 标记补充` +10→**92/3/0**、`L25 祖安武将` +46 纯 UI→**239/18/0**（保留 18 项规则/耦合项待 Q2-L25）、`L28 雷霆万钧` +1→**55/1/0**。详见 [Batch UI-6 证据摘要](../../operations/evidence/2026-11-21-ui-l24-l25-l29-l10/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch U01-1（U01 3D精选首批 20 项→53/95）更新（待提交／发布）

基线 HEAD `b7ac33cb6d6164be6b4f3e124c24c668e666cac6`（Batch Q2-L19 前向修复已发布，CI Run `44954522303` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules` 三轮独立审查（#93 **`APPROVE`**）与主线全量门禁复测，清单更新为 **已复核等价 1,333／简化近似 3,091／缺失 0／上游未定义 0**（合计 4,424）：`U01 3D精选` 由 `33 / 62 / 0` 升为 **53 / 42 / 0**（+20，`dddzhishu`、`dddxiaxing`、`dddjuxian`、`dddjungui`、`dddjinggou`、`dddmoyan`、`dddshilie`、`dddyeshen`、`dddqiaoduan`、`ddddiedang`、`dddanliu`、`dddguiying`、`dddbingjian`、`dddtongyu`、`dddzhilian`、`dddjijian`、`dddjijian_old`、`dddfushi`、`dddlangzhi`、`dddfuzong`）。详见 [Batch U01-1 证据摘要](../../operations/evidence/2026-11-21-batch1-6b-u01-1/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch Q2-L19（L19 末日乱斗终 9 项→69/69）更新（待提交／发布）

基线 HEAD `8a879f8e479406e3788011885eba2a16f0d96613`（Batch UI-5 已发布，CI Run `44944693459` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 三轮独立审查（#83 **`APPROVE`**）与主线全量门禁复测，清单更新为 **已复核等价 1,313／简化近似 3,111／缺失 0／上游未定义 0**（合计 4,424）：`L19 末日乱斗` 由 `60 / 9 / 0` 升为 **69 / 0 / 0**（+9，`ganran2`/`_jiangshi2`/`_huzhu`/`chanceIdentity10`/`chanceIdentity0`/`wujinmoshiRE1`/`_wujinmoshiRecover`/`_relifeskill`/`_guandieskill`，全源 100% 单项等价）。详见 [Batch Q2-L19 证据摘要](../../operations/evidence/2026-11-21-batch2-2-l19/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch UI-5（L14 终 33 项、L20 27 项与 L09 58 项共 +118）更新（待提交／发布）

基线 HEAD `586b51cdc8124ab37c92f1015ce562157f911df9`（Batch 1.6a 已发布，CI Run `44938308161` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-ui` 独立审查（#77 **`APPROVE`**）与主线 7 套 Chrome CDP 及全量门禁复测，清单更新为 **已复核等价 1,304／简化近似 3,120／缺失 0／上游未定义 0**（合计 4,424）：`L14 手杀ui` 由 `108 / 33 / 0` 升为 **141 / 0 / 0**（+33）；`L20 标记补充` 由 `55 / 40 / 0` 升为 **82 / 13 / 0**（+27：3 表面＋24 `ui_hooks` 升为 `已复核等价`，10 无 DOM 消费 `ui_hooks`＋3 技能维持 `简化近似`）；`L09 千幻聆音` 由 `0 / 102 / 0` 升为 **58 / 44 / 0**（+58：12 表面＋8 `ui_hooks`＋38 配置升为 `已复核等价`，44 未消费配置维持 `简化近似`）。详见 [Batch UI-5 证据摘要](../../operations/evidence/2026-11-21-ui-l14-l20-l09/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch 1.6a（L06 wuxing ＋ L10 史诗卡牌 4 项规则收口）更新（待提交／发布）

基线 HEAD `e119f5518b32911ed45c3559881d97751966eddb`（Batch UI-4 已发布，CI Run `44929144693` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules-2` 三轮独立审查（#70 **`APPROVE`**）与主线全量门禁复测，清单更新为 **已复核等价 1,186／简化近似 3,238／缺失 0／上游未定义 0**（合计 4,424）：`L06 wuxing` 由 `2 / 1 / 0` 升为 **3 / 0 / 0**（+1，`wuxingpan`）；`L10 史诗卡牌` 由 `8 / 9 / 0` 升为 **11 / 6 / 0**（+3，`reguhuo`/`rangjie`/`k_danggu`，4 技能 100% 单项等价，3 UI 配置＋`_emojiRebate1`＋2 CSS 保持 `简化近似`）。详见 [Batch 1.6a 证据摘要](../../operations/evidence/2026-11-21-batch1-6a-l10-l06/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-22 Batch UI-4（L08 终 14 项与 L14 108 项共 +122）更新（待提交／发布）

基线 HEAD `bd43569e164a7692bd18a88d96d1f196a1fa2ad1`（Batch UI-3 已发布，CI Run `44922862013` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-ui` 独立审查（#65 **`APPROVE`**）与主线 Chrome CDP 及全量门禁复测，清单更新为 **已复核等价 1,182／简化近似 3,242／缺失 0／上游未定义 0**（合计 4,424）：`L08 十周年UI` 由 `392 / 14 / 0` 升为 **406 / 0 / 0**（+14）；`L14 手杀ui` 由 `0 / 141 / 0` 升为 **108 / 33 / 0**（+108：`39` 配置＋`52` 钩子＋`17` 表面升为 `已复核等价`，其余 #65 指出的 33 项维持 `简化近似` 待 UI-5 闭环）。详见 [Batch UI-4 证据摘要](../../operations/evidence/2026-11-21-ui-l14-shousha/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-21 Batch UI-3（L08 十周年UI 371 项）更新（待提交／发布）

基线 HEAD `1906e7f151848f2852edb90e75803e2ed3ff93f9`（Batch Q2-L27 已发布，CI Run `44916917571` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-ui` 独立审查（#51 **`APPROVE`**）与主线 Chrome CDP 及全量门禁复测，清单更新为 **已复核等价 1,060／简化近似 3,364／缺失 0／上游未定义 0**（合计 4,424）：`L08 十周年UI` 由 `21 / 385 / 0` 更新为 **392 / 14 / 0**（+371：`20` 项 `configs`、`32` 项 `surfaces`、`319` 项 `ui_hooks` 升为 `已复核等价`；仅 `3` 项配置 `dynamicBackground`/`JBDDZ`/`SanGuoShaGameOver` 与 `11` 项表面保留 `简化近似` 待 UI-4 闭环）。详见 [Batch UI-3 证据摘要](../../operations/evidence/2026-11-21-ui-l08-hooks/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-21 Batch Q2-L27（L27 选将美化 12 项）更新（待提交／发布）

基线 HEAD `d9e1c9a1fcea4f19f1555bedb416aee8b51fc9b8`（Batch UI-2 已发布，CI Run `44899269373` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-rules` 三轮独立审查（#47 **`APPROVE`**，Chrome CDP 零异常）与主线全量门禁复测，清单更新为 **已复核等价 689／简化近似 3,735／缺失 0／上游未定义 0**（合计 4,424）：`L27 选将美化` 由 `41 / 13 / 0` 更新为 **53 / 1 / 0**（+12：`xiefang`、`new_meibu`、`new_zhixi`、`clandianzhan`、`reqimou`、`bolan`、`mbcmqingzheng`、`sbqingzheng`、`xingchong`、`dcxiaoxi`、`twfengji`、`twzhouzu` 全部规则/钩子项闭环；仅 `L27:surfaces:e.css` 保留 `简化近似` 归 UI 线）。详见 [Batch Q2-L27 证据摘要](../../operations/evidence/2026-11-21-batch2-1-l27/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-21 Batch UI-2（L08 十周年UI 44 配置与 43 表面）更新（待提交／发布）

基线 HEAD `3686468ca8e05a04440fce62dd0e14cfe3517547`（Batch UI-1 已发布，CI Run `44894025555` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经 `verifier-ui` 独立审查（#39 **`APPROVE`**）与主线 Chrome CDP 及全量门禁复测，清单更新为 **已复核等价 677／简化近似 3,747／缺失 0／上游未定义 0**（合计 4,424）：`L08 十周年UI` 由 `0 / 406 / 0` 更新为 **21 / 385 / 0**（+21：仅升格 #39 核验通过的 21 项 `L08:configs:*`，其余 23 项配置及 43 项 `surfaces` 保留 `简化近似` 待 UI-3/后续消费闭环）。详见 [Batch UI-2 证据摘要](../../operations/evidence/2026-11-21-ui-l08-configs-surfaces/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-21 Batch 1.5（L29 龙血玄黄）更新（待提交／发布）

基线 HEAD `af6ef4899052ad1258f38a242960f8f0ea0fd066`（Batch 1.4 `U07` 已发布，CI Run `44785367168` 10/10）；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经独立复核 agent `3488fd64-4fea-4218-be3f-f5fb6d38add2` 两轮逐行对照与 Chrome CDP 实跑探针审查（Round 2 **`APPROVE`**），清单更新为 **已复核等价 656／简化近似 3,768／缺失 0／上游未定义 0**（合计 4,424）：`L29 龙血玄黄` 更新为 **5 / 4 / 0**（净 +4：`mbqianlong`、`mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin` 5 项技能升为已复核等价，`L29:configs:shengjian` 依复核建议由历史已复核等价下调为简化近似归队列第 5 项界面阶段，`_CMeffects`、`main1`、`main1.css` 维持简化近似）。队列第 1 项 L29 完成本地收口，整包八项仍 `0/37`。详见 [Batch 1.5 证据摘要](../../operations/evidence/2026-11-21-batch1-5-l29/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-21 Batch 1.4（U07 玩点论杀）更新（待提交／发布）

基线 HEAD `08ad0b37a447a21f515da955204923bdaf3e6aa8` 合入 `swarm/author-u07-l10` `63c0411ddf3f0189395f224cb571a050c5d8588e`；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经独立复核 agent `5a16b8ee-86c7-497d-97bf-3ba0a35aa54b` 三轮逐行对照与上游实跑探针审查（`14 PASS / 0 FAIL / APPROVE`），清单更新为 **已复核等价 652／简化近似 3,772／缺失 0／上游未定义 0**（合计 4,424）：`U07 玩点论杀` 更新为 **36 / 0 / 0**（+9：`choudu`、`diaobingqianjiang`、`fuchou`、`fuchou2`、`youzhan`、`youdishenru` 牌与技、`zhucheng2`、`jinyan` 升为已复核等价，另有 `g_youdishenru`、`wdb_xizhenxihong`、`diyjuntun`、`zhucheng`、`kangyin2` 抽样复审通过）。队列第 1 项 U07 完成本地收口，整包八项仍 `0/37`。详见 [Batch 1.4 证据摘要](../../operations/evidence/2026-11-21-batch1-4-u07/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-20 Batch 1.3（U08 英雄杀）更新（待提交／发布）

基线 HEAD `748e2974e6f63067e7770278f1f90b5138a60009` 合入 `swarm/author-u08-l28` `00c6c7412cc7e54484f11176649dc57dd62409f5`；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经独立复核 agent `080e0e55-1095-4fc2-961b-cf5bb2bca2e5` 三轮逐行对照与上游实跑探针审查（`23 PASS / 0 FAIL / APPROVE`），清单更新为 **已复核等价 643／简化近似 3,781／缺失 0／上游未定义 0**（合计 4,424）：`U08 英雄杀` 更新为 **150 / 0 / 0**（+7：`yxsdili`、`heqin2`、`xumou`、`bolehuiyan`、`bolehuiyan2`、`bolehuiyan3`、`nvquan3` 升为已复核等价，另有 `cike`、`nichang*`、`wange*`、`ducai*`、`tongyu`/`lguiyin`、`qiandu`、`xiadao`、`jieyong`/`feigong` 等重审通过）。队列第 1 项 U08 完成本地收口，整包八项仍 `0/37`。详见 [Batch 1.3 证据摘要](../../operations/evidence/2026-11-20-batch1-3-u08/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-20 Batch 1.2（L28 雷霆万钧）更新（待提交／发布）

基线 HEAD `c119a64408e9556f58c415ddd0c5eceb1f11fdf0` 合入 `swarm/author-l28` `d50723fe23c05edfbe5fe43e6bd874c4b553a565`；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经独立复核 agent `571d828a-de8d-4c1a-b9cc-e06219f5006b` 三轮逐行对照与上游 19 步探针审查，清单更新为 **已复核等价 636／简化近似 3,788／缺失 0／上游未定义 0**（合计 4,424）：`L28 雷霆万钧` 更新为 **54 / 2 / 0**（+4：`th_buxu`、`th_tianshu`、`th_yeyan`、`cards:th_taipingyaoshu` 升为已复核等价；`th_zhengjing` 与 `thunder.css` 维持简化近似；同步完成 `th_shen_zhouyu`/`pro_qinyin`、`th_bushi`、`th_jingxie`、`th_shengong` 18 专属装备等 15 项重审通过）。队列第 1 项 L28 完成本地收口，整包八项仍 `0/37`。详见 [Batch 1.2 证据摘要](../../operations/evidence/2026-11-20-batch1-2-l28/README.md) 与 [inventory_status.json](inventory_status.json)。

## 2026-11-19 Batch 1.1（U06 欢乐卡牌 ＋ L06 wuxing）更新（待提交／发布）

基线 HEAD `352236724b5e173cc1ff74c0fb706509129ab7a1` ＋ Batch 1.1 修改；规则 `333`／NGSA `107`／schema `5`／ABI `759`／WS `8`。经独立复核 agent `602b9dd3-b8e7-4e3c-81fe-b57137a6f312` 对 14 项逐行核对与上游实际探针审查，清单更新为 **已复核等价 632／简化近似 3,792／缺失 0／上游未定义 0**（合计 4,424）：`U06` **11 / 0 / 0**（+6），`L06` **2 / 1 / 0**（`_shengke`、`wuxingpan_skill` 升为已复核等价，`wuxingpan` 卡牌依 S2 固定 `1/3` vs 默认 `0.3` 差异下调为简化近似）。队列第 1 项 U06/L06 修复完成本地验收流程收口，整包八项仍 `0/37`。详见 [Batch 1.1 证据摘要](../../operations/evidence/2026-11-19-batch1-1-u06-l06/README.md) 与 [inventory_status.json](inventory_status.json)；下方 2026-11-14 及更早记录保留为历史快照。

## 2026-11-14T03:16:45.676673+08:00 三来源运行复核日期表（文档批待提交／发布）

源码HEAD `5c73d1d9158145e33f98f2018013914667d13964` + 文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。酒诗代码已正常发布且首次精确CI10/10成功，本纯文档批尚未提交/发布。当前逐来源按[37来源新日期表](../../operations/evidence/2026-11-14-queue1-runtime/source-counts-runtime-04.md)；本批明确撤回慧眼2/3与业炎3旧等价、新签0，不从局部投影或恢复通过上调签署。[逐案与原始证据](../../operations/evidence/2026-11-14-queue1-runtime/README.md)、[变更前后原文](../../operations/evidence/2026-11-14-queue1-runtime/inventory-runtime-update-05.json)。第①项只计已复核等价，第⑧项保留各来源全部尚存差异；八项整包仍0/37，队列1仍部分、队列2未开始。原表/日期记录保持历史快照。

## 2026-11-14T02:33:04+08:00 队列1运行补课当前日期表（未提交／发布）

源码HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8不变。 当前逐来源数量以[37来源新日期表](../../operations/evidence/2026-11-14-queue1-runtime/source-counts-runtime-03.md)为准，旧表保留历史语境；U07本次再撤回2条旧等价，本批合计撤回6、新签0。酒诗恢复本地完整代码门禁与首轮preflight21项通过，最终文档复核与发布另记。队列1仍部分、队列2未开始，整包八项0/37；局部指定投影与恢复一致不替代八项验收。

## 2026-11-14T02:17:53+08:00 队列1卡牌条目补充撤回（未提交／发布）

HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。现630／3794／0、合4424，本批撤回4旧等价、新增等价0。U06猴子/写轮眼两卡绑定的规则真实不一致，原条目未只签静态字段，不能事后缩口径保留无差异；[独立补充裁定](../../operations/evidence/2026-11-14-queue1-runtime/independent/u06-l29/card-entry-adjudication.md)。[37来源日期表](../../operations/evidence/2026-11-14-queue1-runtime/source-counts-runtime-02.md)，U06现5／6／0、L29为1／8／0。酒诗只修恢复定义、其余规则差异保留；本地门禁仍在收口。队列1部分、八项0/37；本次先提交已冻结U06/L29证据，U07/L10继续采样，不进入队列2。

## 2026-11-14T02:11:43+08:00 队列1实际运行补课撤回旧等价（未提交／发布）

HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；333／107／5／759／WS8。清单现632／3792／0、合4424；清正/酒诗2项旧整技能等价因实际上游差异撤回，本批新增等价0，L29现1／8／0、U06仍7／4／0。其余来源见[37来源日期快照](../../operations/evidence/2026-11-14-queue1-runtime/source-counts-runtime-01.md)，具体分支与首次失败见[本批证据](../../operations/evidence/2026-11-14-queue1-runtime/README.md)。整包八项0/37，不因选择恢复修复或部分投影相同上调；队列1仍部分，未开始队列2。

## 2026-11-14T01:33:42+08:00 队列1部分增量发布与CI

代码 `1fa8ac6cc3a70446436d97ff49aa19ea4626e22d` 已按裁决25正常发布；[精确SHA CI](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43468568765) attempt1全部10作业success／0失败／0取消，未由agent补跑。记录时HEAD为该代码加文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。 [本地及精确代码CI记录](../../operations/evidence/2026-11-13-queue1-review/publication/code-exact-ci-first/README.md)。[37来源清单](../../operations/evidence/2026-11-13-queue1-review/source-counts-reviewed.md)634／3790／0、本批撤回215、新增等价0；八项0/37、队列1部分。旧状态沿用不作本轮重新认证，保留下方全部历史表。

## 2026-11-14T00:58:36+08:00 队列1部分增量本地门禁通过（未提交／发布）

源码HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。当前[37来源清单](../../operations/evidence/2026-11-13-queue1-review/source-counts-reviewed.md)634／3790／0；撤回215旧等价、新增等价0，沿用旧状态不作为本轮认证。三crate2446／0／6、workspace7842／0／8及其他必跑命令均exit0；完整preflight复跑21项通过，见[命令、首次失败和未验范围](../../operations/evidence/2026-11-13-queue1-review/final-local-gates.md)。八项0/37、队列1部分；实际[对照样本](../../operations/evidence/2026-11-13-queue1-review/runtime-comparison.md)仅5技能指定分支。尚未提交/发布本批。

## 2026-11-14T00:30:42+08:00 队列1复核补课增量（尚未提交／发布）

源码HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。当前清单634／3790／0，旧等价215项按确定源码差异撤回；新增等价签署0。仅静态未找到消费链、键改名或缺字段的建议保持待核，已纠正11条误/过度确认。保留的原等价不是本轮重新签署。

| 来源 | 名称 | 已复核等价 | 简化近似 | 缺失 |
| --- | --- | ---: | ---: | ---: |
| U01 | 3D精选 | 33 | 62 | 0 |
| U02 | boss | 0 | 17 | 0 |
| U03 | cardpile | 0 | 30 | 0 |
| U04 | coin | 0 | 7 | 0 |
| U05 | 杀海拾遗 | 0 | 1621 | 0 |
| U06 | 欢乐卡牌 | 7 | 4 | 0 |
| U07 | 玩点论杀 | 29 | 7 | 0 |
| U08 | 英雄杀 | 145 | 5 | 0 |
| L01 | AI优化 | 0 | 133 | 0 |
| L02 | SJ Settings | 0 | 5 | 0 |
| L03 | boss | 0 | 8 | 0 |
| L04 | cardpile | 0 | 21 | 0 |
| L05 | coin | 0 | 4 | 0 |
| L06 | wuxing | 1 | 2 | 0 |
| L07 | 全能搜索 | 0 | 13 | 0 |
| L08 | 十周年UI | 0 | 406 | 0 |
| L09 | 千幻聆音 | 0 | 102 | 0 |
| L10 | 史诗卡牌 | 8 | 9 | 0 |
| L11 | 太虚幻境 | 2 | 934 | 0 |
| L12 | 如真似幻 | 0 | 6 | 0 |
| L13 | 手杀MVP | 0 | 10 | 0 |
| L14 | 手杀ui | 0 | 141 | 0 |
| L15 | 手杀全扩 | 0 | 14 | 0 |
| L16 | 拖拽读取 | 0 | 3 | 0 |
| L17 | 斗转星移 | 0 | 11 | 0 |
| L18 | 更新保护 | 0 | 13 | 0 |
| L19 | 末日乱斗 | 60 | 9 | 0 |
| L20 | 标记补充 | 55 | 40 | 0 |
| L21 | 武皇争霸 | 0 | 9 | 0 |
| L22 | 段位制裁 | 6 | 2 | 0 |
| L23 | 电脑适配 | 0 | 1 | 0 |
| L24 | 皮肤切换 | 0 | 46 | 0 |
| L25 | 祖安武将 | 193 | 64 | 0 |
| L26 | 笨蛋插件 | 0 | 7 | 0 |
| L27 | 选将美化 | 41 | 13 | 0 |
| L28 | 雷霆万钧 | 51 | 5 | 0 |
| L29 | 龙血玄黄 | 3 | 6 | 0 |
| 全部37来源 | 4424项 | 634 | 3790 | 0 |

[应用复审后的计数及更正链](../../operations/evidence/2026-11-13-queue1-review/source-counts-reviewed.md)；[U01逐技能重审](../../operations/evidence/2026-11-13-queue1-review/independent/u01/README.md)、[其余12源141样本](../../operations/evidence/2026-11-13-queue1-review/independent/other-sources/samples.md)、[224分类项](../../operations/evidence/2026-11-13-queue1-review/nonrule-audit/README.md)、[5技能13分支实际上游对照](../../operations/evidence/2026-11-13-queue1-review/runtime-comparison.md)。下方日期表均保留为历史快照，旧整源等价措辞不代表当前验收。

## 2026-11-13T23:03:20+08:00 第0项发布核验

代码 `1d59f42b8b493e1fbac7a0e8b8c781ccef1ea61c` 已推送，精确SHA的[CI](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43442212867)通过。规则333／NGSA107／schema5／ABI759／WS8；记录时该HEAD加文档dirty。当前清单仍为849／3575／0，U01和L20保留本批下调，未新增等价认证；[逐来源表](../../operations/evidence/2026-11-13-queue0-gates/source-counts.md)及[独立分支样本](../../operations/evidence/2026-11-13-queue0-gates/independent-review/final-report.md)。整包八项0/37，队列1以后未开始。

## 2026-11-13T21:42:47+08:00 第 0 项逐来源计数核验

HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d` + tracked dirty；规则 333／NGSA 107／schema 5／ABI 759／WS8。清单逐对象重算，共 4,424 项。暗织及 Fz 两项降为简化近似，其余状态沿用原清单，未由本批重新认证；13 来源的上游实际触发补课仍待。完整产品门禁执行中，尚未提交／发布。原始计数与 SHA256 见[统计证据](../../operations/evidence/2026-11-13-queue0-gates/source-counts.json)，逐分支复核见[独立报告](../../operations/evidence/2026-11-13-queue0-gates/independent-review/final-report.md)。

| 来源 | 名称 | 已复核等价 | 简化近似 | 缺失 |
| --- | --- | ---: | ---: | ---: |
| U01 | 3D精选 | 94 | 1 | 0 |
| U02 | boss | 0 | 17 | 0 |
| U03 | cardpile | 0 | 30 | 0 |
| U04 | coin | 0 | 7 | 0 |
| U05 | 杀海拾遗 | 0 | 1621 | 0 |
| U06 | 欢乐卡牌 | 11 | 0 | 0 |
| U07 | 玩点论杀 | 36 | 0 | 0 |
| U08 | 英雄杀 | 150 | 0 | 0 |
| L01 | AI优化 | 0 | 133 | 0 |
| L02 | SJ Settings | 0 | 5 | 0 |
| L03 | boss | 0 | 8 | 0 |
| L04 | cardpile | 0 | 21 | 0 |
| L05 | coin | 0 | 4 | 0 |
| L06 | wuxing | 3 | 0 | 0 |
| L07 | 全能搜索 | 0 | 13 | 0 |
| L08 | 十周年UI | 0 | 406 | 0 |
| L09 | 千幻聆音 | 0 | 102 | 0 |
| L10 | 史诗卡牌 | 14 | 3 | 0 |
| L11 | 太虚幻境 | 2 | 934 | 0 |
| L12 | 如真似幻 | 0 | 6 | 0 |
| L13 | 手杀MVP | 0 | 10 | 0 |
| L14 | 手杀ui | 0 | 141 | 0 |
| L15 | 手杀全扩 | 0 | 14 | 0 |
| L16 | 拖拽读取 | 0 | 3 | 0 |
| L17 | 斗转星移 | 0 | 11 | 0 |
| L18 | 更新保护 | 0 | 13 | 0 |
| L19 | 末日乱斗 | 69 | 0 | 0 |
| L20 | 标记补充 | 91 | 4 | 0 |
| L21 | 武皇争霸 | 0 | 9 | 0 |
| L22 | 段位制裁 | 8 | 0 | 0 |
| L23 | 电脑适配 | 0 | 1 | 0 |
| L24 | 皮肤切换 | 0 | 46 | 0 |
| L25 | 祖安武将 | 257 | 0 | 0 |
| L26 | 笨蛋插件 | 0 | 7 | 0 |
| L27 | 选将美化 | 53 | 1 | 0 |
| L28 | 雷霆万钧 | 55 | 1 | 0 |
| L29 | 龙血玄黄 | 6 | 3 | 0 |
| 全部 37 来源 | 4424 项 | 849 | 3575 | 0 |

上游未定义 0；整包八项签署 0/37。以下旧纠偏表与原生成表保留为各自日期的历史快照。


## 2026-11-13 清单纠偏快照（第 0 项执行中）

- 核验时刻：2026-11-13T21:18:34+08:00；HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d`，tracked dirty；规则 333／NGSA 107／schema 5／ABI 759／WS8。
- 独立复核确认 `dddanzhi` 仍有 Prepare／真回合开始及选序差异，`fz_liegong` 仍有选序差异，两项从“已复核等价”撤回“简化近似”。本批生产修复与完整门禁仍在进行，尚未提交／发布。

| 当前清单范围 | 已复核等价 | 简化近似 | 缺失 |
| --- | ---: | ---: | ---: |
| U01 3D精选 | 94 | 1 | 0 |
| L20 标记补充 | 91 | 4 | 0 |
| 全部 37 来源 | 849 | 3575 | 0 |

当前数据以 [inventory_status.json](inventory_status.json) 为准；其余 35 来源计数不变。下方原生成表及结论保留为历史快照。独立源码复核及执行状态见[本批证据](../../operations/evidence/2026-11-13-queue0-gates/README.md)。


> **2026-11-11 核对勘正（Claude Code）**：本目录各表的「忠实实现」状态由生成脚本写死，未经逐项比对；抽查发现多数技能为简化近似。在逐项复核完成前，「忠实实现」一栏只能读作「已有原生落点」。本段为手工添加，重新生成时须保留。见[扫描证据](../../operations/evidence/2026-11-11-progress-scan/README.md)。

核验日期：2026-11-11。每份文档含：
1. 功能清单、菜单/规则分支语义、目标接口、验证入口与旧行为样本引用；
2. **裁决 34 逐项上游清单（第 7 节）**：逐项列出上游每个武将、技能、卡牌、模式/战役节点、配置项、界面钩子与渲染面，标明上游 `file:line`、行为摘要、原生落点（`原生规则` / `rule-vm 包` / `原生前端组件`）和当前状态（`已复核等价` / `简化近似` / `缺失` / `上游未定义`），以及现有 `decision34*` 代码逐条归类。

生成：`python3 scripts/legacy-harness/generate-g0-docs.py`  
基线：`docs/operations/evidence/2026-10-07-e01-feature-inventory/` + `archive/upstream/` 源码静态扫描 + `packages/extensions/` 正式包对照 + `docs/migration/g0/inventory_status.json` 数据驱动状态。

| ID | 名称 | 主文档 / 分册 | 武将 | 技能 | 卡牌 | 模式/节点 | 配置 | UI钩子 | 渲染面 | 已复核等价 | 简化近似 | 缺失 | 上游未定义 |
|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| U01 | 3D精选 | [U01-3D精选.md](./U01-3D精选.md) | 31 | 64 | 0 | 0 | 0 | 0 | 0 | 95 | 0 | 0 | 0 |
| U02 | boss | [U02-boss.md](./U02-boss.md) | 15 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | 17 | 0 | 0 |
| U03 | cardpile | [U03-cardpile.md](./U03-cardpile.md) | 0 | 0 | 29 | 1 | 0 | 0 | 0 | 0 | 30 | 0 | 0 |
| U04 | coin | [U04-coin.md](./U04-coin.md) | 0 | 0 | 0 | 2 | 0 | 1 | 4 | 0 | 7 | 0 | 0 |
| U05 | 杀海拾遗 | [U05-杀海拾遗.md](./U05-杀海拾遗.md) · [U05-杀海拾遗-part2.md](./U05-杀海拾遗-part2.md) | 332 | 1007 | 275 | 0 | 7 | 0 | 0 | 0 | 1621 | 0 | 0 |
| U06 | 欢乐卡牌 | [U06-欢乐卡牌.md](./U06-欢乐卡牌.md) | 0 | 5 | 6 | 0 | 0 | 0 | 0 | 11 | 0 | 0 | 0 |
| U07 | 玩点论杀 | [U07-玩点论杀.md](./U07-玩点论杀.md) | 9 | 24 | 3 | 0 | 0 | 0 | 0 | 36 | 0 | 0 | 0 |
| U08 | 英雄杀 | [U08-英雄杀.md](./U08-英雄杀.md) | 43 | 107 | 0 | 0 | 0 | 0 | 0 | 150 | 0 | 0 | 0 |
| L01 | AI优化 | [L01-AI优化.md](./L01-AI优化.md) | 0 | 69 | 1 | 0 | 63 | 0 | 0 | 0 | 133 | 0 | 0 |
| L02 | SJ Settings | [L02-SJ-Settings.md](./L02-SJ-Settings.md) | 0 | 0 | 0 | 0 | 4 | 0 | 1 | 0 | 5 | 0 | 0 |
| L03 | boss | [L03-boss.md](./L03-boss.md) | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 0 | 0 |
| L04 | cardpile | [L04-cardpile.md](./L04-cardpile.md) | 0 | 0 | 21 | 0 | 0 | 0 | 0 | 0 | 21 | 0 | 0 |
| L05 | coin | [L05-coin.md](./L05-coin.md) | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 4 | 0 | 0 |
| L06 | wuxing | [L06-wuxing.md](./L06-wuxing.md) | 0 | 2 | 1 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 |
| L07 | 全能搜索 | [L07-全能搜索.md](./L07-全能搜索.md) | 0 | 0 | 0 | 0 | 4 | 0 | 9 | 0 | 13 | 0 | 0 |
| L08 | 十周年UI | [L08-十周年UI.md](./L08-十周年UI.md) | 0 | 0 | 0 | 0 | 44 | 319 | 43 | 0 | 406 | 0 | 0 |
| L09 | 千幻聆音 | [L09-千幻聆音.md](./L09-千幻聆音.md) | 0 | 0 | 0 | 0 | 82 | 8 | 12 | 0 | 102 | 0 | 0 |
| L10 | 史诗卡牌 | [L10-史诗卡牌.md](./L10-史诗卡牌.md) | 0 | 4 | 0 | 0 | 10 | 1 | 2 | 14 | 3 | 0 | 0 |
| L11 | 太虚幻境 | [L11-太虚幻境.md](./L11-太虚幻境.md) | 320 | 374 | 32 | 206 | 0 | 0 | 4 | 2 | 934 | 0 | 0 |
| L12 | 如真似幻 | [L12-如真似幻.md](./L12-如真似幻.md) | 0 | 0 | 0 | 0 | 0 | 0 | 6 | 0 | 6 | 0 | 0 |
| L13 | 手杀MVP | [L13-手杀MVP.md](./L13-手杀MVP.md) | 0 | 0 | 0 | 0 | 7 | 1 | 2 | 0 | 10 | 0 | 0 |
| L14 | 手杀ui | [L14-手杀ui.md](./L14-手杀ui.md) | 0 | 0 | 0 | 0 | 48 | 53 | 40 | 0 | 141 | 0 | 0 |
| L15 | 手杀全扩 | [L15-手杀全扩.md](./L15-手杀全扩.md) | 0 | 0 | 0 | 3 | 11 | 0 | 0 | 0 | 14 | 0 | 0 |
| L16 | 拖拽读取 | [L16-拖拽读取.md](./L16-拖拽读取.md) | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 | 3 | 0 | 0 |
| L17 | 斗转星移 | [L17-斗转星移.md](./L17-斗转星移.md) | 0 | 0 | 0 | 0 | 9 | 1 | 1 | 0 | 11 | 0 | 0 |
| L18 | 更新保护 | [L18-更新保护.md](./L18-更新保护.md) | 0 | 0 | 0 | 0 | 12 | 0 | 1 | 0 | 13 | 0 | 0 |
| L19 | 末日乱斗 | [L19-末日乱斗.md](./L19-末日乱斗.md) | 2 | 66 | 0 | 0 | 1 | 0 | 0 | 69 | 0 | 0 | 0 |
| L20 | 标记补充 | [L20-标记补充.md](./L20-标记补充.md) | 0 | 28 | 3 | 0 | 15 | 46 | 3 | 92 | 3 | 0 | 0 |
| L21 | 武皇争霸 | [L21-武皇争霸.md](./L21-武皇争霸.md) | 0 | 0 | 0 | 1 | 7 | 0 | 1 | 0 | 9 | 0 | 0 |
| L22 | 段位制裁 | [L22-段位制裁.md](./L22-段位制裁.md) | 0 | 5 | 0 | 0 | 3 | 0 | 0 | 8 | 0 | 0 | 0 |
| L23 | 电脑适配 | [L23-电脑适配.md](./L23-电脑适配.md) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | 0 |
| L24 | 皮肤切换 | [L24-皮肤切换.md](./L24-皮肤切换.md) | 0 | 0 | 0 | 0 | 14 | 22 | 10 | 0 | 46 | 0 | 0 |
| L25 | 祖安武将 | [L25-祖安武将.md](./L25-祖安武将.md) | 60 | 84 | 5 | 0 | 68 | 40 | 0 | 257 | 0 | 0 | 0 |
| L26 | 笨蛋插件 | [L26-笨蛋插件.md](./L26-笨蛋插件.md) | 0 | 0 | 0 | 0 | 5 | 1 | 1 | 0 | 7 | 0 | 0 |
| L27 | 选将美化 | [L27-选将美化.md](./L27-选将美化.md) | 0 | 31 | 0 | 0 | 10 | 12 | 1 | 53 | 1 | 0 | 0 |
| L28 | 雷霆万钧 | [L28-雷霆万钧.md](./L28-雷霆万钧.md) | 14 | 33 | 1 | 0 | 7 | 0 | 1 | 55 | 1 | 0 | 0 |
| L29 | 龙血玄黄 | [L29-龙血玄黄.md](./L29-龙血玄黄.md) | 0 | 5 | 0 | 0 | 1 | 1 | 2 | 6 | 3 | 0 | 0 |
| **合计** | **37 来源** | **37 主文档 + 1 分册** | **834** | **1908** | **377** | **215** | **434** | **507** | **149** | **851** | **3573** | **0** | **0** |
