# ABI 编号协调（ABI Number Coordination）

并行 agent 共享同一工作区时，引擎 Rule VM ABI 编号与 `docs/adr/` 编号必须**串行领取**。
本文件是唯一领取点：**改 Rust 引擎代码或写 ADR 之前，先在本文件登记**，再动代码。

## 规则（Rules）

1. 新增任何以 `ADDITIVE_RULE_VM_ABI_VERSION_N` 门控的原语（trigger 过滤器 / effect /
   modifier / 回合态 flag / hash 变化）时：先把 `N` 写入下方「ABI 领取表」，并把
   `ABI-MAX:` 更新为 `684`（当前值）；
2. `crates/engine-api/src/version.rs` 的 `RULE_VM_ABI_VERSION` 只能 **≤ ABI-MAX**；
   engine-api 有单元测试强制此不变量（`abort_bump_without_claim`），违反即构建失败，
   失败信息会指向本文件。领取表内编号不得重复。
3. 历史 ABI 1..=353 视为已领取（祖父条款，不逐条登记）。
4. 写 ADR（`docs/adr/NNNN-*.md`）前，先在下方「ADR 领取表」认领下一个编号，
   避免两个 agent 写同号 ADR 互相覆盖。
5. 每个 ABI 批次编译、测试和打包结束后，必须按架构计划 §37.10 删除工作区
   `target/` 与外部 `CARGO_TARGET_DIR`，避免 Rust 编译产物和增量缓存无限累积；
   这里清理的是可再生成的构建内容，不是源码、扩展包，也不是取消构建目录配置；
   `.npack`、`extensions/` 及其他发布产物不属于清理对象。统一使用
   `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`；确认没有 Cargo/rustc 占用后执行
   `cargo clean` 和 `cargo clean --target-dir /private/tmp/noname-next-cargo-target`，并以两个
   `target` 目录均不存在作为批次收尾验收条件。

本次会话收尾执行示例（已完成）：`cd /Users/ruyimg/Documents/Claude/sanguosha/noname-next && CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target cargo clean && cargo clean --target-dir /private/tmp/noname-next-cargo-target`。  
验收确认：`/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target/` 与 `/private/tmp/noname-next-cargo-target` 均不存在（均不保留编译产物，`.npack` / `extensions` 发布物保留）。

## 编号段（Segments）

| 段 | 范围 | 归属 |
|---|---|---|
| B 段 | 354–399 | 另一 agent（Codex 会话，当前活跃于 offline/refresh 等包） |
| A 段 | 400–449 | 本 agent（Claude 会话，负责 shiji 等包） |
| C 段 | 450–469 | 本 agent（omp 会话，负责 shenhua 等包） |
| D 段 | 470–489 | 另一 agent（opencode 会话，负责 xianding 等包） |
| 溢出段 | 490–499 | 任一段耗尽后协商续领 |
| E 段 | 562–599 | 本 agent（Hermes 会话，负责 standard/newjiang/clan/onlyOL/key/extra/yijiang 等未认领包；559-561 已被其他 agent 占用） |
| 第二溢出段 | 600–699 | 原编号段全部耗尽后，继续按本表逐号串行领取 |

ABI-MAX: 753

> 2026-10-10 账面对齐（进度主稿 T15①）：735–753 批原语均已在
> content-compiler（additive 常量 726…753，缺 743 与领取表一致）与
> engine-core 宿主（如 `ResolveXiangshu`）落地，源 npack
> `content/original/ported/{sp,sp2,xianding}` 分别要求 745/753/751，
> 但 `RULE_VM_ABI_VERSION` 与 ABI-MAX 停在 734，按 compat
> `package ∈ [3, host)` 判据被拒装。本次把 ABI-MAX 与
> `RULE_VM_ABI_VERSION` 升至 753 并重打包部署这三包（部署副本
> 714/713/706 → 745/753/751）；兼容判据对 host 单调，已部署低版本包
> 不受影响。

## ABI 领取表（Claims）

| ABI | 原语 / 技能 | 领取者 | 日期 | 备注 |
|---|---|---|---|---|
| 353 | `zj_duanbing`：回合对历史 + 近距【闪】数（`choose_add_current_sha_target_within_distance` 等） | B (Codex) | 2026-08-01 | 已在 `version.rs` 与 content-compiler 落地 |
| 354 | `spniluan`：转化【杀】未造成伤害时退还本次使用次数（`refund_use_count_if_no_damage`） | B (Codex) | 2026-08-01 | 复用既有单次【杀】伤害追踪与 `hand_equip_expansion` 来源区 |
| 355 | `twdaoji`：可获得手牌/装备目标约束与 TW 盗戟结算（`has_gainable_hand_equip`、`resolve_tw_daoji`） | B (Codex) | 2026-08-01 | 复用既有 `daoji` 选牌、获得与强制使用管线 |
| 356 | `yijue`：指定目标非锁定技失效至回合结束、指定回复目标的可取消选择（`suppress_non_locked_skills_until_turn_end`、`choose_recover_target`） | B (Codex) | 2026-08-01 | 复用既有拼点、临时技能和回复管线 |
| 357 | `twdanji`：每回合首次使用转化【杀】（`owner_first_transformed_sha_used_this_turn`） | B (Codex) | 2026-08-01 | 复用既有卡牌使用历史、单骑觉醒与加技能原语 |
| 358 | `twbaimei`：空手牌时防止属性或普通锦囊伤害（`owner_target_hand_empty_elemental_or_trick_damage_before`） | B (Codex) | 2026-08-01 | 复用既有伤害前置上下文与 `prevent_current_damage` |
| 359 | `twrangyi`：交出全部手牌后使用其中一张或受伤（`give_all_hand_then_choose_use_one_or_damage`） | B (Codex) | 2026-08-01 | 复用既有技能牌合法目标形状与用牌管线 |
| 360 | `zhengfu`：声明类别，目标交同类别牌或本次【杀】不可响应（`choose_category_then_target_give_matching_or_sha_unrespondable`） | B (Codex) | 2026-08-01 | 复用现有类别判定、赠牌和当前【杀】直击管线 |
| 361 | `twfenwu`：选定目标后失去体力并使用无距离虚拟【杀】，按本回合基本牌变体数加伤（`choose_lose_hp_use_virtual_sha_by_basic_variants`） | B (Codex) | 2026-08-01 | 复用虚拟【杀】选择/响应管线，按效果标签开启最小历史位 |
| 362 | `twfupan`：伤害后按点摸牌、交牌并按目标累计首次摸二/再次可伤害后封锁（`resolve_twfupan`） | B (Codex) | 2026-08-01 | 复用伤害数量上下文、手牌装备选牌/赠牌/伤害管线；仅在技能实例保存每目标 0/1/2 状态 |
| 363 | `twjiaohua`：自己或最低体力角色摸牌后获得本次未获类别牌（`owner_alive_draw_player_owner_or_min_hp_twjiaohua_available`、`resolve_twjiaohua`） | B (Codex) | 2026-08-01 | 复用 `CardsDrawn` 事件卡、摸牌/弃牌堆扫描和回合旗标；每名受益角色每回合记录基本/锦囊/装备类别 |
| 364 | `twkujian`：赠予并追踪“谏”实体牌，按使用/打出或其他失去分支结算（`UseOrRespond` 移动原因、`resolve_twkujian`） | B (Codex) | 2026-08-01 | 复用主动选牌/赠牌、技能实例状态、摸牌与强制弃牌管线；移动原因用于禁止区域猜测 |
| 365 | `twruilian`：每轮选择角色，记录其回合弃牌数量/类别并于回合结束按类别依次获得弃牌堆牌（`resolve_twruilian`） | B (Codex) | 2026-08-01 | 复用回合开始/结束、移动原因、技能实例状态、目标/类别选择与获得牌管线 |
| 366 | `twfujian`：无武器时从牌堆装备武器，回合外失去武器后失去体力（`equip_weapon_from_draw_pile`、对应移动过滤器） | B (Codex) | 2026-08-01 | 复用装备区、牌堆扫描、实体移牌与失去体力管线；不引入技能专属状态 |
| 367 | `twjianwei`：准备阶段双向发起拼点、武器攻击范围点数修正与胜负分区获牌（`resolve_twjianwei`） | B (Codex) | 2026-08-01 | 复用既有拼点开始/选牌/展示/移牌事件及目标卡选择屏障，仅新增技能专属状态机 |
| 368 | `twtanfeng`：弃置目标区域牌后令其选择实体牌当【杀】，否则受火焰伤害并选择跳过阶段（`resolve_twtanfeng`） | B (Codex) | 2026-08-01 | 复用区域选牌、实体牌转化【杀】、伤害与既有阶段跳过机制；仅新增技能串联状态机 |
| 369 | `twzhibian`：出牌阶段开始拼点，胜后移动对应区域牌/回复/背水，未胜失去体力（`resolve_twzhibian`） | B (Codex) | 2026-08-01 | 复用既有拼点、非基本牌弃置、区域移动与回复管线；仅新增技能串联状态机 |
| 370 | `twyuyan`：高体力【杀】使用者交高点数/非基本牌，否则取消当前目标（`resolve_twyuyan`） | B (Codex) | 2026-08-01 | 复用当前【杀】目标移除、手牌/装备选牌和赠牌管线；新增严格物理【杀】与体力差过滤器 |
| 371 | `twguoyi`：目标禁用手牌/弃牌二选一，并在满足双条件时结算后复制原牌（`resolve_twguoyi`） | B (Codex) | 2026-08-01 | 复用手牌使用响应禁用 modifier、动态弃牌和虚拟牌使用管线；技能状态仅记录本回合已选分支与当前牌的重放意图 |
| 372 | `twchuhai`：记录本回合造成濒死的不同角色，使命成功后废除判定区并依次收牌；获得他人牌后弃置其中一张（`resolve_twchuhai`） | B (Codex) | 2026-08-01 | 复用伤害/濒死历史、座次赠牌和获得牌事件；新增权威判定区废除状态与精确“本批新获牌”选择 |
| 373 | `twyizhu`：开局/结束阶段埋置“遗珠”，被他人单目标使用时修改目标并摸牌，进入弃牌堆时解除追踪（`resolve_twyizhu`） | B (Codex) | 2026-08-01 | 复用技能实例实体牌追踪、牌堆插入、当前用牌目标修改和移动事件；不引入独立牌标记系统 |
| 374 | `twluanchou/twgonghuan`：重置两名“共患”持有者，并将一次他人伤害完整转移给自己（`replace_shared_skill_on_two_targets`、`redirect_current_damage_to_owner`） | B (Codex) | 2026-08-01 | 复用双目标主动技、技能增删、伤害前置取消与带来源/牌/属性伤害管线；仅增加通用“已转移”伤害标志防链式转移 |
| 375 | `twxiawei/twqiongji`：建立可如手牌使用/打出的“威”技能牌堆，准备阶段妄行补充，使用后摸牌、空堆受伤加伤（`resolve_twxiawei`） | B (Codex) | 2026-08-01 | 复用技能牌堆、实体牌来源历史、临时弃牌/减上限与伤害前置；补充受技能约束的“技能牌堆如手牌”通用来源能力 |
| 376 | `twwanwei/twyuejian`：最低体力伤害防止与回合末顶牌获得/底牌使用；牌置牌堆顶底并按数量获得阶梯收益（`resolve_twwanwei`、`resolve_twyuejian`） | B (Codex) | 2026-08-01 | 复用当前伤害取消、临时技能、牌堆首尾移动、实体牌使用、增上限/回复与标记手牌上限；仅新增两段技能状态机 |
| 377 | `twshengxi/twkuanji`：准备阶段获得【调剂盐梅】、无伤用牌回合末获智囊并摸牌，以及非使用进入弃牌堆后转赠一张（`resolve_twshengxi`、`resolve_twkuanji`） | B (Codex) | 2026-08-01 | 复用按牌类/牌定义取牌、既有智囊选择、实体移动事件卡与单牌转赠屏障；不引入旧 JS 的全局卡牌创建状态 |
| 378 | `twmuyue/twchayi`：弃牌声明牌名并令角色获牌、同名时记录免弃；结束阶段布置展示/下次用牌弃牌并在目标下回合末按手牌数变化切换（`resolve_twmuyue`、`resolve_twchayi`） | B (Codex) | 2026-08-01 | 复用牌类枚举、按牌类取牌、事件牌弃置、公开手牌及用牌后弃牌屏障；技能状态只保存免弃位与察异目标/分支/基准手牌数 |
| 379 | `twqianju/twqingxi`：装备数减距离、近距伤害后填充空装备栏；首杀目标选择摸牌且不可响应，或双方按数量弃装备并令伤害加一（`resolve_twqianju`、`resolve_twqingxi`） | B (Codex) | 2026-08-01 ✓ | 复用装备数距离 modifier、装备栏合法性、共享牌堆装备检索、当前杀目标不可响应与目标伤害加值；双方装备弃置选择串联、Golden 与 TW 扩展包已落地 |
| 380 | `twzaoli`：出牌阶段仅可使用/打出本回合获得的手牌；阶段开始弃全部装备和至少一张非装备手牌，等量摸牌并按原装备副类别补装备，补入超过两张时失去体力（`resolve_twzaoli`、本回合新获手牌来源约束） | B (Codex) | 2026-08-01 ✓ | 复用实体移牌、装备槽、共享牌堆检索与手牌使用/响应校验；按实体牌记录的本回合获牌集合、躁厉结算、Golden 与 TW 扩展包已落地 |
| 381 | `twzhengjian/twzhongchi`：开局选择征建条件，其他角色出牌阶段未用非基本牌/未获得牌时交牌并可切换；累计半数不同角色后回复、改为可造成伤害且受到杀伤害加一（`resolve_twzhengjian`、出牌阶段获牌历史） | B (Codex) | 2026-08-01 ✓ | 复用出牌阶段牌类别历史、赠牌、回复与 `sha_damage_taken`；当前分支、觉醒位、不同交牌角色集合、Golden 与 TW 扩展包已落地 |
| 382 | `twzhenxi/twyangshi`：每回合一次杀目标弃至多距离张手牌、移动场上牌或背水依次执行；受伤后若攻击范围未覆盖所有其他角色则永久加一，否则获得【杀】（`resolve_twtianyu`） | B (Codex) | 2026-08-01 ✓ | 复用距离查询、目标手牌选择、场上牌合法迁移、攻击范围 modifier 与按牌名获牌管线；Golden 与 TW 扩展包已落地 |
| 383 | `zhenshan`：每回合一次声明基本牌，选择手牌更少的其他角色交换整手牌后使用/打出（source-free `swap_lower_hand` cost） | B (Codex) | 2026-08-01 ✓ | 复用 source-free 基本牌声明、整手实体移牌、通用响应恢复与技能 `use_count`；Golden 与 TW 扩展包已落地 |
| 384 | `twfenghan/twcongji`：伤害牌首目标后按目标数任选角色摸牌；回合外红色牌因弃置进入弃牌堆后任选转赠（`resolve_twwujing`） | B (Codex) | 2026-08-01 ✓ | 复用事件目标数、任选目标摸牌、事件实体牌绑定与通用移牌管线；Golden 与 TW 扩展包已落地 |
| 385 | `twyingji/twshanghe`：回合外空手牌摸一后声明基本/锦囊牌；濒死时其他角色逐人交牌且无【酒】则回复至一（`draw_one_outside_turn_empty_hand`、`resolve_twshanghe`） | B (Codex) | 2026-08-01 ✓ | 复用 source-free 声明牌、摸牌、座次选牌赠予与限定技计数；Golden、conformance 与 TW 扩展包已落地 |
| 386 | `twkaiji/twshepan`：按整局不同濒死角色数选人摸牌；目标牌后摸牌/置顶，手牌相等返还次数并可排除当前目标（`resolve_twwangchang`） | B (Codex) | 2026-08-02 ✓ | 复用权威濒死事件、目标选牌、牌堆顶移动与当前牌目标排除；Golden、conformance 与 TW 扩展包已落地 |
| 387 | `twfuzuan/twchongqi/twfeifu`：转换技元数据、切换目标转换技及单目标【杀】赠牌后可装备（`resolve_twcaozhao`） | B (Codex) | 2026-08-02 ✓ | 复用技能分支位、全场授予技能、手牌/装备实体移牌与装备使用管线；Golden、conformance 与 TW 扩展包已落地 |
| 388 | `twjingce/yuzhang`：出牌阶段第 X 张牌摸牌并累计“策”、受伤禁牌/弃牌及六阶段跳过（`resolve_twguohuai`） | B (Codex) | 2026-08-02 ✓ | 复用精确用牌伤害追踪、临时禁用 modifier、标记和阶段跳过；Golden、conformance 与 TW 扩展包已落地 |
| 389 | `twlihuo/twchunlao`：普通【杀】转火、加目标及濒死失血代价；全场唯一“醇”的置入、增伤与濒死救援（`resolve_twchengpu`） | B (Codex) | 2026-08-02 ✓ | 复用当前【杀】属性/目标/伤害基数、技能牌堆、赠牌、回复与精确濒死历史；Golden、conformance 与 TW 扩展包已落地 |
| 390 | `twxiaolian`：替换他人的单目标【杀】目标，受伤后可将牌作为“马”置于原目标并累计其被动距离（`resolve_twcaoang`） | B (Codex) | 2026-08-02 ✓ | 复用当前【杀】目标替换、技能实例精确牌状态、技能牌堆与全局距离 modifier；Golden、conformance 与 TW 扩展包已落地 |
| 391 | `twcuorui/twliewei`：限定摸至场上最高手牌（至多摸五张）并废除判定区/伤害；杀人后摸二或重置限定技（`resolve_twniujin`） | B (Codex) | 2026-08-02 ✓ | 复用动态补牌、判定区废除状态、任选目标伤害及限定技次数重置；Golden、conformance 与 TW 扩展包已落地 |
| 392 | `twtijin`：攻击范围内其他角色的单目标【杀】改指自己，精确结算后弃置使用者一张牌（`owner_alive_other_source_sha_resolved_twxiaolian_redirect_has_hand_equip`、`resolve_twcaoang` 清理动作） | B (Codex) | 2026-08-02 ✓ | 复用 ABI390 改目标状态、`sha_resolved` 收尾事件与通用目标牌弃置管线；Golden、全量 compiler、engine-api 与 TW 扩展包已落地 |
| 393 | `twmoukui`：每个【杀】目标可摸牌/弃牌/背水两项；背水目标未因该【杀】进入濒死时在完整结算后反弃来源牌（`resolve_twmoukui`、Sha backwater/dying target history） | B (Codex) | 2026-08-02 ✓ | 复用当前【杀】-目标记录、通用摸牌与目标牌弃置管线；Golden、全量 compiler、engine-api、conformance 与 TW 扩展包已落地 |
| 394 | `twfuman`：每个出牌阶段向每名其他角色各赠一张手牌并追踪实体牌；追踪牌动态视为【杀】，使用/打出结算后按是否造成伤害摸一或两张（`resolve_twfuman`、技能追踪牌 selector） | B (Codex) | 2026-08-02 ✓ | 复用主动目标去重、`twkujian` 实体牌追踪和【杀】完整结算记录；复合状态、离手清理、Golden、全量 compiler、engine-api、conformance 与 TW 扩展包已落地 |
| 395 | `cuijin`：自己或攻击范围内角色使用【杀】时弃一牌令基础伤害+1；完整结算未造成伤害则摸一并伤害使用者（`resolve_cuijin`、精确【杀】追踪） | B (Codex) | 2026-08-02 ✓ | 复用通用弃牌、当前【杀】基础伤害修改和 ABI394 的 `ShaResolved.card_flags`；命中/未命中 Golden、全量 compiler、engine-api、conformance 与 TW 扩展包已落地 |
| 396 | `twqingkou`：准备阶段视为对一名其他角色使用【决斗】，该决斗伤害来源摸一；若来源为自己则跳过判定和弃牌阶段（`resolve_twqingkou`、精确虚拟【决斗】追踪） | B (Codex) | 2026-08-02 ✓ | 复用 `choose_use_virtual_card`、`DuelResolved` 与既有阶段跳过状态；双方胜负 Golden、决斗回归、全量 compiler、engine-api、conformance 与 TW 扩展包已落地 |
| 397 | `twchenxun`：每轮可对未以此法选择过的角色连续使用虚拟【决斗】；目标受伤则摸一并可继续，否则失去体力（`resolve_twchenxun`、轮次目标与精确虚拟牌组合追踪） | B (Codex) | 2026-08-02 ✓ | 已落地；复用 ABI396 虚拟【决斗】物理牌绑定、`DuelResolved` 和牌移动收尾，仅保存本轮已选目标集合；Golden 覆盖连续追击、失败失体力与取消 |
| 398 | `choose_discard_equipment_prevent_current_damage(player)`：弃置一张有效装备牌并防止整段当前伤害（`xinjiaojin` 骄矜） | B (Codex) | 2026-08-02 ✓ | 已落地；复用 ABI57 装备牌筛选/选择屏障与权威 `prevented_damage_ids`，不新增技能专属状态；配合既有 ABI57 `choose_card_user_or_add_target` 闭环 TW 孙鲁班；Golden 覆盖 2 点整段防止并回归原“减 1”语义 |
| 399 | `resolve_twfenxian(player, target, card, action)`：目标选择以双方场上牌当【决斗】使用，或令拥有者视为对其使用【火攻】（幻黄盖「焚险」） | B (Codex) | 2026-08-02 ✓ | 已落地；新增通用场上实体牌独立锦囊使用来源，复用原目标校验、无懈、决斗/火攻结算与 Custom choice，不新增技能状态；Golden、全量 compiler、engine-api、决斗回归及 TW 扩展包已落地 |
| 400 | `spjungong`：峻攻弃牌/失体力 + 虚拟【杀】 + 命中禁用（`spjungong_discard_or_lose_then_sha`、`owner_source_sha_damage_recorded_by_skill`、`ban_active_until_turn_end`） | A (Claude) | 2026-08-01 | A 段首个 |
| 401 | `spwanwei`：主动回复/失体力与濒死回复/失体力（`spwanwei_active`、`spwanwei_dying`） | A (Claude) | 2026-08-01 | 补齐已落地 ABI 401 的协调登记 |
| 402 | `spfenming`：主动目标约束 `hp_lte_owner` + DSL 横置/获牌分支 | A (Claude) | 2026-08-01 | 复用 `MARK_CHAIN`/`take_one_card`；闭环 SP 陈董 |
| 403 | `spxizhan`：弃牌或失体力后按花色视为用牌（`resolve_spxizhan`） | A (Claude) | 2026-08-01 | 复用弃牌选择、虚拟用牌与 `disable_skill_by_def_until_turn_end`；闭环 SP 花鬘 |
| 404 | `spsongshu`：摸牌阶段改为从仁库选牌并禁止指定其他角色（`resolve_spsongshu`、`owner_alive_draw_player_hp_gt_owner_skill_pile_*`） | A (Claude) | 2026-08-01 | 仁库复用割卜技能牌堆；闭环 SP 张温。DSL 接线 + engine-core 结算（resolve_spsongshu / ChooseSpsongshuCards / MoveCards+AddSkillUntilTurnEnd）已由 Claude 补全；.skill 文件待写 |
| 405 | `owner_alive_lost_hand_this_turn`：拥有者本回合失去过手牌（`spdiancai` 点才） | A (Claude) | 2026-08-01 | 复用 `turn.card_loss_count`；已落地 engine-core/compiler，`spdiancai.skill` 编译通过 |
| 406 | `spcunsi`：翻面 + 从牌堆取一张【杀】交给目标 + 授予嗣标记技能（`resolve_spcunsi`） | A (Claude) | 2026-08-01 | 复用 `turn_over`、`gain_card_by_kind` 牌堆查找与 `add_skill`/`add_mark`；闭环 SP 孙尚香 |
| 407 | `owner_source_first_sha_this_play_phase`：拥有者本出牌阶段首次使用【杀】（`dangmo` 当陌 / `liubing` 流冰共用） | A (Claude) | 2026-08-01 | 复用回合内使用牌计数位；配合 `choose_add_current_sha_target`/`card_suit` 直译当陌/流冰 |
| 408 | `owner_source_target_mark_positive_<id>`：拥有者使用牌指定有标记目标（`scfuhai` 覆海） | A (Claude) | 2026-08-01 | 复用 `card_targeted` 事件与 `owner_is_target_mark_positive_` 判定；配合 `make_current_card_unrespondable`+`draw_cards` 直译覆海，闭环神孙策 |
| 409 | `resolve_yuli(player)`：造成伤害前非雷改雷、已雷则伤害+1（prevent+同源同量同牌雷属性重放）；自己受伤前取消并摸伤害点数张（`mb_shen_machao` 驭雳） | A (Claude) | 2026-08-02 | 复用 ABI 374 伤害重放管线（prevented_damage_ids + damage_redirect_in_progress）与 `prevent_current_damage`/`draw_cards`；配合 `jimie`（寂灭）霆标记联动 |
| 410 | `resolve_tingwei(player, sha_target)`：杀指定目标后加 4 霆标记，目标从四选一（非锁失效/交装备/该杀对其伤害+1/随机弃牌）或取消横置（`mb_shen_machao` 霆威） | A (Claude) | 2026-08-02 ✓ | 复用 `suppress_non_locked_skills`（ABI 356）、赠装备、`increase_current_sha_target_damage`、随机弃牌与 `MARK_CHAIN` 横置；两段 choice |
| 411 | `resolve_xinmingshi(player, source)`：受伤后来源强制弃一张手牌/装备/判定区牌，弃红色牌则回复 1、弃牌进弃牌堆则获得（`sp_kongrong` 新名士） | A (Claude) | 2026-08-02 ✓ | 复用强制弃牌选择、`recover_hp` 与获牌管线；配合 `xinlirang` 谦标记（mark 2286538070）条件 |
| 412 | `resolve_xunyi(player, mode)`：开局选“义”目标（获义标记）并与拥有者共轭守护：任一造成伤害另一人摸 1、任一受伤另一人强制弃 1 张手牌/装备（`wangfuzhaolei` 殉义） | A (Claude) | 2026-08-02 ✓ | 复用目标选择、标记、`draw_cards` 与强制弃牌管线；damage 事件按来源/目标分流 |
| 413 | `resolve_boming(player, target)`：将本次主动技选中的手牌赠给目标（`sp_xujing` 博明，触发 CardsGiven 事件） | A (Claude) | 2026-08-02 | 复用 selected_cards 与赠牌管线 |
| 414 | `resolve_ejian(player, target)`：赠牌后目标若有同类牌则二选一——受 1 点伤害 / 展示手牌并弃所有同类牌（`sp_xujing` 恶荐） | A (Claude) | 2026-08-02 ✓ | 复用 `CardsGiven` 事件、伤害与弃牌管线；二选一 choice |
| 415 | `resolve_yiyong(player)`：受伤后获得造成伤害的杀实体牌，可对来源反击（来源无武器则反击杀伤害基数 2）（`sp_wangshuang` 异勇） | A (Claude) | 2026-08-02 | 复用弃牌堆检索、实体牌当杀与 `increase_current_sha_base_damage` 管线 |
| 416 | `resolve_shanxie(player)`：出牌阶段限一次，牌堆有武器则获得，无则随机夺一名有武器角色的武器（`sp_wangshuang` 善械） | A (Claude) | 2026-08-02 ✓ | 复用牌堆检索、权威 RNG 与获得装备管线 |
| 417 | `resolve_sheyi(player, target)`：其他角色受伤前（每轮一次）交至少 max(1,hp) 张 he 牌防伤（`caizhenji` 舍医） | A (Claude) | 2026-08-02 | 复用赠牌与 `prevent_current_damage` 管线；一段选牌 choice |
| 418 | `resolve_tianyin(player)`：结束阶段获得牌堆顶每个本阶段未用类别牌的第一张（`caizhenji` 天荫） | A (Claude) | 2026-08-02 ✓ | 复用 `owner_card_category_mask` 与牌堆移动管线 |
| 419 | `resolve_dbzhuifeng(player)`：失 1 点体力并对一名其他角色视为使用【决斗】（`db_wenyang` 追锋，魏势力） | A (Claude) | 2026-08-03 | 复用失体力、目标选择与虚拟【决斗】管线 |
| 420 | `resolve_dbquedi(player, target, card)`：单目标杀/决斗指定后三选一——获得目标一张手牌 / 弃一张基本牌令此牌伤害+1 / 背水减 1 上限并全执行（`db_wenyang` 曲迪）+ 新过滤器 `owner_source_sha_or_juedou_single_target` | A (Claude) | 2026-08-03 ✓ | 复用获牌、弃牌、`increase_current_card_base_damage` 与减上限管线 |
| 421 | `resolve_spyinju(player, target)`：目标二选一——对拥有者使用一张【杀】/ 获得跳过标记（其下回合准备阶段跳过出牌与弃牌阶段，纯 DSL 辅助技能消费）（`sp_xinpi` 隐据） | A (Claude) | 2026-08-03 | 复用受迫用杀、标记与 `skip_next_play/discard_phase` 管线 |
| 422 | `resolve_spchijie(player, source, card)`：成为其他角色单目标牌目标时判定，点数 > 6 取消该目标（`sp_xinpi` 持节） | A (Claude) | 2026-08-03 ✓ | 复用 `flip_judgment` 与目标排除管线 |
| 423 | `resolve_spshangyi(player, target)`：展示目标手牌并获得其中一张（`sp_jiangqing` 上议） | A (Claude) | 2026-08-03 | 复用展示与获牌管线 |
| 424 | `resolve_spjianyi(player)`：其他角色结束阶段获得弃牌堆中的防具牌（`sp_jiangqing` 简议，近似本回合） | A (Claude) | 2026-08-03 ✓ | 复用弃牌堆检索与获牌管线 |
| 425 | `resolve_fyjianyu(player, source, target)`：出牌阶段有喻标记角色用牌指定另一喻标记角色时目标摸 1（`feiyi` 谏喻）+ 主动选 2 名角色获喻标记 | A (Claude) | 2026-08-03 ✓ | 复用标记与摸牌管线 |
| 426 | `resolve_liubing(player, card)`：出牌阶段首次使用物理【杀】时将其花色改为方块（`wujing` 流冰） | A (Claude) | 2026-08-03 | 直接改实体牌 suit，无新状态 |
| 427 | `resolve_heji(player, target, card)`：其他角色单目标使用决斗/红杀后，可对原目标视为使用【杀】或【决斗】（`wujing` 合击） | A (Claude) | 2026-08-03 ✓ | 复用虚拟用牌管线 |
| 428 | `resolve_spyajun(player, less_def, stage)`：出牌阶段开始本回合新获牌时，选新获牌拼点，胜后可选将拼点牌置牌堆顶，败则加“戒”标记（手牌上限-标记数）（`sp_cuiyan` 压军 + 纯 DSL 辅助技能 spyajun_less 戒 / spyajun_draw 势） | A (Claude) | 2026-08-03 ✓ | 复用拼点提交（hand_compare_commit_context）、牌堆顶移动与手牌上限标记管线；过滤器 owner_alive_hand_gained_this_turn_can_compare_other 改为按 turn.gained_hand_cards 实际判定；Golden 与 shiji 扩展包已落地 |
| 429 | `resolve_spzundi(player, target)`：弃 1 手牌给目标后判定，黑则目标摸 3 / 红则目标移动一张场上牌（复用 movable_field_cards_from + field_card_move_destinations 移动管线）（`sp_cuiyan` 尊帝） | A (Claude) | 2026-08-03 ✓ | 复用判定、摸牌与场上牌移动管线；Golden 与 shiji 扩展包已落地 |
| 450 | 跳过判定+摸牌阶段：`skip_judge_and_draw_phases(player)`（`clbjisu` 济溯） | C (omp) | 2026-08-01 ✓ | 复用 `pending_judge_skips` 与 `turn_flag::SKIP_DRAW` 既有机制；闭环 collab 叶诗文。设计见 ADR 0303，落地见 ADR 0304 |
| 451 | `min(a,b)` 表达式：上限截断（`drlt_zhengu` 镇骨、`beizhan` 备战等） | C (omp) | 2026-08-01 ✓ | 新增 VM MIN 操作码 + ExprIr::Min，编译为 min 语义；闭环 shenhua 郝昭、sp2 备战。落地见 ADR 0306 |
| 452 | 目标过滤器 `owner_is_target_sha_or_trick`：成为【杀】或锦囊牌目标（`weipo` 威迫、`tianming` 天命等 33 个消费技能） | C (omp) | 2026-08-01 ✓ | 复用 `card_is_sha` 与锦囊类型判定，新增 TriggerFilter 变体 + 分发臂；闭环 sp2 韩馥。落地见 ADR 0307 |
| 453 | `current_phase_player` 表达式：当前回合角色（`stdrenzheng` 仁政、`stdshuchen` 等 294 个消费技能） | C (omp) | 2026-08-01 ✓ | 新增 VM 操作码 + ExprIr 变体，求值当前回合玩家；闭环 sixiang 陆通 |
| 454 | 过滤器 `owner_source_or_within_distance_one_sha_or_juedou`：任意角色使用【杀】/【决斗】时使用者是拥有者或与其距离 ≤ 1（`dccuijin` 崔进、`twzhenliang` 等） | C (omp) | 2026-08-01 ✓ | 复用 `card_is_sha`/`card_is_juedou` 与距离判定；闭环 huicui 界沮授。落地见 ADR 0320 |
| 455 | `choose_target_draw_to_max_hp_cap_then_add_skill(player, cap, skill_def)`：选择目标摸至 min(5, 体力上限) 后为其添加技能（`beizhan` 备战等） | C (omp) | 2026-08-02 ✓ | 复用 `ChooseTargetDraw` 目标选择与 `AddSkill` 管线；沿用 `ContinuationKind::Custom` 载荷魔法标记（B 段先例）；闭环 sp2 审配。落地见 ADR 0323 |
| 456 | `choose_discard_heart_redirect_current_damage(player, draw_skill_def)`：弃一张红桃手牌、选择其他角色并将当前伤害重定向（`tianxiang` 天香等） | C (omp) | 2026-08-02 ✓ | 弃红桃+选目标三段选择，取消+同源/同量/同属性/同牌重放（沿用 ABI 374 共患）；新目标不再重评估护甲（文档化近似）；闭环 old 小乔。落地见 ADR 0324 |
| 457 | `choose_discard_heart_cancel_damage_branch(player)`：弃红桃、选目标、取消当前伤害并二选一（1 点来源伤害+摸至多 5 张已损体力 / 失去 1 点体力+获得弃牌）（`retianxiang` 界天香） | C (omp) | 2026-08-02 ✓ | 复用天香红桃选牌/选目标骨架与 `prevent_current_damage`；分支内伤害/失体/摸牌/获牌管线；闭环 shenhua 小乔。落地见 ADR 0325 |
| 458 | `choose_link_other_same_hp(player)`：准备阶段选择任意名体力值与拥有者相同且未横置的其他角色并横置之（`lianzhou` 链舟） | C (omp) | 2026-08-02 ✓ | 复用 `MARK_CHAIN` 横置管线与 `ChooseLinkOther` 目标过滤骨架；新增同体力+多目标选择；闭环 huicui 蔡瑁张允。落地见 ADR 0327 |
| 459 | `choose_give_hand_equip_to_event_player(player)`：将任意张可弃置手牌/装备交给触发事件角色（`stddechong` 得宠等） | C (omp) | 2026-08-02 ✓ | 复用可弃置牌选择与赠牌/移牌管线；配合 bound 技能表达后续阶段结算；闭环 sixiang 标毛皇后。落地见 ADR 0328 |
| 460 | `discard_hand_equip_count(player, count)`：选择恰好 count 张可弃置手牌/装备并弃置（通用弃牌代价，`yangzhong` 殃众等） | C (omp) | 2026-08-02 ✓ | 复用可弃置牌选择与 discard_cards 管线；配合 `optional by event_source` 表达伤害来源弃牌代价；闭环 sp2 赵忠。落地见 ADR 0330 |
| 461 | `resolve_jieliang_gain(player)`：当前回合角色弃牌阶段弃牌后，选择一张仍处弃牌堆的弃牌获得（`jieliang` 劫粮） | C (omp) | 2026-08-02 ✓ | 复用 `cards_moved` 事件卡与弃牌堆取牌管线；配合 `draw_count`/`hand_limit` 修饰符表达摸牌与手牌上限削减；闭环 sp2 韩猛。落地见 ADR 0332 |
| 462 | 过滤器 `owner_source_wuxie_used` + `choose_other_gain_event_cards(player)`：使用【无懈可击】后选其他角色获得该牌（`stdjujian` 举荐） | C (omp) | 2026-08-02 ✓ | 复用 `CardKind::TrickWuxie` 判定与 `gain_event_cards` 管线；闭环 sixiang 标徐庶。落地见 ADR 0333 |
| 463 | `resolve_lulve(player)`：选择手牌数更少的其他角色，由其二选一（交全部手牌并令拥有者翻面 / 自己翻面并视为对拥有者使用【杀】）（`lulve` 掠卤） | C (omp) | 2026-08-02 ✓ | 复用整手移牌、`TurnOver` 与 `UseVirtualCardOn` 管线；分支选择观察者为目标；闭环 sp2 梁兴。落地见 ADR 0334 |
| 464 | `resolve_dctingxian(player)`：使用【杀】时摸装备数+1 张，并令此【杀】对至多等量目标无效（`dctingxian` 铤险） | C (omp) | 2026-08-02 ✓ | 复用 `exclude_saved_sha_target` 当前【杀】目标排除管线；闭环 sp2 孙狼。落地见 ADR 0335 |
| 465 | `choose_other_add_skill(player, skill_def)`：选择一名其他角色并授予指定技能（`hfjieying` 界营等） | C (omp) | 2026-08-02 ✓ | 复用目标选择与 `AddSkill` 管线；配合 `choose_add_current_sha_target` 表达获授者的【杀】加目标；闭环 sp2 韩馥。落地见 ADR 0336 |

纯 DSL 批次（无需新 ABI，2026-08-01 A 段）：shenzhu（神助，choose+临时技能）、yingba（英霸，
change_max_hp+mark）、gebo（割卜，仁库→skill_pile）、spyilie（毅烈，三选一）、spxiangzhen
（向镇，card_target_cancel(nanman)）、spfangzong（放纵，摸至+target_cancel_from_current 近似）、
spsanchen（三陈，觉醒，spmiewu 空定义占位）。杜预（sp_duyu）闭环入选将池。

纯 DSL 批次（无需新 ABI，2026-08-01 D 段 opencode，xianding 限定包）：dcshenzhi（神智，
discard_hand+recover_hp）、dcshushen（淑慎，复用 ABI 101 choose_other_draw_or_recover）、
dcjidi（诋敌，damage_before 体力/手牌比较）、dczhijue（制略，distance_from-2+adjust_hand_to+
ban_active_until_turn_end）、dcguixiu（闺秀，phase_begin 标记+摸牌）、dczhaohan（昭汉，
increase_current_draw_count）。DC 甘夫人（dc_ganfuren）闭环入选将池。

纯 DSL 批次（无需新 ABI，2026-08-01 D 段 opencode，xianding 限定包批次二）：dcjinggong（精弓，
view_as 装备当杀）、dczhongjie（忠节，濒死回复+摸牌，复用 owner_alive_other_dying_once_per_round）、
dchuahuo（花火，view_as 红牌当火杀）、dcjiexing（节行，伤害/回复/失体力后摸一，近似不计手牌上限）、
dcjingyin（静隐，杀结算后获得弃牌堆实体牌，复用 ABI 151 sha_resolved 过滤器）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次三）：dccunsi（存嗣，
限定技，add_skill 授予勇决 + 目标非自己摸二）、dcyongjue（勇决，出牌阶段首次杀后
choose：refund_current_sha_use 不计次数 / gain_event_cards 获牌）。DC 糜夫人（dc_mifuren）
由既有闺秀 + 存嗣闭包方向推进，勇决由存嗣授予。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次四）：dcanjing（安境，
damage 来源后 choose_optional_targets_draw(wounded) 选受伤角色摸牌 + choose_other_recover
最低体力回复）、dcweiti（伪涕，主动 choose 二选一：choose_other_damage 或 choose_other_recover）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次五）：dcyuzhi（逾制，
成为杀目标时 increase_current_card_base_damage 伤害+1，二选一弃装分支未建模）、dczhangji
（长姬，成为多目标牌目标时按 event_target_count-1 摸牌，排序调整未建模）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次六）：dcchenyong（沉勇，
出牌阶段结束时按本阶段逻辑牌类别数摸牌，draw_by_play_phase_card_category_count 精确表达）、
dclingfang（绞，准备阶段或成为非红牌目标时加标记；旧 JS 的“自己用黑牌且非目标”分支未建模）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次七）：dcyuxi（虞袭，
造成或受到伤害时摸一张，owner_is_source_or_target 精确表达）、dcsbfuzhan（复盏，角色脱离濒死
时回复至体力上限；限定技整局一次未建模，以每轮一次近似）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次八）：dcxinyou（心忧，
出牌阶段限一次回复至上限并补牌至上限；结束阶段副作用未建模）、dcniaoxiang（鸟翔，距离 ≤ 1
的杀目标需额外闪，sha_response_count_if_source_owner_target_distance_le_one 近似）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次九）：dcsbfangong（返攻，
限定技回复至上限并摸 min(5, 手牌数) 张，复用 ABI 451 min 表达式）、xinlvli（新膂力，造成伤害
后按体力与手牌差摸牌或回复，owner_source_once_per_turn；已受伤条件未建模）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次十）：dcminshan（敏善，
受伤后选目标摸两张，owner_is_target_per_damage_point + choose_target_draw；育贤花色记录联动
未建模）、dcshouze（受责，结束阶段有绞标记时移除并获弃牌堆【杀】后失体；旧 JS 黑牌约束以
【杀】近似，标记复用 dclingfang）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次十一）：dcduzhang（独仗，
成为非红牌目标时加凛标记并摸一，hand_limit 按凛标记数加成；旧 JS 的“使用黑牌指定目标”场景
未建模，凛标记复用 dclinjie 定义）、dcfengshi（风势，杀指定目标且拥有者在目标攻击范围外时弃
其一张装备，owner_source_sha_target_alive_owner_outside_target_attack_range 精确表达；弃防具/
防御马以弃装备近似）。

## ADR 领取表（Next ADR）

当前最大 ADR 编号：0481（预留给 XIANDING 傅干 ABI 586）。

| 编号 | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0292 | extra-shen-simayi | 既有 | — |
| 0293 | shiji-spwuku | A (Claude) | 2026-08-01 |
| 0294 | shiji-spjungong-gaolan | A (Claude) | 2026-08-01 |
| 0295 | shiji-sp-bianfuren | A (Claude) | 2026-08-01 |
| 0296 | shiji-dsl-batch | A (Claude) | 2026-08-01 |
| 0297 | shiji-spfenming-chendong | A (Claude) | 2026-08-01 |
| 0298 | shiji-spxizhan-huaman | A (Claude) | 2026-08-01 |
| 0299 | shiji-spsongshu-zhangwen | A (Claude) | 2026-08-01 |

| 0300 | shenhua-batch1（陆绩怀橘/遗礼/争论） | C (omp) | 2026-08-01 ✓ |
| 0301 | shenhua-batch2（陆抗谦节/破势） | C (omp) | 2026-08-01 ✓ |
| 0302 | yingbian-batch1（追击） | C (omp) | 2026-08-01 ✓ |
| 0303 | ABI 450 济溯跳过阶段原语设计 | C (omp) | 2026-08-01 ✓ |
| 0304 | collab 叶诗文（济溯/水泳，ABI 450 落地） | C (omp) | 2026-08-01 ✓ |
| 0305 | shenhua 神速系列（界夏侯渊/旧夏侯渊） | C (omp) | 2026-08-01 ✓ |
| 0306 | ABI 451 min 表达式原语 | C (omp) | 2026-08-01 ✓ |
| 0307 | ABI 452 目标过滤器 + SP2 威迫 | C (omp) | 2026-08-01 ✓ |
| 0308 | shiji-spdiancai-diancai | A (Claude) | 2026-08-01 |
| 0309 | xianding-batch1-dcganfuren | D (opencode) | 2026-08-01 |
| 0310 | shiji-spcunsi-cunsi | A (Claude) | 2026-08-01 |
| 0311 | diy 孙翊（国裂） | C (omp) | 2026-08-01 ✓ |
| 0312 | shenhua 郝昭（镇骨） | C (omp) | 2026-08-01 ✓ |
| 0313 | shiji-dangmo-dangmo | A (Claude) | 2026-08-01 |
| 0314 | ABI 453 current_phase_player + SIXIANG 陆通 | C (omp) | 2026-08-01 ✓ |
| 0315 | SB 界华雄（扬威） | C (omp) | 2026-08-01 ✓ |
| 0316 | COLLAB 桃神（怒涛） | C (omp) | 2026-08-01 ✓ |
| 0317 | SXRM 界刘备（逞变） | C (omp) | 2026-08-01 ✓ |
| 0318 | shiji-scfuhai-fuhai | A (Claude) | 2026-08-02 |
| 0319 | SIXIANG 关兴（无有） | C (omp) | 2026-08-01 ✓ |
| 0320 | ABI 454 目标过滤器 + HUICUI 界沮授（崔进） | C (omp) | 2026-08-01 ✓ |
| 0321 | xianding-batch2（精弓/忠节/花火/节行/静隐） | D (opencode) | 2026-08-01 |
| 0322 | TW 张飞（虚吓，twxuhe + twxuhe_damage） | C (omp) | 2026-08-02 ✓ |
| 0323 | ABI 455 备战原语 + SP2 审配（beizhan） | C (omp) | 2026-08-02 ✓ |
| 0324 | ABI 456 天香伤害重定向原语 + old 小乔 | C (omp) | 2026-08-02 ✓ |
| 0325 | ABI 457 界天香取消+分支原语 + shenhua 小乔 | C (omp) | 2026-08-02 ✓ |
| 0326 | COLLAB TW魔周瑜 选将池闭包（跨包技能解析） | C (omp) | 2026-08-02 ✓ |
| 0327 | ABI 458 链舟同体力横置原语 + huicui 蔡瑁张允 | C (omp) | 2026-08-02 ✓ |
| 0328 | ABI 459 赠牌给触发角色原语 + sixiang 标毛皇后 | C (omp) | 2026-08-02 ✓ |
| 0329 | shiji-pinghe-fenghe | A (Claude) | 2026-08-02 |
| 0330 | ABI 460 通用弃牌数量原语 + sp2 赵忠（殃众） | C (omp) | 2026-08-02 ✓ |
| 0331 | DI/YIJIANG RE步练师 选将池闭包（anxu 复用 resolve_reanxu） | C (omp) | 2026-08-02 ✓ |
| 0332 | ABI 461 劫粮获牌原语 + sp2 韩猛 | C (omp) | 2026-08-02 ✓ |
| 0333 | ABI 462 无懈过滤器 + 举荐获牌原语 + sixiang 标徐庶 | C (omp) | 2026-08-02 ✓ |
| 0334 | ABI 463 掠卤目标二选一原语 + sp2 梁兴 | C (omp) | 2026-08-02 ✓ |
| 0335 | ABI 464 铤险摸牌+杀目标排除原语 + sp2 孙狼 | C (omp) | 2026-08-02 ✓ |
| 0336 | ABI 465 选人授技原语 + sp2 韩馥（界营） | C (omp) | 2026-08-02 ✓ |

| 466 | `resolve_dcnuchen(player)` 复合（揭示目标手牌花色 → 弃同花色弃牌等量伤害或获得其同花色手牌）+ 过滤器 `owner_is_target_hej_above_hp`（手牌+装备+判定牌数 > 体力值，SP关羽【单骑】觉醒） | C (omp) | 2026-08-02 ✓ | 闭环 sp2 新杀SP关羽（dcdanji 觉醒 → mashu + dcnuchen；refresh:new_rewusheng 已移植）。落地见 ADR 0337 |
| 0337 | ABI 466 怒嗔揭示-弃置-伤害复合 + 单骑觉醒过滤器 + sp2 新杀SP关羽 | C (omp) | 2026-08-02 ✓ |

| 467 | `resolve_dczhiheng(player)` 复合（弃置任意张手牌/装备牌，摸等量牌；若弃置了所有手牌额外摸一张）+ 经典孙权【制衡】 | C (omp) | 2026-08-02 ✓ | 闭环 collab 经典孙权（dc_sunquan）；伤害增加额外发动次数的 collab 特化机制文档化延后。落地见 ADR 0338 |
| 0338 | ABI 467 制衡弃牌摸牌复合 + collab 经典孙权 | C (omp) | 2026-08-02 ✓ |

| 468 | `resolve_stdgaobian(player)` 复合（其他角色结束阶段：若本回合恰好一名角色受到伤害，其选择视为使用一张【杀】或重置标王沈【安忍】计数）+ 标王沈【高变】 | C (omp) | 2026-08-02 ✓ | 闭环 sixiang 标王沈（stdgaobian；stdanran 已移植）；弃牌堆中具体【杀】实体近似为虚拟【杀】。落地见 ADR 0339 |
| 0339 | ABI 468 高变受损者二选复合 + sixiang 标王沈 | C (omp) | 2026-08-02 ✓ |

| 469 | `resolve_spjiedao(player)` 复合（伤害结算前：受损的伤害来源可弃置 X 张手牌/装备牌并使本次伤害 +X，X 为 1..受伤体力数，可取消）+ 忙牙长【劫刀】 | C (omp) | 2026-08-02 ✓ | 闭环 sp2 忙牙长（mangyachang）；旧 JS 在伤害结算后弃牌，DSL 前置文档化近似。落地见 ADR 0340 |
| 0340 | ABI 469 劫刀伤害增幅复合 + sp2 忙牙长 | C (omp) | 2026-08-02 ✓ |

| 470 | `resolve_sibian(player)` 复合（摸牌阶段：改为展示牌堆顶 4 张，获得最大/最小点数的牌，其余牌交给一名手牌数最少的角色或置入弃牌堆；以 draw_phase_fixed_count(0) 跳过摸牌）+ 张温【思变】 | C (omp) | 2026-08-02 ✓ | 闭环 sp2 张温（zhangwen）；牌堆不足 4 张时取现有张数（文档化近似）。落地见 ADR 0341 |
| 490 | `owner_source_sha_or_juedou_single_target_hand_nonempty` + `hand_suit_count/hand_color_count`：精确统计目标手牌并支持每次单目标【杀】/【决斗】触发（TW 马云騄「凤魄」） | B (Codex overflow) | 2026-08-02 ✓ | B 段耗尽后使用溢出段首号；已落地手牌花色/颜色查询、技能实例持久位与移除重置语义，闭环 `tw_mayunlu`，黄金/闭包/一致性回归通过 |
| 491 | `reveal_choose_category_distribute_self_threshold_hp_loss(player, reveal_count, recipient_cap)`：展示牌堆顶牌、选择一种类别并分配，所得牌数不少于技能拥有者者失去体力（TW 樊稠「兴乱」） | B (Codex overflow) | 2026-08-02 | ✓；闭环 `tw_fanchou`，严格保留展示六张、每名角色至多三张与按拥有者所得数比较的旧 JS 语义 |
| 492 | `choose_other_random_gain_region_then_draw_compare_color_lose_hp(player)`：选择一名区域内有牌的其他角色，权威随机获得其区域牌并摸一张，异色则失去体力（DIY 蔡瑁「定州」） | B (Codex overflow) | 2026-08-02 | ✓；闭环 `ns_caimao`，随机数只使用可回放 RuleRng，旧 JS 的客户端 LuckyStar 偏置不进入规则层；黄金/闭包/一致性回归通过 |
| 493 | `resolve_refuhai(player)`：其他角色依座次私密选择潮起/潮落，统一公开后按从下家起连续相同选择数摸牌（手杀 魏文诸葛直「浮海」） | B (Codex overflow) | 2026-08-02 ✓ | 复用 `luanqun` 的逐座次私密选择/统一公开状态机模式，闭环 `re_weiwenzhugezhi`；黄金/闭包/一致性回归通过，落地见 ADR 0355 |
| 494 | `card_heart` 主动选牌约束 + `choose_gain_target_hand_ceil_half_difference(player, target)`（OFFLINE 樊稠「扬武」） | B (Codex overflow) | 2026-08-02 ✓ | 目标资格按弃牌前 `hand_gt_owner` 校验，获得数量按旧 JS 在成本结算后的手牌差计算；黄金/闭包/一致性回归通过，落地见 ADR 0357 |
| 495 | `resolve_gzjinfa(player, target)`：弃置所选牌后，目标交装备或由拥有者获得其一张牌；黑桃装备触发目标无距离虚拟【杀】（OFFLINE 文钦「矜伐」） | B (Codex overflow) | 2026-08-02 ✓ | 复用实体移牌、目标牌选择与回合外强制虚拟【杀】管线；Golden、闭包、全量 compiler 与 conformance 通过，落地见 ADR 0358 |
| 496 | `resolve_wn_qifeng(player, event_player)`：回合结束时按本回合失去牌数对回合角色使用等额伤害虚拟【杀】（OFFLINE 杨秋「齐锋」） | B (Codex overflow) | 2026-08-02 ✓ | 复用可重复计数的回合失牌历史与带伤害加成的虚拟【杀】管线；Golden、闭包、全量 compiler 与 conformance 通过，落地见 ADR 0360 |
| 497 | `give_selected_then_target_may_use_draw_owner(player, target)` + `card_non_equip` 主动选牌约束（SP2 张郃「远略」） | B (Codex overflow) | 2026-08-02 ✓ | 复用实体赠牌、目标合法用牌与摸牌管线；仅新增串联状态机和非装备选牌过滤，黄金/闭包/一致性回归通过，落地见 ADR 0363 |
| 498 | `choose_category_then_target_give_matching_or_damage_plus_one(player, target)` + `owner_source_target_not_wei`（OFFLINE 荆扬曹操「统北」） | B (Codex overflow) | 2026-08-02 ✓ | 复用征服的类别声明与匹配交牌选择链；无牌或拒绝时增加当前待结算伤害，黄金/闭包/conformance 通过，落地见 ADR 0365 |
| 499 | `choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(player, source)`（手杀 杨阜「借兵」） | B (Codex overflow) | 2026-08-02 ✓ | 复用权威 RuleRng、可获得牌、实体移牌与普通装备使用管线；闭环 `mobile:yangfu` |
| 500 | `resolve_mobiledanshou(player, target)`（手杀界朱然「胆守」） | B (Codex overflow) | 2026-08-02 ✓ | 复用本回合用牌目标历史、精确弃牌选择与伤害管线；闭环 `mobile:xin_zhuran` |
| 501 | `resolve_jsrgchiying(player, target)`（江山如故高翔「驰应」） | B (Codex overflow) | 2026-08-02 ✓ | 复用攻击范围查询、强制弃牌选择与弃牌堆获牌管线；闭环 `jsrg:jsrg_gaoxiang` |
| 502 | source-free 实体手牌/装备材料 + 标记消耗 + 结算后摸牌（史纪杜预「灭吴」） | B (Codex overflow) | 2026-08-02 ✓ | 扩展既有通用 view-as 声明与实体牌使用/响应管线；闭环 `shiji:spmiewu` |
| 503 | 通用 `card_resolved` 结算边界 + `resolve_mbqingdao` / `resolve_mbxiugeng` / `resolve_mbchenshe`（兵势国渊） | B (Codex overflow) | 2026-08-02 ✓ | 复用实体牌、目标选择、技能状态、弃牌/回复管线；Golden、闭包、全量 compiler 与 conformance 通过，落地见 ADR 0374 |
| 504 | `resolve_mbquanchong(player, helper_def)` + `resolve_mbrenxing(player)`（兵势陈祗） | B (Codex overflow) | 2026-08-02 ✓ | 复用全弃、额外回合、轮次技能状态、摸牌与目标区域弃牌管线；Golden、闭包、全量 compiler 与 conformance 通过，落地见 ADR 0375 |
| 505 | 伤害牌距离选择器 + `resolve_mbchizhang` / `resolve_mbduanyang`（兵势黄祖） | B (Codex overflow) | 2026-08-02 ✓ | 复用实体牌响应、技能牌堆、阶段结束用牌、重铸与摸牌管线；闭环 `bingshi:mb_huangzu`，落地见 ADR 0376 |
| 506 | `resolve_mbrunwei` / `resolve_mbshuanghuai`（兵势陆郁生） | B (Codex overflow) | 2026-08-02 ✓ | 复用展示、按颜色获牌、阶段技能次数、伤害防止、弃牌堆取桃与距离查询；闭环 `bingshi:mb_luyusheng`，落地见 ADR 0377 |
| 507 | 首个手牌花色用牌不限次数修正器 + `resolve_mbxiongtu`（兵势孙峻） | B (Codex overflow) | 2026-08-02 ✓ | 复用手牌/弃牌堆花色历史、目标手牌展示/弃置、伤害与阶段技能次数；Golden、闭包、全量 compiler 与 conformance 通过，落地见 ADR 0378 |
| 508 | `resolve_mbganggeng` / `resolve_mbsijian`（兵势田丰） | B (Codex overflow) | 2026-08-02 ✓ | 复用多牌交给、手牌极值、失牌/濒死、摸牌与弃牌管线；Golden、闭包、内容锁、compiler 459/459 与 conformance 8/8 通过，落地见 ADR 0379 |
| 509 | `resolve_mbjianji` / `resolve_mbyuanmo`（兵势杨弘） | B (Codex overflow) | 2026-08-02 ✓ | 复用拼点事件、虚拟杀、场上牌移动、攻击范围与摸牌管线；Golden、闭包、内容锁、compiler 460/460 与 conformance 8/8 通过，落地见 ADR 0380 |
| 510 | `resolve_mbfeijing` / `resolve_mbxiaoge`（兵势张燕） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：伤害锦囊转杀、座次路径弃牌/同色追加目标、绑定防伤回血取牌与单目标杀后决斗；闭环 `bingshi:mb_zhangyan`，落地见 ADR 0382 |
| 511 | `resolve_shuangxiong(player)` 复合（摸牌阶段可选改为判定并获判定牌，本回合记录判定颜色并开启“非该色手牌当【决斗】”视为用牌）+ 颜良文丑【双雄】 | C (omp) | 2026-08-02 ✓ | 复用 draw_phase_fixed_count(0) 改判、判定+获牌管线与 skill 实例颜色记录；view_as 以新增 filter 读取记录色 |
| 512 | `resolve_zhijian(player)` 复合（出牌阶段限一次：选一张手牌区装备牌交给一名可装备该牌的其他角色，然后摸一张）+ 张昭张纮【直谏】 | C (omp) | 2026-08-02 ✓ | 复用实体赠牌/装备使用合法性与摸牌管线；目标可装备性在选牌屏障重校验 |
| 513 | `resolve_guzheng(player, event_player)` 复合（其他角色弃牌阶段结束后：选一张其本阶段弃置仍处弃牌堆的牌归还，并可选获得其余弃牌）+ 过滤器 `owner_alive_other_discarded_in_discard_phase` + 张昭张纮【固政】 | C (omp) | 2026-08-02 ✓ | 复用弃牌阶段事件牌追踪与弃牌堆取牌管线 |
| 523 | `resolve_zhiba(player, event_player)` 复合（主公技：其他吴势力角色出牌阶段限一次可与拥有者拼点；拥有者未赢时可选获得两张拼点牌，已觉醒时可先拒绝）+ 过滤器 `owner_alive_other_wu_play_phase_can_compare_owner` + 孙策【制霸】 | C (omp) | 2026-08-02 ✓ | 复用既有拼点选择/展示/移牌管线与技能实例状态；落地见 ADR 0395 |
| 524 | `resolve_drlt_congjian(player, targets)` 复合（成为多目标锦囊目标后：交给其中一名其他目标一张手牌/装备牌，然后摸一张；所给为装备牌则摸两张）+ 张绣【从俭】 | C (omp) | 2026-08-02 ✓ | 复用实体赠牌/移牌管线与摸牌效果；目标集合来自触发事件并在屏障重校验；落地见 ADR 0395 |
| 526 | `resolve_nzry_feijun(player)` 复合（出牌阶段限一次弃一张牌：按可用性选“手牌数大于你者交给你一张牌”或“装备数大于你者弃置一张装备牌”分支；目标首次被选时摸两张【兵略】）+ 过滤器 `owner_is_target_feijun_new_target`（兵略摸牌触发）+ 王平【飞军/兵略】 | C (omp) | 2026-08-02 ✓ | 复用通用选牌/赠予/弃牌管线与稳定标记；兵略摸牌以回合旗标 + phase_play_end 触发表达（旧 JS 紧随飞军结算，DSL 阶段末文档化近似）；落地见 ADR 0398 |
| 527 | `resolve_drlt_yongsi(player, stage)` 复合（摸牌阶段摸牌数改为势力数；出牌阶段结束时按本回合是否造成伤害：未造成伤害摸至体力值，否则授予既有庸肆弃牌阶段状态）+ 袁术【庸肆】 | C (omp) | 2026-08-02 ✓ | 复用 draw_phase_fixed_count 与既有 drlt_yongsi1 定义；回合内伤害以 turn 计数近似（出牌阶段前无法造成伤害，文档化近似）；临时技能以 add_skill_until_turn_end 表达（旧 JS 弃牌阶段后移除，文档化近似）；落地见 ADR 0400 |
| 528 | `resolve_drlt_weidi(player)` 复合（主公技，弃牌阶段开始：把至多 min(需弃置数, 其他群势力角色数) 张手牌分别交给等量其他群势力角色，弃牌需求随手牌减少自然下降）+ 过滤器 `owner_is_target_discard_phase_owe_limit_other_qun` + 袁术【伪帝】 | C (omp) | 2026-08-02 ✓ | 复用弃牌阶段事件、可弃置手牌选择与赠予管线；目标与牌按序配对；落地见 ADR 0400 |
| 530 | 过滤器三件（祝融【巨象/烈刃】，纯 DSL 批次）：`owner_source_sha_damage_target_alive_both_hand`（杀伤害且双方有手牌）、`owner_alive_other_nanman_cards_discard`（他人使用南蛮且事件牌在弃牌堆）、`owner_alive_skill_bound_target_is_event_source`（绑定目标为拼点第二参与者）；配 `gain_event_cards`/`choose_compare_selected_hand_card`/`take_one_card` | C (omp) | 2026-08-02 | 复用拼点胜负绑定辅助技能、通用获牌与南蛮取消修正器；拼点胜者获牌以 bound 技能精确限定目标 |
| 531 | `resolve_lieren(player, target, win_skill_def, lose_skill_def)`（烈刃拼点启动：以固定受伤害角色为目标打开通用拼点选牌续体，胜者获牌以绑定辅助技能表达）+ 祝融【烈刃】 | C (omp) | 2026-08-02 | 复用 ChooseHandComparePlayerCard 续体与胜负绑定辅助管线；触发内无法使用 active 专属 choose_compare_selected_hand_card（需要选中牌），故以专用启动原语表达 |
| 532 | `distinct_suits` 主动选牌约束 + `resolve_potnigu` / `resolve_potlulian`（兵势孙綝） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：复用主动选牌、座次赠牌、伤害加值、横置与摸牌管线；Golden、兵势闭包、26 包扩展导入与 conformance 通过，落地见 ADR 0407 |
| 533 | shenhua 邓艾【屯田】：过滤器 `owner_lost_cards_outside_turn`（回合外本角色手/装/判区牌移出己方控制区）；`resolve_tuntian(player)`（判定后非红桃判定牌移入本技能牌堆）；修正器 `global_distance_minus_skill_pile_count`（与其他角色距离减本技能牌堆牌数） | C (omp) | 2026-08-02 | 复用技能牌堆/判定/距离修正机制；田映射为技能牌堆，急袭以 view_as hand_expansion 表达 |
| 534 | shenhua 邓艾【凿险】：过滤器 `owner_phase_begin_originating_skill_pile_gte_three`（准备阶段本技能牌堆≥3）；`resolve_zaoxian(player, jixi_skill_def)`（减1点体力上限并授予急袭） | C (omp) | 2026-08-02 | 觉醒复用 ChangeMaxHp+AddSkill 通用效果 |
| 535 | `pair_hand_difference_lte_three` 双目标约束 + `resolve_potdimeng(player, targets)`（兵势鲁肃【缔盟】） | B (Codex overflow) | 2026-08-02 ✓ | 复用整手原子交换、手牌/装备弃置选择与摸牌管线；Golden、兵势闭包、26 包扩展导入与 conformance 通过，落地见 ADR 0413 |
| 536 | `resolve_pothaoshi(player, action)` + 跨角色手牌授权（兵势鲁肃【好施】） | B (Codex overflow) | 2026-08-02 ✓ | 授权统一进入可用手牌、响应来源与观察者投影；两次空手补至三张；下个回合开始清理 |
| 537 | shenhua 姜维【挑衅】：`resolve_tiaoxin(player, skill)` 复合原语（出牌阶段限一次：选择其他角色——其攻击范围含持有者且有手牌/装备；该角色二选一：对持有者使用一张杀，或令持有者弃置其一张牌） | C (omp) | 2026-08-02 | 复用乱武 prompt_sha 受迫杀源与 PickTargetCard 弃牌续体；目标合法性在 Choice 恢复时重校验 |
| 538 | shenhua 姜维【志继】：过滤器 `owner_phase_begin_hand_empty`（准备阶段持有者无手牌）+ `resolve_zhiji(player, guanxing_def)`（摸2张、回复1点体力、减1点体力上限并授予观星） | C (omp) | 2026-08-02 | 觉醒一次性以持有观星守卫；观星复用标准包既有定义 |
| 539 | `resolve_spmiaolve(player, action)` / `resolve_spyingjia(player, action, helper)` + 锦囊【瞒天过海】（势董昭） | B (Codex overflow) | 2026-08-02 ✓ | 补齐衍生牌实体创建、多目标逐个无懈/获牌后依次回赠，妙略受伤分支与迎驾同名锦囊历史/额外回合 |
| 540 | shenhua 孟获【再起】：`resolve_zaiqi(player, extra)` 复合原语（摸牌阶段替换：已受伤时弃摸牌并翻亮牌堆顶 X 张（X=已损失体力+extra），红桃每张回复 1 点体力，其余获得） | C (omp) | 2026-08-02 | 复用摸牌替换 set_draw_phase_fixed_count 与翻亮/获得管线；红桃按张计数回复 |
| 541 | shenhua 孟获【祸首】：修正器 `nanman_damage_source_as_owner` + prepare_damage 来源重定向钩子（南蛮牌伤害的来源改为持有者；南蛮对持有者无效复用通用 card_target_cancel） | C (omp) | 2026-08-02 | 钩子仅在南蛮实体牌伤害路径生效，其他伤害路径零影响 |
| 542 | shenhua 孟获【求首】（主公技）：过滤器 `owner_alive_nanman_resolved_target_dead_or_damaged_gt_three`（南蛮结算后目标死亡或受创目标数>3）+ `resolve_twqiushou(player)`（所有存活蜀/群角色各摸一张） | C (omp) | 2026-08-02 | 受创目标数以 CardResolved damaged_targets 数量近似伤害总数 |
| 543 | `resolve_pottuntian(player, action, potzaoxian_def)` / `resolve_potjixi(player, target, action)` + `owner_lost_non_damage_card_pottuntian_charge_not_full`（势邓艾） | B (Codex overflow) | 2026-08-02 ✓ | 复用标记、实体移牌、逐目标无懈/顺手牵羊管线，完整表达蓄力上限、红桃分发、凿险阈值与急袭目标历史；Golden、门禁与扩展包已落地 |
| 544 | `potqingyan` 多牌响应代价 + `resolve_potceduan(player,target,action)` + `owner_in_target_attack_range` / `owner_source_sha_resolved_tracked_potceduan_dealt_damage`（势陈矫） | B (Codex overflow) | 2026-08-02 ✓ | 复用多牌响应、展示、实体多牌转【杀】与伤害追踪管线；完整表达逐轮递增展示锁、反向攻击范围和策断并列最多颜色；Golden、门禁与扩展包已落地 |
| 545 | shenhua 毌丘俭【征荣】：过滤器 `owner_source_first_damage_card_target_hand_gte_owner`（首个目标、伤害标签基本/锦囊、目标手牌数不小于持有者且有手/装备）+ `resolve_zhengrong(player, target)`（持有者取目标一张手牌/装备移入本技能牌堆「荣」） | C (omp) | 2026-08-02 | 复用技能牌堆与取牌续体 |
| 546 | shenhua 毌丘俭【鸿举】：`resolve_hongju(player, qingjian_def)`（荣≥3 觉醒：可选与等量手牌交换、减 1 点体力上限、授予清俭） | C (omp) | 2026-08-02 | 觉醒一次性以持有清俭守卫 |
| 547 | shenhua 毌丘俭【清俭】：`resolve_qingjian(player)`（出牌阶段限一次：弃一张手牌获得一张「荣」，弃一名其他角色装备/判定区一张牌） | C (omp) | 2026-08-02 | 弃装备/判定区复用目标卡选择续体 |
| 548 | `resolve_pottaishici(player,target,action,hanzhan_def,zhanlie_def)` + `owner_alive_sha_entered_discard_potzhanlie_available` / `owner_source_sha_resolved_tracked_potzhanlie_draw`（势太史慈【酣战/战烈/振锋】） | B (Codex overflow) | 2026-08-02 ✓ | 复用补牌、虚拟决斗/杀与杀结算续体；完整表达振锋独立改写 X、战烈逐目标响应代价和结算后摸三，门禁与扩展包已落地 |
| 549 | shenhua 张角【雷击】：过滤器 `owner_responded_shan`（持有者使用/打出闪）+ `resolve_releiji(player, target)`（选目标后其判定：黑桃 2 点雷电伤害，梅花持有者回 1 并对其 1 点雷电伤害） | C (omp) | 2026-08-03 | 复用判定与雷电伤害管线 |
| 550 | shenhua 张角【鬼道】：修正器 `judgment_rewrite_black_hand_equip`（黑色手/装备牌可替换判定牌，复用既有判定改写机制与选择续体） | C (omp) | 2026-08-03 | 判定改写统一由 judge 流程征询 |
| 551 | shenhua 张角【黄天】（主公技）：过滤器 `owner_alive_other_qun_play_phase_can_give_owner`（其他群势力角色出牌阶段且有闪/闪电手牌）+ `resolve_huangtian(player, actor)`（该角色可选交一张闪/闪电给持有者，出牌阶段限一次） | C (omp) | 2026-08-03 | 复用制霸 by event_player 主公技模式与赠予管线 || 529 | `resolve_potgongmou(player)` / `resolve_potzhengshuo(player)`（兵势桓阶） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：共谋交换整手牌并临时授予奇策/看破；正朔全场依次弃手牌、洗牌并各摸四张；落地见 ADR 0406 |
| 552 | shenhua 于吉【禅缘】：修正器 `skills_disabled_when_hp_one`（锁定技：持有者体力值为 1 时其非锁定技能失效，skill_effectively_enabled 增加钩子）+ 于吉【新蛊惑】纯 DSL 声明 | C (omp) | 2026-08-03 | 蛊惑复用引擎 cost guhuo 质疑流程；禅缘「蛊惑结算中不能质疑」未建模（文档化近似） || 525 | `resolve_potfuan` / `resolve_potyinxian`（兵势张任） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：基础攻击范围归零并记录花色、他人回合弃牌花色触发无距离杀与追击/失效分支；最远目标杀增伤及回合外用牌累加攻击范围，Golden、兵势闭包、编译器与 conformance 通过，落地见 ADR 0397 |
| 553 | `resolve_mbzhonghui` + `queried_player_qun_owner_lord_alive`（兵势钟会【俟资/挟志/狱难/克昌】） | B (Codex overflow) | 2026-08-02 ✓ | 已落地；复用蓄力点、回合杀历史、伤害意图、势力覆盖与当前杀不可响应管线 |
| 554 | `resolve_potjiejie` / `resolve_potqingshi` + `owner_alive_play_phase_actor_hand_nonempty_potjiejie_unused`（兵势辛宪英【诫节/清识】） | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用私有手牌展示、花色修正器、阵营关系、弃牌/摸牌与选择续体，落地见 ADR 0437 |
| 555 | shenhua 鲁肃【好施】：`resolve_haoshi(player, stage)` 复合原语（摸牌阶段额外摸两张（复用摸牌替换）；摸牌阶段结束时手牌>5 则选手牌数最少角色并给 floor(手牌/2) 张） | C (omp) | 2026-08-03 | 复用摸牌替换与赠予管线 |
| 556 | shenhua 鲁肃【缔盟】：`resolve_dimeng(player, targets, stage)` 复合原语（出牌阶段限一次：选两名其他角色，弃置 X 张牌（X=两者手牌差，可 0），随后交换其手牌） | C (omp) | 2026-08-03 | 复用整手交换与弃牌管线；势鲁肃 potdimeng 的 ≤3 差约束为兵势变体，不复用 || 514 | `resolve_mbxuye` / `resolve_mbkuangxiang`（兵势庞羲）+ `hand_lte_owner` 主动技目标约束 | B (Codex overflow) | 2026-08-02 ✓ | 已完成：摸二/新达手牌上限后的 HEJ 置顶、整手原子交换、交换牌耗尽双角色追踪与下个出牌阶段清理；Golden、闭包、内容锁、compiler 462/462 与 conformance 8/8 通过，落地见 ADR 0383 |
| 558 | `resolve_potweiyan`（势魏延【忠傲/壮誓/饮战/狂骨】）+ 前 N 张牌无距离/不可响应/不计次数助手 | B (Codex overflow) | 2026-08-03 ✓ | 复用使命技技能增删、当前牌响应、弃牌/失去体力、【杀】结算与距离查询管线；Golden、全量 compiler、ABI/VM、conformance、冷导入均通过 |
| 430 | `resolve_xinyangjie(player, stage)`：出牌阶段限一次与一名其他角色拼点（**点数小者胜**，引擎首个小者胜拼点变体）；拼点失败时可选另一名其他角色视为对拼点目标使用火【杀】（`sp_zhujun` 佯解） | A (Claude) | 2026-08-03 ✓ | 复用拼点提交管线（小者胜以胜者反转表达）；目标合法性重校验；Golden 与 shiji 扩展包已落地 |
| 431 | `resolve_qingyu(player, action)`：使命技——受伤前可弃 2 张 he 牌防伤（defend）/ 准备阶段满血且手牌<轮数时使命成功获【选存】（achieve）/ 濒死时使命失败减 1 上限并选一人获【勇决】（fail）（`sp_mifuren` 清雨，xuancun/mbyongjue 为既有纯 DSL 草稿） | A (Claude) | 2026-08-03 ✓ | 复用 potweiyan 使命状态机模式（state_blob status + 唤醒）、强制弃牌防伤与减上限/授技管线；Golden 与 shiji 扩展包已落地 |
| 432 | `resolve_zhengsu(player, mode, stage)`：整肃系统三条件跟踪与判定——垒金（出牌阶段点数严格递增且≥3，card_used 逐张状态跟踪）/ 变阵（出牌阶段同花色且≥2，花色掩码+计数）/ 明志（弃牌阶段花色全异且≥2，discard_phase_discards_for）；成功奖励受益者（授予者记录段 + 自己）各自摸2或回复1，随后移除临时整肃技能（`zhengsu_leijin/bianzhen/mingzhi`，核心库通用系统，朱儁侯封/吕范燕集共用） | A (Claude) | 2026-08-03 ✓ | 复用 ChooseDrawOrRecover 奖励与 state_blob 状态段（ZS0 跟踪段与 GR 授予者段以 0xFE 分隔共存）；新过滤器 owner_alive_other_play_phase_zhengsu_missing_in_range / owner_is_target_missing_zhengsu |
| 433 | `resolve_houfeng(player, target, mode, stage)`：出牌阶段开始选整肃类型授予目标（mode 0 侯封：他人，记录授予者）/（mode 1 燕集：自己），目标弃牌阶段结束后由 ABI 432 判定并奖励 | A (Claude) | 2026-08-03 ✓ | 复用整肃系统与临时技能授予；Golden 与 shiji 扩展包已落地 |
| 434 | `resolve_mbdiancai(player, actor)`：其他角色出牌阶段结束时，若其本回合失去牌数不小于其体力值且手牌数小于体力上限，其摸至上限（`sp_lvfan` 点才变体） | A (Claude) | 2026-08-03 ✓ | 复用 turn.card_lost_by_player 计数与 draw_cards；Golden 与 shiji 扩展包已落地 |
| 435 | `resolve_rechuhai(player, target, card, target_card, stage)`：职责技——出牌阶段限一次摸一张并与一名其他角色拼点（装备数<4 时拼点牌点数 +(4-装备数)，上限 13）；赢则展示目标手牌并按类别从牌堆各取一张获得，输且拼点牌点数<7 使命失败；装备后装备数>2 使命成功：回复至满、以【彰名】替换【乡害】（`zhouchu` 战海；zhangming 梅花牌使用不可响应为纯 DSL + 新过滤器 owner_source_club） | A (Claude) | 2026-08-03 ✓ | 复用拼点提交（点数修正变体）、RevealCards、first_shared_pile_card_by_category 与使命状态机（state_blob）；Golden 与 shiji 扩展包已落地 |
| 436 | `resolve_xiongshu(player, target, card, action, stage)`：其他角色出牌阶段开始时弃置 X 张牌（X=凶竖标记数，每轮递增）并展示其一张手牌，预测其是否会使用该牌；该出牌阶段结束时预测对则其受 1 点伤害，错则获得该牌（目标区域或弃牌堆）（`yingbian` 贾充【凶竖】；新过滤器 owner_alive_other_play_phase_discardable_gte_mark） | A (Claude) | 2026-08-03 ✓ | 复用强制弃牌选择、展示、伤害与获牌管线；预测与结算状态以 state_blob 记录；“是否使用”以牌是否离开目标区域近似 |
| 437 | `resolve_jianhui(player, source, target)`：伤害结算后——作为伤害来源且受伤者与上次相同则摸一张；受伤且来源与上次相同（连续同来源）则来源强制弃一张手牌或装备（`yingbian` 贾充【奸回】；blob "JH1" 记录上次来源/受伤者） | A (Claude) | 2026-08-03 ✓ | 复用 ChooseDiscardOneHandOrEquip 与 draw_cards；连续同来源/同目标以 blob 追踪 |
| 438 | 过滤器 `owner_alive_other_trick_first_target_source_max_hand`（他人单目标锦囊首个目标、使用者手牌全场最多可并列）已落地引擎（`yingbian` 辛敞【参谋】） | A (Claude) | 2026-08-03 | 首个目标判定复用 ctx.targets.first() 模式；**锦囊目标追加机制未落地**（引擎 saved-targets 深埋杀结算 ResumePlayedSha，需平行机制），canmou DSL 暂缓，辛敞待【从俭·应变】一并发布 |
| 439 | `resolve_spqiai(player, target, card)`：出牌阶段限一次将一张非基本牌交给一名其他角色——满血则摸两张，已受伤则目标二选一（令拥有者回复 1 点体力 / 摸两张）（`shiji` 王粲【弃哀】；DSL select_hand card_nonbasic + vm_host selected_cards 填充） | A (Claude) | 2026-08-03 ✓ | 复用赠牌（CardsGiven 事件）与二选一 choice 管线；Golden 与 shiji 扩展包已落地 |
| 440 | `resolve_spshanxi(player, target, stage)`：出牌阶段开始选一名无“檄”标记的角色获得“檄”标记（全局唯一，同时清除他人标记）（`shiji` 王粲【擅袭】） | A (Claude) | 2026-08-03 ✓ | 复用目标选择与标记管线；檄标记键 = 擅袭技能 def id |
| 441 | `resolve_spshanxi_suoming(player, actor, stage)`：其他角色回复体力后（其有檄标记且体力>0）其交给你 2 张牌或失去 1 点体力（可交不足时直接失体）（`shiji` 王粲【索命】；新过滤器 owner_alive_other_marked_xi） | A (Claude) | 2026-08-03 ✓ | 复用交牌选择、赠牌与失体力管线 |
| 442 | `resolve_tongduo(player, target, card, stage)`：成为其他角色单目标牌目标时（每回合一次）选一名有牌角色强制重铸一张牌（`shiji` 刘巴【通夺】；新过滤器 owner_is_target_single_target_other_source_once_per_turn；重铸复用 recast_cards） | A (Claude) | 2026-08-03 ✓ | 复用目标选择、强制选牌与重铸管线 |
| 443 | `resolve_yuanqing(player, stage)`：出牌阶段结束时，本阶段用过的每种类别（基本/锦囊/装备）从弃牌堆权威随机取一张置入仁库（skill pile）（`shiji` 华歆【怨清】） | A (Claude) | 2026-08-03 ✓ | 复用类别掩码、RuleRng 权威随机与 skill pile 管线 |
| 444 | `resolve_shuchen(player, dying, renku_def, stage)`：其他角色濒死时（每轮一次）若仁库（怨清技能牌堆）>3 张则获得全部并令其回复 1（`shiji` 华歆【疏陈】） | A (Claude) | 2026-08-03 ✓ | 复用 skill_pile_cards 读取与回复管线；跨技能共享仁库以 renku_def 参数传递 |
| 445 | `resolve_duanbi(player, targets, stage)`：限定技——全场手牌总数 > 人数×2 时可选任意名角色，逐目标强制弃 min(3, floor(手牌/2)) 张；全部弃完后再选一名角色权威随机获得至多 3 张仍处弃牌堆的被弃牌（`shiji` 刘巴【断笔】；targets 1 255 other 多目标 + blob 记录被弃牌） | A (Claude) | 2026-08-03 ✓ | 复用多目标 active、强制弃牌选择、RuleRng 权威随机与获牌管线 |
| 446 | `resolve_zaoli(player, mark, stage)`：回合开始时有“躁厉”标记（使用/打出牌时 +1 至多 4）→ 弃至少一张牌（可全弃）并摸（标记数+弃牌数）张，总数 > 2 则失去 1 点体力；出牌阶段仅可使用本回合获得的牌（复用 `hand_card_gained_this_turn_only` modifier）（`shiji` 孙翊【躁厉】，与 TW 孙翊标记驱动变体不同） | A (Claude) | 2026-08-03 ✓ | 复用强制弃牌选择、摸牌与失体力管线；标记上限以 DSL if 条件表达 |
| 447 | `resolve_guying(player, actor, card, stage)`：你失去一张手/装/判区牌后（每回合一次，当前回合角色为他人）其须二选一——随机交给你一张其可获得的牌 / 你收回失去的牌（若仍在弃牌堆）（`shiji` 向宠【孤影】；新过滤器 owner_lost_one_hand_equip_judge_other_current_phase_once） | A (Claude) | 2026-08-03 ✓ | 复用 RuleRng 权威随机、赠牌与弃牌堆获牌管线 |
| 448 | `resolve_muzhen(player, branch, stage)`：出牌阶段限一次三选一（每回合各一次，blob 记录已用分支）——①装备换手牌：将一张装备牌置于目标装备区并获得其一张手牌；②给牌换装备：将至多 2 张牌交给目标并获得其装备区一张牌；③睦阵：选任意名其他角色，手牌/装备数与拥有者相同的一项摸 1（两项同摸 2），全部摸 2 则拥有者摸目标数张（`shiji` 向宠【睦阵】） | A (Claude) | 2026-08-03 | 复用装备使用、赠牌、取牌与条件摸牌管线 |
| 559 | shenhua 张郃【巧变】：`resolve_qiaobian(player, phase)` 复合原语（四阶段开始时弃一张手牌并跳过该阶段；摸牌阶段随后可选获得至多两名其他角色各一张手牌，出牌阶段随后可选移动场上的一张牌到另一名角色的同类别区域） | C (omp) | 2026-08-03 ✓ | 复用弃牌选择、`skip_current_phase` 与整牌移动管线；阶段收益在跳过前开选择避免嵌套阶段分发打断；落地见 ADR 0445 |
| 560 | shenhua 刘禅【享乐/放权/若愚】：`exclude_saved_sha_target(card, target)` 效果（card_targeted 时移除当前杀保存目标，享乐拒绝弃基本牌即令杀对持有者无效）+ `resolve_xiangle(player, source, card)`（使用者弃基本牌或杀无效）+ `resolve_fangquan_skip/grant`（跳过出牌阶段、结束阶段弃牌授额外回合）+ `resolve_ruoyu(player, jijiang_def)`（主公最低体力觉醒：加上限、回复、授激将） | C (omp) | 2026-08-03 ✓ | 复用 card_targeted 事件、杀保存目标排除、`skip_current_phase`、extra_turn_queue 与觉醒管线；落地见 ADR 0446 |
| 563 | shenhua 诸葛瞻【罪论/父荫】：`resolve_zuilun(player, stage)` 复合（结束阶段揭示牌堆顶三张：按「本回合失去过牌/手牌非最少/未受伤」计数 0-3；0 全获、N 置顶余获、3 置顶并选择一名其他角色一起失去 1 点体力）+ `resolve_fuyin(player, source, card)`（锁定技：其他角色以不小于你手牌数的【杀】指定你为目标且本回合首次时，移除该杀保存目标） | C (omp) | 2026-08-03 ✓ | 复用揭示/置顶/获得/失体力管线与 ABI 560 保存目标排除；落地见 ADR 0453 |
| 571 | shenhua 严颜【拒战】：转换技双分支复合 `resolve_juzhan(player, source, card)`（阴：你成为其他角色【杀】的目标后与其各摸一张牌并令其本回合不能对你使用牌；阳：你使用【杀】指定目标后获得其一张牌并令你本回合不能对其使用牌）+ `ban_use_against(player, target)` 回合禁用效果 + `card_target_cancel_owner_turn_banned` 修正器 | C (omp) | 2026-08-03 ✓ | 复用摸牌/获得牌管线与保存目标排除；转换技以技能状态翻转位表达；落地见 ADR 0454 |
| 573 | shenhua 王基【奇制/进趋】：`resolve_qizhi(player, targets, stage)` 复合（使用牌指定目标后可选弃置一名非目标角色的一张手牌/装备牌并令其摸一张牌，本回合奇制次数 +1）+ `resolve_jinqu(player)` 复合（出牌阶段结束时摸两张并将手牌弃置至本回合奇制次数张） | C (omp) | 2026-08-03 ✓ | 复用 SelectCards/DiscardCards/DrawCards 与回合技能计数管线；落地见 ADR 0465 |
| 574 | shenhua 蒯良蒯越【荐相/审时】：`resolve_jianxiang(player)` 复合（成为其他角色使用牌的目标后令手牌数最少的一名角色摸一张）+ `resolve_shenshi(player, source, card, stage)` 转换技复合（阳：出牌阶段限一次交一张牌给手牌最多角色并对其造成 1 点伤害，若其因此死亡可令一名角色手牌摸至四张；阴：受伤后观看来源手牌交其一张，其回合结束时该牌仍在其手/装备区则你摸至四张） | C (omp) | 2026-08-03 ✓ | 复用 card_targeted、赠予、伤害、追踪牌与摸至四张管线；落地见 ADR 0466 |
| 575 | `add_first_missing_skill(player, first, second, third)`：按参数顺序永久获得第一个尚未拥有的技能；全部拥有则无事发生（SP【豹变】/ XIANDING 新杀夏侯霸） | B (Codex overflow) | 2026-08-03 ✓ | 复用既有技能持有检查与 `AddSkill` 管线；不打开选择；落地见 ADR 0470 |
| 576 | shenhua 卢植【明任/贞良】：`resolve_mingren(player, stage)` 复合（首轮准备阶段摸两张并将一张手牌置于「任」技能牌堆；结束阶段可选手牌替换「任」）+ `resolve_zhenliang(player, source, stage)` 转换技复合（阴：出牌阶段限一次弃一张与「任」同色的牌并对攻击范围内角色造成 1 点伤害；阳：回合外使用/打出同色牌后令一名角色摸一张牌） | C (omp) | 2026-08-03 ✓ | 复用技能牌堆、花色、攻击范围与转换技管线；落地见 ADR 0471 |
| 577 | shenhua 孙亮【溃诛/掣政/立军】：`resolve_kuizhu(player, stage)` 复合（弃牌阶段结束后按本回合弃牌数 X 选择：令至多 X 名角色摸一张，或对任意名体力值之和为 X 的角色各造成 1 点伤害）+ `resolve_zhizheng(player)` 复合（出牌阶段结束若本回合用牌数少于攻击范围不包含你的角色数，弃置其中一名角色的一张牌）+ `resolve_lijun(player, source)` 复合（主公技：吴势力角色出牌阶段使用实体杀后你获得该牌并令其摸一张） | C (omp) | 2026-08-03 ✓ | 复用弃牌计数、目标选择、伤害、弃牌与主公势力管线；落地见 ADR 0472 |
| 578 | `resolve_dccunsi(player, target, guixiu_def, yongjue_def, stage)`：XIANDING 糜夫人【存嗣】授【勇决】，目标非自己则摸二；授技/摸牌及嵌套触发完成后，仍拥有有效【闺秀】且受伤时回复 1 | B (Codex overflow) | 2026-08-03 | ✓ 已落地；补齐旧 `useSkillAfter` 时序，复用 AddSkill、DrawCards、deferred tail 与 Recover 管线 |
| 579 | shenhua 周妃【良姻/箜声】：`resolve_liangyin(player, stage)` 复合（其他角色失去牌后每回合一次选一名其他角色双方各摸一张，任一方手牌等于箜数且受伤时可回复 1；获得牌后每回合一次弃一张牌并令一名其他角色弃一张牌）+ `resolve_kongsheng(player, stage)` 复合（准备阶段将任意张牌置为「箜」；结束阶段非装备箜回手、剩余装备箜令一名角色依次使用） | C (omp) | 2026-08-03 | 复用技能牌堆、摸牌/弃牌/回复与装备使用管线 |
| 580 | `owner_source_heart_sha` + `record_current_sha_all_targets(player, card)`：XIANDING 新杀神关羽【武神】精确标记红桃杀的全部目标，后续仅该实体杀造成伤害时摸牌 | B (Codex overflow) | 2026-08-03 | 复用现有 Sha resolution `skill_targets` 与 `owner_source_sha_damage_recorded_by_skill`，不新增专属状态 | ✓
| 581 | `choose_give_or_discard_hand(player, payment_count, hand_empty)`：XIANDING 杨彪【昭汉】摸牌后强制交两张手牌给无手牌角色或弃两张 | B (Codex overflow) | 2026-08-03 | 复用 ABI116 的交牌/弃牌选择续体与既有 `hand_empty` 目标范围；一次性辅助技能承接本次摸牌阶段结束，不新增引擎状态 | ✓
| 582 | `choose_discard_hand_equip_then_recover_target(chooser, target, count, amount)`：XIANDING 杨彪【尽节】可取消地弃等量手牌/装备后回复指定濒死角色 | B (Codex overflow) | 2026-08-03 | 复用 ABI134 精确弃牌选择续体与 ABI356 指定目标回复；声明数、摸牌及零张分支由既有 DSL 表达 | ✓
| 583 | `resolve_dcjue(player, event_player, stage, remaining)`：XIANDING 杨彪【举讹】按本回合实际弃置且仍在弃牌堆的实体牌数与目标体力上限取小值，逐次随机选择首个对本人/目标有合法目标的虚拟【杀】/【过河拆桥】/【五谷丰登】并使用 | B (Codex overflow) | 2026-08-03 ✓ | 仅在加载对应过滤器时追踪弃置实体牌；复用权威 RNG、虚拟牌完整使用/响应/无懈管线与技能每轮状态；ADR 0478 |
| 584 | `resolve_ruanji(player, source, card, action)` + 昭文展示牌追踪/黑牌普通锦囊转化成本 + 酒遁精确过滤器：XIANDING 阮籍【昭文/酒遁】 | B (Codex overflow) | 2026-08-03 ✓ | 复用技能实例实体牌集合、按牌名每回合位图、source-free 实体牌转化、当前牌目标排除与【酒】标记管线；落地见 ADR 0479 |
| 585 | `resolve_zhengbi(player, target, action)` + `resolve_fengying(player, helper_def, action)`：XIANDING 崔琰毛玠【征辟/奉迎】 | B (Codex overflow) | 2026-08-03 ✓ | 复用出牌阶段获牌历史、手牌/装备实体选牌赠予、额外回合队列与一次性辅助技能；仅新增两段技能结算状态机；落地见 ADR 0480 |
| 586 | `resolve_qiaojian(player, target, helper_def, action)` + `resolve_xicha(player, source, card, action)`：XIANDING 傅干【巧谏/析察】 | B (Codex overflow) | 2026-08-03 ✓ | 复用出牌阶段目标唯一性、手牌/装备多选弃置、牌实体追踪、临时辅助技能、当前牌效果次数与保存目标排除管线；落地见 ADR 0481 |
| 587 | `resolve_huzun(player, source, card, action)` + 通用逻辑牌实体材料上下文：XIANDING 胡遵【斩涛/安境】 | B (Codex overflow) | 2026-08-03 ✓ | 复用判定、虚拟【杀】、技能实例状态、座次顺序与权威 RNG；补齐转化牌材料在用牌/伤害触发中的共用来源；Golden、全量 XIANDING、ABI/VM/compiler/conformance 与空 store 导入通过 |
| 588 | `resolve_zhugemengxue(player, action)` + 卡牌名字数/牌堆装备直接使用：XIANDING 诸葛梦雪【寄春/寒英】 | B (Codex overflow) | 2026-08-03 | 复用主动选牌、权威目标选择、装备替换与完整用牌管线；卡牌名字数进入规则元数据 | ✓
| 589 | SHENHUA 许攸【成略/恃才/寸目】：`resolve_chenglve`、`resolve_shicai`、牌堆底摸牌 | C (omp) | 2026-08-03 | 复用翻面、弃牌、花色修饰、牌堆移动与摸牌管线 |
| 590 | SHENHUA 张绣【雄乱/从谏/踞襄】：`resolve_xiongluan`、`resolve_congjian`、`resolve_twjuxiang` | C (omp) | 2026-08-03 | 复用装备栏、判定区、目标限制、赠牌与装备移动管线 |
| 591 | SHENHUA 旧周泰【不屈】：`resolve_gzbuqu` | C (omp) | 2026-08-03 | 复用技能牌堆、点数去重、濒死与回复管线 |
| 592 | SHENHUA 左慈【化身/新生】：`resolve_huashen`、`resolve_xinsheng` | C (omp) | 2026-08-03 | 复用权威 RNG、手牌获得与回合临时技能管线 |
| 593 | `resolve_bailingyun(player, action)`：XIANDING 柏灵筠【灵慧/黠策】顶三使用/随机获得/余牌置顶与每回合双分支 | B (Codex overflow) | 2026-08-03 ✓ | 复用共享区域实体牌使用、权威 RNG、弃牌回复与 ABI 356 非锁定技失效管线；【御心】沿用既有 DSL；Golden、XIANDING 全量、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0488 |
| 594 | `resolve_dcshaowei(player, action)` + `resolve_dcdichou(player, action)`：XIANDING 威关银屏【绍威/涤仇】每回合花色双分支、装备栏废除/恢复、仇标记与回合首尾结算 | B (Codex overflow) | 2026-08-03 ✓ | 复用 CardsMoved 实体牌上下文、权威 RNG、通用回复、装备栏、目标弃牌、名字数和火焰伤害管线，取代早期【涤仇】近似；Golden、XIANDING 全量、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0489 |
| 595 | `resolve_dcdufeng(player, action, helper_skill_def)`：XIANDING 凌操【独锋】强制失去体力/废除装备栏组合、摸牌与回合内攻击范围/杀次数加成 | B (Codex overflow) | 2026-08-03 ✓ | 复用失去体力、装备栏、摸牌、临时技能与 owner mark 修正器管线；闭合 `dc_lingcao`，见 ADR 0490 |
| 596 | `choose_discard_hand_range(player, min, max)`：强制选择范围内手牌弃置，精确承载 XIANDING 田丰【随势】死亡分支 | B (Codex overflow) | 2026-08-03 ✓ | 复用标准选牌屏障、实体牌所有权重验与弃牌移动管线；闭合 `dc_tianfeng`，见 ADR 0491 |
| 597 | `resolve_dcctjiuxian(player, event_player, action)`：XIANDING 陈泰【矜贤】精确重铸半数手牌、技能来源虚拟【决斗】及伤害后选取受伤角色回复 | B (Codex overflow) | 2026-08-03 ✓ | 复用重铸、普通锦囊完整使用/响应、攻击范围和回复管线；同时补齐共享虚拟牌的来源技能记录，不保留旧 JS helper；Golden、XIANDING 全量、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0492 |
| 598 | `resolve_bianyue(player, target, action)`：XIANDING 卞玥【庇族/无胁】同手牌组重复检测、整手交换及新手牌伤害牌置底 | B (Codex overflow) | 2026-08-03 ✓ | 复用摸牌、主动技临时禁用、回复、标准目标屏障、实体移牌及牌堆底顺序；Golden、XIANDING 全量、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0501 |
| 599 | `resolve_sbbiyue(player)`：SB 界貂蝉【闭月】按本回合受伤角色数摸牌 | C (omp) | 2026-08-03 | 并行实现已先落地，协调表补登记以消除号码冲突 |
| 600 | `resolve_zhupeilan(player, target, card, helper_def, action)`：XIANDING 朱佩兰【辞虑/痛悼】锦囊删项与精确牌结算策略、开局技能重置 | B (Codex overflow) | 2026-08-03 ✓ | 复用标准 Choice、包内非可获得 helper、当前牌目标排除、伤害阻止、实体牌移动、技能增删及回复管线；不执行旧 JS，见 ADR 0503 |
| 601 | `resolve_dc_sb_zhangxiu(player, target, action)`：XIANDING 势张绣【附袭/豪义】二选一给牌摸牌/弃牌出杀，以及本回合未造成伤害的伤害牌回收分配 | B (Codex overflow) | 2026-08-03 ✓ | 复用标准 Choice、实体牌移动、虚拟【杀】、回合弃牌/伤害牌材料记录及批量分配管线；Golden、XIANDING 全量、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0504 |
| 602 | `resolve_dc_sb_zhouyu(player, source, card, action)`：XIANDING 新杀谋周瑜【融火/英谋】火焰伤害蓄势、火杀/火攻势力数增伤与一次性额外目标，以及转换状态下补牌火攻/最多手牌角色串行使用实体伤害牌 | B (Codex overflow) | 2026-08-03 ✓ | 已完成：复用当前牌目标与基础伤害、权威 RNG、实体牌完整使用、CardResolved 精确受伤目标及技能实例状态；Golden、XIANDING 全量、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0513 |
| 603 | `resolve_dc_sb_lusu(player, source, card, action)`：XIANDING 新杀谋鲁肃【明势/盟谋】摸二展示三张择一获得，以及每回合阳阴各一次的转换状态【杀】使用/响应序列 | B (Codex overflow) | 2026-08-03 ✓ | 已完成：复用标准 Choice、公开展示、实体转移、普通/转化【杀】完整使用与响应、精确伤害回复及回合分支位；Golden、compiler/VM/API、conformance 与 26 包冷导入通过，不执行旧 JS，见 ADR 0514 |
| 604 | `resolve_zhangjian(player, card, grant_skill_def, action)`：XIANDING 张臶【技能A/技能B】按牌点数改写伤害，以及弃置所有牌后授技至来源下回合开始 | B (Codex overflow) | 2026-08-03 ✓ | 已完成：复用 damage_before 绑定伤害、标准 Choice、弃置全部手牌/装备及 source-turn-bound 通用授技状态；Golden、compiler/VM/API、conformance 与 26 包冷导入通过，不执行旧 JS，见 ADR 0515 |
| 605 | `resolve_zhugeruoxue(player, target, card, action)`：XIANDING 诸葛若雪【琼英/暖惠】场上牌移动后同花色弃牌/展示，以及按装备数连续使用基本牌并在重名后弃置装备 | B (Codex overflow) | 2026-08-03 ✓ | 已完成：复用通用场上牌合法移动、实体花色、公开展示、标准 Choice、虚拟基本牌完整使用及装备区弃置；Golden、compiler/VM/API、conformance 与 26 包冷导入通过，不执行旧 JS，见 ADR 0516 |
| 606 | `resolve_caoyi(player, card, helper_skill_def, action)` + `owner_source_single_target_sha_or_trick_all_materials_from_hand_can_sha`：XIANDING 曹轶【蜜饴/寅君】跨结束阶段反向结算、全手牌材料精确过滤及无来源虚拟【杀】 | B (Codex overflow) | 2026-08-03 ✓ | 已完成：复用标准多目标 Choice、包内一次性 helper、CardResolved 实体材料、无距离虚拟【杀】与回合技能禁用管线；Golden、compiler/VM/API、conformance 与 26 包冷导入通过，不执行旧 JS，见 ADR 0517 |
| 607 | `resolve_malingli(player, card, helper_skill_def, action)`：XIANDING 马伶俐【硝引】按距离亮牌、连续座次分配实体扩展牌，以及伤害来源消费“硝引”增伤或转火焰 | B (Codex overflow) | 2026-08-03 ✓ | 复用权威距离、展示/实体移牌、技能牌堆、标准 Choice 与 damage_before 绑定伤害管线；Golden、整包、compiler/VM/API、conformance 与 26 包冷导入通过，见 ADR 0518；旧 JavaScript 仅作语义证据 |
| 608 | `resolve_nsshuaiyan(player, recovered_player)` + `lebu` 牌选择器：归档扩展“玩点论杀”费诗【率言/墨守】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 回复角色强制二选一、跨角色选牌弃置与仅屏蔽【乐不思蜀】/【兵粮寸断】目标；Golden、compiler/VM/API、engine-core、conformance 与 27 包冷导入通过，见 ADR 0521；旧 JavaScript 仅作语义证据 |
| 609 | `resolve_wdb_hanlong(player, target, card, helper_skill_def, action)` + 弃牌阶段杀计数/健康目标杀伤害过滤器：归档扩展“玩点论杀”韩龙【伺机/刺酋】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 【伺机】按弃牌阶段实际弃置杀数摸两倍牌；【刺酋】对未受伤目标的杀伤害+1，预判致命时授临时强制死亡 helper 并失去本技能；Golden、compiler/VM/API、engine-core、conformance 与 27 包冷导入通过，见 ADR 0522；旧 JavaScript 仅作语义证据 |
| 610 | `resolve_wdb_liufu(player, target, card, zhucheng_skill_def, action)` + “筑”技能牌堆/出牌阶段弃牌/杀目标过滤器：归档扩展“玩点论杀”刘馥【筑城/夺气】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 复用公开技能牌堆、精确弃牌选择、当前杀取消与当前出牌阶段结束管线；Golden、compiler/VM/API、engine-core、conformance 与 27 包冷导入通过，见 ADR 0523；旧 JavaScript 仅作语义证据 |
| 611 | `resolve_wdb_liuyan(player, card, action)` + linked 距离值源 + 势力均不在攻击范围内过滤器：归档扩展“玩点论杀”刘焉【绝道/割据】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 复用主动选手牌弃置、横置标记、双向距离修正、权威攻击范围与摸牌管线；Golden、compiler/VM/API、engine-core、conformance 与 27 包冷导入通过，见 ADR 0524；旧 JavaScript 仅作语义证据 |
| 612 | `resolve_wdb_liuzan(player, target, helper_skill_def, action)` + 负技能状态值源：归档扩展“玩点论杀”留赞【亢音】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 复用失去体力、目标牌弃置、可选多目标摸牌与回合末临时技能管线；Golden、compiler/VM/API、engine-core、conformance 与 27 包冷导入通过，见 ADR 0525；旧 JavaScript 仅作语义证据 |
| 613 | `resolve_wdb_tianyu(player, target, card, action)` + `youdishenru` 卡牌种类/可恢复结算 + 装备阈值布尔修正：归档扩展“玩点论杀”田豫【车阵/诱战】与【诱敌深入】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 复用杀目标续体、无懈窗口、实体/视为杀响应、伤害记录与标准 Choice；Golden、compiler/VM/API、engine-core、conformance 与 27 包冷导入通过，见 ADR 0526；旧 JavaScript 仅作语义证据 |
| 614 | `resolve_wdb_xizhenxihong(player, target, card, action)` + 分类响应禁用 + 回合末逐来源续体：归档扩展“玩点论杀”习珍习宏【负仇/噤言】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | Golden、compiler/VM/API、engine-core、conformance、27 包冷导入、打包投放与编译目录清理均通过，见 ADR 0527；旧 JavaScript 仅作语义证据 |
| 615 | `resolve_wdb_yangyi(player, target, card, action)` + `diaobingqianjiang` 全体锦囊可恢复结算：归档扩展“玩点论杀”杨仪【筹度/立断】与【调兵遣将】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | Golden、compiler/VM/API、engine-core、conformance、27 包冷导入、打包投放与编译目录清理均通过，见 ADR 0528；旧 JavaScript 仅作语义证据 |
| 616 | `resolve_wdb_zaozhirenjun(player, target, card, action)` + `wangmeizhike` + 五谷剩余牌/多目标锦囊排除可恢复结算：归档扩展“玩点论杀”枣祗任峻【粮策/坚壁/军屯】与【望梅止渴】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；Golden 9/9、共享门禁、整包打包投放与编译目录清理完成，见 ADR 0529 |
| 617 | `resolve_dddxianxi(player, card, action)` + 本技能新增【杀】目标/精确受伤追踪过滤器：归档扩展“3D精选”韩当【险袭】完整语义 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；无距离追加任意目标、同步最终 CardResolved 目标，结算后对未受伤新增目标按旧 JS 二选一；Golden、共享门禁、29 包冷导入与打包投放通过，见 ADR 0534 |
| 618 | `resolve_dddxiaheng(player, action)`：归档扩展“3D精选”周处【侠横】完整可恢复结算与三名不同受伤目标状态 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；强制选弃牌角色、目标自弃至多两张、再选伤害角色，伤害后结算减上限/失去技能，累计三名后永久删除惩罚条款；Golden、共享门禁、30 包冷导入与打包投放通过，见 ADR 0536 |
| 619 | `resolve_ddd_liuchong(player, actor, card, action)`：归档扩展“3D精选”刘宠【精彀/末焱】武器结算伤害与轮首亮牌分配/受伤增伤状态 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用攻击范围、牌堆展示、标准 Choice、实体移牌、伤害与当前伤害修改管线，Golden 与累积 npack 已通过 |
| 620 | `dddshilie` source-free view-as cost：明置未公开手牌且点数和不小于双方体力，精确相等时补牌 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用 source-free 基本牌声明、标准选牌续体、公开牌、每回合次数与摸牌管线，Golden、共享门禁、31 包冷导入与编译产物清理通过，见 ADR 0538 |
| 621 | `resolve_ddd_duanbing(player, target, card, action)`：3D精选丁奉【短兵】自置兵粮、摸牌、无距离虚拟杀及伤害后转移判定牌 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用延时锦囊判定区、标准 Choice、摸牌、虚拟杀、伤害触发与实体移牌管线；Golden、共享门禁、31 包冷导入与编译产物清理通过，见 ADR 0539 |
| 622 | `resolve_ddd_wuzhi(player, card, action)` + `phase_any_skipped`：3D精选吴质【灵涌/虚孝】任意阶段跳过后循环判定/使用判定牌，以及弃牌堆黑色基本牌直置兵粮并令目标摸二 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用判定、标准 Choice、实体牌使用、判定区、实体移牌和摸牌管线；Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0541 |
| 623 | `resolve_ddd_xujing(player, source, card, action)`：3D精选许靖【虚誉/实荐】下一张牌弃置债务、第二张牌赠予及下一张有目标牌额外结算/摸牌 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用 CardResolved、出牌阶段计数、标准 Choice、实体赠牌、技能实例状态与当前牌结算后重放管线；Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0545 |
| 624 | `resolve_ddd_caomao(player, action)` + 曹髦【太思/驱车】回合历史与主动连续结算：选择本回合进入弃牌堆的实体牌赠予、伤害来源补牌、红牌赠予/黑牌转无次数【杀】及临时距离标记 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；按译文执行驱车至多体力上限轮，Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0547 |
| 625 | `resolve_ddd_xinxianying(player, actor, action)` + `dddzhijie` 技能牌堆双牌响应成本：3D精选辛宪英【洞察/智解】双方依次置“鉴”、回合末依次取“鉴”，同/异色双“鉴”转【闪/无懈】并摸二 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用技能牌堆、标准 Choice 与多牌响应管线，Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0548 |
| 626 | `resolve_ddd_xianglang(player, actor, action)` + `dddfusi` 明置手牌借用成本与全局手牌限制：3D精选向朗【恰化/腹笥/拓籍】明置手牌、临时【恂恂】、经牌主确认由他人使用/打出明置牌、全明置时锁手牌及借牌后可摸三 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用公开手牌、source-free 转化、标准 Choice、观星 modifier 与结算尾效果；Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0550 |
| 627 | `resolve_ddd_yujin(player, actor, action)` + “整军”体力/手牌/装备变化候选过滤器：3D精选于禁【整军】在自己的出牌阶段三类各限一次，数值相等后选择回复/失去体力、摸牌/弃牌或移动装备 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用出牌阶段分类位、标准 Choice、体力/摸弃牌及场上牌移动管线，Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0552 |
| 628 | `resolve_ddd_liuye(player, actor, card, action)` + 【察势】阶段目标/弃牌花色类别状态与【齐策】锦囊解锁状态：3D精选刘晔两技能精确语义 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用手牌装备弃置、实体牌有效花色/类别、标准摸牌与技能实例持久状态，Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0553 |
| 629 | `resolve_ddd_baosanniang(player, target, card, action, cards)` + 回合恰好两名用牌/响应角色过滤器：3D精选鲍三娘【枝连/赍剑】完整可恢复结算 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用逐角色回合用牌/响应计数、进入弃牌堆实体牌、同色展示、实体赠牌与虚拟牌目标结算管线，Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0556 |
| 630 | `resolve_ddd_zhenji(player, target, action, stage, piles, cards)` + 两组隐藏技能牌堆：3D精选甄宓【淼形/浮世】起始手牌三倍拆组、摸牌阶段后平衡与整手交换 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用 `initial_hand_count multiply`、暗置技能牌堆、标准 Choice、实体移牌与牌堆补牌管线；Golden、共享门禁、31 包冷导入与打包投放通过，见 ADR 0557 |
| 631 | `resolve_ddd_zhaoang(player, action, stage, card, kind, count)` + `owner_alive_ddd_fenji_available`：3D精选赵昂【奋计】阶段首未用普通锦囊虚拟使用、永久牌名去重及摸/弃牌阶段数值改写 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用现有普通锦囊目标规格、完整牌结算、阶段固定摸牌数、跳过弃牌与标准 Choice 管线；Golden、共享门禁、31 包冷导入、打包投放与编译产物清理通过，见 ADR 0559 |
| 632 | `resolve_ddd_liuba(player, actor, action, cards)` + 【丰政】全员点数和 13 转【无中生有】/上一轮使用者统一分配 + 【玉律】弃牌点数响应与第二次交换 | B (Codex overflow) | 2026-08-03 ✓ | 复用每出牌阶段限次、点数和选择、多实体牌普通用牌、全员 non-gainable helper、公开技能牌堆与标准 Choice；Golden、共享门禁、31 包冷导入与累积 DDD npack 已闭环，旧 JavaScript 仅作语义证据 |
| 633 | `resolve_ddd_jianshuo(player, target, action, cards)`：多人加入两方并按平均点数判定的【分野】，以及按手牌名次调整与伤害防止升级的【逝潮】 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用实体拼点牌移动、座次顺序虚拟【杀】、主公查询、手牌调整与当前伤害防止；Golden、共享门禁、31 包冷导入与累积 DDD npack 闭环，见 ADR 0565 |
| 634 | `resolve_ddd_guanning(player, actor, action, card)`：全场唯一【游学】标记轮转/距离摸牌、回合末最后一张合格实体牌置“经”，以及以任意牌转化为“经”并可转赠 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用角色标记、距离查询、按需回合实体用牌历史、公开技能牌堆、普通牌目标规格与完整实体牌使用管线；Golden、共享门禁、31 包冷导入与累积 DDD npack 闭环，见 ADR 0570 |
| 635 | `resolve_ddd_caoshuang(player, actor, card, action)`：曹爽【专摄】赠牌名追踪、同名基本牌/普通锦囊追加合法目标与未使用惩罚，以及【危秋】无手牌时以摸牌替代他人回复 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；补齐通用 `recover_before`/当前回复取消根能力，复用实体赠牌、逻辑牌名状态、当前用牌追加目标与标准 Choice；Golden、共享门禁、31 包冷导入、打包投放及编译目录清理通过，见 ADR 0574；旧 JavaScript 仅作语义证据 |
| 636 | `resolve_ddd_xuelingyun(player, actor, card, action)`：薛灵芸【涟洏】回收红色牌/点数使用上限与【暗织】上一张黑牌后的回合首个黑色目标失效 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用 CardResolved 实体材料、有效颜色/点数、当前牌目标移除、技能实例回合状态与标准 Choice；Golden、共享门禁、31 包冷导入、打包投放及编译目录清理通过，见 ADR 0576；旧 JavaScript 仅作语义证据 |
| 637 | `resolve_ddd_liuhong(player, actor, action)`：刘宏【失兴】动态派生技能、【党锢】连环/弃牌阶段装备计入与【覆宗】开局摸牌/群势力死亡弃主公牌 | B (Codex overflow) | 2026-08-04 ✓ | 已落地；同时修复复用的【酒诗】为翻面使用无实体【酒】及背面受伤后可翻回；Golden、共享门禁、31 包冷导入、打包投放及编译目录清理通过，见 ADR 0577；旧 JavaScript 仅作语义证据 |
| 638 | `resolve_ddd_xiahouxuan(player, action)`：夏侯玄【朗怀】转换摸牌与【玄论】伤后摸牌、回合末顶底置牌/强制朗怀状态 | B (Codex overflow) | 2026-08-04 ✓ | 已落地；复用固定摸牌数、有效花色、公开手牌、标准可恢复 Choice 与牌堆顶底顺序；Golden、共享门禁、31 包冷导入、打包投放及编译目录清理通过，见 ADR 0578；旧 JavaScript 仅作语义证据 |
| 639 | `resolve_ddd_zhangkai(player, actor, card, action)`：张闿【劫行】截取他人回复/装备及后续重置、【拜泪】唯一最多装备或 1 体力目标分支 | B (Codex overflow) | 2026-08-04 ✓ | 已落地；复用回复前取消、装备使用取消、限定技计数、标准可恢复 Choice、获得牌与伤害/死亡管线；Golden、共享门禁、31 包冷导入与打包投放通过，见 ADR 0579；旧 JavaScript 仅作语义证据 |
| 640 | `resolve_ddd_liangxi(player, actor, card, action)`：梁习【仝御】异花色牌转【五谷丰登】、结算方向/额外亮牌、逐目标封手或无效及余牌赠予 | B (Codex overflow) | 2026-08-04 ✓ | 已落地；复用现有视为牌、五谷丰登、目标排除、回合临时限制与实体牌移动管线，Golden、共享门禁、31 包冷导入、打包投放及编译目录清理通过，见 ADR 0580 |
| 641 | `GuozhanUseZhulianBihe(player)`：国战【珠联璧合】标记当【无中生有】使用原语 | A (Claude) | 2026-08-05 ✓ | 已落地；完整 8 文件 VM 驱动接线 + ABI-MAX/RULE_VM_ABI_VERSION bump 641 + **国战内容全量**：256 武将 skills 全填充 + 464 技能 DSL + bootstrap_guozhan 切国战武将池 + 12 装备卡牌 + 12 锦囊进国战牌堆（CardKind::Other）；conformance 4 全过、abort_bump_without_claim 过、国战单测 21 + 烟雾 2、失败集合 257→247（0 新增）、warnings 59，见 ADR 0604。12 锦囊的结算（各含 choice 流）需新增 CardKind+choice 原语，判为独立 ABI 642+ 工作 |
| 642 | `ResolveShoulijian(player, target, skill)`：欢乐卡牌【手里剑】使用结算（目标弃一张装备牌或受 1 伤害）| A (Claude) | 2026-08-05 ✓ | 已落地；完整 8 文件 VM 驱动接线 + 原语复用 `open_xiaoguo_target_equipment_or_damage`（有装备开「弃装备或伤害」choice、无装备直接受 1 伤害）+ shoulijian DSL active 技能 + gwent 2 技能（gw_wuyao→杀/gw_lang→酒 view_as）；conformance 4 全过、abort_bump_without_claim 过、247=247 零回归、warnings 59，见 ADR 0607 |
| 643 | `CancelIncomingCardUse(player, source, card, skill)`：「取消使用牌」——成为他人使用牌目标时，取消该牌对你生效并获取对你使用的实体牌（gwent 杜达元帅1 特殊卡）| A (Claude) | 2026-08-05 ✓ | 已落地；完整 8 文件 VM 驱动接线 + 原语复用 `exclude_saved_card_target`（杀走 exclude_saved_sha_target、锦囊走 WuxieWindow resume_payload 排除）+ `gain_event_cards`（获得对你使用的牌）+ gw_dudayuanshuai1/2 DSL 运行时技能（3583584906/3832623127）；conformance 4 全过、abort_bump_without_claim 过、247=247 零回归、warnings 60（本次改行 0 新增），见 ADR 0608 |
| 644 | `ChooseOtherGainOneOfTwoSkills(player, target, first_def, second_def, skill)`：择一加技能——玩家在两个技能定义间二选一并置入目标（gwent 呼朋 hupeng 择一杜达元帅）| A (Claude) | 2026-08-06 ✓ | 已落地；复用 `ContinuationKind::Custom` 魔数分发（不新增 ContinuationKind 变体、不破坏回放序数）+ 完整 VM 驱动接线 + hupeng 真实 DSL（弃一牌 + 选目标 + 择一杜达元帅1/2 置入，def 3607688462 与 gw_zhuoertan 引用天然联动）；conformance 4 全过、abort_bump_without_claim 过、247=247（双向差异仅 2 项他人域：standard_remaining 出集/standard_npack 入集）、改行 0 警告，见 ADR 0612 |
| 645 | `GwyewuRandomDiscardChain(player, wuyao_def, lang_def)`：叶舞造卡入口随机弃牌链（gwent 叶舞）| A (Claude) | 2026-08-07 ✓ | 0627 批次 claim；RULE_VM_ABI_VERSION bump 645；C 段补 codegen.rs 常量导入（半落地遗漏）并同步 ABI-MAX |
| 646 | `ChooseTargetGainMaxHp(player, skill)`：选目标加上限（pxshouchun 寿春 batch AJ）| B (Codex) | 2026-08-15 ✓ | 补记领取：原会话已落地全链路（tag 665/Cont 266、原测试 6/6 过）但未领号；本次代为补 ABI-MAX/RULE_VM_ABI_VERSION 同步 |
| 647 | `GainTargetHandCard(player, skill)`：两阶段获得目标手牌（pxsizhou 肃州 batch AN）| B (Codex) | 2026-08-15 ✓ | 补记领取：原会话已落地全链路（tag 666/Cont 267、原测试 4/4 过）但未领号；sp npack 已按 647 发布（13:23），本次补齐 host=647 使 extensions 锁恢复 |
| 648 | `ChooseClanNotMaxHandLimitGain(player, skill)`：选同族或自己且手牌上限非全场最高者上限+1（穆荫 clanmuyin，GameState.hand_limit_bonus 持久化）| A (Codex) | 2026-08-15 ✓ | 已落地：tag 667/Cont 268/引擎 ChooseClanNotMaxHandLimitGain + GameState.hand_limit_bonus + HandLimit 查询计入；host RULE_VM_ABI_VERSION bump 648 |
| 649 | `ExtraResolveCurrentTrick(player, card, skill)`：为挂起锦囊追加一次效果结算（额外结算家族 clanfenshi/fuyu/olufuyou/twchengxi/dckeshui，上游 effectCount++）| A (Codex) | 2026-08-15 ✓ | 已落地：tag 668 纯同步原语 + GameState.extra_effect_count + apply_deferred_trick_with_extras 执行循环（上限 2 次）；host bump 649 |
| 650 | `PlaceHandCardToExpansion(player, skill)`：选一张手牌置入自身技能牌堆（expansion P2 置入，ADR 0641，复用 CommitStoreHandShaInSkillPile 落堆管线，Cont 269）| A (Codex) | 2026-08-15 ✓ | 已落地：tag 669/Cont 269/host bump 650；候选 ollihuo/olchunlao/xinzhoufu2/ymfriendfangqiu（§6b 审计）|
| 651 | `GainFromSkillPile(player, skill)`：从自身技能牌堆选一张牌加入手牌（expansion P1 取用，ADR 0641，Cont 270；用户 2026-08-16 批准门槛降至实面 2 后铸造）| A (Codex) | 2026-08-16 ✓ | 已落地：tag 671/Cont 270/host bump 651；首批 dchaoshi(collab)/ao_diegui(key) |
| 652 | `ChooseOtherAddMark(player, mark, delta, skill)`：选一名其他存活角色，对其施加标记增减（choose_other_add_mark；护甲/横置族 mbzhenfu+cangchizhibi×4，5 例同构立项，tag 672/Cont 271）| A (Codex) | 2026-08-21 ✓ | 已落地：tag 672/Cont 271/host bump 652；mbzhenfu(mobile)+cangchizhibi×4(shahai) 迁移，两包 pin 652 |
| 653 | `ResolveJsrgchaozheng(player, skill)`：朝争议事结算（ADR 0644 P1：所有其他角色亮首张手牌为意见→红黑分组→多数决→红=红意见者各回复1/黑=红意见者各失去1→全同意见发起者摸参与者数张；tag 673）| A (Codex) | 2026-08-22 ✓ | 已落地：tag 673/674/host bump 654；jsrgchaozheng+jsrgsheju 迁移，jsrg pin 654，红/黑/无结果三分支 Golden 5/5 |
| 654 | `ResolveJsrgsheju(player, counterpart, skill)`：设居双人议事（ADR 0644 P1：双方亮牌→黑=双方各减1体力上限；tag 674）| A (Codex) | 2026-08-22 ✓ | 已落地：tag 673/674/host bump 654；jsrgchaozheng+jsrgsheju 迁移，jsrg pin 654，红/黑/无结果三分支 Golden 5/5 |
| 655 | `StartDebateAllOthers(player, skill)`：发起议事——所有其他有手牌角色亮首张手牌分组，只发 DebateResolved 事件（结算由反应技 DSL 承担；ADR 0644 P2，tag 675）| A (Codex) | 2026-08-22 ✓ | 已落地：tag 675/676/Cont 272/host bump 656；DebateResolved 事件+三过滤器；jsrgfayi 反应技迁移（jsrg pin 654：fayi 未用新原语，655/656 待后续反应技消费）|
| 656 | `StartDebateTargets(player, skill)`：选 1..N 名其他角色议事（Cont 272），同上只发事件（ADR 0644 P2，tag 676）| A (Codex) | 2026-08-22 ✓ | 已落地：tag 675/676/Cont 272/host bump 656；DebateResolved 事件+三过滤器；jsrgfayi 反应技迁移（jsrg pin 654：fayi 未用新原语，655/656 待后续反应技消费）|
| 657 | `DiscardBasicDrawEqual(player, skill)`：弃置任意张基本牌并摸等量（玉蝉族 g_yuchan_equip 引擎，40 例消费者；tag 677/Cont 273）| A (Codex) | 2026-08-22 | 领取：host bump 657 待落地 |
| 564 | `resolve_potlvyi(player, target, card, action)`（势吕壹【隳律/颂谳/势施】）+ 轮次弃牌颜色、出牌阶段实体牌历史与受伤来源响应锁过滤器 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用牌堆按颜色取牌、实体牌使用历史、虚拟伤害、赠牌、无中生有/无懈可击与逐目标不可响应管线；Golden、兵势整包、compiler、ABI/schema、conformance 与 26 包冷导入通过，落地见 ADR 0458 |
| 570 | `resolve_potyuji(player, target, card, action)`（势于吉【道转/符济】）+ 每轮基本牌名、符济实体牌标签及下一张杀/闪效果 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用声明基本牌、实体牌选择/赠予、花色检索、伤害增量与逐牌结算管线；Golden、兵势整包、compiler、ABI/schema、conformance 与 26 包冷导入通过，见 ADR 0467 |
| 561 | `resolve_potzhouyu(player, target, card, action)`（势周瑜【炽沄/焰洄/焚涛/雄姿】）+ 获牌/展示/连环火焰过滤器 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用实体赠牌、公开展示、阶段/回合历史、连环状态、火焰伤害与限定技管线；Golden、BINGSHI 整包、compiler、ABI/schema、conformance 与 26 包冷导入通过 |
| 515 | `resolve_potchengpu`（兵势程普：督作/避憾） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：获得牌后选择角色取得牌堆最靠顶火杀、专属移牌原因阻断自递归、自己/受伤角色弃手牌至体力并令绑定杀伤害减一；Golden、闭包、内容锁、compiler 463/463 与 conformance 8/8 通过，落地见 ADR 0384 |
| 516 | `lose_max_hp_1` 无实体牌视为使用成本、`owner_alive_card_targeted_healthy`、`resolve_potjugu`（兵势臧洪：烈志/拒孤） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：复用无实体牌虚拟牌、技能回合计数、摸牌与强制弃牌管线；Golden、兵势闭包、扩展导入与 conformance 8/8 通过，落地见 ADR 0385 |
| 517 | `resolve_potsifeng` + 伺锋牌堆触发过滤器（兵势曹真） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：复用处理区、目标技能牌堆、牌色、弃牌、伤害与获得牌管线；Golden、兵势闭包、扩展导入与 conformance 8/8 通过，落地见 ADR 0386 |
| 518 | `resolve_potfaen` / `resolve_potdingpin` + 任意角色额外摸牌阶段队列（兵势陈群） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：复用技能实例上一名用牌者/反向项状态、回合失牌计数与普通摸牌阶段管线；Golden、兵势闭包、扩展导入与 conformance 8/8 通过，落地见 ADR 0387 |
| 519 | `resolve_potwanglie` / `resolve_pothongyi` + 精确策略牌结算过滤器（兵势陈到） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：复用精确实体牌策略、仅可自目标修正、技能标记与伤害/阶段事件；Golden、兵势闭包、26 包扩展导入与 conformance 8/8 通过，落地见 ADR 0388 |
| 520 | `resolve_potguansha` / `resolve_potjiyu` + 灌沙限定触发过滤器（兵势娄圭） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：复用全区移牌、基本牌检索、手牌上限标记与技能实例追踪牌状态；Golden、兵势闭包、26 包扩展导入与 conformance 8/8 通过，落地见 ADR 0391 |
| 521 | `view_as no_use_count` / `target_lacks_source_suit_unrespondable`、`resolve_mbzhuji`、护甲吸收普通伤害（兵势孙韶） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：装备实体牌转无距离/次数杀、按目标同花色手牌决定不可响应；同花色整手弃置、检索并使用装备及三选一收益，Golden、兵势闭包、26 包扩展导入与 conformance 8/8 通过，落地见 ADR 0392 |
| 522 | `resolve_potlibing` / `resolve_potpoxi`（兵势夏侯霸） | B (Codex overflow) | 2026-08-02 ✓ | 已完成：非伤害牌三段循环增益、伤害牌重置；弃置杀从弃牌堆无距离/次数使用，按目标手牌数决定不可响应，Golden、兵势闭包、编译器与 conformance 通过，落地见 ADR 0394 |
| 0341 | ABI 470 思变展示分配复合 + sp2 张温 | C (omp) | 2026-08-02 ✓ |

| 471 | `skip_next_discard_phase(player)` 效果 + `resolve_spfengyin(player, event_player)` 复合（他人准备阶段且体力不低于你：可选给其一张【杀】令其跳过出牌阶段和弃牌阶段）+ SP伏完【封印】 | C (omp) | 2026-08-02 ✓ | 闭环 collab SP伏完（sp_fuwan；spchizhong 既有）。落地见 ADR 0342 |
| 0342 | ABI 471 封印授杀跳阶段复合 + collab SP伏完 | C (omp) | 2026-08-02 ✓ |

| 472 | 过滤器 `owner_source_play_card_repeats_previous_targets`（出牌阶段使用的牌与上一张牌存在共同目标，每回合至多三次）+ `resolve_xinfu_lianpian(player, targets)` 复合（摸一张并可将之交给共同目标之一）+ 苏飞【连篇】 | C (omp) | 2026-08-02 ✓ | 闭环 sp2 苏飞（xf_sufei）；引擎新增 LAST_CARD_TARGETS 回合位掩码记录上一张牌的目标。落地见 ADR 0343 |
| 0343 | ABI 472 连篇共同目标摸牌授牌复合 + sp2 苏飞 | C (omp) | 2026-08-02 ✓ |

| 473 | `resolve_gongjian(player, targets)` 复合（本回合使用的【杀】与上一张【杀】存在共同目标时：对每个共同目标弃置 1-2 张手牌/装备牌，然后获得其中被弃置的【杀】）+ 过滤器 `owner_source_sha_repeats_previous_sha_targets` + 朱儁【拱坚】 | C (omp) | 2026-08-02 ✓ | 闭环 sp2 朱儁（zhujun；kuimang 既有）；引擎新增 LAST_SHA_TARGETS 记录上一张【杀】的目标。落地见 ADR 0344 |
| 0344 | ABI 473 拱坚共同目标弃牌夺杀复合 + sp2 朱儁 | C (omp) | 2026-08-02 ✓ |

| 474 | 过滤器 `owner_equip_zone_changed`（装备区有牌进出）+ `resolve_xinfu_qinguo(player)` 复合（使用装备牌后可视为使用一张【杀】）+ 吕岱【勤国】 | C (omp) | 2026-08-02 ✓ | 闭环 sp2 吕岱（lvdai）；回复分支以 empty_equip_slots == 5 - hp 等价装备数==体力。落地见 ADR 0345 |
| 0345 | ABI 474 勤国装备授杀复合 + sp2 吕岱 | C (omp) | 2026-08-02 ✓ |

| 475 | `resolve_dcmanzhi(player)` 复合（准备/结束阶段二选一：①令目标交给你至多两张牌并视为使用一张无距离【杀】；②获得目标区域内至多两张牌并交给其等量牌后摸一张；每分支每回合一次，以技能 mark 记录）+ 孟优【蛮智】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 孟优（mengyou；manyi 既有）。落地见 ADR 0346 |
| 0346 | ABI 475 蛮智双分支复合 + huicui 孟优 | C (omp) | 2026-08-02 ✓ |

| 476 | `resolve_xinxingluan(player)` 复合（出牌阶段使用牌后每回合一次三选一：①观看牌堆两张点数6并获其一（无则摸六）；②获得目标装备/判定区一张点数6；③目标弃一张点数6手牌或交给你一张牌）+ 樊稠【心星乱】 | C (omp) | 2026-08-02 ✓ | 闭环 sp2 樊稠（fanchou）；每回合一次以过滤器 owner_source_once_per_turn 表达。落地见 ADR 0347 |
| 0347 | ABI 476 心星乱三点数6分支复合 + sp2 樊稠 | C (omp) | 2026-08-02 ✓ |

| 477 | `resolve_dckuizhen(player)` 复合（出牌阶段限一次，选目标（手牌≥你或体力≥你）：其视为对你使用一张【决斗】；若你因此受伤则获得其所有手牌【杀】，否则其失去 1 点体力）+ 公孙修【溃阵】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 公孙修（gongsunxiu；dcgangu 既有）；双阶段：决斗使用以 mark 记录发起者，DuelResolved 事件按胜负结算。落地见 ADR 0348 |
| 0348 | ABI 477 溃阵决斗分支复合 + huicui 公孙修 | C (omp) | 2026-08-02 ✓ |

| 478 | `resolve_nsdingzhou(player)` 复合（出牌阶段限一次，选目标：随机获得其一张手牌/装备/判定牌（确定性近似取首张），摸一张，若花色颜色不同则失去 1 点体力）+ 蔡瑁【定州】 | C (omp) | 2026-08-02 ✓ | 闭环 diy 蔡瑁（ns_caimao）；随机取牌以确定性取首张近似。落地见 ADR 0349 |
| 0349 | ABI 478 定州取牌摸牌比色复合 + diy 蔡瑁 | C (omp) | 2026-08-02 ✓ |

| 479 | `resolve_sxrmlanjiao(player)` 复合（出牌阶段限一次每目标：循环查看其两张手牌，目标可选失去1点体力并摸一张令你重新选择；停止后获得最后查看的两张牌；每目标每回合一次以技能 mark（key=目标id）记录）+ 嗔曹操【揽娇】 | C (omp) | 2026-08-02 ✓ | 闭环 sxrm 嗔曹操（sxrm_chen_caocao）；随机/自选查看以确定性取前两张近似。落地见 ADR 0350 |
| 0350 | ABI 479 揽娇循环查看复合 + sxrm 嗔曹操 | C (omp) | 2026-08-02 ✓ |

| 0355 | ABI 493 浮海私密选择与连续计数复合 + mobile 魏文诸葛直 | B (Codex overflow) | 2026-08-02 ✓ |
| 0357 | ABI 494 扬武红桃成本与差值取牌 + offline 樊稠 | B (Codex overflow) | 2026-08-02 ✓ |
| 0358 | ABI 495 矜伐交装备/取牌与黑桃虚拟杀复合 + offline 文钦 | B (Codex overflow) | 2026-08-02 ✓ |

| 480 | `resolve_rekuanshi(player)` + `resolve_rekuanshi_resolve(player, event_player)` 复合（结束阶段选目标标记；其后该角色本回合第 2 次受伤时回复 1 点体力，每回合一次，以技能 mark 记录）+ 阚泽【宽释】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 阚泽（re_kanze）；以 turn.damage_taken_count==2 判定第 2 次伤害。落地见 ADR 0351 |
| 0351 | ABI 480 宽释二伤回复复合 + huicui 阚泽 | C (omp) | 2026-08-02 ✓ |

| 481 | `resolve_dcmoyu(player)` 复合（出牌阶段限一次每目标：获得目标 1（或 2，若上轮未被打）张手牌/装备/判定牌，其视为对你使用一张无距离【杀】；若你因此受伤则本回合禁用本技，否则下次多拿一张）+ 裴元绍【魔狱】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 裴元绍（peiyuanshao）；闪响应窗口时分支跳过（文档化近似）。落地见 ADR 0352 |
| 0352 | ABI 481 魔狱夺牌反击复合 + huicui 裴元绍 | C (omp) | 2026-08-02 ✓ |

| 482 | `resolve_xizhen(player)` 复合（出牌阶段开始选目标，视为对其使用一张【杀】或【决斗】）+ 高览【袭阵】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 高览（dc_gaolan）；响应代偿子技能（目标回复+你摸牌）依赖响应窗口事件，文档化省略。落地见 ADR 0354 |
| 0354 | ABI 482 袭阵视杀/决斗复合 + huicui 高览 | C (omp) | 2026-08-02 ✓ |

| 483 | `resolve_xuxie(player)` + `resolve_xuxie_add(player)` 复合（出牌阶段开始失去 1 点体力上限，对距离 ≤1 的相邻角色二选一：弃各一张牌或各摸一张；出牌阶段结束时若有人体力上限大于你则恢复）+ 邢道荣【虚泄】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 邢道荣（xingdaorong）；距离以 query 距离近似。落地见 ADR 0356 |
| 0356 | ABI 483 虚泄上限牺牲复合 + huicui 邢道荣 | C (omp) | 2026-08-02 ✓ |

| 484 | `resolve_dcchiying(player)` 复合（出牌阶段限一次，选目标（体力≤你）：目标循环选择攻击范围内一名未选过的角色弃其一张牌，停止后获得所弃的全体基本牌）+ 高翔【驰应】 | C (omp) | 2026-08-02 | 闭环 huicui 高翔（gaoxiang）；范围内以 attack_range 查询近似 |
| 0359 | ABI 484 驰应循环弃牌夺基复合 + huicui 高翔 | C (omp) | 2026-08-02 |
| 0360 | ABI 496 齐锋失牌计数与等额伤害虚拟杀 + offline 杨秋 | B (Codex overflow) | 2026-08-02 ✓ |

| 485 | `resolve_xinkuangfu(player)` 复合（出牌阶段限一次，选目标（可为自己）弃其一件装备，视为使用一张无距离【杀】；若弃自己装备且此杀造成伤害则摸两张，若弃他人装备且此杀未造成伤害则弃两张手牌）+ 潘凤【匡扶】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 潘凤（re_panfeng）；闪响应窗口时分支跳过（文档化近似）。落地见 ADR 0361 |
| 0361 | ABI 485 匡扶弃装授杀复合 + huicui 潘凤 | C (omp) | 2026-08-02 ✓ |

| 486 | `resolve_dcxiangshu(player, target)` + `resolve_dcxiangshu_lottery(player, target)` 复合（他人出牌阶段开始（手牌≥体力）时选择秘密数字 0-5；其出牌阶段结束时按 |手牌数-数字| 结算：≤1 获得其一张牌，==0 对其造成 1 点伤害）+ 张闿【相术】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 张闿（zhangkai）；弃牌保密步骤省略（近似）。落地见 ADR 0362 |
| 0362 | ABI 486 相术猜数复合 + huicui 张闿 | C (omp) | 2026-08-02 ✓ |
| 0363 | ABI 497 远略赠牌后受赠者可使用、成功后摸牌 + sp2 张郃 | B (Codex overflow) | 2026-08-02 ✓ |

| 487 | `resolve_dcyingtu(player, cards)` 复合（牌移动后：若你的上家/下家手牌获得牌（非其摸牌阶段），获得其一张 he；将一张 he 交给其对位角色，若所给为装备牌则其使用之）+ 陈珪【应图】 | C (omp) | 2026-08-02 ✓ | 闭环 huicui 陈珪（chengui；dccongshi 既有）；精确实现摸牌阶段豁免与每回合限一次。落地见 ADR 0364 |
| 0364 | ABI 487 应图邻位取予复合 + huicui 陈珪 | C (omp) | 2026-08-02 ✓ |
| 0365 | ABI 498 统北声明类别、交牌或伤害增加 + offline 荆扬曹操 | B (Codex overflow) | 2026-08-02 ✓ |
| 0366 | shiji 神孙策既有三技能角色闭包（无新 ABI） | B (Codex overflow) | 2026-08-02 ✓ |
| 0367 | huicui 孟优、界阚泽跨包既有技能闭包修复（无新 ABI） | B (Codex overflow) | 2026-08-02 ✓ |
| 0368 | sp2 三名既有双技能武将闭包修复（无新 ABI） | B (Codex overflow) | 2026-08-02 ✓ |
| 0369 | tw 周处、桥蕤内部 helper 折叠与源角色闭包修复（无新 ABI） | B (Codex overflow) | 2026-08-02 ✓ |
| 0370 | ABI 499 借兵随机获牌并使用装备 + mobile 杨阜 | B (Codex overflow) | 2026-08-02 ✓ |
| 0371 | ABI 500 胆守目标历史计数、弃牌伤害 + mobile 手杀界朱然 | B (Codex overflow) | 2026-08-02 ✓ |
| 0372 | ABI 501 江山如故驰应顺序弃牌、条件夺基 + jsrg 高翔 | B (Codex overflow) | 2026-08-02 ✓ |
| 0373 | ABI 502 灭吴实体牌转化、武库消耗与结算摸牌 + shiji 杜预 | B (Codex overflow) | 2026-08-02 ✓ |

| 0374 | ABI 503 通用整牌结算边界与兵势国渊三技能 | B (Codex overflow) | 2026-08-02 |
| 0375 | ABI 504 兵势陈祗（全弃/额外回合/仁行） | B (Codex overflow) | 2026-08-02 |
| 0376 | xianding-batch3（存嗣/勇决） | D (opencode) | 2026-08-02 |
| 0377 | ABI 505 兵势黄祖（驰张/断粮：伤害牌距离选择器 + 实体牌响应/重铸） | B (Codex overflow) | 2026-08-02 ✓ |
| 0378 | ABI 506 兵势陆郁生（润威/双还） | B (Codex overflow) | 2026-08-02 ✓ |
| 0379 | ABI 507 兵势孙峻（凶图） | B (Codex overflow) | 2026-08-02 ✓ |
| 0380 | ABI 509 兵势杨弘（坚机/远谋） | B (Codex overflow) | 2026-08-02 ✓ |
| 0381 | shenhua 颜良文丑双雄 + 张昭张纮直谏/固政（ABI 511-513） | C (omp) | 2026-08-02 ✓ |
| 0382 | ABI 510 兵势张燕（飞径/骁戈） | B (Codex overflow) | 2026-08-02 ✓ |
| 0383 | ABI 514 兵势庞羲（蓄业/狂想） | B (Codex overflow) | 2026-08-02 ✓ |
| 0384 | ABI 515 兵势程普（督作/避憾） | B (Codex overflow) | 2026-08-02 ✓ |
| 0385 | ABI 516 兵势臧洪（烈志/拒孤） | B (Codex overflow) | 2026-08-02 ✓ |
| 0386 | ABI 517 兵势曹真（伺锋） | B (Codex overflow) | 2026-08-02 ✓ |
| 0387 | ABI 518 兵势陈群（法恩/定品） | B (Codex overflow) | 2026-08-02 ✓ |
| 0388 | ABI 519 兵势陈到（往烈/弘毅） | B (Codex overflow) | 2026-08-02 ✓ |
| 0389 | xianding-batch3（存嗣/勇决） | D (opencode) | 2026-08-02 |
| 0390 | xianding-batch4（安境/伪涕） | D (opencode) | 2026-08-02 |
| 0391 | ABI 520 兵势娄圭（灌沙/集驭） | B (Codex overflow) | 2026-08-02 ✓ |
| 0392 | ABI 521 兵势孙韶（敢决/筑墼） | B (Codex overflow) | 2026-08-02 ✓ |
| 0393 | shiji-spsongshu-zhangwen（SP 张温闭环，ABI 404 送枢） | A (Claude) | 2026-08-02 ✓ |
| 0394 | ABI 522 兵势夏侯霸（砺兵/破袭） | B (Codex overflow) | 2026-08-02 ✓ |
| 0395 | shenhua 孙策制霸 + 张绣从俭（ABI 523-524） | C (omp) | 2026-08-02 ✓ |
| 0396 | xianding-batch5（逾制/长姬） | D (opencode) | 2026-08-02 | ✓
| 0397 | ABI 525 兵势张任（伏暗/引弦） | B (Codex overflow) | 2026-08-02 ✓ |
| 0398 | shiji-mb-shen-machao（神马超闭环，ABI 409 驭雳 / 410 霆威） | A (Claude) | 2026-08-02 ✓ |
| 0399 | xianding-batch6（沉勇/绞） | D (opencode) | 2026-08-02 | ✓
| 0406 | ABI 529 兵势桓阶（共谋/正朔） | B (Codex overflow) | 2026-08-02 ✓ |
| 0407 | ABI 532 兵势孙綝（逆固/戮连） | B (Codex overflow) | 2026-08-02 ✓ |
| 0401 | shenhua 祝融巨象/烈刃（ABI 530 过滤器批次 + ABI 531 resolve_lieren） | C (omp) | 2026-08-02 ✓ |
| 0402 | xianding-batch7（虞袭/复盏） | D (opencode) | 2026-08-02 | ✓
| 0403 | shiji-sp-kongrong（孔融闭环，ABI 411 新名士） | A (Claude) | 2026-08-02 ✓ |
| 0404 | xianding-batch8（点才/心忧） | D (opencode) | 2026-08-02 | ✓
| 0405 | shiji-wangfuzhaolei（王甫赵累闭环，ABI 412 殉义） | A (Claude) | 2026-08-02 ✓ |

| 0408 | xianding-batch9（返攻/新膂力） | D (opencode) | 2026-08-02 | ✓
| 0409 | shiji-sp-xujing（许靖闭环，ABI 413 博明 / 414 恶荐） | A (Claude) | 2026-08-02 ✓ |
| 0410 | xianding-batch10（敏善/受责） | D (opencode) | 2026-08-02 | ✓
| 0411 | shiji-sp-wangshuang（王双闭环，ABI 415 异勇 / 416 善械） | A (Claude) | 2026-08-02 ✓ |

| 0412 | shenhua 邓艾屯田/凿险（ABI 533 屯田全套：`owner_lost_cards_outside_turn` 过滤器 + `resolve_tuntian` + 负牌堆计数修正器 `owner_skill_pile_count_negative_<def>`；ABI 534 凿险：`owner_phase_begin_skill_pile_gte_three_<田def>` 过滤器 + `resolve_zaoxian`）+ 邓艾武将（含急袭 view_as） | C (omp) | 2026-08-02 ✓ |

| 0413 | ABI 535 兵势鲁肃【缔盟】（双目标手牌差约束、整手交换与交换后代价选择） | B (Codex overflow) | 2026-08-02 ✓ |
| 0414 | xianding-batch11（独仗/风势） | D (opencode) | 2026-08-02 | ✓
| 0415 | shiji-caizhenji（蔡贞姬闭环，ABI 417 舍医 / 418 天荫） | A (Claude) | 2026-08-02 ✓ |
| 0416 | ABI 536 兵势鲁肃【好施】（跨角色手牌授权、响应、可见性与两次空手补牌） | B (Codex overflow) | 2026-08-02 ✓ |
| 0419 | shiji-sp-zongyu（宗预闭环，复用 ABI 369 直辩 / 370 预言，零新 ABI） | A (Claude) | 2026-08-02 ✓ |

| 0417 | shenhua 姜维挑衅/志继（ABI 537 `resolve_tiaoxin`：目标限定「攻击范围含本角色且有牌」+ 目标二选一（对持有者使用杀或令持有者弃其一张牌），复用乱武受迫杀源与弃牌续体；ABI 538 `owner_phase_begin_hand_empty` 过滤器 + `resolve_zhiji(player, guanxing_def)`（摸2回1减上限并授观星））+ 姜维武将 | C (omp) | 2026-08-02 ✓ |

| 0418 | ABI 539 势董昭【妙略/迎驾】+衍生锦囊【瞒天过海】 | B (Codex overflow) | 2026-08-02 ✓ |

| 0420 | xianding-batch12（诛寇） | D (opencode) | 2026-08-02 | ✓

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次十二）：zhukou（诛寇，
周夷分支②，结束阶段本回合未造成伤害时选两名其他角色各受 1 点伤害，复用
owner_alive_phase_end_used_card_no_damage 过滤器近似“本回合未造成伤害”——旧 JS 无“使用过牌”
要求，未使用牌时漏触发，已注明；分支①摸已用牌数与周夷另一技 mengqing 觉醒均未建模）。

纯 DSL 批次（无需新 ABI，2026-08-02 D 段 opencode，xianding 限定包批次十三）：dcjinjie（金玠，
濒死时令其摸 2 张并可回复 1 点体力，每轮限一次复用 owner_alive_other_dying_once_per_round；
旧 JS 的 X∈[0,3] 整数声明以固定 X=2 近似，且“弃置 X 张牌”代价省略直接回复，已注明）。

| 0421 | shenhua 孟获祸首/再起/求首（ABI 540 `resolve_zaiqi(player, extra)` 摸牌替换翻顶揭示复合；ABI 541 修正器 `nanman_damage_source_as_owner`（南蛮伤害来源视为持有者）+ prepare_damage 重定向钩子；ABI 542 过滤器 `owner_alive_nanman_resolved_target_dead_or_damaged_gt_three` + `resolve_twqiushou(player)`（蜀/群各摸一））+ 孟获武将 | C (omp) | 2026-08-02 ✓ |

| 0422 | shiji-db-wenyang（标文鸯闭环，ABI 419 追锋 / 420 曲迪 + 重剑 view_as） | A (Claude) | 2026-08-03 ✓ |

| 0423 | ABI 543 兵势邓艾【屯田/凿险/急袭】 | B (Codex overflow) | 2026-08-02 ✓ |

| 0424 | ABI 544 兵势陈矫【清严/策断】 | B (Codex overflow) | 2026-08-02 ✓ |

| 0425 | shenhua 毌丘俭征荣/鸿举/清俭（ABI 545 过滤器 `owner_source_first_damage_card_target_hand_gte_owner` + `resolve_zhengrong`（选目标取一张手/装备入「荣」技能牌堆，vm event_targets 注入修复）；ABI 546 `resolve_hongju(player, qingjian_def, rong_def)`（荣≥3 觉醒：可选交换荣与等量手牌、减 1 点体力上限、授予清俭）；ABI 547 `resolve_qingjian(player, rong_def)`（出牌阶段限一次：弃一张手牌获得一张荣、弃一名其他角色装备/判定区一张牌））+ 毌丘俭武将 | C (omp) | 2026-08-02 ✓ |

| 0426 | xianding-batch14（武神/递仇） | D (opencode) | 2026-08-03 | ✓

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次十四）：dcwushen（武神，
红桃手牌当【杀】使用且无距离与次数限制，view_as sha heart 近似旧 JS 锁定技“红桃手牌自动视为
杀”，“红桃杀造成伤害后摸一”未建模）、dcdichou（递仇，观银屏：其他角色对你使用牌时其获得
1 枚“仇”标记（owner_is_target_other_source_alive + add_mark(event_source)），你对有“仇”目标的
角色使用牌时摸一并移去其 1 枚“仇”（owner_source_target_mark_positive_ + remove_mark(event_player)）；
旧 JS 的废除装备栏子效果②与回合结束火伤④未建模）。

| 0428 | shiji-sp-xinpi（辛毗闭环，ABI 421 隐据 / 422 持节） | A (Claude) | 2026-08-03 ✓ |

| 0429 | shiji-sp-jiangqing（江庆闭环，ABI 423 上议 / 424 简议） | A (Claude) | 2026-08-03 ✓ |

| 0433 | ABI 548 兵势太史慈【酣战/战烈/振锋】 | B (Codex overflow) | 2026-08-02 ✓ |

| 0431 | shenhua 张角雷击/鬼道/黄天（ABI 549 过滤器 `owner_responded_shan` + `resolve_releiji(player, target)`（目标判定：黑桃 2 雷伤 / 梅花回1+1雷伤，自定义判定改判征询）；ABI 550 修正器 `judgment_rewrite_black_hand_equip`（鬼道：黑色手/装备牌改判）；ABI 551 过滤器 `owner_alive_other_qun_play_phase_can_give_owner` + `resolve_huangtian(player, actor)`（群势力角色出牌阶段限一次可交一张闪/闪电））+ 张角武将 | C (omp) | 2026-08-03 ✓ |

| 0434 | xianding-batch16（聚土/复学） | D (opencode) | 2026-08-03 | ✓

| 0435 | shenhua 于吉蛊惑/禅缘（ABI 552 修正器 `skills_disabled_when_hp_one`（禅缘：持有者体力值为 1 时其非锁定技能失效，skill_effectively_enabled 钩子）+ 蛊惑（新蛊惑：纯 DSL view_as cost guhuo 声明任意基本/普通锦囊，复用引擎蛊惑质疑流程））+ 于吉武将 | C (omp) | 2026-08-03 ✓ |

| 0436 | ABI 553 兵势钟会【俟资/挟志/狱难/克昌】 | B (Codex overflow) | 2026-08-02 ✓ |

| 0437 | ABI 554 兵势辛宪英【诫节/清识】 | B (Codex overflow) | 2026-08-03 ✓ |

| 0438 | shenhua 鲁肃好施/缔盟（ABI 555 `resolve_haoshi(player, stage)`（摸牌阶段额外摸两张；摸牌阶段结束手牌>5 时选手牌最少角色给一半手牌）；ABI 556 `resolve_dimeng(player, targets, stage)`（出牌阶段限一次：选两名其他角色，弃置 X 张牌（X=两者手牌差）后交换手牌））+ 鲁肃武将 | C (omp) | 2026-08-03 ✓ |

| 0439 | xianding-batch17（梦魇） | D (opencode) | 2026-08-03 | ✓
| 0440 | xianding-batch18（忿肆/典财） | D (opencode) | 2026-08-03 | ✓
| 0442 | xianding-batch19（立刃/讽言） | D (opencode) | 2026-08-03 | ✓
| 0443 | xianding-batch20（酒盾/全守） | D (opencode) | 2026-08-03 | ✓
| 0456 | xianding-batch21（守户/烈骧） | D (opencode) | 2026-08-03 | ✓
| 0457 | xianding-batch22（下策） | D (opencode) | 2026-08-03 | ✓
| 0494 | xianding-batch23（清棱/接缨） | D (opencode) | 2026-08-03 | ✓
| 0533 | xianding-batch25（义指） | D (opencode) | 2026-08-03 | ✓
| 0543 | xianding-batch26（溃击/膂力·闪/膂力·无懈） | D (opencode) | 2026-08-03 | ✓
| 0544 | xianding-batch27（施仇/壮魄/剑出） | D (opencode) | 2026-08-03 | ✓
| 0547 | xianding-batch28（恩遇） | D (opencode) | 2026-08-03 | ✓
| 0551 | xianding-batch29（清识/残肆） | D (opencode) | 2026-08-03 | ✓
| 0552 | xianding-batch30（醮影） | D (opencode) | 2026-08-03 | ✓
| 0555 | xianding-batch31（追还·二/沙舞） | D (opencode) | 2026-08-03 | ✓
| 0556 | xianding-batch32（防毒/赏誉） | D (opencode) | 2026-08-03 | ✓
| 0560 | xianding-batch33（识锦/斗威） | D (opencode) | 2026-08-03 | ✓
| 0561 | xianding-batch34（追猎/夺月） | D (opencode) | 2026-08-03 | ✓
| 0563 | xianding-batch35（弹翼） | D (opencode) | 2026-08-03 | ✓
| 0567 | xianding-batch36（华衣） | D (opencode) | 2026-08-03 | ✓
| 0568 | xianding-batch37（毒露/猝袭） | D (opencode) | 2026-08-03 | ✓
| 0569 | xianding-batch38（孔武） | D (opencode) | 2026-08-03 | ✓
| 0571 | xianding-batch39（困欲/矜节） | D (opencode) | 2026-08-03 | ✓
| 0572 | xianding-batch40（势援） | D (opencode) | 2026-08-03 | ✓
| 0580 | xianding-batch41（天匠） | D (opencode) | 2026-08-03 | ✓
| 0582 | xianding-batch42（夺锐） | D (opencode) | 2026-08-03 | ✓
| 0583 | xianding-batch43（奇绝/毒逝·诅咒） | D (opencode) | 2026-08-03 | ✓
| 0593 | xianding-batch51（dcpeiniang 佩娘） | D (opencode) | 2026-08-04 | ✓
| 0592 | xianding-batch50（onedcspshichou 施仇 OL版） | D (opencode) | 2026-08-04 | ✓
| 0591 | xianding-batch49（dcpijing 僻境） | D (opencode) | 2026-08-04 | ✓
| 0590 | xianding-batch48（dcsbxunshi 迅势 / dcjujian 据谏） | D (opencode) | 2026-08-04 | ✓
| 0589 | xianding-batch47（xunshi 巡使 / dcsbwankang 王抗 / old_dcrenshuang 纫霜旧版） | D (opencode) | 2026-08-04 | ✓
| 0590 | xianding characters.json 引用修复（batch45 遗留，1 武将）：v_lvbu（吕布变体）第一技能 2203577701→dcxiaowu=4153740280（骁武），重打包并同步 extensions | E (Hermes) | 2026-08-04 ✓ |
| 0588 | xianding-batch46（dushi 毒逝） | D (opencode) | 2026-08-04 | ✓
| 0587 | xianding-batch45（dcxiaowu 笑武 / dcruxian 儒贤 / dcsbsushen 苏慎） | D (opencode) | 2026-08-04 | ✓
| 0586 | xianding-batch44（oldjue 决·旧版 / dczhangcai 张采） | D (opencode) | 2026-08-04 | ✓
| 0587 | xianding characters.json 引用修复（batch43 遗留，1 武将）：dc_sb_yangfeng（界杨奉）第三技能 3506229050→dcsbqijue=2527119333（奇绝），重打包并同步 extensions | E (Hermes) | 2026-08-04 ✓ |
| 0584 | xianding characters.json 引用修复（batch41/42 遗留，2 武将）：dc_shen_zhangliao（神张辽）dcduorui 引用 3677114600→2823548533（opencode 重写 id）；puyuan（蒲元）pytianjiang 引用 3088049727→2167112704（天匠），重打包并同步 extensions | E (Hermes) | 2026-08-04 | ✓
| 0573 | xianding characters.json 引用修复（batch37/38 遗留，1 武将）：shen_pangtong（神庞统）第三技能 2478060157→kunyu=3623529353（困愚），重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0564 | ABI 632 3D精选刘巴【丰政/玉律】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0565 | ABI 633 3D精选蹇硕【分野/逝潮】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0566 | xianding characters.json 引用修复（batch35 遗留，1 武将）：xizheng（希正）第一技能 3109120588→dcdanyi=2264670487（弹翼），重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0571 | xianding characters.json 引用修复（batch36 遗留，4 武将）：dc_shen_zhangliao（神张辽）3767673053→dccuxi=4093257724；liufuren（刘夫人）3688714569→dcdulu=2779276477；dc_sb_hucheer（界虎车儿）3316374594→dcsbkongwu=3614891392；duanqiaoxiao（段巧笑）3042801245→dchuayi=2287394739，重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0570 | ABI 634 3D精选管宁【游学/承经】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0574 | ABI 635 3D精选曹爽【专摄/危秋】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0576 | ABI 636 3D精选薛灵芸【涟洏/暗织】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0577 | ABI 637 3D精选刘宏【失兴/党锢/覆宗】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-04 ✓ |
| 0578 | ABI 638 3D精选夏侯玄【朗怀/玄论】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-04 ✓ |
| 0579 | ABI 639 3D精选张闿【劫行/拜泪】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-04 ✓ |
| 0580 | ABI 640 3D精选梁习【仝御】完整语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-04 ✓ |
| 0575 | xianding characters.json 引用修复（batch40 遗留，1 武将）：liubian（刘辩）第一技能 3653071969→shiyuan=4115301600（势援），重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0441 | ABI 558 兵势魏延【忠傲/壮誓/饮战/狂骨】 | B (Codex overflow) | 2026-08-03 ✓ |

| 0444 | ABI 428 压军 / 429 尊帝 + sp_cuiyan（崔琰闭环，shiji） | A (Claude) | 2026-08-03 |
| 0445 | shenhua 张郃巧变（ABI 559 `resolve_qiaobian(player, phase)`：四阶段开始时弃一张手牌并跳过该阶段；摸牌阶段可选获至多两名其他角色各一张手牌，出牌阶段可选移动场上牌到另一角色同类别区域）+ 张郃武将 | C (omp) | 2026-08-03 ✓ |
| 0446 | shenhua 刘禅（ABI 560 `exclude_saved_sha_target(card, target)`（card_targeted 时移除当前杀保存目标）+ `resolve_xiangle`（享乐：使用者弃基本牌或杀无效）+ `resolve_fangquan_skip/grant`（放权：跳出牌阶段、结束阶段弃牌授额外回合）+ `resolve_ruoyu`（若愚：主公最低体力觉醒加上限/回复/授激将））+ 刘禅武将 | C (omp) | 2026-08-03 ✓ |
| 0453 | shenhua 诸葛瞻（ABI 563 `resolve_zuilun`（罪论：结束阶段揭示牌堆顶三张按条件计数分派/一起失体力）+ `resolve_fuyin`（父荫：其他角色以不小于你手牌数的杀指定你且本回合首次时该杀无效））+ 诸葛瞻武将 | C (omp) | 2026-08-03 ✓ |
| 0454 | shenhua 严颜（ABI 571 `ban_use_against(player, target)`（回合禁用效果）+ `card_target_cancel_owner_turn_banned` 修正器 + `resolve_juzhan`（拒战转换技：受杀各摸一张并禁其对你用牌 / 用杀获其一张牌并禁你对其用牌））+ 严颜武将 | C (omp) | 2026-08-03 ✓ |
| 0465 | shenhua 王基（ABI 573 `resolve_qizhi`（奇制：用牌后弃非目标角色一张牌并令其摸一张，计数）+ `resolve_jinqu`（进趋：出牌阶段结束摸二并弃至奇制次数））+ 王基武将 | C (omp) | 2026-08-03 ✓ |
| 0466 | shenhua 蒯良蒯越（ABI 574 `resolve_jianxiang`（荐相：受指目标后选手牌最少者摸一张）+ `resolve_shenshi`（审时转换技：阳交牌伤害摸至四 / 阴观手交牌回合末追踪摸至四））+ 蒯良蒯越武将 | C (omp) | 2026-08-03 ✓ |
| 0471 | shenhua 卢植（ABI 576 `resolve_mingren`（明任：首轮准备阶段摸二置一张入「任」牌堆；结束阶段替换）+ `resolve_zhenliang`（贞良转换技：阴弃同色牌对攻击范围内角色伤害 / 阳回合外同色牌令一名角色摸一张））+ 卢植武将 | C (omp) | 2026-08-03 ✓ |
| 0472 | shenhua 孙亮（ABI 577 `resolve_kuizhu`（溃诛：弃牌阶段后按弃牌数选择摸牌或伤害）+ `resolve_zhizheng`（掣政：出牌阶段结束弃置不包含你的角色的一张牌）+ `resolve_lijun`（立军：吴势力角色用杀后你获牌并令其摸一张））+ 孙亮武将 | C (omp) | 2026-08-03 ✓ |
| 0473 | ABI 578 XIANDING 糜夫人【闺秀/存嗣】补全 + `dc_mifuren` 完整武将 | B (Codex overflow) | 2026-08-03 | ✓
| 0474 | shenhua 周妃（ABI 579 `resolve_liangyin`（良姻：失牌/获牌后双分支摸牌回复/弃牌）+ `resolve_kongsheng`（箜声：准备阶段置箜、结束阶段回手/令角色使用装备））+ 周妃武将 | C (omp) | 2026-08-03 ✓ |
| 0475 | ABI 580 XIANDING 新杀神关羽【武神/武魂】补全 + `dc_shen_guanyu` 完整武将 | B (Codex overflow) | 2026-08-03 | ✓
| 0476 | ABI 581 XIANDING 杨彪【昭汉】摸牌后交牌/弃牌精确闭环 | B (Codex overflow) | 2026-08-03 | ✓
| 0477 | ABI 582 XIANDING 杨彪【尽节】声明摸牌、可选等量弃牌回复精确闭环 | B (Codex overflow) | 2026-08-03 | ✓
| 0478 | ABI 583 XIANDING 杨彪【举讹】弃牌历史、随机虚拟牌循环精确闭环 + `dc_yangbiao` 完整武将 | B (Codex overflow) | 2026-08-03 | ✓
| 0479 | ABI 584 XIANDING 阮籍【昭文/酒遁】精确闭环 + `ruanji` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0480 | ABI 585 XIANDING 崔琰毛玠【征辟/奉迎】精确闭环 + `cuimao` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0481 | ABI 586 XIANDING 傅干【巧谏/析察】精确闭环 + `fugan` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0482 | ABI 587 XIANDING 胡遵【斩涛/安境】精确闭环 + `huzun` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0483 | ABI 588 XIANDING 诸葛梦雪【寄春/寒英】精确闭环 + `zhugemengxue` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0458 | ABI 564 兵势吕壹【隳律/颂谳/势施】 | B (Codex overflow) | 2026-08-03 ✓ |
| 0459 | ABI 568 兵势曹爽【典易/奢权/渐专/溺尊】 | B (Codex overflow) | 2026-08-03 ✓ |

| 0449 | ABI 561 兵势周瑜【炽沄/焰洄/焚涛/雄姿】 | B (Codex overflow) | 2026-08-03 ✓ |
| 0450 | ABI 430 佯解 / 431 清雨 + sp_mifuren（糜夫人闭环，shiji） | A (Claude) | 2026-08-03 |
| 0451 | ABI 432 整肃系统 / 433 侯封燕集 / 434 点才 + sp_zhujun、sp_lvfan（朱儁/吕范闭环，shiji） | A (Claude) | 2026-08-03 |

| 0455 | ABI 435 战海 + zhouchu（周处闭环，shiji；彰名纯 DSL） | A (Claude) | 2026-08-03 |

| 0456 | ABI 436 凶竖 + yingbian 包首技能（贾充凶竖入包，武将待奸回） | A (Claude) | 2026-08-03 |

| 0457 | ABI 437 奸回 + yingbian 贾充（jin_jiachong）武将发布 | A (Claude) | 2026-08-03 |

| 0458 | ABI 439 弃哀（王粲首技能，shiji） | A (Claude) | 2026-08-03 |

| 0462 | ABI 440 擅袭 / 441 索命 + sp_wangcan（王粲闭环，shiji） | A (Claude) | 2026-08-03 |

| 0463 | ABI 442 通夺（刘巴首技能，shiji；断笔待多目标逐弃+弃牌堆随机获牌原语） | A (Claude) | 2026-08-03 |
| 0464 | ABI 443 怨清 / 444 疏陈 + sp_huaxin（华歆闭环，shiji） | A (Claude) | 2026-08-03 |
| 0465 | ABI 445 断笔 + liuba（刘巴闭环，shiji） | A (Claude) | 2026-08-03 |
| 0466 | ABI 446 躁厉 + sunyi（孙翊闭环，shiji） | A (Claude) | 2026-08-03 |
| 0467 | ABI 447 孤影（向宠首技能，shiji；武将待睦阵） | A (Claude) | 2026-08-03 |
| 0467 | ABI 570 兵势于吉【道转/符济】 | B (Codex overflow) | 2026-08-03 ✓ |
| 0468 | ABI 572 SP【横江】+ HUICUI `zangba` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0469 | NEWJIANG `sunli` / `kangli` 已重写闭包重新打包投放 | B (Codex overflow) | 2026-08-03 ✓ |
| 0470 | ABI 575 SP【豹变】+ XIANDING `dc_xiahouba` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |

| 0488 | ABI 593 XIANDING 柏灵筠【灵慧/黠策/御心】精确闭环 + `bailingyun` 完整武将 | B (Codex overflow) | 2026-08-03 ✓ |
| 0489 | ABI 594 XIANDING 威关银屏【绍威/涤仇】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0490 | ABI 595 XIANDING 凌操【独锋】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0491 | ABI 596 XIANDING 新杀田丰【随势】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0492 | ABI 597 XIANDING 陈泰【矜贤】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |

| 0501 | ABI 598 XIANDING 卞玥【庇族/无胁】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |

下一个可领取：0629（0628 已由 A 段 Claude 领取并 ✓（gw_dudayuanshuai2 禁弃置门禁结构性评估 + 弃牌域收窄）；0627 已由 A 段 Claude 领取并 ✓（gwent 叶舞造卡入口 ABI 645）；0626 已由 A 段 Claude 领取并 ✓（gwent 运行时 VM 弹栈/def 读取修复）；0625 已由 A 段 Claude 领取并 ✓（国战 zhibi 元数据登记，12 锦囊盘点闭环）；0624 已由 A 段 Claude 领取并 ✓（国战敕令结算）；0623 已由 A 段 Claude 领取并 ✓（国战号令天下结算）；0622 已由 A 段 Claude 领取并 ✓（国战文和乱武结算）；0621 已由 A 段 Claude 领取并 ✓；0620 已由 A 段 Claude 领取并 ✓；0619 已由 A 段 Claude 领取并 ✓；0618 已由 A 段 Claude 领取并 ✓；0617 已由 A 段 Claude 领取并 ✓；0616 已由 A 段 Claude 领取并 ✓；0615 已由 A 段 Claude 领取并 ✓；0614 已由 A 段 Claude 领取并 ✓；0613 已由 A 段 Claude 领取并 ✓；0612 已由 A 段 Claude 领取并 ✓；0611 已由 C 段 omp 领取并 ✓；0610 已由 C 段 omp 领取并 ✓；0609 已由 C 段 omp 领取并 ✓；0608 已由 A 段 Claude 领取并 ✓；0607 已由 A 段 Claude 领取并 ✓；0606 fork jiange 已由 C 段 omp 领取并 ✓；0594/0595/0596/0597/0598/0599/0600/0601/0602/0603/0604/0605 已由 A 段 Claude 架构整理领取并 ✓；0582 全包角色索引补齐已由 C 段 omp 领取并 ✓；0530-0532 已由 C 段 omp 领取并 ✓；3D精选 ddd 已由 B 段 Codex overflow 批次一~二十三 + C 段 omp 收尾批次 0581 全部闭环 31/31；0540/0542/0546/0550/0554/0558/0561/0562/0566/0571/0573/0575/0584/0587/0590 已由 E 段 Hermes 领取 xianding/diy 技能引用修复；0535 曾由 Hermes 误领 yxs 后撤销释放回池）。

2026-09-24 补记：0630–0777 的原生线 ADR 未在本表登记，现有编号以 `docs/adr/` 与 [ADR 索引](docs/adr/INDEX.md) 为准；0778（原生扩展包数据键表与自描述装载，C0 续）由审查修复会话领取并 ✓。下一个可领取：0779。

### Claude A 段领取（2026-08-03 第五轮：停滞包收尾 + 内容结构技能补齐）

全包盘点确认剩余缺口分类：xianding(缺400，opencode 11 分钟前活跃不接管)；
shenhua(缺54)/yingbian(缺23)/bingshi(缺3)/extra(缺10) 均已 ≥2 小时无文件写入，
由 A (Claude) 接管补齐；standard(缺6)/diy(缺1)/sxrm(缺1)/tw(缺1) 为内容结构
技能（*1/*2 派生、系统技能、跨包辅助），一并补齐。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0519 | shenhua/yingbian/bingshi/extra 停滞包收尾 + standard/diy/sxrm/tw 内容结构技能补齐（全部闭环：shenhua 176/176、yingbian 92/92、bingshi 80/80、extra 63/63、standard 70/70、diy 144/144、sxrm 51/51、tw 351/351） | A (Claude) | 2026-08-03 ✓ |
| 0520 | xianding 包剩余技能接管（496/496 技能 + 196 武将闭环；504 定义含辅助） | A (Claude) | 2026-08-03 ✓ |
| 0521 | ABI 608 归档扩展“玩点论杀”首批：费诗【率言/墨守】、武将与立绘独立 npack 闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0522 | ABI 609 归档扩展“玩点论杀”韩龙【伺机/刺酋】、武将与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0523 | ABI 610 归档扩展“玩点论杀”刘馥【筑城/夺气】、武将与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0524 | ABI 611 归档扩展“玩点论杀”刘焉【绝道/割据】、武将与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0525 | ABI 612 归档扩展“玩点论杀”留赞【亢音】、武将与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0526 | ABI 613 归档扩展“玩点论杀”田豫【车阵/诱战】、【诱敌深入】卡牌与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0527 | ABI 614 归档扩展“玩点论杀”习珍习宏【负仇/噤言】、武将与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0528 | ABI 615 归档扩展“玩点论杀”杨仪【筹度/立断】、【调兵遣将】卡牌与立绘增量闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0529 | ABI 616 归档扩展“玩点论杀”枣祗任峻【粮策/坚壁/军屯】、【望梅止渴】卡牌、语音与整包闭环 | B (Codex overflow) | 2026-08-03 ✓ |

| 0518 | ABI 607 XIANDING 马伶俐【硝引】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0517 | ABI 606 XIANDING 曹轶【蜜饴/寅君】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0516 | ABI 605 XIANDING 诸葛若雪【琼英/暖惠】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0515 | ABI 604 XIANDING 张臶【技能A/技能B】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0514 | ABI 603 XIANDING 新杀谋鲁肃【明势/盟谋】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0513 | ABI 602 XIANDING 新杀谋周瑜【融火/英谋】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0512 | diy 包武将全量（54 将 DSL 重写与 npack 闭环；可表达技能真实运行时，其余元数据登记 + 文档化近似） | C (omp) | 2026-08-03 ✓ |
| 0504 | ABI 601 XIANDING 势张绣【附袭/豪义】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0503 | ABI 600 XIANDING 朱佩兰【辞虑/痛悼】精确闭环 | B (Codex overflow) | 2026-08-03 ✓ |
| 0502 | sb（界限突破）包武将全量（54 将 DSL 重写与 npack 闭环；制衡/闭月真实运行时，其余界武将差异技能元数据登记 + 文档化近似） | C (omp) | 2026-08-03 ✓ |
| 0500 | yingbian 包武将全量（23 将 DSL 重写与 npack 闭环；才媛/崇信/新婉仪真实运行时，其余元数据登记 + 文档化近似） | C (omp) | 2026-08-03 ✓ |
| 0493 | sxrm-batch1（sxrm 包武将全量：曹仁/曹丕/曹操/伏皇后/关银屏/关羽/华佗/贾诩/蒋干/刘封/陆逊/陆抗/庞德/孙尚香/孙休/太史慈/王平/许褚/张春华/张辽/钟会/左慈等 25 将的 DSL 重写与 npack 闭环；无需新 ABI） | C (omp) | 2026-08-03 ✓ |

### Claude A 段领取（2026-08-03 第三轮：Hermes 遗留包接管）

Hermes E 段会话已结束；standard/extra 已收尾，newjiang/clan/onlyOL/key/yijiang 当前无活跃
认领者，由 A (Claude) 接管，按「旧 JS 语义参考 → DSL 重写 → npack → extensions」流程推进。
策略：优先纯 DSL 复用既有 filter/effect/modifier；确需新原语时按规则先在本表领取
（A 段剩 449，E 段剩 598/599），再改 engine-api 并更新 ABI-MAX。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0495 | newjiang-batch3（接管 Hermes 批次二之后的剩余技能；102/102 技能 + 43/43 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0496 | clan-batch2（接管 Hermes 批次一之后的剩余技能；82/82 技能 + 37/37 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0497 | key-batch2（接管 Hermes 批次一之后的剩余技能；259/259 技能 + 84 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0498 | yijiang-batch8（接管 Hermes 批次七之后的剩余技能；285/285 技能 + 71 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0499 | onlyOL-batch3（接管 Hermes 批次二之后的剩余技能；176/176 技能 + 77/80 武将闭环，junk_zhangjiao/shen_sunquan 依赖 diy/xianding 跨包技能待其包移植） | A (Claude) | 2026-08-03 ✓ |

### Claude A 段领取（2026-08-03 第四轮：停滞包接管 + shiji 收尾）

全包盘点（npack 技能覆盖 vs 上游）确认：clan/key/newjiang/old/onlyOL/sb/yijiang 已闭环；
**shiji（A 段自己负责）仍缺 85 技能**，由本会话补齐。其余停滞包（≥8 小时无文件写入、
无整包完成声明）：offline(缺 661)、mobile(缺 335)、tw(缺 270)、sp2(缺 219)、
huicui(缺 215)、refresh(缺 199)、jsrg(缺 174)、collab(缺 104)、diy(缺 104)、
sixiang(缺 105)、sp(缺 510)、xianding(缺 412，opencode 活跃中暂不接管) 等，
按用户目标「把其他未完成的包也处理」由 A (Claude) 依序接管。活跃包（xianding/
shenhua/bingshi/sxrm/sb/yingbian，其他 agent 近期写入中）不冲突。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0505 | shiji 收尾（补齐 85 个缺失技能 + 48/48 武将闭环；164/164 技能） | A (Claude) | 2026-08-03 ✓ |
| 0506 | offline 包剩余技能接管（748/749 技能 + 339 武将闭环，_taoyuanwange 补齐） | A (Claude) | 2026-08-03 ✓ |
| 0507 | refresh 包剩余技能接管（312/312 技能 + 113 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0508 | mobile 包剩余技能接管（428/428 技能 + 177 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0509 | tw 包剩余技能接管（351/351 技能 + 146 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0510 | jsrg 包剩余技能接管（181/181 技能 + 76 武将闭环） | A (Claude) | 2026-08-03 ✓ |
| 0511 | sp2/huicui/collab/sixiang/diy/sp 剩余技能接管（全部闭环：sp2 281/281、huicui 249/249、collab 136/136、sixiang 174/174、diy 143/144、sp 639/639） | A (Claude) | 2026-08-03 ✓ |

### C 段（omp）领取（2026-08-03 第二轮）

| 编号 | 主题 |
|---|---|
| 0484 | shenhua 许攸（ABI 589 `resolve_chenglve`（成略转换技：翻面摸 1/2 弃 2/1，本回合弃置花色牌无距离/次数限制）+ `resolve_shicai`（恃才：使用后置弃牌堆牌于牌堆顶并摸一张，装备先于同类牌时）+ `resolve_cunmu`（寸目：主公技摸牌改为从牌堆底摸））+ 许攸武将 | C (omp) | 2026-08-03 ✓ |
| 0485 | shenhua 张绣（ABI 590 `resolve_xiongluan`（雄乱限定技：废除装备栏与判定区，回合 flag 限定目标获得距离/次数豁免并临时禁其手牌）+ `resolve_twjuxiang`（踞襄主公技近似：其他群势力角色出牌阶段可将装备置入主公装备区/恢复栏位，回合 flag 限一次）；从谏复用既有 drlt_congjian（ABI 524））+ 张绣武将 | C (omp) | 2026-08-03 ✓ |
| 0486 | shenhua 旧周泰（ABI 591 `resolve_gzbuqu`（旧不屈：体力降至 0 以下时翻牌入「创」技能牌堆，点数重复则进入濒死，不重复则本回合免濒死；恢复后每点回复移去一张「创」））+ 旧周泰武将 | C (omp) | 2026-08-03 ✓ |
| 0487 | shenhua 左慈（化身/新生为元技能：动态复制其他武将技能与性别/势力，DSL/引擎不支持，登记元数据并文档化近似，无运行时行为；无需新 ABI）+ 左慈武将 | C (omp) | 2026-08-03 ✓ |


### C 段（omp）领取（2026-08-03 第三轮：归档内置扩展迁移）

26 角色包 + 玩点论杀已闭环后，上游 `archive/upstream/noname/apps/core/extension/` 尚有
英雄杀/欢乐卡牌/杀海拾遗/3D精选四个含技能、角色或卡牌的内容扩展，按
「旧 JS 语义参考 → DSL 重写 → npack → extensions」流程推进；其中 3D精选并非单纯皮肤层，
还定义 31 名 `ddd_*` 武将及其技能。boss/cardpile/coin 属模式、牌堆配置或本地 UI 功能，
不伪装成规则 npack，另按各自运行时边界处理并登记。优先纯 DSL 复用既有
filter/effect/modifier；确需新原语时先在本表
领取（C 段 450-469 已耗尽，按溢出段 617 起逐号串行领取）再改 engine-api 并更新
ABI-MAX。本批包 ID：`yxs`（英雄杀）、`huanle`（欢乐卡牌）、`shahai`（杀海拾遗）。
全仓剩余非可迁移项的完整登记见 `docs/migration/upstream-noname-inventory.md`
（boss/cardpile/coin 模式与 UI JS、guozhan/huodong/xianxia/yongjian/zhulu 模式
卡组、perfectPairs/rank/replace 客户端元数据、mode 模式脚本与表现层资源）。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0530 | 英雄杀包（yxs）：43 武将 / 113 技能 DSL 重写与 npack 闭环；可表达技能真实运行时，其余元数据登记 + 文档化近似 | C (omp) | 2026-08-03 ✓ | 已落地：105 技能定义（20 真实运行时 + 85 元数据登记）、43 武将入角色索引、standard/shenhua 跨包依赖；content-compiler 514/514、conformance 8/8、28 包空 store 冷导入通过 |
| 0531 | 欢乐卡牌包（huanle）：6 卡 / 5 技能 DSL 重写与 npack 闭环 | C (omp) | 2026-08-03 ✓ | 已落地：5 装备卡（monkey/jiuwei/xuelunyang/mianju/kuwu）+ 2 真实技能（mianju 防翻面、jiuwei 摸/回二选一）+ 3 元数据登记；自定义基本牌手里剑登记为不可迁移；content-compiler 514/514、conformance 8/8、30 包空 store 冷导入通过 |
| 0532 | 杀海拾遗包（shahai）：gujian/gwent/hearth/mtg/ow/swd/xianjian/yunchou 8 系 332 武将 / ~1091 技能 / 201 卡 DSL 重写与 npack 闭环（分批） | C (omp) | 2026-08-03 ✓ | 已落地：332 武将、1092 技能定义（39 真实运行时 + 1053 元数据登记）、41 装备卡、7 包跨包依赖（standard/shenhua/extra/yijiang/mobile/refresh/sp）；content-compiler 515/515、conformance 8/8、31 包空 store 冷导入通过 |
| 0534 | 3D精选包（ddd）批次一：韩当【险袭】语义重写、Golden 与首个可安装 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0535 | ~~英雄杀包（yxs）接管~~（已撤销：误判 omp 停滞。实为 omp 16:52 已交付 yxs 105 技能/43 将并打 ✓（见 0530）；Hermes 16:40 盘点时 omp 尚未写入，17:17 误建 43 个重名 .skill 已全部删除，被覆盖的 characters/metadata 已从 npack 反推恢复，extensions/yxs md5 一致无损。0535 释放回池） | E (Hermes) | 2026-08-03 |
| 0536 | 3D精选包（ddd）批次二：周处【侠横】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0537 | 3D精选包（ddd）批次三：刘宠【精彀/末焱】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 | ✓

### C 段（omp）领取（2026-08-04 第一轮：ddd 剩余批次接管）

用户明确指示「把 codex 认领的做了」；ddd（3D精选）剩余 7 名武将
（ddd_kebineng/ddd_lie/ddd_qianzhao/ddd_sunliang/ddd_wangkanglvkai/ddd_zhangcheng/
ddd_zhangmiao）由 C 段（omp）接管，按「旧 JS 语义参考 → DSL 重写 → npack →
extensions」流程推进。可表达技能以真实 DSL 实现；需引擎复合原语的按既有
`resolve_ddd_*` 模式新增原语并先在本表领取（ABI 641 起逐号串行领取）再改
engine-api 并更新 ABI-MAX；其余登记元数据 + 文档化近似。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0581 | 3D精选包（ddd）收尾批次：剩余 7 名武将（轲比能/列/牵招/孙亮/王伉吕凯/张承/张邈）DSL 重写与 npack 闭环 | C (omp) | 2026-08-04 ✓ | 已落地：31/31 武将入角色索引、66 技能定义（ddd_xiaoxing 真实修正器运行时，其余 13 项元数据登记 + 文档化近似）、31 立绘随包；content-compiler 531/531、conformance 8/8、31 包空 store 冷导入通过 |

### C 段（omp）领取（2026-08-04 第二轮：全包角色索引补齐）

全包盘点（上游 character.js vs 各 npack character_index）发现 14 包共 133 名上游武将
未入角色索引；其引用的 271 项技能已全部存在于已迁移 npack（零未解析），本次为纯
角色登记补齐：按上游 sex/group/hp/skills 生成条目，技能按名解析到既有 def id，
跨包引用以依赖补全；同名歧义优先本包/既有依赖。不新增技能定义、不改引擎。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0582 | 全包角色索引补齐：14 包 133 名缺失武将登记（collab 1/huicui 2/key 3/mobile 24/offline 13/old 36/onlyOL 3/refresh 15/sp 4/sp2 9/standard 3/tw 5/xianding 14/yijiang 1） | C (omp) | 2026-08-04 ✓ | 已落地：13 包重打包角色索引与上游全量对齐（2,897 唯一武将、零重复 ID），技能索引逐包验证与既有 npack 完全一致（仅 character_index 增量）；standard 3 名（甘夫人/公孙瓒/旧界李典）因引擎内嵌 standard 自包含约束（技能位于 sp/refresh）回退并文档化；content-compiler 531/531、conformance 8/8、31 包空 store 冷导入通过 |
| 0538 | 3D精选包（ddd）批次四：罗宪【示烈】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0539 | 3D精选包（ddd）批次五：丁奉【短兵】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0540 | xianding characters.json 技能引用修复（5 个错误 id）：dc_shen_liubei/dc_sb_liuzhang/dongxu/zhangjian 引用不存在的技能 id（3029944376/3234837342/2872120335/2238296709/2540253792），修正为 .skill 实际定义 id（dcjieying/dcsbyizhi/dcqingleng/dc_zj_a/dc_zj_b），重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0541 | 3D精选包（ddd）批次六：吴质【灵涌/虚孝】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0542 | diy characters.json 技能引用修复（61 角色 / 84 失效 id）：按上游技能名 → .skill 定义 id 重建（跨包引用目标包既有 id），dualside 双面标记剔除，deps 扩充至 8 包，重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0545 | 3D精选包（ddd）批次七：许靖【虚誉/实荐】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0546 | xianding characters.json 引用修复（batch27 遗留，3 武将）：shie/shen_xuzhu/dc_sp_machao 第二个技能引用失效 id（3355579768/3222350667/2499124614），修正为 .skill 实际定义（sejianchu/zhuangpo/twodcspshichou），重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0547 | 3D精选包（ddd）批次八：曹髦【太思/驱车/潜龙】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0548 | 3D精选包（ddd）批次九：辛宪英【洞察/智解】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0549 | ⚠️ §37.10 待清理记录（Hermes）：工作区 target/ 被 Codex/opencode 并发编译占用（19:37 cargo test original_xianding_batch28 + cargo check 用工作区 target），按 §37.10 暂缓清理并记录；编译空闲后立即删除工作区 target/（外部 CARGO_TARGET_DIR 保留给 Codex） | E (Hermes) | 2026-08-03 ✓ |
| 0550 | 3D精选包（ddd）批次十：向朗【恰化/恂恂/腹笥/拓籍】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0550 | xianding characters.json 引用修复（batch28 遗留，1 武将）：chengbing（程秉）第二个技能引用失效 id 3359025792，修正为 .skill 实际定义 dcenyu=3753328944（恩遇），重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0552 | 3D精选包（ddd）批次十一：于禁【整军】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0554 | xianding characters.json 引用修复（batch29 遗留，2 武将）：dc_duyu（杜预）第二技能 2617108563→dcdyqingshi=4110806243；zerong（泽荣）第一技能 2171233362→dccansi=3144382148，重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0553 | 3D精选包（ddd）批次十二：刘晔【察势/齐策】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0556 | 3D精选包（ddd）批次十三：鲍三娘【枝连/赍剑】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0557 | 3D精选包（ddd）批次十四：甄宓【淼形/浮世】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0559 | 3D精选包（ddd）批次十五：赵昂【奋计】语义重写、Golden 与累积 npack | B (Codex overflow) | 2026-08-03 ✓ |
| 0561 | xianding characters.json 引用修复（batch33 遗留，2 武将）：dc_sb_dengai（界邓艾）第二技能 3927470045→dcsbshijin=3506294264；dc_sb_dongcheng（董成）第二技能 2918478934→dcsbdouwei=2217274823，重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0562 | xianding characters.json 引用修复（batch34 遗留，2 武将）：v_caocao（曹操变体）第一技能 2513963246→dcduoyue=3729672764；wangshuang（王双）技能 2196883215→spzhuilie=3774269941，重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |
| 0779 | N/C0 名称符号表总量上限与拒装策略（`SYMBOL_TABLE_LIMIT=1e6`，装载器三入口超限拒装，不回收） | A (ZCode) | 2026-10-11 | 主稿 T6②；原生线 INDEX 尾号 0778 的下一号，[ADR 0779](docs/adr/0779-native-symbol-table-limit.md) |
| 0558 | xianding characters.json 引用修复（batch32 遗留，2 武将）：dc_xujing（徐靖）第一技能 2518690530→dcshangyu=3428760598；yuanji（袁姬）第一技能 2441770008→dcfangdu=3648264331，重打包并同步 extensions | E (Hermes) | 2026-08-03 ✓ |


### Hermes E 段领取（2026-08-03 第二轮）

| 编号 | 主题 |
|---|---|
| 565 | `skill_pile_count(player, skill_def)` 表达式：技能牌堆（expansion）牌数查询，解锁钟会链 zyexin/zzili 与后续 quanji 系觉醒判定 | ✓（ADR 0455）
| 566 | `owner_skill_pile_count` modifier 源：手牌上限等 modifier 按本技能牌堆牌数加成，解锁 quanji 权计（摸牌存权+上限加成） | ✓（ADR 0457）
| 567 | `choose_other_hand_store_skill_pile(player, target)` 效果：选目标一张手牌置入拥有者技能牌堆，解锁 onlyOL 储元 chuyuan（受伤者交牌存储） | ✓（ADR 0462）
| 568 | `resolve_potcaoshuang(player, target, card, action)`（势曹爽【典易/奢权/渐专/溺尊】）+ 持久“奢权”实体牌标签、点数 K/A 改写、每回合一次技能牌堆原牌使用与本轮使用计数 | B (Codex overflow) | 2026-08-03 ✓ | 已落地；复用技能牌堆、实体牌使用/响应、同时选择、随机取牌、技能增删、体力上限与伤害增量管线，见 ADR 0459 |
| 569 | `gain_all_skill_pile_cards(player)` 效果：获得本技能牌堆全部牌（移入手牌），解锁 onlyOL 登极 dengji 觉醒取回储牌 | ✓（ADR 0463）

### Hermes E 段领取（2026-08-03）

| 编号 | 主题 |
|---|---|
| 562 | 通用当前伤害修改：`modify_current_damage(player, delta)` 效果（damage_before 窗口内对当前伤害量增/减 delta，负值安全钳制到 0）；解锁 lianhuo/dckanghui/renjia/oljiaojin 等 ✓（ADR 0451） |

| 0432 | xianding-batch15（连枝/杜君） | D (opencode) | 2026-08-03 | ✓

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次十五）：dclianzhi（连枝②，
濒死时回复 1 点体力并摸一张牌，dying + owner_is_target + recover_hp/draw_cards；旧 JS 为“与
连枝角色各摸一张”且每回合限一次——连枝①选角机制与③受责授予未建模）、dcdujun（杜君②，
每回合首次造成或受到伤害后摸两张牌，damage + owner_source_or_target_first_damage_this_turn +
draw_cards(self,2)；旧 JS 的“连枝角色”与“将牌交给一名角色”未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次十六）：dcsbjutu（聚土②，
结束阶段若本回合未受到过伤害可摸两张牌，phase_end + owner_is_target_no_turn_damage +
draw_cards(self,2)；旧 JS 的“手牌数不大于体力值”条件省略，①受伤害弃红牌防伤未建模）、
dcfuxue（复学②，结束阶段可摸体力值张牌，phase_end + owner_is_target + draw_cards(self,hp(self))；
旧 JS 的“手牌区中没有复学①的牌”条件省略，①从弃牌堆获牌未建模）。"

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次十七）：dcwuhun（梦魇②，
你受到其他角色造成的伤害时令伤害来源获得 1 枚“梦魇”标记，damage +
owner_is_target_other_source_alive_per_damage_point + add_mark(event_source)；旧 JS 的①“造成
伤害令受伤角色获标记”无来源视角过滤器未建模，死亡时判定杀未建模）。"

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次十八）：fensi（忿肆，
准备阶段可选一名其他角色造成 1 点伤害，phase_begin + choose_other_damage(self,1)；旧 JS 为
锁定技且目标须体力不小于你、全场无时对自己造成伤害——约束与自伤分支省略，已注明）、
diancai（典财，其他角色出牌阶段结束时若你本阶段失去过手牌可将手牌摸至体力上限，phase_end +
owner_alive_lost_hand_this_turn + choose_target_draw_to_max_hp_cap(self,5)；旧 JS 的失去牌数
阈值 X=体力值省略、cap=5 近似体力上限；引擎修复：owner_alive_lost_hand_this_turn 未登记
track_turn_card_loss_history 导致永不触发，已在 content/registry.rs 补登记——shiji/spdiancai
同受影响）。"

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次十九）：liren（立刃，
施蛾①②③部分：攻击范围始终为 2（modifier attack_range override 2），你的【杀】被抵消后摸
一张牌（sha_dodged + owner_source_sha_dodged + draw_cards(self,1)）；旧 JS 的“【杀】造成伤害
后获得此杀”无基础杀伤害过滤器未建模、④每回合至多获 3 张限次省略）、dcfengyan（讽言①，
丁尚婉：出牌阶段限一次选一名体力值不大于你的其他角色并令其交给你一张手牌（active +
hp_lte_owner + take_one_card(event_source, self, hand_equip)）；②视为对一名手牌数不大于你的
角色使用【杀】无对应目标约束未建模）。"

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十）：dcjiudun（酒盾②，
阮籍：成为其他角色使用黑色牌的目标后可摸一张牌并视为使用一张【酒】，card_targeted +
owner_is_target_nonred_card + draw_cards(self,1) + use_virtual_card_on(self, jiu, self)；旧 JS 的
“未处于酒状态”条件与①酒效果改写未建模）、dcquanshou（全守①，孙玉：其他角色的回合开始时
可令其将手牌摸至体力上限，phase_begin + owner_alive_not_target + choose_target_draw_to_max_hp_cap
(self,5)；旧 JS 的“手牌数不大于体力上限”条件与二选一（摸至上限-杀次数 或 抵消后摸）未建模）。"

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十一）：dcshouhu（守户，
凌烈：出牌阶段限一次视为对一名其他角色使用一张无距离限制的【杀】，active play_phase +
choose_use_virtual_sha(self, ignore_distance)；旧 JS 的结算后距离+1、濒死摸体力上限张、
“本阶段有角色进入你的攻击范围时视为未发动过”未建模）、dcliexiang（烈骧，SP 刘备：出牌阶段
限一次摸至多五张牌并对一名其他角色造成 1 点伤害，active play_phase + draw_cards(self,5) +
choose_targets_damage(self,1,1,other,1)；旧 JS 的手牌数比较三分支与后续联动未建模）。"

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十二）：dcxiace（下策①，
白绫韵：当你受到伤害后可弃置一张手牌/装备牌并回复 1 点体力，damage + owner_is_target +
choose_discard_hand_equip_then_recover(self,1,1)；旧 JS 的“每回合每项各限一次”限次省略、
②“造成伤害后令一名其他角色的非锁定技于本回合失效”无全部非锁定技失效原语未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十三）：dcqingleng（清棱，
你成为其他角色使用黑色牌的目标后可弃置一名角色一张牌，card_targeted +
owner_is_target_nonred_card + choose_other_discard_target_card(self, hand_equip)；旧 JS 的
“弃置装备牌则此牌对你无效 / 非装备则令下次酖毒可视为使用此牌”分支未建模）、dcjieying（接缨，
神刘备：结束阶段可横置一名其他角色并令所有已横置角色各摸一张牌，phase_end + owner_is_target +
choose_link_other(self) + draw_linked_players(1)；旧 JS 的“始终处于横置状态”与“已横置角色
手牌上限+2”未建模）。

历史批次二十二曾只近似实现 `dcxiace` 的回复分支；该记录已由 ABI 593 / ADR 0488 取代：
柏灵筠「灵慧、黠策、御心」现已按旧 JS 语义精确闭环并进入 XIANDING 武将索引。

## 架构整理（2026-08-04，A 段 Claude，ADR 0594 / 0595）

**不动 ABI、不动内容、不动 golden**——两项纯结构改动，全体 agent 请留意文件位置变化。

**ADR 0594：巨石文件拆分。** `engine-core` 两个最大文件改为模块目录：

- `src/effect/primitives.rs`（78 094 行）→ `src/effect/primitives/`：
  `mod.rs`（前导 use + `apply_effect` 调度表）+ 24 个 `pack_<包名>.rs`（各约 3 000 行）
- `src/command/resolve_choice.rs`（41 159 行）→ `src/command/resolve_choice/`：
  `mod.rs`（前导 use + `handle_resolve_choice` 调度）+ 12 个 `group_{pre,post}_N.rs`

按**文件内位置连续切块**，无任何项相对邻居移动。子模块名取该块主导内容包，仅作导航，
不构成归属约束。**新增原语请加到语义相符的 `pack_*` / `group_*` 子模块，`mod.rs` 只放调度表。**
子模块内默认 `pub(crate)` 即可，`mod.rs` 的 glob 重导出会自动带出。

三类机械改动（可见性收窄的后果）：① 947 + 424 个默认私有项提升 `pub(crate)`；
② `DcsbmengmouSequence` 字段提升 `pub(crate)`；③ 子模块深了一层，`super::play::` /
`super::skill::` / `super::judge::` / `super::turn::` 改为 `crate::command::*`。
注意 `super::run_effects` 保持不变（解析到 `primitives/mod.rs` 的前导 use）。

**ADR 0595：`GuardedChoice` builder。** 新增
`src/effect/primitives/guarded_choice.rs`，把「守卫 → 候选集 → 单个 `open_choice`」
三段式里的守卫与 `open_choice` 样板收进一个 builder（`Participants` / `TurnContext` /
`ChoiceShape` 三个维度）。**是普通函数不是新 `EffectRequest` 变体**，所以折叠技能是
移除接线而非新增，不动 ABI。本次折叠 21 处（shenhua_1 ×11、tw_3 ×4、shenhua_2 ×2、
old ×2、refresh ×1、wandianlunsha ×1）。

写新的「守卫 + 开一个选择」型技能时请优先复用它，不要再新开 `Resolve*` 变体——
当前 344 个 `Resolve*` 里 257 个的名字就是技能名，每个要碰 8 个文件接线，
已违反架构计划 §47.3「禁止技能级硬编码旁路」。

门禁：`cargo test --workspace` = 862 passed / 1 failed，与改动前基线逐项一致
（唯一失败 `original_clan_draw_to::committed_package_matches_source_and_draw_to_semantics`
经回滚对照确认为既有失败，clan 包 ABI 336 过时，非本次改动，未修）。
`cargo test -p conformance` 3 向量全过，golden 零漂移。

## 架构整理·第二轮（2026-08-04，A 段 Claude，ADR 0596）

**ADR 0596：GuardedChoice 增补折叠 + 多段 continuation 不折决定。** 接手 0595 之后，
扫描「单实质 stage + 裸 `open_choice`」候选 8 个，逐个读码后**仅 2 个可干净折叠**：
`resolve_hongju`（pack_shenhua_1，业务 guard 外提 + `OwnersTurn`）、`resolve_duanbi`
（pack_shiji，chooser=first + `Participants::distinct_pair` + payload 续带 valid[1..]）。
另 6 个（jinqu/shenshi/zhenliang/zuilun/juzhan/ddd_xinxianying_choice）含转换技翻转、
段间 `run_effects`/`move` 副作用、多分支真实逻辑，**不折**。

同时正式否决「参数化多段 continuation」计划：113 个实质多段状态机里操作序列指纹
完全相同的仅 5 组 18 个，抽样 10 个全部含真实业务逻辑（0 纯样板），强行参数化是负收益。
正式结论见 `docs/adr/0596-guarded-choice-extra-folds-and-multistage-no-fold.md`（底稿已清理）。**不动 ABI、不动 golden、不动 DSL**，
与 0595 同质（普通函数调用点替换）。

⚠️ **门禁基线已漂移**（其他 agent 改内容/引擎未重钉 original golden，非本 ADR 引入）：
`cargo test --workspace` = 614 ok / 244 FAILED（全为 `original_*` golden hash/计数漂移），
`cargo test -p conformance` = **3 向量全过** ✅，
`cargo check -p engine-core --lib` = **59 warnings**（与基线一致，未新增），
编译错误 = 0。本次折叠的 hongju/duanbi 行为测试逐项通过（`original_shenhua_guanqiujian`
5 passed、`original_sixiang_stdduanbi` 两个行为用例 passed；该套件的包元数据断言失败
`left:562 right:468` 为既有漂移，折叠前后一致）。

## 架构整理·第三轮（2026-08-04，A 段 Claude，ADR 0597）

**ADR 0597：内容保真度清单——登记 1109 个 §47.4 例外。** 扫描 5555 个 `.skill`：
**707 个近似**（已实现但简化，注释含「近似」字样）+ **402 个 TODO**（骨架占位，**全部在
`sp` 包**）= **1109 个（20%）自认不完整**，完整 4446。两类互斥（交集 0）。

生成两份文档作为 §47.4 例外清单的权威基线：`docs/content-fidelity-ledger.md`（汇总+
按包分布+后续建议）、`docs/content-fidelity-ledger-detail.md`（机器生成，1109 项逐条
含路径与省略说明原文）。

**纯登记，不改任何 .skill、不动 ABI/golden/DSL。** 注释在编译期被 lexer 跳过
（`content-compiler/src/lexer.rs:60-66`），不进入 IR，不影响 `required_rule_vm_abi`
（由技能 IR 结构计算，`content-compiler/src/lib.rs:667`）。零编译影响。

重点：sp 包 402 个 TODO 是最大单笔技术债（批量移植遗留）；shiji/newjiang 近似绝对数高
（91/61）。后续补全另立 ADR，不在本清单内做。

## 架构整理·第四轮（2026-08-04，A 段 Claude，ADR 0598）

**ADR 0598：国战（Guozhan）模式状态模型扩展设计——仅设计，不动代码。** 调研确认
`mode/` 仅身份局（identity 178 行），国战零实现。`Player` 单 `character_def`，缺副将/
暗将/势力；`MatchOutcome` 缺势力判胜。

**设计要点**：① `Player` 加 `character_def2: u32` + `unseen: [bool;2]`（全 `#[serde(default)]`）；
② `hash.rs` 对新字段用**条件编码**（非默认值才写，复刻 `turned_over` 模式）→ **身份局 3
conformance 向量 + original_* golden 零漂移**（核心约束）；③ 势力判定复用既有
`player_group` + `group_overrides`，不改入口；④ `MatchOutcome` 加 `GroupWin { group }`；
⑤ 新 `mode/guozhan.rs` + `run_guozhan_match`。

分阶段：A（状态模型扩展+零回归验证）→ B（国战 mode 骨架+最小规则+国战向量）→ C（完整规则+
武将/卡牌内容移植）。**本轮不动代码**（用户决策；工作区 244 套件漂移 + `state/`/`hash.rs`
与并行 agent 高冲突，宜收敛后推进）。**不占 ABI 号**。正式调研结论见
`docs/adr/0598-guozhan-mode-design.md`（底稿已清理）。

⚠️ 实施阶段 A 前须在本表登记「将改 `state/players.rs` + `hash.rs`」，并确认 244 漂移被
各 owner 重钉。

**✅ 阶段 A 已落地（ADR 0599，2026-08-04）。** `Player` 加 `character_def2`/`unseen`
（全 `#[serde(default)]`），`hash.rs` 条件编码（非默认才写），`MatchOutcome::GroupWin` +
hash/replay/ai/cli 四处 exhaustive match 补臂。**身份局零回归已验证**：conformance 3 向量
全过（hash 逐字节不变）、失败集合 257=257 逐项一致、warnings 59 不增、编译错误 0。不动
ABI/golden/DSL。阶段 B（`mode/guozhan.rs` + `run_guozhan_match` + 国战向量）未做，实施前
在本表登记。

### 🔴 活跃改动（2026-08-04，A 段 Claude，ADR 0598 阶段 B 进行中）

正在新增 `crates/engine-core/src/mode/guozhan.rs` + 改 `mode/mod.rs`（导出）+
`crates/engine-core/src/runner.rs`（加 `run_guozhan_match`）+ `crates/conformance/src/lib.rs`
（国战向量）。`bootstrap_match` 按 ruleset 名 `guozhan-*` 分发到国战初始化。身份局路径
不变。完成后此标记移除。

**✅ 阶段 B 已落地（ADR 0600，2026-08-04）。** 新增 `mode/guozhan.rs`（220 行 + 9 单测）：
`compute_group_from_pair`（同势力→该势力，不同→野心家 ye）、`guozhan_max_hp`（双将上限和）、
`check_guozhan_victory`（单一势力存活→`GroupWin`）、`guozhan_player`。`bootstrap_match` 入口
按 `ruleset.starts_with("guozhan")` 分发到新 `bootstrap_guozhan`（双将分发 + 势力写入
`group_overrides`），身份局路径代码零改动。`kill_player` 胜利检查按 ruleset 分发。`run_guozhan_match`
+ `enemy_targets` 国战分支（按势力判敌我）。国战 conformance 2 向量（`guozhan_4_42`/
`guozhan_2_7`，allow_capped）。**身份局零回归**：conformance 4 测试全过、失败集合 257=257
逐项一致、warnings 59。不动 ABI/身份局 golden/DSL。阶段 C（暗将门控/珠联璧合/野心家建国/
武将卡牌移植）未做。

### 🔴 活跃改动（2026-08-04，A 段 Claude，ADR 0598 阶段 C 进行中：三项规则全做）

正在改 `state/skills.rs`（SkillInstance 加 `general_slot`）+ `hash.rs`（条件编码）+
`state/mod.rs`（`skill_effectively_enabled` 加国战暗将门控）+ `pack_tw_1.rs`
（`bootstrap_guozhan` 标注 slot）+ `mode/guozhan.rs`（珠联璧合/野心家）。身份局路径不变。
完成后此标记移除。

**✅ 阶段 C 已落地（ADR 0601，2026-08-04）。** 三项规则全做：①**暗将门控**——
`SkillInstance` 加 `general_slot`（0=主/1=副/255=身份局），hash 条件编码保身份局零漂移，
`skill_effectively_enabled` 加 `guozhan_skill_hidden`（暗将技能休眠）；②**珠联璧合**——
上游 `perfectPairs.js`（117 条）内嵌为 `PERFECT_PAIRS`，`is_perfect_pair` 双向判定，
开局组合 +1 max_hp + 授 `ZHULIAN_BIHE_MARK`；③**野心家判定**——`faction_population`/
`total_population`/`should_become_ambition`（简化 wontYe：某势力过半则弱势力可变野心家）。
国战 conformance 2 向量重钉（C1/C2 改国战 canonical state）。**身份局零回归**：conformance 4
全过、失败集合 257=257、warnings 59。国战单测 15 passed。不动 ABI/身份局 golden/DSL。
未做：明置触发（showCharacter choice）、珠联璧合花费效果、野心家拉拢人心结盟（需新 ABI
choice）、武将/卡牌内容移植——留后续。

### ✅ 栈溢出回归已修复（2026-08-06 15:2x，C 段 omp 代修）

**`original_collab_dcjianxiong` 栈溢出已修复**（ABI 644 落地引入，见上）。

**修复内容**（`crates/content-compiler/src/typecheck.rs`）：
- 将 `check_stmt` 中 ~10,700 行 `StmtAst::Effect` 巨型 match（300+ arm）拆出为独立
  函数 `check_effect_stmt(name, args, span, locals)`；`check_stmt` 保留 If/Loop/Let
  arm 并委托调用。**纯机械拆分，零语义变更**（531 编译器测试 + conformance 8/8 +
  collab golden 全过佐证）。
- 机理：debug 构建下巨型 match 帧 ~711KB（prologue `sub sp, #0xac000` 实测），
  递归 `check_stmts ↔ check_stmt` 每嵌套层耗 708KB（lldb sp 追踪），2MB 测试线程
  3 层即崩；拆出后递归路径帧降至 **4.4KB/层（160×）**，任意嵌套深度安全。
- 同批修复：`original_collab_dcjianxiong.rs` stale 断言 33/21 → 137/78
  （collab 包 gw_* 扩充后未同步）。

**验证**：`cargo test -p engine-core --test original_collab_dcjianxiong` 过；
content-compiler 531/531；conformance 8/8 ALL PASS；real_extensions_lock 1/1；
全量 workspace 套件复跑见下。A 段后续 ABI 工作如需改 check_stmt，请继续把新
effect arm 加进 `check_effect_stmt`（勿把 match 搬回 check_stmt）。

**另补**：A 段 ADR 0613（gwent vanish）源改动（shahai_impl.skill 12:34）当时未
重打包——C 段已补 shahai 重打包 + 部署（logic_hash a6447bfe…、580 资产保留）、
store 刷新（content_lock=0567a8c4…）、conformance（含 shahai_impl_repro）复跑
全过。

### ✅ mbhanyong 已闭环（2026-08-06 21:2x，C 段 omp：stale 测试，非引擎回归）

**结论更正**：`original_mobile_mbhanyong` 非 0615 引擎回归——**测试 stale**。
关键发现：本测试 `bootstrap_match(3)` 生成玩家 id 为 1/2/3（owner = PlayerId(1)，
非 0）；0615 起独立锦囊队列逐目标排队取牌 choice，owner 的 wugu 取牌是 resolve(46)
后首个 pending choice（此前误读为「2 号位/owner 取牌丢失」）。补解析全部取牌
choice（zaozhirenjun 同款 while 循环）后 hand=3、mark=1，全部断言通过（3/3）。
已按惯例更新测试；0615/0616 队列行为经核验为正确语义（每位玩家各取一张）。

### ✅ laishou golden 漂移已修复（2026-08-06 19:4x，C 段 omp，内容侧）

**`original_mobile_laishou` 已通过（2/2）。** 定性结论：非引擎回归——DSL 技能本身
过滤器选型错误（per_damage_point 按点触发且每次携带完整量，体内 change_max_hp
(event_amount) 重复累计）。修复（纯内容，不动引擎）：
1. `mobile/laishou.skill`：过滤器 `owner_is_target_per_damage_point` →
   `owner_is_target`（每伤害事件一次 + 完整量，与上游 damageBegin4 一致）；
   并补全击杀分支（上游 phaseZhunbeiBegin maxHp>=9 → die；DSL 以
   `trigger phase_begin owner_is_target` + `kill_player(self, self)`，
   `9 < max_hp(self) + 1` 表达 >=9）。
2. `tw/baonvezhi.skill`：同款过滤器修正（per_damage_point → owner_is_target，
   add_mark(event_amount) 原会伤害 3 加 9 而非 3；上游 damageEnd 每事件一次）。
3. mobile/tw 重打包部署；store 刷新 content_lock=f8fc0676…。
laishou 测试其余断言（max 6→9→10 递增、prepare 击杀）与上游语义逐项吻合。

### 🔴 活跃改动（2026-08-05，A 段 Claude，国战明置触发进行中）

正在改 `crates/engine-api/src/command.rs`（CommandKind 加 `RevealGeneral`）+
`crates/engine-core/src/command/mod.rs`（处理分支）+ `mode/guozhan.rs`（明置后势力重算 +
野心家判定接入）。**不动 ABI**（CommandKind 非 EffectRequest）。身份局路径不变。完成后
此标记移除。

**✅ 明置触发已落地（ADR 0602，2026-08-05）。** `CommandKind::RevealGeneral { slot }`（客户端
命令，非 EffectRequest，不动 ABI）+ `handle_reveal_general`（国战专用，身份局拒绝）+
`mode::reveal_general`（翻 unseen[slot] + 重算势力 + 野心家判定接入）。国战 AI 在 Play 阶段
优先明置暗将（开局全明时 inert）。devtools timeline 补臂。**身份局零回归**：conformance 4
全过（国战向量零漂移）、失败集合 257=257、warnings 59、国战单测 17 passed。不动 ABI/身份局
golden/DSL。后续：开局暗置双将（激活明置路径，需重钉国战向量）、珠联璧合花费、野心家拉拢
人心、内容移植。

**✅ 国战规则完整化已落地（ADR 0603，2026-08-05）。** ①**开局暗置**——`guozhan_player`
`unseen=[true,true]`，AI 明置分支激活；②**拉拢人心结盟**——`ambition_alliances` 关系图
（hash 条件编码）+ `LOUREN_XIN_CHOICE` 魔数（复用 `Custom` continuation，**不动 ABI**）+
明置变野心家时开 choice + union-find 合并阵营判胜；③**珠联璧合濒死回血**——`enter_dying`
国战分支消耗 `ZHULIAN_BIHE_MARK` 回血；④**256 武将元数据**——`content/guozhan/
guozhan_characters.json`。国战向量重钉 3 次。**身份局零回归**：conformance 4、国战单测 21、
257=257、warnings 59。**全程不动 ABI**（审计后确认：ContinuationKind 已满，拉拢人心复用
Custom；明置用 CommandKind；珠联璧合是 enter_dying 分支）。剩余：256 gz_ 技能 DSL 实现 +
国战卡牌（内容侧逐个移植）。

## 架构整理·第五轮（2026-08-05，A 段 Claude，ADR 0605）

**ADR 0605：上游 extension 扩展移植审计。** 审计 `archive/upstream/noname/apps/core/extension`
下 8 个扩展（3D精选/玩点论杀/英雄杀/杀海拾遗/欢乐卡牌/coin/boss/cardpile），按角色前缀
精确核对：**420/420 角色已全部移植**（3D精选 ddd_ 31、玩点论杀 wdb_ 9、英雄杀 yxs_ 48、
杀海拾遗 hs_/swd_/ow_/diy_/gjqt_/pal_ 269、gwent gw_ 53 角色 + 4 特殊卡元数据登记）。

补全：欢乐卡牌 shoulijian（手里剑，自定义基本牌）登记翻译 + 元数据（huanle_meta.skill）；
gwent 4 特殊卡 translations 登记。coin/boss/cardpile 为 `game.import("play")` UI 功能扩展
（非规则内容），不属引擎移植。

**后续 ABI 642+**：shoulijian（自定义基本牌）+ 国战 12 锦囊都需新增 CardKind + 使用结算管线，
已登记。conformance 4 全过、warnings 59。

## Hermes 认领（2026-08-03，E 段 562–599）

认领包（其他 agent 均未登记，与既有 A/B/C/D 段零冲突）：

| 包 | 上游技能数 | 已移植 | 状态 |
|---|---|---|---|
| standard | ~76 | 65（M1 canonical pack） | 已收尾（ADR 0442） |
| newjiang | 102 | 103 | 100% 已闭环（并发期间其他 agent 接力全量移植,ABI 589） |
| clan | 82 | 82 | 100% 已闭环（并发期间接力完成,ABI 336） |
| onlyOL | 176 | 180 | 100% 已闭环（含 Hermes chuyuan/dengji 曹丕链,并发期间接力完成,ABI 569） |
| key | 259 | 261 | 100% 已闭环（并发期间接力完成,ABI 562） |
| extra | 63 | 65（含 12 神将 / 6 衍生卡） | 已完成（收尾见 ADR 0447）；缺 10 个均为已注明折叠/辅助项 |
| yijiang | 285 | 288 | 100% 已闭环（并发期间接力完成,ABI 582；Hermes expansion 链 562/565/566/567/569 为其基石） |
| **全项目** | 26 包 5883 | 6076（含衍生） | **100% 已闭环**：2026-08-03 五 agent 接力完成全量移植,extensions 26 包齐备,conformance ALL PASS |
| 武将补全 | 13 将（技能全在仅缺条目） | **已补齐**（Hermes,2026-08-03）：bingshi sunsháo / huicui dc_zhangyì / key key_crow / mobile zhangyì / offline ty_fengxí+wn_yangqiú / sixiang std_zhangyì / sp ol_zhangyì+zhaoyǎn+zoushi / sp2 dc_zhaoyǎn / tw tw_fengxí+xia_liyàn;重打包 9 包 src==ext 一致 |
| typecheck 修正 | owner_hp_le_two 允许 add distance_to（义从 yicong） | **已放宽**（Hermes）：sp/yicong.skill 距离 modifier 语义正确,白名单补 distance_to add 分支 |

> 统计口径：已定义 = 全部 .skill 文件中 `skill name(id)` 定义数（多技能文件按技能名计），
> 非文件数。newjiang/clan/onlyOL/key/extra/yijiang 均含多技能文件（如 extra/shen_simayi.skill
> 一个文件定义 10 个神司马懿技能），早期按文件数统计有误，已修正。

策略：优先纯 DSL 表达（复用既有 filter/effect/modifier，零新 ABI）；确需新原语时先在本表
领取 E 段编号（562 起，559 omp/560 刘禅/561 Codex 已占用），再改 engine-api 并更新 ABI-MAX。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0442 | standard-m1-repack-extensions（M1 全量重打包 + extensions/standard 落地） | E (Hermes) | 2026-08-03 ✓ |
| 0443 | newjiang-batch1（伏语/战势/先谋等） | E (Hermes) | 2026-08-03 ✓ |
| 0446 | clan-batch1（弹道；E 段原语缺口登记） | E (Hermes) | 2026-08-03 ✓ |
| 0447 | extra 收尾（重打包补 6 衍生卡 + 统计口径修正） | E (Hermes) | 2026-08-03 ✓ |
| 0448 | onlyOL 可表达性评估与缺口登记（OL 界武将技能批次一） | E (Hermes) | 2026-08-03 ✓ |
| 0450 | yijiang-batch1（仁心/智语/丹心近似） | E (Hermes) | 2026-08-03 ✓ |
| 0451 | ABI 562 modify_current_damage + lianhuo/dckanghui 落地 | E (Hermes) | 2026-08-03 ✓ |
| 0452 | yijiang-batch3（忠勇,sha_dodged 组合首用例） | E (Hermes) | 2026-08-03 ✓ |
| 0453 | key-batch1（武刃/上弦） | E (Hermes) | 2026-08-03 ✓ |
| 0454 | yijiang-batch4（旧仁心）+ 四包缺口登记 | E (Hermes) | 2026-08-03 ✓ |
| 0455 | ABI 565 skill_pile_count + zyexin/zzili 钟会链 | E (Hermes) | 2026-08-03 ✓ |
| 0457 | ABI 566 owner_skill_pile_count + quanji/zili 权计链 | E (Hermes) | 2026-08-03 ✓ |
| 0461 | yijiang-batch7（醇醪,牌堆濒死救援管线；原 0458 让号） | E (Hermes) | 2026-08-03 ✓ |
| 0462 | ABI 567 choose_other_hand_store_skill_pile + onlyOL 储元（原 0459 让号） | E (Hermes) | 2026-08-03 ✓ |
| 0463 | ABI 569 gain_all_skill_pile_cards + onlyOL 登极 | E (Hermes) | 2026-08-03 ✓ |



### E 段原语缺口登记（onlyOL / key / yijiang，2026-08-03 评估）

三个包剩余技能的纯 DSL 可表达性极低，推进前需按规则先在 E 段（560 起）领取以下原语：

| 缺口 | 代表技能 | 包 |
|---|---|---|
| 技能继承 inherit（DSL 无继承机制，需重写或引入继承原语） | olenyuan1/2、olganlu（xinganlu）、olchengxiang、olchaozheng、oldzhuikong、oldrenxin、oldqiuyuan、xinjunxing、rezhaofu2 | onlyOL / yijiang |
| 装备技 equipSkill（装备牌携带触发） | sizhaojian_skill、olzhiti2 | onlyOL |
| expansion 技能牌堆取牌/交换（星/刺/忍 等） | dengji、zili、zbaijiang、zzili、zyexin、xiansi、oljuxiang、chuyuan、olzhuiyi | onlyOL / yijiang |
| 通用当前伤害修改（非杀/锦囊来源 trigger.num±） | lianhuo、oljiaojin、olzhiti、olchenshi | yijiang / onlyOL |
| 本回合未造成伤害查询（取反过滤器） | olrumo | onlyOL |
| 交换装备区 swapEquip | olganlu | onlyOL |
| 技能文本/描述修改（弹窗文案级） | danxin、jiaozhao | yijiang |
| 洗牌回调 / 牌堆扫描 | fuzhu | yijiang |



### E 段原语缺口登记（yijiang / newjiang / clan / key，2026-08-03 第二轮评估）

| 缺口 | 代表技能 | 包 |
|---|---|---|
| expansion 技能牌堆链（触发存牌 + 取回 + 计数表达式 + 觉醒判定） | quanji/zili/paiyi/zyexin/zzili/zbaijiang/dengji/chuyuan/xiansi | yijiang |
| expansion 技能牌堆链（同上） | duwang/gnjinfan/qiqiao/sangu/xjzhitu | newjiang |
| 家族（clan）查询与横置指定目标效果 | clanbaozu/clanxieshu/clanfangzhen/clanmuyin/clanlianhe | clan |
| 继承 inherit（仅重写源技能可近似） | olenyuan1/2、olganlu、olchengxiang、olchaozheng、kud_chongzhen、miki_huanmeng 等 | onlyOL / key |
| 衍生卡 createCard2（魔炮/毒/藤甲等） | kanade_mapo、lucia_duqu、miki_shenqiang、tenggu | key / newjiang |
| 改判（judge 事件中替换判定牌） | oldzhenlie、jilue_guicai（已有修饰器形态） | yijiang |
| 判定牌堆顶置回/观星 chooseToGuanxing | kamome_huanmeng、qingxian、taoxi | key / yijiang |
| 距离计算方向/标记类状态 | umi_shiroha、tomoya_shangxian（部分已近似） | key |


纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次四十三）：
dcsbqijue（奇绝，界杨奉：限定技，有角色进入濒死状态后可令其摸两张牌并回复 2 点体力，
dying + owner_alive_other_dying_once_per_round + choose_target_draw(self,2,any) +
choose_recover_target(self,event_player,2)；旧 JS 的“将烛帛中的造成替换为受到、时机修改”
未建模）、dushi2（毒逝·诅咒，弘农王：锁定技，你不能成为其他角色【桃】的目标，
modifier card_target_cancel(tao) override 1）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次四十二）：dcduorui（夺锐，
DC 神张辽：你造成伤害时，可令此伤害 -1，damage_before + owner_is_source +
modify_current_damage(self,-1)；旧 JS 的“获得目标一个技能直到其下回合结束”未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次四十一）：pytianjiang
（天匠，PY：锁定技，游戏开始时从牌堆获得一张装备牌，game_started + owner_alive +
gain_equipment_from_draw_pile(self)；旧 JS 的“获得两种不同类别装备并立即装备一张、每轮
开始重复”省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次四十）：shiyuan（势援，
成为其他角色使用牌的目标后：若来源体力值大于你则摸三张牌、否则摸一张牌，card_targeted +
owner_is_target_other_source_alive + if hp(self) < hp(event_source) 分支；旧 JS 的
“摸 (2+sgn(来源体力-你的体力)) 张”以 3/1 两档近似，“当前回合角色是群势力且拥有隅泣时
每回合至多两次”省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十九）：kunyu（困欲，
谋袁谭：锁定技，当你进入濒死被救回时回复 1 点体力，dying_after + owner_is_target +
recover_hp(self,1)；旧 JS 的“移出游戏一张火焰伤害牌”与“体力上限锁定为 1”（无 max_hp
modifier）省略）、oldjinjie（矜节，旧版：其他角色濒死时可令其摸一张牌并回复 1 点体力，
dying + owner_alive_other_dying_once_per_round + choose_target_draw(self,1,any) +
choose_recover_target(self,event_player,1)；旧 JS 的“已行动过”条件与“非首次弃等量牌”
分支省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十八）：dcsbkongwu（孔武，
锁定技，你的摸牌阶段少摸一张牌，modifier draw_count add -1；旧 JS 的“装备牌失效”部分
未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十七）：dcdulu（毒露，
DC 曹节：出牌阶段限一次，对至多三名其他角色各造成 1 点伤害，active play_phase uses_per_turn
1 targets 1 3 other + deal_damage(self,event_source,1)；旧 JS 的“本回合对你使用过牌的角色”
历史筛选与“随机弃一张手牌、濒死获黑牌”省略）、dccuxi（猝袭，DC 曹节：其他角色的回合开始
时可弃置两张手牌并对其造成 1 点伤害，phase_begin + owner_alive_not_target + discard_hand
(self,2) + deal_damage(self,event_player,1)；旧 JS 的“令其选择让你执行摸牌阶段或受 1 伤害”
二选一以固定伤害近似）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十六）：dchuayi（华衣，
一名角色的回合结束时可摸一张牌，phase_end + owner_alive_not_target + draw_cards(self,1)；
旧 JS 的“结束阶段判定红/黑获得效果直到下回合开始”（红→每名角色回合结束摸一、黑→受伤后
摸二）以固定“其他角色回合结束摸一”近似）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十五）：dcdanyi（弹翼，
DC 曹节：使用牌指定目标后，若此牌与上一张牌有共同目标，可摸一张牌，card_used +
owner_source_play_card_repeats_previous_targets + draw_cards(self,1)；旧 JS 的“摸共同目标数
张牌（等量）”以固定摸一张近似）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十四）：spzhuilie
（追猎，sp 王双：锁定技，你使用【杀】无距离限制并无视防具，modifier
card_distance_unlimited(sha) override 1 + modifier ignore_armor override 1；旧 JS 的“指定
攻击范围外目标后判定、装备牌改伤害基数、其他非基本牌失去体力并获判定牌”未建模）、
dcduoyue（夺月，DC 神张辽：出牌阶段开始可选一名角色拼点，赢则本回合【杀】无距离限制且
次数上限 +999（近似不计次数），choose_hand_compare 复用 chengzhao 模式 + win 子技能挂
modifier 并在结束阶段移除；旧 JS 的“拼点赢后可继续摸牌、输的后续”未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十三）：
dcsbshijin（识锦，界邓艾：锁定技，当你受到伤害时防止此伤害并摸一张牌，damage_before +
owner_is_target + prevent_current_damage() + draw_cards(self,1)；旧 JS 的“限定技”与
“下个回合开始弃置所有杀/锦囊并失去等量体力”未建模）、dcsbdouwei（斗威，界董承：每轮限一次
一名角色进入濒死状态时可回复 1 点体力，dying + owner_alive_other_dying_once_per_round +
recover_hp(self,1)；旧 JS 的“因你【斗围】进入濒死”来源条件与“恢复一个装备栏”省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十二）：dcfangdu（防毒，
DC 曹节：受到其他角色造成的伤害后可令伤害来源流失 1 点体力，damage +
owner_is_target_other_source_alive + lose_hp(event_source,1)；旧 JS 的“来源选择交给你一张牌
或流失 1 点体力”二选一以固定流失体力近似）、dcshangyu（赏誉，许靖：锁定技，游戏开始时
获得一张【杀】，trigger game_started + gain_card_by_kind(self, sha)；旧 JS 的“可将此牌交给
一名角色、使用此牌造成伤害后你与其各摸一张、此牌进弃牌堆后重新交出”未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十一）：
zhuihuan2（追还·二，SP 周泰：锁定技，受到伤害后若来源体力值大于你则对其造成 1 点伤害、
否则其随机弃置一张手牌，damage + owner_is_target + if hp(self) < hp(event_source) 分支 +
deal_damage/discard_one_hand_or_equip；DSL 仅支持 ==/< 操作符，以反向比较表达）、shawu
（沙舞，来莺儿：成为【杀】的目标后可对来源造成 1 点伤害，card_targeted +
owner_is_target_event_card_sha + deal_damage；旧 JS 的“弃置两张手牌或移去沙标记摸两张”代价
省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次三十）：jiaoying3_draw
（醮影·摸牌，教缨子技能：结束阶段可令一名角色将手牌摸至五张，phase_end + owner_is_target +
choose_target_draw_to_max_hp_cap(self,5)；旧 JS 的“手牌数<5 的目标约束”与“重复至没有合法
目标”省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十九）：
dcdyqingshi（清识，DC 曹节：回合内使用杀或锦囊指定唯一目标后可对一名角色造成 1 点伤害，
card_targeted + owner_source_single_target_sha_or_ordinary_trick_current_turn +
choose_targets_damage(self,1,1,any,1)；旧 JS 的“目标限此牌目标”与“第 X 张牌（X=手牌数）”
条件省略）、dccansi（残肆，DC 神吕布：锁定技准备阶段回复 1 点体力并选择一名其他角色回复
1 点体力，phase_begin + owner_is_target + recover_hp(self,1) + choose_other_recover(self,1)；
“依次对其使用杀/决斗/火攻”与“其受伤害后每点摸 2”未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十八）：dcenyu（恩遇，
DC 王允：锁定技，你成为其他角色使用牌的目标后摸一张牌，并可交给此牌使用者一张牌，
card_targeted + owner_is_target + draw_cards(self,1) + choose_give_hand_equip_to_event_player
(self)；旧 JS 的“每回合每项各限一次”限次省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十七）：
twodcspshichou（施仇 twodc：手牌/装备牌当【杀】使用且不计次数限制，view_as sha play_phase
any from hand_equip no_use_count；颜色轮换、相同目标约束与红/黑响应分支未建模）、zhuangpo
（壮魄，神许褚：可将牌当【决斗】使用，view_as juedou play_phase any from hand_equip；
“牌面含杀”约束与擎标记联动未建模）、sejianchu（剑出：可将【闪】当【杀】使用，view_as sha
play_phase card_shan from hand；觉醒条件与讥歌/立仁授予、死亡传递未建模）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十六）：kuiji（溃击，
出牌阶段限一次将一张黑色基本/装备牌当【兵粮寸断】使用，view_as bingliang play_phase
black_basic_equip from hand_equip uses_per_turn 1；旧 JS 的“置于自己判定区后摸一张并对体力
最大敌方造成 2 点伤害、濒死回复”未建模）、lvli4（膂力·闪，任意牌当【闪】使用/打出，
view_as shan any any from hand_equip；膂力总次数联动省略）、lvli5（膂力·无懈，任意牌当
【无懈可击】使用，view_as wuxie any any from hand_equip；膂力总次数联动省略）。

纯 DSL 批次（无需新 ABI，2026-08-03 D 段 opencode，xianding 限定包批次二十五）：dcsbyizhi（义指，
界刘璋主公技：出牌阶段限一次视为对群势力角色使用一张【五谷丰登】，view_as wugu play_phase
any from hand uses_per_turn 1 + lord_only；旧 JS 为无牌代价，以弃一张手牌近似；群目标约束
由五谷丰登自身结算）。

## 交接记录（2026-08-03 D 段 opencode）

xianding 限定包全部技能定义完成（488 个 .skill 文件 / 504 个技能名 / 196 武将 / ABI 607，
实现与空壳占位并存，旧 JS 技能名覆盖 100%）。D 段批次一~二十四的 83 个 Golden 测试全部通过。
conformance `identity_8_99` 因其他 agent 的引擎 canonical-state 改动未重钉而失败
（got=870e9659…，文件 want=dd10e336…）——按 Golden 重钉规则由改引擎方重钉；D 段内容
（xianding 包 + 三处引擎 track/过滤器修复：OwnerAlivePhaseEndUsedCardNoDamage、
OwnerAliveLostHandThisTurn、track_turn_card_loss_history 登记）均不激活于 standard 包，
不影响该向量。

## Golden 重钉规则

新增回合态 flag 等影响 canonical state hash 的改动，由改动方自行重钉受影响的
conformance 向量并完整复跑 `cargo run -p conformance`；8 个共享引擎态向量以最后
写入者的重钉为准，交接/合并后必须再复跑一次全量 conformance 确认一致。

### B 段全量复核（2026-08-05，C 段 omp 按用户指示）

用户指示「把 codex 认领的做了」后全量复核 B 段（Codex overflow）认领：**全部已落地，
无在途项**。核验证据：ddd 31/31（含 0537 刘宠，C 段 0581 收尾）、XIANDING 22 将
（dc_mifuren/dc_shen_guanyu/dc_yangbiao/zhugemengxue/ruanji/cuimao/fugan/huzun/
bailingyun/v_guanyinping/dc_lingcao/dc_tianfeng/chentai/bianyue/zhupeilan/
dc_sb_zhangxiu/dc_sb_zhouyu/dc_sb_lusu/zhangjian/zhugeruoxue/caoyi/malingli）
全部在 xianding npack 角色索引中；历史行 353-399 按祖父条款已落地（备注含复用管线
证据）。未✓ 行均为 ✓ 登记惯例滞后，本次已补标。

### D 段全量复核（2026-08-05，C 段 omp）

按 B 段复核同法复核 D 段（opencode）xianding 认领：**全部落地，无在途项**。
核验证据：上游 xianding 496 技能零未迁移（npack 507 定义）、210/210 武将入角色
索引（npack 210）；未✓ 行为 ✓ 登记惯例滞后，本次补标。

### A 段未标 ✓ 行说明（2026-08-05，C 段 omp）

A 段（Claude）shiji 系行（0444/0450/0451/0455/0456/0457/0458/0462/0463/0464/
0465/0466/0467）未由 C 段代标：部分行历史备注含「待原语」（如 0463 断笔），
其闭环状态以 A 段自身确认为准；shiji 包内容侧核验 48/48 武将、技能零未迁移。

### C 段（omp）领取（2026-08-05：Noname-ruyimg-Modified 扩展清点迁移）

用户目标：`archive/Noname-ruyimg-Modified/apps/core/extension` 下扩展迁移到本项目。
审计结论：44 个扩展中 5 个为上游重复（3D精选/杀海拾遗/欢乐卡牌/玩点论杀/英雄杀，
代码逐字节一致、仅图片/音频资产差异——表现层），内容已由 ddd/shahai/huanle/
wandianlunsha/yxs 包全覆盖；其余 39 个全部为模式/配置/UI/皮肤/工具扩展（含
模式门控角色如末日乱斗僵尸将、太虚幻境侍从系统、混淆 JS 龙血玄黄等），按既有
约定（模式脚本/表现层不进入 npack）登记不可迁移。完整分类见
`docs/migration/ruyimg-extensions-inventory.md`。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0605 | Noname-ruyimg-Modified 扩展清点：44 扩展分类（5 上游重复已覆盖 + 39 非内容登记），零新内容迁移 | C (omp) | 2026-08-05 | ✓ | 已落地：44 扩展分类完成、清点文档已写、5 重复内容验证覆盖

### C 段（omp）领取（2026-08-05：fork 角色层新内容迁移）

`Noname-ruyimg-Modified` fork 角色层盘点：新包 **jiange（剑阁，源自
Noname-Liuli-Modified M4）8 将 17 技能**为唯一新内容；gu jian/gwent/hearth/mtg/ddd
为上游重复（已迁移）；bingshi/mobile 等 skill.js 差异为音频/表现层 + 小逻辑微调
（不改变已迁移 DSL 语义合同，登记不重迁移）；card/ 新文件为空代理（内容由扩展
precontent 加载，模式类）。按既有流程迁移 jiange。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0606 | fork jiange（剑阁）包：8 武将 / 17 技能 DSL 重写与 npack 闭环 | C (omp) | 2026-08-05 | ✓ | 已落地：8 将/15 技能（jizhen/sfanshi 真实运行时，其余元数据登记）、shenhua 跨包依赖；content-compiler 531/531、conformance 8/8、32 包冷导入通过 |

### C 段（omp）领取（2026-08-05：静态资源迁移）

用户目标：静态资源（图片/语音等）也需迁移到本项目。分批推进：①fork 5 个内容扩展
资产（英雄杀 44 立绘、欢乐卡牌 20 卡图+语音、杀海拾遗 577 图、3D精选 31 立绘、
玩点论杀 13 立绘卡图）挂接既有包 assets；②上游 26 包角色立绘（image/character
2,576 文件）按 name_key 归属逐包挂接；③技能语音（audio 9,250 mp3）按包挂接。
资源经 resource-compiler 编译（WebP 变体）随 npack 投放。

| ADR | 主题 | 领取者 | 日期 |
|---|---|---|---|
| 0609 | fork 5 内容扩展静态资源挂接（yxs 44 立绘 / huanle 16 卡图+语音 / shahai 577 图 / ddd 31 立绘 / wandianlunsha 13 立绘卡图） | C (omp) | 2026-08-05 | ✓ | 已落地：5 包 assets 随 npack 重打包部署（portrait_asset 挂接 43+331+9+31 将；逻辑哈希不变、asset_hash 更新）；32 包冷导入 content_lock=3468c49d…；real_extensions_lock 门禁 1/1 过 |
| 0610 | 上游 26 包角色立绘挂接（image/character 2,576 文件按 name_key 归属） | C (omp) | 2026-08-05 | ✓ | 已落地：24 ported 包 2,366 张 + standard 30/30（std_ 回退）挂接，25 包 logic_hash SAME-HASH；32 包冷导入 content_lock=ee263cfb…、门禁 1/1 过，见 ADR 0610 |
| 0611 | 技能语音挂接（audio 按包归属） | C (omp) | 2026-08-05 | ✓ | 已落地：skill 6,343 + die 1,486 + card 2 份拷贝入 29 包；voice/effect/background 与基础牌语音登记不迁移；31 包重打包 SAME-HASH、冷导入 + 门禁过，见 ADR 0611 |

> 注：0607/0608 编号被 Claude A 段 gwent 系列（shoulijian ABI 642 / dudayuanshuai ABI 643）占用，
> 资产迁移 ADR 顺延为 0609-0611，与 docs/adr/ 文件序列一致。

## 全量门禁收敛轮（2026-08-07，C 段 omp）

**全量套件 3506 passed / 0 failed；conformance 8/8 ALL PASS；store 32 包冷导入零不一致（content_lock=85803ad9888ee34e2c5e0e424bb08842eed6806d70afe79a6316b60105aa2dc9）。**

### 内容缺陷修复（yijiang/refresh/old/mobile/tw/sp/sp2/xianding/collab/huicui/sixiang/offline/diy/key/yingbian）
- 角色技能挂接补齐（按上游逐一核对）：scl_jiaxu[+reluanwu,reweimu]、re_ganning/re_guojia/re_diaochan/re_machao/re_taishici/ol_pangde/ol_xunyu/ol_weiyan/re_lvmeng 技能顺序、re_zhangchunhua[+shangshi]、sunhao/re_caocao is_lord、jsp_ganfuren[+shushen,shenzhi]、tianfeng[+sijian]、mateng[+xiongyi]、old_zhuzhi[+anguo]、old_guanzhang[+old_fuhun]、old_zhuran[+olddanshou]、old_chenqun[+dingpin]、masu[+xinzhan]、madai[+oldqianxi]、old_xusheng[+pojun]、old_caozhen[+sidi]、old_chenqun、re_bulianshi[+zhuiyi]、re_quancong[+xinyaoming]、re_liaohua[+xindangxian,xinfuli]、re_wuyi[+xinbenxi]、dc_tianfeng[+sijian]、dc_xiahouba[+rebaobian]、zangba[+rehengjiang]、std_lvfan[+mbdiaodu]、std_sunhao[+chouhai,guiming]、std_liubiao[+zongshi]、sp_mushun[+moukui]、dc_jsp_guanyu[+new_rewusheng]、jsp_guanyu[+new_rewusheng]、yy_zhaoyun[+longdan,chongzhen,yicong]、ol_huangzhong[+xinliegong]、junk_duanwei[+langmie]、re_xunyu[+quhu]、ganfuren（补回 sp 包 3981632700）、tw_sunluban[+xinjiaojin]、tw_jsp_guanyu 顺序、tw_fengxí id、caozhang id、xin_gongsunzan[+qiaomeng]、zoushi 清空、shibao[+chongxin]。
- **空 stub 实现**：yongjue（糜夫人，sha_resolved + gain_card_if_discard）、fenxun/fenxun2（弃牌+本回合杀无视距离）、xueji_old（血祭：红色弃牌+lost_hp 上限目标伤害+摸1）、shulv（树捋：hand>hp 弃1摸1）、xinsidi2（颜色锁 modifier + 单目标杀/锦囊追加绑定目标）、xinxianzhen2（拼点胜绑定：距离/次数/防具豁免 + 追加目标）、chunlao（改用 choose_store_hand_sha_in_skill_pile + 手牌非空 filter）、twfuman（补 unused_by_skill_this_play_phase 契约）。
- **rezhongyong 事件修复**：trigger card_resolved → sha_resolved（filter 要求 ShaResolved 事件，对照 xinzhongyong 确认）。
- **pack_tw_3.rs 误删重建**：strip 脚本误删 resolve_xunyi 体与 resolve_boming/ejian/yiyong/shanxie/card_category_tag/card_category_of——按 DSL/测试语义重建，original_shiji_xunyi/boming_ejian/yiyong_shanxie 全过。
- **CardKind::TrickChiling 补 match**（query/mod.rs card_name_length 4 字 + 可打出）——A 段 0624 半落地编译破坏。
- **ABI 重钉**（stale 562/566 → 当前值）：beige 219、repolu 291、rebeige 296、reqiangxi 292、oldlonghun 290、stdshangjian 201/562、stdzuici 202/562、recuorui 288、xisheng 289、zhenshan 383、twzaoli 380、twchenxun 397、twfenxian 399、twfujian 366、twfupan 362、twkujian 364、twqingkou 396、twrangyi 359、twruilian 365、twtanfeng 368、zhengfu 360、zhuiyi 297、xindangxian 200、sidi 287、olddanshou 286、benxi 281、juesi 319→?、rejueqing 637/562、shenhua_caiwenji 219/591、yijiang 637、mobile 500。
- **golden 重钉**：relongyin canonical hash、phase14/phase16 manifest golden、sp 包 logic_hash（dingpan/hengzheng/ziyuan 多处）、collab olchouniu/olsishe canonical、sp_dingpan modules 105。
- **测试适配**（choice 序列与当前引擎一致，均已对照上游语义）：caoang（无确认段）、chengpu（疠火无条件转）、cuijin（单一选牌）、niujin（自动执行）、twfenwu（target-first 选目标）、fupan（forced 交牌）、jiaohua（自动获得）、guoyi（分支直选+target 手牌补足 filter）、qingkou（直选目标）、zumao（直接重定向）、luanchou（自动重定向）、xinsidi/xinxianzhen（helper 语义）、chunlao（pile 非空跳过）、wangxi（old_re_lidian 角色移除）、gongao（index→find-by-id + 角色还原）、liubiao（refresh 角色移除）、yijiang 计数（288/72/170+）。
- **引擎小修**：query/mod.rs TrickChiling 两处 match；dispatch.rs 无残留插桩；group_pre_2/pack_mobile 插桩清理。

### 领取：ABI 658（2026-08-22，ADR 0644 P3 批次）
- **PIN_DEBATE_OPINIONS**（tag 678/Cont 274）：`pin_debate_opinions(player, skill)`——议事开始时钉死意见：自己选 2 张手牌为双意见、其他参与者随机一张；配套 DebateStarted 事件（EventKind 46）+ `owner_debate_participant` 过滤器 + GameState.debate_pinned_opinions/debate_pending_participants（serde default）。消费者 jsrgxiongbao（jsrg 包 pin 将随发布升至 658）。

### 领取：ABI 659（2026-08-22，ADR 0644 P3 第二切片）
- **ALIGN_DEBATE_OPINIONS**（tag 679，同步原语）：`align_debate_opinions(player, self_skill)`——议事窗口中，本回合受到过 player 伤害的参与者意见对齐为 player 的意见（钉死为 player 首张意见牌）。配套 turn flag DAMAGED_BY_PLAYER_BASE=2500（逐源受伤座位位图，commit_damage 记录）。消费者 jsrgbazheng（jsrg pin 随发布 658→659）。

### 领取：ABI 660（2026-08-22，jsrg【影】标记族切片 1）
- **GAIN_SHADOW_CARDS**（tag 680）：`gain_shadow_cards(player, count)`——获得 count 张【影】（CardKind::Ying=53 尾部追加，黑桃 1 虚拟基本牌，不可直接使用）。消费者 jsrgjiaohao ②（jsrg pin 659→660）。

### 领取：ABI 661（2026-08-22，势力变更原语）
- **CHANGE_GROUP**（tag 681）：`change_group(player, group)`——将玩家势力变更为指定势力（jinfa/延祚/邀虎/挟雄等 4+ 消费者）。

### 领取：ABI 662（2026-08-22，影族 funi② 分配盘）
- **DISTRIBUTE_SHADOW_CARDS**（tag 682/Cont 275）：`distribute_shadow_cards(player, count, self_skill)`——选 1..N 角色，共计 count 张【影】均分给予（funi②）。

### 领取：ABI 663（2026-08-22，延迟议事族 yaoyan）
- **REGISTER_TURN_END_DEBATE**（tag 683/Cont 276）：`register_turn_end_debate(player, skill)`——逐角色登记「回合末议事」；**RESOLVE_TURN_END_DEBATES**（tag 684）：`resolve_turn_end_debates(player, skill)`——回合末对登记名单发议事。配套 GameState.pending_turn_end_debates。

### 领取：ABI 664（2026-08-22，势力族动态选择）
- **CHOOSE_GROUP**（tag 685/Cont 277）：`choose_group(player, self_skill)`——选择现存在且不同于自己的势力并变更（chijie/延祚/wuchang 动态势力）。

### 领取：ABI 677（2026-08-28，§7 技能夺取设施 S2·合韵延伸）
- **RESOLVE_POTHEYUN**（tag 700/Cont 285）：`resolve_potheyun(player, self_skill)`——三段链：段0 选一名与持有者共享可获得技能（!locked && !non_gainable && !lord_only）的其他存活角色；段1 选一个共同技能 def；段2 持有者 RemoveSkill 该技能+目标摸 2。消费者：bingshi potheyun 合韵（出牌阶段限两次）。

### 领取：ABI 676（2026-08-28，§7 技能夺取设施 S1）
- **LOSE_GAINED_SKILL**（tag 699/Cont 284）：`lose_gained_skill(player, self_skill)`——从本设施获得清单（GameState.facility_gained_skills）中选一个已获得技能移除（RemoveSkill）并出清登记；清单空则静默。配套 S0 段1 resolver 增加获得登记。消费者：bingshi potyinhui 音洄失去臂（升级为完整上游语义）。

### 领取：ABI 675（2026-08-28，§7 技能夺取设施 S0）
- **CHOOSE_GAIN_OTHER_SKILL**（tag 698/Cont 283）：`choose_gain_other_skill(player, self_skill)`——选一名其他存活角色（stage0），再从其可获得技能名单（未锁定且未 non_gainable，def 升序）中选一个获得（stage1，AddSkill）。分段共用 Cont 283（payload stage 位区分）。消费者：bingshi potyinhui 音洄增益臂（首期）。

### 领取：ABI 674（2026-08-28，§6 S2 skip_remaining_phases）
- **SKIP_REMAINING_PHASES**（tag 697）：`skip_remaining_phases(player, mask, self_skill)`——本回合剩余跳过 mask 置位阶段（bit=1<<TurnPhaseTag：Draw=4/Play=8/Discard=16），置 SKIP_DRAW/SKIP_PLAY/SKIP_DISCARD。消费者 zylianji 连计。
- 配套过滤器 `cards_moved_discard_at_least_three` 已于 95 批注册（复用）。

### 领取：ABI 672（2026-08-27，§6 相位裁剪队列 S0）
- **QUEUE_EXTRA_PHASES**（tag 695）：`queue_extra_phases(player, phases_mask, self_skill)`——裁剪额外回合：仅执行 mask 置位阶段（bit=1<<TurnPhaseTag），经 `turn.extra_phase_queue` 与 extra_turn_queue 同级消费；SKIP_*/pending_judge_skips 武装法。

### 领取：ABI 673（2026-08-28，tw twjieqiu 装备栏编排）
- **RESOLVE_TWJIEQIU**（tag 696）：`resolve_twjieqiu(player, target, subaction, self_skill)`——多路器：sub0 选目标禁全六槽+摸 k 张（k=废除前装备区牌数）+BUFF 标记；sub2 余禁用栏判持有者额外回合。配套过滤器 `owner_alive_event_target_twjieqiu_buff_positive_<mark>`。

### 领取：ABI 671（2026-08-27，shenhua 放权目标插相）
- **CHOOSE_OTHERS_INSERT_EXTRA_TURN**（tag 694）：`choose_others_insert_extra_turn(player, check, self_skill)`——选一名其他角色入队 extra_turn_queue（check 非零同时登记尾随代价标记）。消费者 shenhua fangquan2 放权。

### 领取：ABI 670（2026-08-27，政略①双授分支）
- **CHOOSE_OTHERS_ADD_MARK**（tag 693/Cont 280）：`choose_others_add_mark(player, mark, delta, max_count, self_skill)`——选 1..max_count 名其他存活角色各获 delta 标记。配套过滤器对 `owner_alive_event_player_lord_{no_damage,damaged}`。

### 领取：ABI 666（2026-08-27，jsrg 蓄谋族切片 3b）
- **PLACE_DECK_TOP_XUMOU**（tag 687）：`place_deck_top_xumou(player, self_skill)`——牌堆顶实体牌移入判定区作蓄谋牌（殆谋「他人」分支；真实移动语义）。
- **CHOOSE_PLACE_HAND_XUMOU**（tag 688/Cont 278）：`choose_place_hand_xumou(player, self_skill)`——选一张手牌移入判定区作蓄谋牌，cancel=停止（遏前①单次近似）。
- **DISCARD_ONE_OWN_XUMOU**（tag 689）：`discard_one_own_xumou(player, self_skill)`——确定性弃置判定区最后一张蓄谋牌（殆谋「我」分支）。
- 配套 TriggerFilter×2：`owner_alive_other_targeted_by_sha`、`owner_alive_targeted_by_other_sha_judge_xumou`。

### 领取：ABI 668（2026-08-27，城族 S2b 沙城消耗）
- **CITY_CONSUME**（tag 691/Cont 279）：`city_consume(player, self_skill)`——移去「城」堆一张并令一名本回合失去过牌的其他角色摸 min(5, 失去牌数)。配套 SkillPileRef.wants_lost_count opt-in 计数 + TriggerFilter `owner_alive_sha_resolved_city_<def>`。

### 领取：ABI 669（2026-08-27，猎族 S3 额外回合）
- **QUEUE_EXTRA_TURN**（tag 692）：`queue_extra_turn(player, self_skill)`——入队引擎既有 extra_turn_queue 并登记尾随 check 标记；该额外回合结束时若 DAMAGE_DEALT==0 则失去 1 点体力（结算位于 continue_turn_advance 头部）。

### 领取：ABI 667（2026-08-27，城族 S2a 筑城）
- **PLACE_DECK_TOP_EXPANSION**（tag 690）：`place_deck_top_expansion(player, count, self_skill)`——牌堆顶 count 张实体牌明置移入自己「城」扩张堆（沙城 build 子技；扩张堆按需创建、以技能 def 键控）。

### 领取：ABI 665（2026-08-26，jsrg 蓄谋族切片 1）
- **PLACE_XUMOU**（tag 686）：`place_xumou(player, card, self_skill)`——将一张牌作为蓄谋牌置入判定区（特殊延时，可重复，结算=视为使用原牌）。消费者 jsrgeqian①/jsrgdaimou 等。

### 协调表状态
- 🔴 区无活跃项（mbhanyong 已于 2026-08-06 21:2x 闭环——stale 测试，非引擎回归）。
- A 段 0624+（TrickChiling 半落地）已由 C 段补 match 修编译——若 A 段继续锦囊批次请先跑全量确认无新破坏。
- 共享 target 已按 §37.10 清理（58G 释放）。
| 678 | `ChooseHandCompareGrantTargetOnWin(player, win_grant_def, lose_skill_def, skill)`：拼点（复用 Cont 75/76/49，payload 尾 flag=1），发起者严格胜→**目标**挂 win_grant_def（AddSkill 到 target 侧）；负/平→无操作。批 166 tanhu 探虎（offline）消费；同步修正批 152/154「拼点 win/lose 挂钩语义缺口」过时登记——旧原语胜负挂钩（胜→AddBoundSkill 绑 target 挂发起者侧/负→AddSkill 发起者侧）早已存在，本批仅补「胜→挂目标侧」参数化 | A (ZCode) | 2026-09-05 | 领取：host bump 678 待落地 |
| 679 | `GiveSelectedThenRecoverSelf(player, target, amount)`：active select he 2..255 交给目标后**使用者**回复 amount 点体力（批 171 立项书「交牌回自己族」首原语，jun_renwang 君仁望消费；tag 702，selected_cards 注入同 GIVE_SELECTED_THEN_RECOVER_TARGET）| A (ZCode) | 2026-09-06 | 领取：host bump 679 待落地 |
| 680 | `ChooseHandCompareBetween(player, first, skill)`：第三方拼点——first 选一名对手（排除 player），双方各选一张手牌拼点（复用 Cont 75/76/49，payload 尾 flag=2）；非平局时**胜方对败方**使用无距离虚拟【杀】（近似：上游 user.chooseUseTarget 自选目标，引擎固定以败方为目标）；拼点牌入弃牌堆（批 171 组合原语批，jun_zhaoshou 昭算消费；tag 703）| A (ZCode) | 2026-09-06 | 领取：host bump 680 待落地 |
| 681 | `GuanxingTop(player, count, skill)`：观看牌堆顶 count 张并逐张二选（置于牌堆顶序列/置于牌堆底），完成后重排落位（观星族 pxlangya/kamome_huanmeng/xinfu_dianhua 消费；逐张二选近似上游拖拽任意排列——堆内自由序未建模，tag 704，Cont 289 GuanxingPick 多轮）| A (ZCode) | 2026-09-06 | 领取：host bump 681 待落地 |
| 682 | `ChooseOtherGiveThenRecover(player, skill)`：选一名其他有手牌角色，其交给你一张手牌，然后**其**回复 1 点体力（psdaohe 踏鹤消费——「目标交牌+目标自回」方向；Cont 290 单段选目标，交牌复用 DrawThenGiveOne ready_to_give 链）| A (ZCode) | 2026-09-08 | 领取：host bump 682 待落地 |
| 683 | `GuanxingTopByFaluMarks(player, spade, heart, club, diamond, skill)`：数四法箓标记中非零种数 N，观星牌堆顶 N 张（逐张二选同 681）；配合 4 个参数化过滤器 `owner_discard_suit_no_own_mark_<suit>`（cards_moved to discard + 含花色 X 牌 + owner 无 X 标记 → add_mark(X)）——xinfu_falu 花色化标记族（批 175 立项；tag 706，Cont 复用 289）| A (ZCode) | 2026-09-08 | 领取：host bump 683 待落地 |
| 684 | `DiscardSkillPile(player, skill)`：自身技能堆（武将牌上）全部牌移入弃牌堆（expansion S1-B，ADR 0641 §8.2 铸造；rebiaozhao3 型 63 件消费池；tag 707，纯同步零选择续体）| A (ZCode) | 2026-09-09 | 领取：host bump 684 随本批落地 |
| 685 | `PlaceTopDeckToExpansion(player, skill, pile_def, count)`：牌堆顶 count 张置入自身 (player, pile_def) 技能堆（expansion P2-var，ADR 0641 §8.7；jibing 集兵/shiroha_yuzhao 兆/zyexin 权消费；tag 708，纯同步零选择续体）| A (ZCode) | 2026-09-10 | 领取：host bump 685 随本批落地 |
| 687 | `GainWholeSkillPile(player, skill, pile_def)`：自身 (player, pile_def) 技能堆全部牌置入手牌（expansion S1-B 深化，ADR 0641 §8.7；zhenwei2/tsumugi_huilang2 消费；tag 710，纯同步零选择续体）| A (ZCode) | 2026-09-10 | 领取：host bump 687 随本批落地 |
| 688 | `ChooseTargetHandCardToPile(player, skill, pile_def)`：选一名其他有手牌角色，将其一张手牌置入 (player, pile_def) 技能堆（expansion A1 他人置入，ADR 0641 §8.7；olquanji 权计等消费；tag 711，两段选择续体 Cont 294）| A (ZCode) | 2026-09-10 | 领取：host bump 688 随本批落地 |
| 690 | `RearrangePileToDeckTop(player, skill, pile_def, count)`：从自身技能堆选 count 张牌置牌堆顶（chooseToMove 堆重排族，ADR 0641 §8.8；41 件触达；tag 713，选择续体 SelectCards count/count）| A (ZCode) | 2026-09-10 | 领取：host bump 690 随本批落地 |
| 691 | `ChooseHandCardsToSkillPile(player, skill, min_count, max_count)`：变张置入——从自身手牌选 [min,max] 张置入自身技能堆（可取消=不置入；A0 置入面，ADR 0641 §8.12；tunchu 屯储/ymfriendyance/tsumugi_huilang 回浪 3 件消费达门槛；tag 714，选择续体 SelectCards min/max + Cont 298）| A (ZCode) | 2026-09-10 | 领取：host bump 691 随本批落地 |
| 692 | `ResolveDcbingji(player, skill, stage, kind)`：秉纪专属状态机——手牌同花色+本回合该花色未用校验 → 虚拟杀/桃双选 → 选目标使用并记花色史（经典转化簇旗舰，ADR 0641 §8.30；resolve_xinxingluan/ABI 476 工艺；tag 715，Custom 续体 Cont 300）| A (ZCode) | 2026-09-10 | 领取：host bump 692 随本批落地 |
| 693 | `ResolvePxqixian(player, skill, stage, kind, material, target)`：祁县专属状态机——重铸一张装备牌（置入弃牌堆）并视为对一名其他角色使用虚拟【杀】/【过河拆桥】（经典转化簇第二件，ADR 0641 §8.31；dcbingji 工艺复用；tag 716，7 槽 Custom 续体 Cont 301）| A (ZCode) | 2026-09-10 | 领取：host bump 693 随本批落地 |
| 694 | `ResolveQuanjia(player, skill, source, target)`：拳甲专属状态机——来源为当前回合角色的伤害事件后，随机取样来源 (2+标记) 张手牌，有【杀】则对其使用虚拟杀，否则获得标记（经典转化簇第三件，ADR 0641 §8.32；RNG Fisher–Yates 取样；tag 717，单发无续体）| A (ZCode) | 2026-09-11 | 领取：host bump 694 随本批落地 |
| 695 | `ResolveDuwang(player, skill)`：度望专属状态机——游戏开始时将牌堆顶前 5 张非【杀】牌置于自身技能堆（距离修正面保守省略；newjiang 转化簇，ADR 0641 §8.34；tag 718，单发无续体）| A (ZCode) | 2026-09-11 | 领取：host bump 695 随本批落地 |
| 696 | `ResolveJsrgdaoren(player, skill)`：蹈仁专属状态机——交一张手牌给一名其他角色，对双方攻击范围交集内所有角色各造成 1 点无来源伤害（jsrg 转化/交牌簇，ADR 0641 §8.35；tag 719，6 槽续体 Cont 302）| A (ZCode) | 2026-09-11 | 领取：host bump 696 随本批落地 |
| 697 | `ResolveQianlong(player, skill)`：潜龙专属状态机——受伤后观牌堆顶三张，获得至多已损失体力值张（SelectCards 0..cap 可取消），余者置牌堆底（xianding 转化簇，ADR 0641 §8.36；tag 720，7 槽续体 Cont 303）| A (ZCode) | 2026-09-11 | 领取：host bump 697 随本批落地 |
| 698 | `ResolveJsrgzuozhan(player, skill)`：坐瞻专属状态机——游戏开始时选至多两名其他角色，射程标记 = min(所选最高体力, 5) 加于自身（attack_range add owner_skill_mark 消费），所选角色各记「坐瞻」标记（jsrg 转化/开局簇，ADR 0641 §8.37；tag 721，单发含一段 SelectTargets 续体 Cont 304）| A (ZCode) | 2026-09-12 | 领取：host bump 698 随本批落地 |
| 699 | `ResolveJsrgzuozhanGain(player, skill, event_player)`：坐瞻组技补牌状态机——「坐瞻」标记角色死亡后，令一名存活的坐瞻角色从弃牌堆获得至多 X（射程标记）张牌名各不相同的基本牌（jsrgzuozhan 组技面，ADR 0641 §8.38；tag 722，选目标续体 Cont 305）| A (ZCode) | 2026-09-12 | 领取：host bump 699 随本批落地 |
| 700 | `ResolveTwmouzhi(player, skill, mode, event_card)`：谋制专属状态机——mode 0（damage_before）：事件牌有颜色且与记录的上次受伤牌颜色相同则防止本次伤害（防止不更新记录）；mode 1（damage）：记录事件牌颜色（红/黑，无牌或无色清除）。上次颜色存于技能 content_storage（tw 防护簇，ADR 0641 §8.39；tag 723，单发无续体）| A (ZCode) | 2026-09-12 | 领取：host bump 700 随本批落地 |
| 701 | `ResolveTwdaigui(player, skill, stage, count)`：待归多段状态机第一批（slice 1）——出牌阶段结束时手牌同色门控（机内权威校验），选 [1, 手牌数] 名任意角色（可含自己），亮出牌堆底等量牌并入弃牌堆暂存（逐目标选获面留待 slice 2 接续，Cont 306 续体在 stage 2 前后扩展；tw 多段选择链专项，ADR 0641 §8.40；tag 724）| A (ZCode) | 2026-09-12 | 领取：host bump 701 随本批落地 |
| 702 | `ResolveTwmitu(player, skill, stage, count)`：迷途多段拼点链状态机 slice 1——准备阶段可选至多 3 名任意角色（上游无已受伤过滤，代码语义），各摸 2 张打「迷途」gaintag（blake2b("twmitu")=3195907057，content_card_tags）并公开亮出；拼点轮与崩溃面留 slice 2+（Cont 307 轮次扩展；tw 多段选择链专项，ADR 0641 §8.41；tag 725）| A (ZCode) | 2026-09-12 | 领取：host bump 702 随本批落地 |
| 703 | `ResolveQiantun(player, skill, stage)`：谦吞专属状态机——出牌阶段限一次，令一名其他角色展示任意张手牌（只能用这些牌拼点），与其拼点：赢则获得其展示牌、负/平则获得其其余手牌（jsrg 拼点族，ADR 0641 §8.42；tag 726，Cont 308 展示/选拼点牌分段续体）| A (ZCode) | 2026-09-12 | 领取：host bump 703 随本批落地 |
| 704 | `ResolveYangming(player, skill, stage)`：养名循环拼点状态机——出牌阶段限一次，与一名角色拼点；目标赢→目标摸「已输次数」张+owner 回 1 体力结束；否则计输+owner 选择是否重复（循环拼点，机内拼点子第三次复用；jsrg 拼点族，ADR 0641 §8.43；tag 727，Cont 309 循环续体）| A (ZCode) | 2026-09-12 | 领取：host bump 704 随本批落地 |
| 705 | `ResolveCanyun(player, skill, stage)`：残殒专属状态机——出牌阶段弃置一张牌，选 1-2 名其他角色比较装备数：owner 多则目标回 1、相等则目标摸 1、目标多则目标失 1 体力；目标体力为 1 时 owner 摸 1（sp2 装备比较族，ADR 0641 §8.44；tag 728，Cont 310 分段续体）| A (ZCode) | 2026-09-12 | 领取：host bump 705 随本批落地 |
| 707 | `ResolveBiaozhao3(player, skill, stage)`：表召·归状态机——准备阶段若「表」堆有牌：弃置首张，令一名角色（可选自己，可取消）摸至全场最大手牌数（至多 5）并回复 1 体力；全场最大手牌数机内计算（sp2 表召族消费面，批 216 登记债清偿，ADR 0641 §8.46；tag 730，Cont 311）| A (ZCode) | 2026-09-12 | 领取：host bump 707 随本批落地 |
| 708 | `ResolveChuanyun(player, skill, event_player, event_card)`：穿云专属状态机——杀指定目标后若其有装备：随机弃置其一（RNG Fisher–Yates 既有工艺；sp2 衍生技族，ADR 0641 §8.47；tag 731，单发无续体）| A (ZCode) | 2026-09-13 | 领取：host bump 708 随本批落地 |
| 709 | `ResolveBiaozhao2(player, skill, event_player)`：表召·应状态机——任意角色弃置与「表」同花色同点数的牌时：将「表」交给弃牌者，owner 失去 1 体力（新参数化过滤器 owner_alive_discard_matches_own_pile_card_<def>，零 ABI；sp2 表召族收官，ADR 0641 §8.48；tag 732，单发无续体）| A (ZCode) | 2026-09-13 | 领取：host bump 709 随本批落地 |
| 710 | `ResolveDcniji(player, skill, mode, event_card)`：逆击专属状态机——成为非装备牌目标后摸 1 张并打「逆击」标记（blake2b("dcniji")）；结束阶段弃置手牌中全部标记牌（可先使用其一未建模；sp2 gaintag 族，ADR 0641 §8.49；tag 733，双触发共调单发，twmouzhi 模式）| A (ZCode) | 2026-09-13 | 领取：host bump 710 随本批落地 |
| 713 | `ResolveChumei(player, skill, stage)`：狐魅专属状态机——出牌阶段，选一名体力≤owner 本回合造成伤害总数的角色，执行一项（摸 1/给 owner 一张牌/回 1，per-activation 一次；damage_dealt_count 机内直读；sp2 狐魅族，ADR 0641 §8.52；tag 736，Cont 314 分段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 713 随本批落地 |
| 714 | `ResolveQinmian(player, skill, mode, amount)`：勤勉状态机——准备阶段/受伤后摸 1 张打「勤勉」标记（按伤害点数多次）；全场结束阶段可弃全部标记牌并对当前回合角色造成等量（全手牌皆标记）/1 点伤害（sp2 同构 dcniji+伤害值源；ADR 0641 §8.53；tag 737，双触发共调单发）| A (ZCode) | 2026-09-13 | 领取：host bump 714 随本批落地 |
| 715 | `ResolveShuaiyan(player, skill, stage)`：率言状态机——弃牌阶段开始，若手牌>1：选一名其他角色，其交给你一张牌（手+装备；mobile 简单主动族，ADR 0641 §8.54；tag 738，Cont 315 分段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 715 随本批落地 |
| 718 | `ResolveMbbeiming(player, skill, stage)`：孛明状态机——游戏开始时选至多两名角色，各获得牌堆中一张攻击范围=其手牌花色数的武器牌（武器按 weapon_range 检索；mobile 开局族，ADR 0641 §8.57；tag 742，Cont 318）| A (ZCode) | 2026-09-13 | 领取：host bump 718 随本批落地 |
| 719 | `ResetLinkedSelectedTargets(source, targets[])`：断索主动原语——出牌阶段限一次，重置任意名连环状态角色后对各造成 1 点火焰伤害（先全体重置再全体伤害两遍循环，重置后伤害不经链传导；mobile，ADR 0641 §8.58；tag 743）| A (ZCode) | 2026-09-13 | 领取：host bump 719 随本批落地 |
| 720 | `ResolveGaoyuan(player, source, card)`：告援杀目标转移状态机——成为【杀】目标时可弃一张牌，将此杀转移给一名有「诤」标且为此杀合法目标的其他角色（candidates=source 射程内合法目标∩有诤标；复用 redirect_saved_sha_target 提交；mobile 杀目标族，ADR 0641 §8.59；tag 744，Cont 319 两段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 720 随本批落地 |
| 721 | `ResolveDuoduan(player, source, card)`：度断杀目标多段状态机——成为【杀】目标（每回合一次）可重铸一张牌，再选一项：①使用者摸 2 且此杀对 owner 无效（CancelIncomingCardUse）；②使用者弃一张牌且 owner 不可响应（make_saved_sha_target_unrespondable）（复用 RecastCards/取消/不可响应三既有件；mobile 杀目标族，ADR 0641 §8.60；tag 745，Cont 320 三段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 721 随本批落地 |
| 722 | `ResolveDcanxu(player, targets[])`：暗蓄简单主动状态机——出牌阶段限一次，选两名手牌数不同的角色，手牌少者从多者处选得一张手牌；所得牌非黑桃则 owner 摸 1，此后两人手牌数相等则 owner 回 1（refresh 简单主动族，ADR 0641 §8.61；tag 746，Cont 321 单段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 722 随本批落地 |
| 723 | `ResolveChulao(player, targets[])`：黜老简单主动状态机——出牌阶段限一次，弃一张牌（黑桃则 owner 摸 1），对任意名势力各异角色逐个由 owner 选弃其一张牌（黑桃则该目标摸 1）（refresh 简单主动族，ADR 0641 §8.62；tag 747，Cont 322 逐目标循环续体）| A (ZCode) | 2026-09-13 | 领取：host bump 723 随本批落地 |
| 724 | `ResolveFriendbiwei(player, target, card, skill)`：鄙位点数比较状态机——出牌阶段限一次，弃手牌中点数最大的一张，令一名有手牌的其他角色弃置所有点数≥该牌的手牌；未弃任何牌则重置技能出牌阶段使用计数（mobile 点数比较族，ADR 0641 §8.63；tag 748，单发无续体）| A (ZCode) | 2026-09-13 | 领取：host bump 724 随本批落地 |
| 725 | `ResolveMbzhenann(player)`：镇南杀目标族状态机——成为其他角色使用的锦囊目标（目标数>max(1,使用者体力)）时可弃一张牌并对任意角色造成1点伤害（targets 数门槛为新过滤器面；mobile 杀目标族，ADR 0641 §8.64；tag 749，Cont 323 两段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 725 随本批落地 |
| 726 | `ResolveMeiyong(player)`：姝勇状态机——使用/打出【杀】时可获得一名其他角色 hej 区一张牌（owner 选牌），然后其摸 1（双触发 card_used/card_responded+双新过滤器 sha 门；mobile 杀目标族，ADR 0641 §8.65；tag 750，Cont 324 两段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 726 随本批落地 |
| 727 | `ResolveQiangyong(player, target, skill)`：羌勇锁定状态机——【杀】造成伤害时弃置受伤角色 X 张牌（X=本回合 owner 用【杀】次数=SHA_PLAYED 旗），若其无手牌此伤害 +1（ModifyCurrentDamage 既有件；mobile 点数/伤害门控族，ADR 0641 §8.66；tag 751，Cont 325 单段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 727 随本批落地 |
| 728 | `owner_phase_end_lost_cards` 触发过滤器：镇抚门槛——own 结束阶段且本回合失去过牌（card_loss_count>0；回合计数超集近似，登记债；mobile，ADR 0641 §8.67；零 bytecode，manifest ABI 门控）| A (ZCode) | 2026-09-14 | 领取：host bump 728 随本批落地 |
| 729 | `owner_is_target_damage_amount_gt_one` 触发过滤器：照月狮子盔 arm1——owner 为伤害目标且伤害量>1（配合既有 modify_current_damage DSL 实现伤害改 1；arm2 失去装备触发因卡牌不在牌池登记债；mobile，ADR 0641 §8.68；零 bytecode，manifest ABI 门控）| A (ZCode) | 2026-09-14 | 领取：host bump 729 随本批落地 |
| 730 | `ResolveDingce(player, source)`：定策伤害后续状态机——受到伤害后弃自己一张手牌，再令伤害来源弃一张手牌（owner 代选）；同色分支（虚拟【洞烛先机】）因卡牌不在牌池登记债（jsrg 伤害后续族，复用 owner_is_target_source_alive 过滤器；ADR 0641 §8.69；tag 752，Cont 326 两段续体）| A (ZCode) | 2026-09-14 | 领取：host bump 730 随本批落地 |
| 731 | `pending_bound_damage_deltas`（Resolution 回合态字段 + increase/reduce 暂存回退 + defer_damage_tail 套用）：damage_before 触发执行期的 bound 增减暂存桥——修复批 280 在册「触发期部分减伤/伤害改值时序」缺口（mobile rw_baiyin 迁移落地，ADR 0641 §8.68 更新；无 bytecode，manifest ABI 门控）| A (ZCode) | 2026-09-14 | 领取：host bump 731 随本批落地 |
| 732 | `ResolveDcqiaolve(player)`：巧略弃牌代价补全状态机——用牌指定目标后先弃自己一张牌（强制），再取事件目标一张随机手牌（首现过滤/多目标仍为登记债；newjiang，ADR 0641 §8.71；tag 753，Cont 327 单段续体）| A (ZCode) | 2026-09-15 | 领取：host bump 732 随本批落地 |
| 733 | `ResolveTwxiongjun(player)`：雄军批量摸牌状态机——owner 造成伤害后，每名拥有〖雄军〗的角色各摸一张（按 def 遍历存活持有者；tw 锁定触发，ADR 0641 §8.72；tag 754，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 733 随本批落地 |
| 734 | `ResolveYixiang(player)`：义襄状态机——每名角色回合限一次，成为使用牌目标且来源体力>自身时，随机获得牌堆中一张自己没有的基本牌（单发无续体；mobile，ADR 0641 §8.73；tag 755）| A (ZCode) | 2026-09-15 | 领取：host bump 734 随本批落地 |
| 716 | `ResolveZhengjian(player, skill, stage, event_player)`：诤荐状态机——结束阶段选一名角色授「诤」标；其用/打出牌时标+1；owner 回合开始：各有标角色摸 min(5,上限,标数) 张并清标（mobile 标记族，ADR 0641 §8.55；tag 739，Cont 316）| A (ZCode) | 2026-09-13 | 领取：host bump 716 随本批落地 |
| 717 | `ResolveLiangyin(player, skill, mode)`：良姻状态机——①牌被置于武将牌上时可令一名角色摸 1；②牌从武将牌上被获得时可令一名角色弃 1（两个静态过滤器 cards_moved_into/out_of_expansion，零 ABI；mobile 良姻，ADR 0641 §8.56；tag 741，分段续体 Cont 317）| A (ZCode) | 2026-09-13 | 领取：host bump 717 随本批落地 |
| 711 | `ResolveDcmoukui(player, skill, stage, target)`：谋溃状态机——杀指定目标后 chooseButton [1,2]：摸 1／弃目标一张牌（owner 从目标手牌选），可同选两项；未命中追罚面留 slice 2（sp2 杀目标族，ADR 0641 §8.50；tag 734，Cont 312 分段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 711 随本批落地 |
| 712 | `ResolveZhongyan(player, skill, stage)`：忠言状态机——出牌阶段限一次，亮牌堆顶三张，一名有手牌角色以其中一张与其一张手牌交换（chooseToMove 族，多段选择链；ADR 0641 §8.51；tag 735，Cont 313 三段续体）| A (ZCode) | 2026-09-13 | 领取：host bump 712 随本批落地 |
| 706 | `ResolveHengwu(player, skill, event_card)`：横骛专属状态机——使用/打出牌后，若手牌无此花色且场上有同花色装备：摸「场上该花色装备数」张（frequent 锁定自动；xianding 花色计数族，ADR 0641 §8.45；tag 729，单发无续体）| A (ZCode) | 2026-09-12 | 领取：host bump 706 随本批落地 |
| 689 | `PlaceHandCardsToDeckTop(player, skill, count)`：选 count 张手牌置牌堆顶（chooseToMove 堆重排族首原语，ADR 0641 §8.8；41 件触达；tag 712，选择续体 SelectCards count/count）| A (ZCode) | 2026-09-10 | 领取：host bump 689 随本批落地 |
| 735 | `ResolveXinfuQiai(player)`：求援濒死交牌循环状态机——限定技，owner 濒死时其他角色依座次各强制交给 owner 一张手/装备牌（无牌者跳过；复用 owner_dying_zhupeilan_once_per_game 过滤器零新 filter；sp 濒死族，ADR 0641 §8.75；tag 756，Cont 328 giver 循环续体）| A (ZCode) | 2026-09-15 | 领取：host bump 735 随本批落地 |
| 736 | `ResolveOlwenren(player, targets[])`：温仁条件多目标摸牌状态机——出牌阶段限一次任选任意名目标，每目标无手牌摸 1、手牌不大于 owner 再摸 1（可叠 2 可摸 0；uses_per_turn 1 权威限次；vm 臂 host.selected_targets() 注入；sp 简单主动族，ADR 0641 §8.76；tag 757，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 736 随本批落地 |
| 737 | `ResolveSpolaige(player)` + `owner_alive_second_global_dying` 过滤器 + `lifetime_dying_events` 全局计数器：霸业觉醒状态机——全局第二次濒死结算后失去〖西向〗获得〖朔北〗并摸/回复至体力上限（计数==2 天然限一次；差值摸/回零新原语；sp 觉醒换技族，ADR 0641 §8.77；tag 758，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 737 随本批落地 |
| 738 | `ResolveOlzhijie(player)` + `owner_used_and_was_targeted_this_turn_in_range` 过滤器 + `card_targeted_count` track：直谏结束阶段射程伤害状态机——任意角色结束阶段，若 owner 本回合用过/打出过牌且被指定为目标过且射程内有人：对射程内一名角色造成 1 点伤害（三条件过滤器+射程选目标续体；sp 直谏族，ADR 0641 §8.78；tag 759，Cont 329）| A (ZCode) | 2026-09-15 | 领取：host bump 738 随本批落地 |
| 739 | `ResolveOlsuyi(player, target)`：俗义牌堆装备检索组合状态机——出牌阶段限一次选一名其他角色：牌堆检索其可装备的装备牌令其获得并装备（equip_slot_enabled 门控+use_equipment_from_hand），再对其视为使用杀（UseVirtualCardOnSelectedTargets 完整结算；随机检索以顺序近似登记债；sp 组合机器族，ADR 0641 §8.79；tag 760，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 739 随本批落地 |
| 740 | `ResolveOlqiaozhi(player, stage)`：巧织亮顶选获状态机——弃一张牌，亮牌堆顶两张选获其一打「巧织」gaintag（blake2b=1909178934），未选入弃；手牌有巧织牌机内门控不可发动（亮牌/gaintag 族复用，ADR 0641 §8.80；tag 761，Cont 330 两段续体）| A (ZCode) | 2026-09-15 | 领取：host bump 740 随本批落地 |
| 741 | `owner_basic_uses_plus_one_cards` view_as 动态张数原语 + `lifetime_basic_card_uses` 全局计数器：转换素材最少 1+本局基本牌使用数张、上限不封顶（五层接线~40 点照 owner_hp_min_one_cards 同型；track 门控 opt-in；解锁西向 spolxixiang approx→complete；ADR 0641 §8.81；零 bytecode）| A (ZCode) | 2026-09-15 | 领取：host bump 741 随本批落地 |
| 742 | `at_least N` view_as 不封顶张数语法 + 255 哨兵校验豁免：垦伤 rekenshang/olkenshang 张数面收窄（min=N/max 不封顶；typecheck+registry 哨兵豁免；改目标 effect 子技留债；ADR 0641 §8.82；零 bytecode，manifest ABI 门控）| A (ZCode) | 2026-09-15 | 领取：host bump 742 随本批落地 |
| 744 | `once_per_turn_per_branch` view_as 每规则每回合限次原语：实体转换适用，规则索引位记账（复用 view_as_use_round/used_branches 位图零新状态）；清偿西向「每种牌名每回合限一次」债（ADR 0641 §8.84；零 bytecode，manifest ABI 门控）| A (ZCode) | 2026-09-15 | 领取：host bump 744 随本批落地 |
| 745 | `hand_missing_gaintag <u32>` active 命令级 gaintag 门控原语：手牌含携带该 gaintag 的牌时发动被拒（五层接线；清偿巧织按钮可见性债；ADR 0641 §8.85；零 bytecode，manifest ABI 门控）| A (ZCode) | 2026-09-15 | 领取：host bump 745 随本批落地 |
| 746 | `ResolveDczhee(player, mode)`：闪电挂判定状态机——开局牌堆/每结束阶段弃牌堆检索【闪电】挂为判定牌（判定区唯一性门控；TrickShandian + MoveCards→Judge，判定结算由引擎管道承接；xianding 闪电技，ADR 0641 §8.86；tag 763，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 746 随本批落地 |
| 747 | `ResolveDchaoxian(player)`：朝献点数检索收集状态机——限定技，弃牌堆全部 3 点牌洗入牌堆底（随机插入以堆底近似）+ 收齐其他角色手牌 3 点牌（rank 扫描 + zone reorder 工艺；xianding 谋杨奉，ADR 0641 §8.87；tag 764，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 747 随本批落地 |
| 748 | `ResolveZhongjie(player)`：张介死亡赠礼状态机——死亡时选一名其他角色 +1 体力上限/回复 1/摸 1（death 触发无 alive 检查承接 forceDie；Cont 332 选目标续体；sp2 张宁，ADR 0641 §8.88；tag 765）| A (ZCode) | 2026-09-15 | 领取：host bump 748 随本批落地 |
| 749 | `owner_source_in_turn_target_unharmed_card_actions_ge_target_hp` 威临三条件过滤器：自己回合内对未受伤目标伤害+1（damage_history+card_action_totals 值源；modify_current_damage 既有 DSL；**须配 codegen trigger filter 门**否则被 677 短路遮蔽；sp2 SP群陈到，ADR 0641 §8.89；零 bytecode）| A (ZCode) | 2026-09-15 | 领取：host bump 749 随本批落地 |
| 750 | `ResolveStarcuxiaWeiming(player, source, variant)` + 双过滤器：粗遐/未命孪生随机弃牌状态机——使用牌者随机弃 1 手牌，条件复验 owner 摸 1（RuleRng shuffle；CardTargeted 逐目标 ctx；历史粒度放宽为债；sp2 星文丑/星丁奉，ADR 0641 §8.90；tag 766，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 750 随本批落地 |
| 751 | `ResolveSanshou(player)` + `lifetime_used_type_groups` 类型位图：三首防伤状态机——亮顶三张存在本局未使用类型则 PreventDamage 当前伤害（CardUsed 排水置位 skip-if-zero；forced 化承接上游 AI check；机器侧直读 current_damage_id；xianding 三首，ADR 0641 §8.92；tag 767，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 751 随本批落地 |
| 752 | `ResolveNhlundao(player, source)`：论道手牌比较双分支状态机——受伤后来源存活且手牌数不同：手牌多摸 1，否则 owner 从来源 he 区强制选弃 1（机器内 gate 零新 filter；选弃续体 dcmoukui 工艺；sp2 南华老仙，ADR 0641 §8.93；tag 768，Cont 333）| A (ZCode) | 2026-09-15 | 领取：host bump 752 随本批落地 |
| 753 | `ResolveXiangshu(player, targets[])`：襄戍限定支援状态机——结束阶段令一名受伤角色回复 X/摸 X（X=min(5, 本回合造成伤害事件数)；once_per_game+wounded spec；selected_targets 注入工艺；sp2 朱灵，ADR 0641 §8.94；tag 769，单发无续体）| A (ZCode) | 2026-09-15 | 领取：host bump 753 随本批落地 |
