//! PA12 迁移批⑨（2026-11-06，零版本号变更）：
//! 1. `rvm_ops` 拆分 `commit.rs` 子模块并接入 9 个既有 ABI（<= 759）确定性效果原语（tags 49..=57）：
//!    - `IncreaseCurrentDrawCount`（tag 49）：累加当前摸牌阶段额外摸牌数（`DrawPhaseMachine` `DP::Begin` 消费）；
//!    - `SkipNextPlayPhase`（tag 50）：置当前回合角色 `turn.skip_play_phase = true`；
//!    - `SkipNextDiscardPhase`（tag 51）：置目标角色 `skip_discard_phase = true`（`begin_discard` 记录跳过，
//!      `intrinsic_machine` 消费并清零以抑制 `DiscardToLimitMachine`）；
//!    - `SkipNextJudgePhase`（tag 52）：置 `(p, "skip_judge") = 1`（下次判定阶段 `intrinsic_machine` 消费并记跳过）；
//!    - `SkipJudgeAndDrawPhases`（tag 53）：同时置 `skip_judge` 与 `turn.skip_draw_phase = true`；
//!    - `SkipRemainingPhases`（tag 54）：按 `phases_mask`（bit2=Draw、bit3=Play、bit4=Discard）置跳过位；
//!    - `QueueExtraPhases`（tag 55）：按 `phases_mask`（tags 1..=5 对应 Judge..=End）入队 `turn.extra_phases`；
//!    - `GainTargetEquipment`（tag 56）：按槽位编码（1=武器、2=防具、3=+1马、4=-1马、5=宝物）获得目标装备；
//!    - `RefundCurrentShaUse`（tag 57）：重置当前回合角色 `turn.sha_used_play_phase = false`。
//! 2. `content-native-loader` 与 `rvm_bridge` 接入 3 个触发事件（及 `extra_events` 传播）：
//!    - `"damage_prevented"` → `DamageCancelled` + `extra_events: [DamageZero]`；
//!    - `"character_state_changed"` → `TurnOvered` + `extra_events: [Linked]`，补齐
//!      `OwnerTurnedOver`、`OwnerBecameFaceUpOrLinked`、`OwnerTurnChangedOrBecameLinked`；
//!    - `"skill_added"` → `SkillToggled`（仅 `enabled: true` 触发），补齐 `OwnerSkillAddedSelf(u32)` 与
//!      `EffectRequest::AddSkill` 在存在匹配监听技能时发射 `SkillToggled`。
//! 3. `rvm_named` 补齐 `"event_player_alive"`、`"delayed_trick"`、`"pending_judge_skip"`、
//!    `"first_sha"`（合取 `CardSha` 且 `CardUsed` 计入 `hits`）、`"source_has_weapon"`、`"sha_dodged"`/
//!    `"dodged"`；`GainEventCards` 支持从 `table` 回收正在结算的实体牌，`Ctx::to_discard` 增补跨区唯一性守卫。

use content_compiler::compile_source_named;
use engine_native::catalog::EquipSlot;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{CardId, EffectRequest, PlayerId, SkillId};

const STDFENYIN: &str = include_str!("../../../content/original/ported/sixiang/stdfenyin.skill");
const OLDZISHOU: &str = include_str!("../../../content/original/ported/old/oldzishou.skill");
const SPYINJU2: &str = include_str!("../../../content/original/ported/shiji/spyinju2.skill");
const ZHENGJING2: &str = include_str!("../../../content/original/ported/mobile/zhengjing2.skill");
const ZHUANGSHU_TRICK: &str =
    include_str!("../../../content/original/ported/sp/zhuangshu_trick.skill");
const ZJ_FULI: &str = include_str!("../../../content/original/ported/offline/zj_fuli.skill");
const XINDUODAO: &str = include_str!("../../../content/original/ported/mobile/xinduodao.skill");
const DCYONGJUE: &str = include_str!("../../../content/original/ported/xianding/dcyongjue.skill");
const OLDHUXIAO: &str = include_str!("../../../content/original/ported/sp/oldhuxiao.skill");
const RENZHENG: &str = include_str!("../../../content/original/ported/xianding/renzheng.skill");
const STDRENZHENG: &str =
    include_str!("../../../content/original/ported/sixiang/stdrenzheng.skill");
const FAEN: &str = include_str!("../../../content/original/ported/yijiang/faen.skill");
const REFAEN: &str = include_str!("../../../content/original/ported/refresh/refaen.skill");
const PXYANMEN: &str = include_str!("../../../content/original/ported/sp/pxyanmen.skill");
const PXBEIHAI: &str = include_str!("../../../content/original/ported/sp/pxbeihai.skill");
const ZJ_HANCI: &str = include_str!("../../../content/original/ported/offline/zj_hanci.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑨不引入新 ABI：{} > {}",
            pkg.manifest.rule_vm_abi_version,
            rule_vm::RULE_VM_ABI_VERSION
        );
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑨ 事件映射与回合/装备/杀次数确定性效果原语",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let rev = e.revision();
    let out = e.submit(GameCommand {
        actor,
        expected_revision: rev,
        dedup_id: (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64)
            | 1,
        action: action.clone(),
    });
    assert!(
        !matches!(out, engine_native::CommandOutcome::Rejected(_)),
        "命令不应被拒绝: actor={actor}, action={action:?}, out={out:?}"
    );
}

fn confirm(e: &mut Engine, actor: u8) {
    let ch = e.open_choice().cloned().expect("应有待答确认窗");
    assert_eq!(ch.observer, actor);
    assert_eq!(ch.spec.prompt, "trigger_confirm");
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: ch.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
}

/// 1. 批⑨新增放行的 9 个 `EffectRequest` 变体（tags 49..=57）全部通过 `accept_effect` 与 `enc_effects`/`dec_effects` 往返。
#[test]
fn pa12_batch9_codec_new_9_effects_round_trip_and_validation() {
    let p1 = PlayerId::new(1);
    let p2 = PlayerId::new(2);
    let sk = SkillId::new(7);
    let c1 = CardId::new(11);

    let batch9_effects = vec![
        EffectRequest::IncreaseCurrentDrawCount {
            player: p1,
            count: 2,
        },
        EffectRequest::SkipNextPlayPhase { player: p1 },
        EffectRequest::SkipNextDiscardPhase { player: p2 },
        EffectRequest::SkipNextJudgePhase { player: p1 },
        EffectRequest::SkipJudgeAndDrawPhases {
            player: p1,
            skill: sk,
        },
        EffectRequest::SkipRemainingPhases {
            player: p1,
            phases_mask: 20,
            skill: sk,
        },
        EffectRequest::QueueExtraPhases {
            player: p1,
            phases_mask: 8,
            skill: sk,
        },
        EffectRequest::GainTargetEquipment {
            player: p1,
            target: p2,
            slot: 1,
        },
        EffectRequest::RefundCurrentShaUse {
            player: p1,
            card: c1,
            skill: sk,
        },
    ];

    for eff in &batch9_effects {
        accept_effect(eff, 4).unwrap_or_else(|e| panic!("accept_effect 应放行 {eff:?}: {e:?}"));
    }

    let mut buf = Vec::new();
    enc_effects(&mut buf, &batch9_effects);
    let decoded = dec_effects(&buf).expect("dec_effects 应完整解码 tags 49..=57");
    assert_eq!(decoded, batch9_effects);

    // 越界与零参校验：
    assert!(accept_effect(
        &EffectRequest::IncreaseCurrentDrawCount {
            player: p1,
            count: 0,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GainTargetEquipment {
            player: p1,
            target: p2,
            slot: 0,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GainTargetEquipment {
            player: p1,
            target: p2,
            slot: 6,
        },
        4
    )
    .is_err());
}

/// 2. `stdfenyin.skill`（四象·奋音）+ `oldzishou.skill`（旧自守）：
/// - `stdfenyin` 在摸牌阶段开始确认后执行 `increase_current_draw_count(self, 2)` + `add_skill(self, 3965200877)`，
///   使本次摸牌阶段共摸 2 + 2 = 4 张牌，并在回合结束触发 `stdfenyin_discard` 自移除；
/// - `oldzishou` 在受伤角色摸牌阶段开始确认后按已损体力增加摸牌数并 `skip_next_play_phase(self)`，
///   摸牌后直接跳过出牌阶段进入弃牌阶段。
#[test]
fn pa12_batch9_increase_draw_count_and_skip_next_play_phase_e2e() {
    // (a) stdfenyin:
    let pack = pack_for(&[("stdfenyin", STDFENYIN)], "pa12-batch9-stdfenyin");
    let mut e = Engine::new_with_extensions(2026110641, 2, &[&pack]).expect("装载 stdfenyin");
    e.debug_grant(1, "stdfenyin");

    // 推进至座位 1 摸牌阶段开始询问确认
    while !(e.open_choice().is_some() && e.open_choice().unwrap().observer == 1) {
        let cur = e.turn().current;
        e.debug_clear_hand(cur);
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let hand_before = e.debug_hand(1).len();
    confirm(&mut e, 1);
    assert_eq!(
        e.debug_hand(1).len(),
        hand_before + 4,
        "stdfenyin 应令摸牌阶段多摸 2 张（共 4 张）"
    );
    assert!(
        e.debug_has_skill(1, "stdfenyin_discard"),
        "stdfenyin 应授予子技能 stdfenyin_discard(3965200877)"
    );

    // (b) oldzishou:
    let pack2 = pack_for(&[("oldzishou", OLDZISHOU)], "pa12-batch9-oldzishou");
    let mut e2 = Engine::new_with_extensions(2026110642, 2, &[&pack2]).expect("装载 oldzishou");
    e2.debug_grant(1, "oldzishou");
    // 令座位 1 损失 2 点体力（max_hp=4, hp=2）
    e2.debug_set_max_hp(1, 4);
    e2.debug_set_hp(1, 2);
    e2.debug_clear_hand(1);

    while !(e2.open_choice().is_some() && e2.open_choice().unwrap().observer == 1) {
        let cur = e2.turn().current;
        e2.debug_clear_hand(cur);
        submit(&mut e2, cur, GameAction::NextPhase);
    }
    confirm(&mut e2, 1);
    // 座位 1 摸 2+2=4 张牌后跳过出牌阶段，由于 4 > hp(2)，进入弃牌阶段打开弃牌选择（或不在 Play 阶段）
    assert_ne!(
        (e2.turn().current, e2.turn().phase),
        (1, Phase::Play),
        "oldzishou 应跳过座位 1 的出牌阶段"
    );
}

/// 3. `spyinju2.skill`（隐据·标记）+ `zhengjing2.skill`（正争·终段）+ `zhuangshu_trick.skill`（妆梳·锦囊）：
/// 验证 `SkipNextPlayPhase`、`SkipNextDiscardPhase` 与 `SkipNextJudgePhase` 的真实跳过行为。
#[test]
fn pa12_batch9_skip_play_discard_and_judge_phases_e2e() {
    let pack = pack_for(
        &[
            ("spyinju2", SPYINJU2),
            ("zhengjing2", ZHENGJING2),
            ("zhuangshu_trick", ZHUANGSHU_TRICK),
        ],
        "pa12-batch9-skip-phases",
    );
    let mut e = Engine::new_with_extensions(2026110643, 2, &[&pack]).expect("装载跳过阶段包");
    // 先把 0 号推进到出牌阶段
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }

    // (a) spyinju2: 准备阶段开始锁定触发，跳过出牌阶段和弃牌阶段
    e.debug_grant(1, "spyinju2");
    // 给座位 1 预置 6 张手牌且 hp=1：若弃牌阶段未跳过则必停在弃牌选择；跳过后保留 6+2=8 张手牌直接结束回合！
    e.debug_set_hp(1, 1);
    while e.debug_hand(1).len() < 6 {
        e.debug_place(1, "sha");
    }
    let hand_before = e.debug_hand(1).len();
    e.debug_clear_hand(0);
    // 0 号结束出牌阶段（Play -> Discard -> End -> 1 号 Prepare）
    while e.turn().current == 0 {
        submit(&mut e, 0, GameAction::NextPhase);
    }
    // 座位 1 的准备阶段已触发 spyinju2；推进 1 号走完整个回合（Prepare -> Judge -> Draw -> 跳过 Play -> 跳过 Discard -> End）
    while !(e.turn().current == 1 && e.turn().phase == Phase::End) {
        assert_ne!(
            e.turn().phase,
            Phase::Play,
            "spyinju2 应跳过座位 1 的出牌阶段"
        );
        assert!(
            e.open_choice().is_none(),
            "spyinju2 跳过弃牌阶段后不应打开弃牌选择窗"
        );
        submit(&mut e, 1, GameAction::NextPhase);
    }
    assert_eq!(
        e.debug_hand(1).len(),
        hand_before + 2,
        "spyinju2 应同时跳过出牌阶段与弃牌阶段（手牌超限亦不弃牌）"
    );

    // (b) zhengjing2: 准备阶段开始获得「争」堆全部牌、跳过判定阶段并移去自身
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    let yinju_def = e.catalog().skill_by_name("spyinju2").unwrap();
    e.state_core_mut().skills.remove(&(1, yinju_def));
    e.debug_grant(1, "zhengjing2");

    let pile_card = e.state_core_mut().deck.pop().unwrap();
    e.state_core_mut()
        .vm_aux
        .piles
        .insert((1, "zhengjing2".to_string()), vec![pile_card]);
    // 在座位 1 判定区放一张乐不思蜀：若判定阶段被 zhengjing2 跳过，乐不思蜀不会结算，仍留在判定区！
    let le_card = e.debug_place_judge(1, "le");

    e.debug_clear_hand(0);
    while !(e.turn().current == 1 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(
        e.debug_hand(1).contains(&pile_card),
        "zhengjing2 应将「争」堆卡牌置入手牌"
    );
    assert!(
        e.debug_judge(1).contains(&le_card),
        "被跳过的判定阶段不应消费判定区里的延时锦囊"
    );
    assert!(
        !e.debug_has_skill(1, "zhengjing2"),
        "zhengjing2 结算后应移除自身"
    );
}

/// 4. `zj_fuli.skill`（极乐·界扶利）：
/// 濒死时确认发动：回复至 2 点体力、摸 2 张牌，并 `queue_extra_phases(self, 8, self_skill)`
/// 排入额外出牌阶段；当前出牌阶段结束后立即进入额外出牌阶段。
#[test]
fn pa12_batch9_zj_fuli_queue_extra_play_phase_e2e() {
    let pack = pack_for(&[("zj_fuli", ZJ_FULI)], "pa12-batch9-zjfuli");
    let mut e = Engine::new_with_extensions(2026110644, 2, &[&pack]).expect("装载 zj_fuli");
    e.debug_grant(0, "zj_fuli");
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    e.debug_set_hp(0, 1);
    let hand_before = e.debug_hand(0).len();

    // 对 0 号造成 1 点致命伤害触发濒死与 zj_fuli 确认窗
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    confirm(&mut e, 0);

    assert_eq!(e.debug_hp(0), 2, "zj_fuli 应将体力回复至 2");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 2,
        "zj_fuli 应摸 2 张牌"
    );
    assert_eq!(
        e.turn().extra_phases,
        vec![2],
        "zj_fuli 应排入额外出牌阶段（code=2）"
    );

    // 结束当前出牌阶段 → 弹出 extra_phases[0]=2，立即进入额外出牌阶段！
    submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play);
    assert!(e.turn().extra_phases.is_empty());
}

/// 5. `xinduodao.skill`（新夺刀）：
/// 受到伤害后，若伤害来源装备区有武器（`owner_is_target_source_has_weapon`），可确认获得来源装备区的武器牌
/// （`GainTargetEquipment` slot=1）。
#[test]
fn pa12_batch9_xinduodao_gain_target_weapon_e2e() {
    let pack = pack_for(&[("xinduodao", XINDUODAO)], "pa12-batch9-xinduodao");
    let mut e = Engine::new_with_extensions(2026110645, 2, &[&pack]).expect("装载 xinduodao");
    e.debug_grant(1, "xinduodao");

    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    // 座位 0 装备一张武器
    let weapon = e.debug_place(0, "weapon2");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: weapon,
            targets: vec![],
        },
    );
    assert!(e.debug_equips(0).contains(&weapon));

    // 座位 0 对座位 1 造成 1 点伤害
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    confirm(&mut e, 1);

    assert!(
        !e.debug_equips(0).contains(&weapon),
        "伤害来源的武器应离开其装备区"
    );
    assert!(
        e.debug_hand(1).contains(&weapon),
        "xinduodao 应将伤害来源的武器收入手牌"
    );
}

/// 6. `dcyongjue.skill`（勇决）+ `oldhuxiao.skill`（旧虎啸）：
/// - `dcyongjue` 分支 1（`gain_event_cards`）：从 `table` 回收刚使用的【杀】实体牌入手牌，且结算尾 `to_discard`
///   不会把已在手牌中的该牌重复推入弃牌堆（快照恢复通过卡牌唯一性守卫）；
/// - `oldhuxiao`：【杀】被【闪】抵消时锁定触发返还出杀次数（`sha_used_play_phase = false`）。
#[test]
fn pa12_batch9_dcyongjue_and_oldhuxiao_sha_refund_and_gain_card_e2e() {
    let pack = pack_for(
        &[("dcyongjue", DCYONGJUE), ("oldhuxiao", OLDHUXIAO)],
        "pa12-batch9-sha-refund",
    );
    let mut e = Engine::new_with_extensions(2026110646, 2, &[&pack]).expect("装载勇决与旧虎啸");
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    e.debug_grant(0, "dcyongjue");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);

    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");

    // 使用第一张【杀】 → 触发 dcyongjue confirm → 选分支 1（获得此【杀】实体牌）
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    // 1 号无手牌，直接 Pass 响应闪
    if let Some(ch_shan) = e.open_choice().cloned() {
        if ch_shan.observer == 1 {
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: ch_shan.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }
    confirm(&mut e, 0);
    let branch_ch = e.open_choice().cloned().expect("dcyongjue 分支选择");
    let opt1 = branch_ch.spec.options[1];
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: branch_ch.id,
            selection: vec![opt1],
        },
    );
    // 若刚才 CardUsed 在 Sha 结算前触发，则再處理 1 号响应闪
    if let Some(ch_shan) = e.open_choice().cloned() {
        if ch_shan.observer == 1 {
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: ch_shan.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }

    assert!(
        e.debug_hand(0).contains(&sha1),
        "dcyongjue 分支 1 应将使用的【杀】收回手牌"
    );
    assert!(
        !e.state_core().discard.contains(&sha1),
        "已被收回手牌的【杀】不得在结算结束时重复进入弃牌堆"
    );
    // 快照恢复验证卡牌唯一性守卫：
    let snap = e.save();
    let restored =
        Engine::restore_with_extensions(&snap, &[&pack]).expect("快照恢复应通过卡牌唯一性校验");
    assert_eq!(restored.state_hash(), e.state_hash());

    // 测试 oldhuxiao：【杀】被【闪】抵消后返还 sha_used_play_phase
    e.debug_grant(0, "oldhuxiao");
    e.state_core_mut().turn.sha_used_play_phase = false;
    let shan = e.debug_place(1, "shan");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    // 第二张【杀】不再满足 first_sha，dcyongjue 不再触发；直接由 1 号打出【闪】抵消！
    let ch_shan = e.open_choice().cloned().expect("1 号应响应第二张杀");
    assert_eq!(ch_shan.observer, 1);
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ch_shan.id,
            selection: vec![ChoiceOption::Card(shan)],
        },
    );
    assert!(
        !e.turn().sha_used_play_phase,
        "oldhuxiao 在【杀】被闪避后应返还本阶段出杀次数"
    );
}

/// 7. `renzheng.skill` + `stdrenzheng.skill`（`damage_prevented` → `DamageCancelled` + `DamageZero`）：
/// 当伤害在 `DamageBefore` 被防止或减至 0 时，触发 `renzheng`（摸 2 张）与 `stdrenzheng`（当前回合角色摸 1 张）。
#[test]
fn pa12_batch9_damage_prevented_renzheng_and_stdrenzheng_e2e() {
    const PREVENT_HELPER: &str = r#"
skill test_prevent_damage(3900000001) locked {
  trigger damage_before owner_is_target {
    effect prevent_current_damage();
  }
}
"#;
    let pack = pack_for(
        &[
            ("renzheng", RENZHENG),
            ("stdrenzheng", STDRENZHENG),
            ("test_prevent_damage", PREVENT_HELPER),
        ],
        "pa12-batch9-damage-prevented",
    );
    let mut e = Engine::new_with_extensions(2026110647, 2, &[&pack]).expect("装载仁政包");
    e.debug_grant(0, "stdrenzheng");
    e.debug_grant(1, "renzheng");
    e.debug_grant(1, "test_prevent_damage");

    let hand0_before = e.debug_hand(0).len();
    let hand1_before = e.debug_hand(1).len();
    let hp1_before = e.debug_hp(1);

    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);

    assert_eq!(e.debug_hp(1), hp1_before, "伤害应被防止");
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_before + 2,
        "renzheng 在伤害被防止时应令持有者摸 2 张牌"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0_before + 1,
        "stdrenzheng 在伤害被防止时应令当前回合角色（座位 0）摸 1 张牌"
    );
}

/// 8. `faen.skill`（法恩）+ `refaen.skill`（界法恩）：
/// 验证 `character_state_changed` 对 `TurnOvered`（翻至正面 vs 翻至背面）与 `Linked`（横置 vs 重置）的精确区分。
#[test]
fn pa12_batch9_character_state_changed_faen_and_refaen_e2e() {
    const FLIP_ON_DMG: &str = r#"
skill test_flip_on_damage(3900000003) locked {
  trigger damage owner_is_target {
    effect turn_over(self);
  }
}
"#;
    let pack = pack_for(
        &[
            ("faen", FAEN),
            ("refaen", REFAEN),
            ("test_flip_on_damage", FLIP_ON_DMG),
        ],
        "pa12-batch9-character-state",
    );
    let mut e = Engine::new_with_extensions(2026110648, 2, &[&pack]).expect("装载法恩包");
    while !(e.turn().current == 0 && e.turn().phase == Phase::Play) {
        let cur = e.turn().current;
        submit(&mut e, cur, GameAction::NextPhase);
    }
    e.debug_grant(0, "faen");
    e.debug_grant(1, "test_flip_on_damage");
    e.debug_set_hp(1, 10);

    // (a) 1 号受伤翻至背面（face_down: true）：faen 不触发
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.debug_face_down(1));
    assert!(
        e.open_choice().is_none(),
        "faen 在翻至背面时不应触发"
    );

    // (b) 1 号再次受伤翻回正面（face_down: false）：faen 触发，令事件角色（座位 1）摸 1 张牌
    let hand1_before = e.debug_hand(1).len();
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(!e.debug_face_down(1));
    confirm(&mut e, 0);
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_before + 1,
        "faen 在翻至正面时应令事件角色摸 1 张牌"
    );

    // (c) 使用铁索连环横置 1 号（linked: true）：faen 触发；再次铁索重置 1 号（linked: false）：faen 不触发
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let ts1 = e.debug_place(0, "chain");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ts1,
            targets: vec![1],
        },
    );
    assert!(e.debug_linked(1));
    confirm(&mut e, 0);
    assert_eq!(e.debug_hand(1).len(), 1);

    let ts2 = e.debug_place(0, "chain");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ts2,
            targets: vec![1],
        },
    );
    assert!(!e.debug_linked(1));
    assert!(
        e.open_choice().is_none(),
        "faen 在重置（解除连环）时不应触发"
    );

    // (d) 换成 refaen：翻至背面（face_down: true）也会触发！
    let faen_def = e.catalog().skill_by_name("faen").unwrap();
    e.state_core_mut().skills.remove(&(0, faen_def));
    e.debug_grant(0, "refaen");
    let hand1_now = e.debug_hand(1).len();
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(e.debug_face_down(1));
    confirm(&mut e, 0);
    assert_eq!(
        e.debug_hand(1).len(),
        hand1_now + 1,
        "refaen 在翻至背面时也应触发摸 1 张牌"
    );
}

/// 9. `pxyanmen.skill`（雁门）+ `pxbeihai.skill`（北海）+ `zj_hanci.skill`（韩辞）：
/// 验证 `skill_added`（`SkillToggled { enabled: true }`）与 `owner_skill_added_self_{def}` 及 `AddSkill` 联动。
#[test]
fn pa12_batch9_skill_added_pxyanmen_pxbeihai_and_zj_hanci_e2e() {
    const GRANT_HELPER: &str = r#"
skill test_grant_skills(3900000002) locked {
  trigger damage owner_is_target {
    effect add_skill(self, 3339454948);
    effect add_skill(event_source, 3401654349);
  }
}
"#;
    let pack = pack_for(
        &[
            ("pxyanmen", PXYANMEN),
            ("pxbeihai", PXBEIHAI),
            ("zj_hanci", ZJ_HANCI),
            ("test_grant_skills", GRANT_HELPER),
        ],
        "pa12-batch9-skill-added",
    );
    let mut e = Engine::new_with_extensions(2026110649, 2, &[&pack]).expect("装载技能获得包");
    e.debug_grant(0, "test_grant_skills");
    e.debug_grant(1, "zj_hanci");

    // 清空座位 0 与座位 1 手牌以便观察：
    // - 座位 0 受伤触发 test_grant_skills：
    //   1) 座位 0 获得 pxyanmen(3339454948) → pxyanmen 自触发获得 1 张武器 + 1 张 -1 马（进攻马），同时座位 1 的 zj_hanci 触发令双方各摸 1 张；
    //   2) 座位 1 获得 pxbeihai(3401654349) → pxbeihai 自触发将手牌补至体力上限，同时座位 1 的 zj_hanci 再次触发令双方各摸 1 张！
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);

    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    while let Some(ch) = e.open_choice().cloned() {
        let selection = if ch.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else {
            vec![ch.spec.options[0]]
        };
        submit(
            &mut e,
            ch.observer,
            GameAction::Answer {
                choice_id: ch.id,
                selection,
            },
        );
    }

    let hand0 = e.debug_hand(0);
    assert!(
        hand0
            .iter()
            .any(|c| e.catalog().card_kind(*c).equip_slot() == Some(EquipSlot::Weapon)),
        "pxyanmen 应为获得者检索一张武器牌"
    );
    assert!(
        hand0
            .iter()
            .any(|c| e.catalog().card_kind(*c).equip_slot() == Some(EquipSlot::HorseMinus)),
        "pxyanmen 应为获得者检索一张进攻坐骑（-1 马）"
    );
    assert!(
        e.debug_hand(1).len() >= e.state_core().player(1).max_hp as usize,
        "pxbeihai + zj_hanci 应令座位 1 手牌补至不少于体力上限"
    );
}
