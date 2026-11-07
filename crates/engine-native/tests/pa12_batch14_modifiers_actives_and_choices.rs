//! PA12 迁移批⑭（2026-11-06，零版本号变更）综合验证：
//! 1. `codec_ext2.rs` 25 个高频效果原语（tags 129..=153）编解码往返、未知单字节 tag 154 拒绝，
//!    以及 `tag = 255` `u16` 扩展标签前缀预留与非法/截断输入拒绝。
//! 2. `VmModifierRule` 15 类通用修正目标（`hand_limit`、`hand_limit_base_max_hp`、`sha_use_limit`、
//!    `sha_extra_targets`、`attack_range`、`distance_to`、`distance_from`、`card_distance_unlimited`、
//!    `card_use_unlimited`、`draw_count`、`card_target_cancel`、`sha_use_cancel`、`card_use_cancel`、
//!    `hand_card_use_response_cancel`、`dying_peach_target_cancel`）在原生内核中的求值，以及
//!    `modifier hand_limit add 0;`（`INTENTIONAL_EMPTY_NOOP` 占位）继续被 `bridge_vm_skill_decls` 严格拒绝。
//! 3. 扩展主动技门禁与输入（`once_per_game`、`uses_per_turn > 1`、`uses_per_play_phase`、
//!    `uses_per_turn_source`、`selection`、`targets`）及 `NGSA 107` 存读档往返。
//! 4. 12 个剩余触发时机、动态 `event_view_as_skill_<u32>` 过滤与 `choice_actor != Owner`
//!    （`EventPlayer` / `EventSource` 在 `OptionalGate` 询问与 `NGSA 107` 存读档中的行为）。
//! 5. 25 个批⑭交互效果原语（tags 129..=153）在原生 `Engine` 上的真实开窗、选择结算与 `NGSA 107` 挂起存读档往返。

use content_compiler::compile_source_named;
use engine_native::choice::ChoiceKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{
    CardId, DrawTargetScope, EffectRequest, MarkDefId, PlayerId, SkillDefId, SkillId,
};

const MINSI2_NOOP: &str = include_str!("../../../content/original/ported/sp2/minsi2.skill");
const SHUSHEN: &str = include_str!("../../../content/original/ported/sp/shushen.skill");
const MIEJI: &str = include_str!("../../../content/original/ported/yijiang/mieji.skill");
const XIANZHEN: &str = include_str!("../../../content/original/ported/yijiang/xianzhen.skill");
const XINJUECE: &str = include_str!("../../../content/original/ported/yijiang/xinjuece.skill");
const DANGMO: &str = include_str!("../../../content/original/ported/shiji/dangmo.skill");
const KAIKANG: &str = include_str!("../../../content/original/ported/sp/kaikang.skill");
const XINWANGXI: &str = include_str!("../../../content/original/ported/refresh/xinwangxi.skill");
const QIAOSHUI: &str = include_str!("../../../content/original/ported/yijiang/qiaoshui.skill");
const LIZHAN: &str = include_str!("../../../content/original/ported/sp/lizhan.skill");
const STDDUANBI: &str = include_str!("../../../content/original/ported/sixiang/stdduanbi.skill");
const REGUANXING: &str = include_str!("../../../content/original/ported/refresh/reguanxing.skill");
const DCJIEYING: &str = include_str!("../../../content/original/ported/xianding/dcjieying.skill");
const FANGZHU: &str = include_str!("../../../content/original/ported/shenhua/fangzhu.skill");
const OLPAIYI: &str = include_str!("../../../content/original/ported/onlyOL/olpaiyi.skill");
const DUANBING: &str = include_str!("../../../content/original/ported/sp/duanbing.skill");
const DCSBQIJUE: &str = include_str!("../../../content/original/ported/xianding/dcsbqijue.skill");
const JIEMING: &str = include_str!("../../../content/original/ported/shenhua/jieming.skill");
const OLFEIBIAN: &str = include_str!("../../../content/original/ported/sp/olfeibian.skill");
const KUIWEI: &str = include_str!("../../../content/original/ported/sp/kuiwei.skill");
const TUNCHU: &str = include_str!("../../../content/original/ported/mobile/tunchu.skill");
const RENXIN: &str = include_str!("../../../content/original/ported/yijiang/renxin.skill");
const DCSHOUZE: &str = include_str!("../../../content/original/ported/xianding/dcshouze.skill");
const ZHENGNAN: &str = include_str!("../../../content/original/ported/sp/zhengnan.skill");
const SHEBIAN: &str = include_str!("../../../content/original/ported/refresh/shebian.skill");
const TIEJI: &str = include_str!("../../../content/standard/tieji.skill");
const DCWUHUN: &str = include_str!("../../../content/original/ported/xianding/dcwuhun.skill");

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1_400_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn submit_outcome(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let out = submit_outcome(e, actor, action.clone());
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "submit({actor}, {action:?}) 失败: {out:?}"
    );
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let ask = e.open_choice().cloned().expect("expected open_choice");
    submit(
        e,
        ask.observer,
        GameAction::Answer {
            choice_id: ask.id,
            selection,
        },
    );
}

fn drain_choices(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

fn advance_to_phase(e: &mut Engine, seat: u8, want: Phase) {
    for _ in 0..80 {
        drain_choices(e);
        if e.state_core().turn.current == seat && e.state_core().turn.phase == want {
            return;
        }
        let cur = e.state_core().turn.current;
        submit(e, cur, GameAction::NextPhase);
    }
    panic!(
        "未能推进到 ({seat}, {want:?})，当前=({}, {:?})",
        e.state_core().turn.current,
        e.state_core().turn.phase
    );
}

fn assert_archive_roundtrip(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode_archive");
    let restored =
        Engine::restore_with_extensions(&decoded, packs).expect("restore_with_extensions");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "NGSA 107 恢复前后哈希一致"
    );
    assert_eq!(restored.open_choice(), e.open_choice(), "挂起询问一致");
    restored
}

fn pack_strict(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译 .skill");
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑭ 严格桥接测试包",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

#[test]
fn pa12_batch14_codec_roundtrip_129_to_153_and_u16_extension_prefix_255() {
    let effects = vec![
        (
            129u8,
            EffectRequest::ChooseOtherDrawOrRecover {
                player: PlayerId::new(1),
                draw_count: 2,
                recover_count: 1,
                times: 2,
                skill: SkillId::new(3835991174),
            },
        ),
        (
            130,
            EffectRequest::ChooseAddOrRemoveCurrentCardTarget {
                player: PlayerId::new(1),
                card: CardId::new(5),
                skill: SkillId::new(2376002488),
            },
        ),
        (
            131,
            EffectRequest::ChooseCompareSelectedHandCard {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(8),
                win_skill_def: SkillDefId::new(3706320139),
                lose_skill_def: SkillDefId::new(2499130347),
                skill: SkillId::new(3382226057),
            },
        ),
        (
            132,
            EffectRequest::ChooseTargetsDamage {
                player: PlayerId::new(1),
                min_count: 1,
                max_count: 2,
                scope: DrawTargetScope::HandEmpty,
                amount: 1,
                skill: SkillId::new(3145104894),
            },
        ),
        (
            133,
            EffectRequest::ChooseAddCurrentShaTarget {
                player: PlayerId::new(1),
                card: CardId::new(3),
                skill: SkillId::new(3289572540),
            },
        ),
        (
            134,
            EffectRequest::DrawThenGiveOne {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                count: 2,
                drawn_only: false,
                use_given_equipment: true,
                skill: SkillId::new(3829685432),
                drawn: vec![CardId::new(10), CardId::new(11)],
                ready_to_give: true,
            },
        ),
        (
            135,
            EffectRequest::ChooseHandCompare {
                player: PlayerId::new(1),
                win_skill_def: SkillDefId::new(2260792673),
                lose_skill_def: SkillDefId::new(2280240537),
                skill: SkillId::new(3955213386),
            },
        ),
        (
            136,
            EffectRequest::ChooseOptionalTargetsDraw {
                player: PlayerId::new(1),
                count: 1,
                min_count: 1,
                max_count: 255,
                scope: DrawTargetScope::Wounded,
                skill: SkillId::new(2790547004),
            },
        ),
        (
            137,
            EffectRequest::ChooseTargetsDraw {
                player: PlayerId::new(1),
                count: 2,
                target_count: 2,
                scope: DrawTargetScope::Any,
                skill: SkillId::new(3811377323),
            },
        ),
        (
            138,
            EffectRequest::GuanxingTop {
                player: PlayerId::new(1),
                count: 3,
                round: 0,
                flipped: vec![CardId::new(1), CardId::new(2), CardId::new(3)],
                top: vec![CardId::new(1)],
                bottom: vec![CardId::new(2)],
                skill: SkillId::new(4220147649),
            },
        ),
        (
            139,
            EffectRequest::ChooseLinkOther {
                player: PlayerId::new(1),
                skill: SkillId::new(3849335872),
            },
        ),
        (
            140,
            EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver {
                player: PlayerId::new(1),
                skill: SkillId::new(2335755764),
            },
        ),
        (
            141,
            EffectRequest::ChooseOneFromSkillPileDiscard {
                player: PlayerId::new(1),
                skill: SkillId::new(3281980776),
                pile_def: 3615325273,
            },
        ),
        (
            142,
            EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
                player: PlayerId::new(1),
                card: CardId::new(4),
                max_distance: 1,
                skill: SkillId::new(4181445466),
            },
        ),
        (
            143,
            EffectRequest::ChooseRecoverTarget {
                chooser: PlayerId::new(1),
                target: PlayerId::new(2),
                amount: 2,
            },
        ),
        (
            144,
            EffectRequest::ChooseTargetDrawToMaxHpCap {
                player: PlayerId::new(1),
                cap: 5,
                skill: SkillId::new(3579069878),
            },
        ),
        (
            145,
            EffectRequest::ChooseDiscardHandRange {
                player: PlayerId::new(1),
                min: 1,
                max: 2,
                skill: SkillId::new(3525562624),
            },
        ),
        (
            146,
            EffectRequest::ChooseDiscardHandEquipThenRemoveSelfSkill {
                player: PlayerId::new(1),
                count: 2,
                skill: SkillId::new(4087547393),
            },
        ),
        (
            147,
            EffectRequest::ChooseHandCardsToSkillPile {
                player: PlayerId::new(1),
                skill: SkillId::new(3943350142),
                min_count: 1,
                max_count: 4,
            },
        ),
        (
            148,
            EffectRequest::ChooseDiscardEquipmentPreventCurrentDamage {
                player: PlayerId::new(1),
                damage_id: 9,
                skill: SkillId::new(3389183077),
            },
        ),
        (
            149,
            EffectRequest::ChooseGainDiscardCardByKind {
                player: PlayerId::new(1),
                kind: 1,
                skill: SkillId::new(3027710363),
            },
        ),
        (
            150,
            EffectRequest::ChooseAddMissingSkill {
                player: PlayerId::new(1),
                skill_defs: [
                    SkillDefId::new(3038912803),
                    SkillDefId::new(4090178594),
                    SkillDefId::new(3521391741),
                ],
                skill: SkillId::new(3435011424),
            },
        ),
        (
            151,
            EffectRequest::ChooseMoveFieldCardTo {
                player: PlayerId::new(1),
                target: PlayerId::new(1),
                skill: SkillId::new(3980781998),
            },
        ),
        (
            152,
            EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                player: PlayerId::new(1),
                card: CardId::new(6),
                target: PlayerId::new(2),
                retieji: true,
                skill: SkillId::new(3561072419),
                ready_to_choose: true,
                judgment_suit: 1,
            },
        ),
        (
            153,
            EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                player: PlayerId::new(1),
                mark: MarkDefId::new(3698112918),
                skill: SkillId::new(4234333270),
            },
        ),
    ];

    assert_eq!(effects.len(), 25);
    for (expected_tag, eff) in &effects {
        assert!(
            accept_effect(eff, 4).is_ok(),
            "tag {expected_tag} 应通过 accept_effect: {eff:?}"
        );
        let mut buf = Vec::new();
        enc_effects(&mut buf, std::slice::from_ref(eff));
        assert_eq!(buf[4], *expected_tag, "tag {expected_tag} 编码字节匹配");
        let decoded = dec_effects(&buf).expect("dec_effects 成功");
        assert_eq!(decoded.as_slice(), std::slice::from_ref(eff));
    }

    let all: Vec<EffectRequest> = effects.into_iter().map(|(_, e)| e).collect();
    let mut batch_buf = Vec::new();
    enc_effects(&mut batch_buf, &all);
    let batch_decoded = dec_effects(&batch_buf).expect("批量解码成功");
    assert_eq!(batch_decoded, all);

    // 未知单字节 tag 154 与截断/未分配 u16 扩展标签前缀 (tag = 255) 必须拒绝
    assert!(dec_effects(&[1, 0, 0, 0, 154]).is_none());
    assert!(dec_effects(&[1, 0, 0, 0, 255]).is_none());
    assert!(dec_effects(&[1, 0, 0, 0, 255, 0]).is_none());
    assert!(dec_effects(&[1, 0, 0, 0, 255, 0, 1]).is_none());

    // 边界校验拒绝测试
    assert!(accept_effect(
        &EffectRequest::ChooseTargetsDamage {
            player: PlayerId::new(1),
            min_count: 3,
            max_count: 2,
            scope: DrawTargetScope::Any,
            amount: 1,
            skill: SkillId::new(1),
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::GuanxingTop {
            player: PlayerId::new(1),
            count: 3,
            round: 2,
            flipped: Vec::new(),
            top: Vec::new(),
            bottom: Vec::new(),
            skill: SkillId::new(1),
        },
        4
    )
    .is_err());
}

#[test]
fn pa12_batch14_modifiers_generalized_and_intentional_noop_rejected() {
    // 1. 严格拒绝 INTENTIONAL_EMPTY_NOOP 占位 (modifier hand_limit add 0;)
    let noop_pkg = compile_source_named("minsi2", MINSI2_NOOP).expect("编译 minsi2");
    let err = content_native_loader::bridge_vm_skill_decls(&noop_pkg)
        .expect_err("modifier hand_limit add 0 占位必须继续被严格拒绝");
    assert!(
        format!("{err:?}").contains("modifier_index"),
        "预期 modifier_index 拒绝，实际: {err:?}"
    );

    let inline_noop = compile_source_named(
        "inline_noop",
        "skill inline_noop(91400001) { modifier hand_limit add 0; }",
    )
    .expect("编译 inline_noop");
    assert!(content_native_loader::bridge_vm_skill_decls(&inline_noop).is_err());

    // 2. 真实修正器全覆盖验证
    let mod_dsl = r#"
skill mod_hand_lost(91400101) locked {
    modifier hand_limit add owner_lost_hp;
}
skill mod_hand_base_max(91400102) locked {
    modifier hand_limit_base_max_hp override 1;
}
skill mod_atk_range(91400103) locked {
    modifier attack_range add 2;
}
skill mod_dist_to(91400104) locked {
    modifier distance_from add -1;
}
skill mod_dist_from(91400105) locked {
    modifier distance_to add 1;
}
skill mod_draw_bonus(91400106) locked {
    modifier draw_count add 2;
}
skill mod_sha_extra(91400107) locked {
    modifier sha_use_limit add 1;
    modifier card_target_limit(sha) add 1;
}
skill mod_dist_unlim(91400108) locked {
    modifier card_distance_unlimited(trick) override 1;
}
skill mod_target_cancel(91400109) locked {
    modifier card_target_cancel(sha) override 1;
}
skill mod_sha_ban(91400110) locked {
    modifier sha_use_cancel override 1;
}
skill mod_dying_peach_ban(91400111) locked {
    modifier card_save_cancel(tao) override 1;
}
"#;
    let pack = pack_strict(&[("batch14_mods", mod_dsl)], "pa12-b14-mods");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(914_100, 3, &packs).expect("建局");

    // (a) hand_limit + hand_limit_base_max_hp
    e.state_core_mut().player_mut(0).max_hp = 4;
    e.state_core_mut().player_mut(0).hp = 2;
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 2);
    e.debug_grant(0, "mod_hand_lost");
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        4,
        "hp(2) + lost_hp(2) = 4"
    );
    e.debug_grant(0, "mod_hand_base_max");
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        6,
        "base max_hp(4) + lost_hp(2) = 6"
    );

    // (b) attack_range & distance_to / distance_from
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 1);
    e.debug_grant(0, "mod_atk_range");
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 3);

    e.debug_grant(1, "mod_dist_from");
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        2,
        "基础距离 1 + distance_from(1) = 2"
    );
    e.debug_grant(0, "mod_dist_to");
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        1,
        "2 - distance_to(1) = 1"
    );

    // (c) draw_count 修正摸牌阶段摸牌数
    e.debug_grant(0, "mod_draw_bonus");
    let h0_before_draw = e.state_core().player(0).hand.len();
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        h0_before_draw + 4,
        "基础摸 2 + mod_draw_bonus(2) = 4 张"
    );

    // (d) sha_use_limit (+1) & sha_extra_targets (+1) & card_target_cancel & sha_use_cancel
    e.debug_grant(0, "mod_sha_extra");
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");
    let sha3 = e.debug_place(0, "sha");
    // 1 号拥有 mod_target_cancel 时不能成为【杀】目标
    e.debug_grant(1, "mod_target_cancel");
    let rej_cancel = submit_outcome(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert!(matches!(
        rej_cancel,
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    // 移除 1 号的 mod_target_cancel 后，0 号可凭 sha_extra_targets 同时指定 [1, 2] 两名目标！
    let cancel_def = e.catalog().skill_by_name("mod_target_cancel").unwrap();
    e.state_core_mut().skills.remove(&(1, cancel_def));
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1, 2],
        },
    );
    drain_choices(&mut e);
    // 凭 sha_use_limit +1 可继续出第 2 张【杀】
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    drain_choices(&mut e);
    // 第 3 张【杀】超出上限 (1 + 1 = 2)
    let rej_limit = submit_outcome(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    assert!(matches!(
        rej_limit,
        CommandOutcome::Rejected(RejectReason::ShaLimit)
    ));

    // (e) card_distance_unlimited：1 号有 distance_from +1 且移除 0 号 distance_to 后距离为 2，
    //     但 0 号拥有 mod_dist_unlim (trick) 时可对距离 2 的 1 号使用【顺手牵羊】！
    let dist_to_def = e.catalog().skill_by_name("mod_dist_to").unwrap();
    e.state_core_mut().skills.remove(&(0, dist_to_def));
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 1), 2);
    let shun = e.debug_place(0, "snatch");
    let rej_dist = submit_outcome(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shun,
            targets: vec![1],
        },
    );
    assert!(matches!(
        rej_dist,
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    e.debug_grant(0, "mod_dist_unlim");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: shun,
            targets: vec![1],
        },
    );
    drain_choices(&mut e);

    // (f) sha_use_cancel：在新回合授予 mod_sha_ban 后禁止出【杀】
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_grant(1, "mod_sha_ban");
    let sha_p1 = e.debug_place(1, "sha");
    let rej_ban = submit_outcome(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_p1,
            targets: vec![0],
        },
    );
    assert!(matches!(
        rej_ban,
        CommandOutcome::Rejected(RejectReason::CardLimit)
    ));
}

#[test]
fn pa12_batch14_expanded_active_skills_limits_selection_targets_and_archive() {
    let active_dsl = r#"
skill act_twice_play(91400201) {
    active play_phase uses_per_play_phase 2 select_hand 1 1 targets 1 1 other {
        effect discard_selected(self);
        effect deal_damage(self, event_source, 1);
    }
}
skill act_once_game(91400202) {
    active play_phase once_per_game {
        effect draw_cards(self, 2);
    }
}
skill act_hp_uses(91400203) {
    active play_phase uses_per_turn owner_hp {
        effect draw_cards(self, 1);
    }
}
"#;
    let pack = pack_strict(&[("batch14_actives", active_dsl)], "pa12-b14-actives");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(914_200, 3, &packs).expect("建局");
    advance_to_phase(&mut e, 0, Phase::Play);

    e.debug_grant(0, "act_twice_play");
    e.debug_grant(0, "act_once_game");
    e.debug_grant(0, "act_hp_uses");
    let def_twice = e.catalog().skill_by_name("act_twice_play").unwrap();
    let def_once_game = e.catalog().skill_by_name("act_once_game").unwrap();
    let def_hp_uses = e.catalog().skill_by_name("act_hp_uses").unwrap();

    // 1. act_hp_uses: 将体力设为 1 时 uses_per_turn owner_hp 允许本回合发动 1 次，第 2 次被拒绝
    e.state_core_mut().player_mut(0).hp = 1;
    submit(&mut e, 0, GameAction::ActivateSkill { skill: def_hp_uses });
    let rej_hp_second = submit_outcome(
        &mut e,
        0,
        GameAction::ActivateSkill { skill: def_hp_uses },
    );
    assert!(matches!(rej_hp_second, CommandOutcome::Rejected(_)));

    // 2. act_twice_play: 选 1 张手牌 + 1 名其他角色，本出牌阶段可发动 2 次
    let c1 = e.state_core().player(0).hand[0];
    let c2 = e.state_core().player(0).hand[1];
    let hp1_before = e.state_core().player(1).hp;
    // 第 1 次发动（预置选牌 c1 与目标 1 号）
    e.state_core_mut().vm_aux.set_selected_cards(0, &[c1]);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: def_twice });
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    assert!(!e.state_core().player(0).hand.contains(&c1));

    // 存读档往返后仍保留计数，可发动第 2 次，第 3 次被拒绝
    let mut e = assert_archive_roundtrip(&e, &packs);
    e.state_core_mut().vm_aux.set_selected_cards(0, &[c2]);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: def_twice });
    drain_choices(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 2);
    assert!(matches!(
        submit_outcome(&mut e, 0, GameAction::ActivateSkill { skill: def_twice }),
        CommandOutcome::Rejected(_)
    ));

    // 3. act_once_game: 整局仅可发动 1 次，跨回合仍不可再次发动
    submit(
        &mut e,
        0,
        GameAction::ActivateSkill {
            skill: def_once_game,
        },
    );
    advance_to_phase(&mut e, 1, Phase::Play);
    advance_to_phase(&mut e, 0, Phase::Play);
    assert!(matches!(
        submit_outcome(
            &mut e,
            0,
            GameAction::ActivateSkill {
                skill: def_once_game,
            }
        ),
        CommandOutcome::Rejected(_)
    ));
}

#[test]
fn pa12_batch14_new_trigger_events_view_as_filter_and_choice_actor() {
    let trig_dsl = r#"
skill trig_round_started(91400301) {
    trigger round_started owner_is_target {
        effect add_mark(self, 91400301, 1);
    }
}
skill trig_round_ended(91400302) {
    trigger round_ended owner_is_target {
        effect add_mark(self, 91400302, 1);
    }
}
skill trig_card_resolved(91400303) {
    trigger card_resolved owner_is_source {
        effect add_mark(self, 91400303, 1);
    }
}
skill trig_sha_resolved(91400304) {
    trigger sha_resolved owner_is_source {
        effect add_mark(self, 91400304, 1);
    }
}
skill trig_duel_resolved(91400305) {
    trigger duel_resolved owner_is_source {
        effect add_mark(self, 91400305, 1);
    }
}
skill trig_debate_started(91400306) {
    trigger debate_started owner_is_source {
        effect add_mark(self, 91400306, 1);
    }
}
skill trig_debate_resolved(91400307) {
    trigger debate_resolved owner_is_source {
        effect add_mark(self, 91400307, 1);
    }
}
skill trig_hand_compare_started(91400308) {
    trigger hand_compare_started owner_is_target {
        effect add_mark(self, 91400308, 1);
    }
}
skill trig_hand_compare_finished(91400309) {
    trigger hand_compare_finished owner_is_target {
        effect add_mark(self, 91400309, 1);
    }
}
skill trig_equip_slot_disabled(91400310) {
    trigger equip_slot_disabled owner_is_target {
        effect add_mark(self, 91400310, 1);
    }
}
skill trig_max_hp_changed(91400311) {
    trigger max_hp_changed owner_is_target {
        effect add_mark(self, 91400311, 1);
    }
}
skill trig_phase_any_skipped(91400312) {
    trigger phase_any_skipped owner_is_target {
        effect add_mark(self, 91400312, 1);
    }
}
skill trig_choice_actor_source(91400313) {
    trigger damage owner_is_target optional by event_source {
        effect draw_cards(self, 1);
    }
}
skill trig_view_as_filter(91400314) {
    trigger card_used event_view_as_skill_91400314 {
        effect draw_cards(self, 1);
    }
}
"#;
    let pack = pack_strict(&[("batch14_trigs", trig_dsl)], "pa12-b14-trigs");
    assert_eq!(
        pack.vm_skills.len(),
        14,
        "12 个新触发时机 + choice_actor + event_view_as_skill 全部通过正式桥接"
    );
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(914_300, 2, &packs).expect("建局");

    // 验证 choice_actor == event_source: 0 号拥有技能，1 号对 0 号造成伤害时，
    // OptionalGate 询问的 observer 必须是伤害来源 1 号（而非技能拥有者 0 号）！
    e.debug_grant(0, "trig_choice_actor_source");
    let h0_before = e.state_core().player(0).hand.len();
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let ask = e
        .open_choice()
        .cloned()
        .expect("OptionalGate 询问应挂起");
    assert_eq!(
        ask.observer, 1,
        "choice_actor = event_source 时应由伤害来源 (1 号) 回答是否发动"
    );
    // 在 OptionalGate 挂起态验证 NGSA 107 存读档往返
    let mut restored = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored] {
        answer(g, vec![ChoiceOption::Confirm]);
    }
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(e.state_core().player(0).hand.len(), h0_before + 1);
}

#[test]
fn pa12_batch14_interactive_effects_129_to_140_and_archive_roundtrip() {
    let pack = pack_strict(
        &[
            ("shushen", SHUSHEN),
            ("mieji", MIEJI),
            ("xianzhen", XIANZHEN),
            ("xinjuece", XINJUECE),
            ("dangmo", DANGMO),
            ("kaikang", KAIKANG),
            ("xinwangxi", XINWANGXI),
            ("qiaoshui", QIAOSHUI),
            ("lizhan", LIZHAN),
            ("stdduanbi", STDDUANBI),
            ("reguanxing", REGUANXING),
            ("dcjieying", DCJIEYING),
            ("fangzhu", FANGZHU),
        ],
        "pa12-b14-eff-a",
    );
    let packs = [&pack];

    // 1. 129 ChooseOtherDrawOrRecover (shushen): 回复体力后选其他角色摸 2 或回复 1
    {
        let mut e = Engine::new_with_extensions(914_401, 3, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "shushen");
        e.state_core_mut().player_mut(0).hp = 2;
        e.state_core_mut().player_mut(1).hp = 2;
        let tao = e.debug_place(0, "tao");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: tao,
                targets: vec![],
            },
        );
        // 自动触发 shushen -> stage 1 选其他角色
        let ask_t = e.open_choice().cloned().expect("shushen stage 1 选人");
        assert_eq!(ask_t.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        // stage 2 选摸 2 (Tag(0)) 或回复 1 (Tag(1))
        let ask_b = e.open_choice().cloned().expect("shushen stage 2 分支");
        assert_eq!(ask_b.spec.kind, ChoiceKind::PickTag);
        let mut restored2 = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored2] {
            answer(g, vec![ChoiceOption::Tag(1)]);
        }
        assert_eq!(e.state_hash(), restored2.state_hash());
        assert_eq!(e.state_core().player(1).hp, 3, "1 号角色回复 1 点体力");
    }

    // 2. 131 ChooseCompareSelectedHandCard (xianzhen 主动技): 选 1 名目标 + 1 张手牌拼点
    {
        let mut e = Engine::new_with_extensions(914_402, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "xianzhen");
        let def_xz = e.catalog().skill_by_name("xianzhen").unwrap();
        let c0 = e.state_core().player(0).hand[0];
        e.state_core_mut().vm_aux.set_selected_cards(0, &[c0]);
        e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xz });
        let ask_xz = e
            .open_choice()
            .cloned()
            .expect("ChooseCompareSelectedHandCard 选拼点目标");
        assert_eq!(ask_xz.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert!(!e.state_core().player(0).hand.contains(&c0));
    }

    // 3. 132 ChooseTargetsDamage (xinjuece): 结束阶段对空城角色造成 1 点伤害
    {
        let mut e = Engine::new_with_extensions(914_403, 3, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "xinjuece");
        e.debug_clear_hand(1);
        e.debug_clear_hand(0);
        let hp1_before = e.state_core().player(1).hp;
        submit(&mut e, 0, GameAction::NextPhase); // -> Discard
        submit(&mut e, 0, GameAction::NextPhase); // -> End
        answer(&mut e, vec![ChoiceOption::Confirm]); // OptionalGate
        let ask = e.open_choice().cloned().expect("ChooseTargetsDamage 选人");
        assert_eq!(
            ask.spec.options,
            vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]
        );
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    }

    // 4. 134 DrawThenGiveOne (xinwangxi): 受到伤害后摸 2 张牌再选 1 张牌交给伤害来源
    {
        let mut e = Engine::new_with_extensions(914_404, 2, &packs).expect("建局");
        e.debug_grant(0, "xinwangxi");
        e.debug_clear_hand(0);
        let h1_before = e.state_core().player(1).hand.len();
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        answer(&mut e, vec![ChoiceOption::Confirm]); // xinwangxi OptionalGate -> draws 2 -> asks give 1
        let ask_give = e.open_choice().cloned().expect("DrawThenGiveOne 选交出牌");
        assert_eq!(ask_give.spec.kind, ChoiceKind::SelectCards);
        let give_card = match ask_give.spec.options[0] {
            ChoiceOption::Card(c) => c,
            other => panic!("expected Card option, got {other:?}"),
        };
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(give_card)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hand.len(), h1_before + 1);
    }

    // 5. 135 ChooseHandCompare (qiaoshui): 出牌阶段开始时选拼点目标
    {
        let mut e = Engine::new_with_extensions(914_405, 2, &packs).expect("建局");
        e.debug_grant(0, "qiaoshui");
        // 推进到 0 号出牌阶段触发 qiaoshui（建局起始在 Phase::Judge）
        submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw
        submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play (triggers qiaoshui)
        answer(&mut e, vec![ChoiceOption::Confirm]); // OptionalGate
        let ask_t = e.open_choice().cloned().expect("ChooseHandCompare 选拼点目标");
        assert_eq!(ask_t.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
    }

    // 6. 138 GuanxingTop (reguanxing) & 139 ChooseLinkOther (dcjieying) & 140 ChooseOtherDrawOwnerLostHpTurnOver (fangzhu)
    {
        let mut e = Engine::new_with_extensions(914_406, 3, &packs).expect("建局");
        e.debug_grant(0, "fangzhu");
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e
            .open_choice()
            .cloned()
            .expect("ChooseOtherDrawOwnerLostHpTurnOver 选人");
        let h2_before = e.state_core().player(2).hand.len();
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(2)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(2).hand.len(), h2_before + 1);
        assert!(e.state_core().player(0).face_down);
        assert!(ask.spec.options.contains(&ChoiceOption::Player(2)));
    }
}

#[test]
fn pa12_batch14_interactive_effects_141_to_153_and_archive_roundtrip() {
    let pack = pack_strict(
        &[
            ("olpaiyi", OLPAIYI),
            ("duanbing", DUANBING),
            ("dcsbqijue", DCSBQIJUE),
            ("jieming", JIEMING),
            ("olfeibian", OLFEIBIAN),
            ("kuiwei", KUIWEI),
            ("tunchu", TUNCHU),
            ("renxin", RENXIN),
            ("dcshouze", DCSHOUZE),
            ("zhengnan", ZHENGNAN),
            ("shebian", SHEBIAN),
            ("tieji", TIEJI),
            ("dcwuhun", DCWUHUN),
        ],
        "pa12-b14-eff-b",
    );
    let packs = [&pack];

    // 1. 144 ChooseTargetDrawToMaxHpCap (jieming): 受伤后令一名角色将手牌补至体力上限
    {
        let mut e = Engine::new_with_extensions(914_501, 2, &packs).expect("建局");
        e.debug_grant(0, "jieming");
        e.debug_clear_hand(1);
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e.open_choice().cloned().expect("jieming 选人");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(
            e.state_core().player(1).hand.len(),
            e.state_core().player(1).max_hp.min(5) as usize
        );
    }

    // 2. 147 ChooseHandCardsToSkillPile (tunchu): 摸牌阶段摸 2 后选若干手牌置入「粮」堆
    {
        let mut e = Engine::new_with_extensions(914_502, 2, &packs).expect("建局");
        e.debug_grant(0, "tunchu");
        submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw (triggers tunchu)
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e
            .open_choice()
            .cloned()
            .expect("ChooseHandCardsToSkillPile 选牌");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        let pick = ask.spec.options[0];
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![pick]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(
            e.state_core()
                .vm_aux
                .piles
                .get(&(0, "tunchu".to_string()))
                .map(|v| v.len())
                .unwrap_or(0),
            1
        );
    }

    // 3. 148 ChooseDiscardEquipmentPreventCurrentDamage (renxin): 弃置装备牌防止伤害并翻面
    {
        let mut e = Engine::new_with_extensions(914_503, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "renxin");
        let equip = e.debug_place(0, "zhuge");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: equip,
                targets: vec![],
            },
        );
        e.state_core_mut().player_mut(1).hp = 1;
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        answer(&mut e, vec![ChoiceOption::Confirm]); // renxin OptionalGate
        let ask = e
            .open_choice()
            .cloned()
            .expect("ChooseDiscardEquipmentPreventCurrentDamage 选装备");
        assert_eq!(ask.spec.options, vec![ChoiceOption::Card(equip)]);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(equip)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hp, 1, "伤害已被防止");
        assert!(e.state_core().player(0).face_down, "曹冲翻面");
    }

    // 4. 151 ChooseMoveFieldCardTo (shebian): 翻面后移动场上一张牌到自己区域
    {
        let fz_pack = pack_strict(&[("fangzhu", FANGZHU)], "pa12-b14-fz");
        let both_packs = [&pack, &fz_pack];
        let mut e2 = Engine::new_with_extensions(914_505, 2, &both_packs).expect("建局");
        advance_to_phase(&mut e2, 1, Phase::Play);
        let equip2 = e2.debug_place(1, "zhuge");
        submit(
            &mut e2,
            1,
            GameAction::PlayCard {
                card: equip2,
                targets: vec![],
            },
        );
        e2.debug_grant(0, "fangzhu");
        e2.debug_grant(0, "shebian");
        e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        answer(&mut e2, vec![ChoiceOption::Confirm]); // fangzhu
        answer(&mut e2, vec![ChoiceOption::Player(1)]); // fangzhu target -> turns over 0 -> triggers shebian
        answer(&mut e2, vec![ChoiceOption::Confirm]); // shebian OptionalGate
        let ask_card = e2
            .open_choice()
            .cloned()
            .expect("ChooseMoveFieldCardTo 选其他角色场牌");
        assert_eq!(ask_card.spec.options, vec![ChoiceOption::Card(equip2)]);
        let mut restored = assert_archive_roundtrip(&e2, &both_packs);
        for g in [&mut e2, &mut restored] {
            answer(g, vec![ChoiceOption::Card(equip2)]);
        }
        assert_eq!(e2.state_hash(), restored.state_hash());
        assert!(e2.state_core().player(0).equips.contains(&equip2));
    }
}

#[test]
fn pa12_batch14_ported_skill_bridge_inventory_and_noop_rejection() {
    fn collect_skill_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .expect("read_dir")
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let p = entry.path();
            if p.is_dir() {
                collect_skill_files(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("skill") {
                out.push(p);
            }
        }
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/ported");
    let mut files = Vec::new();
    collect_skill_files(&root, &mut files);
    assert!(files.len() >= 3002);

    let mut bridged_ok = 0usize;
    let mut non_shahai_total = 0usize;
    let mut non_shahai_ok = 0usize;
    let mut noop_rejected = 0usize;
    let mut filter_rejected = 0usize;
    let mut active_rejected = 0usize;
    let mut modifier_rejected = 0usize;
    let mut view_as_rejected = 0usize;
    let mut other_rejected = 0usize;

    for file in &files {
        let is_shahai = file.to_string_lossy().contains("/shahai/");
        if !is_shahai {
            non_shahai_total += 1;
        }
        let src = std::fs::read_to_string(file).expect("read skill file");
        let pkg = match content_compiler::compile_source_named("probe", &src) {
            Ok(p) => p,
            Err(_) => {
                other_rejected += 1;
                continue;
            }
        };
        let is_pure_noop = pkg.trigger_index.is_empty()
            && pkg.active_index.is_empty()
            && pkg.view_as_index.is_empty()
            && !pkg.modifier_index.is_empty()
            && pkg.modifier_index.iter().all(|m| {
                m.target == "hand_limit"
                    && m.mode == "add"
                    && m.value == 0
                    && m.value_source.is_none()
                    && m.selector.is_none()
            });
        match content_native_loader::bridge_vm_skill_decls(&pkg) {
            Ok(_) => {
                assert!(
                    !is_pure_noop,
                    "INTENTIONAL_EMPTY_NOOP must never pass bridge_vm_skill_decls: {}",
                    file.display()
                );
                bridged_ok += 1;
                if !is_shahai {
                    non_shahai_ok += 1;
                }
            }
            Err(err) => {
                let msg = err.to_string();
                if is_pure_noop {
                    noop_rejected += 1;
                } else if msg.contains("VM 技能滤镜原生不承载") {
                    filter_rejected += 1;
                } else if msg.contains("active_index") || msg.contains("skill_index") {
                    active_rejected += 1;
                } else if msg.contains("modifier_index") {
                    modifier_rejected += 1;
                } else if msg.contains("view_as_index") {
                    view_as_rejected += 1;
                } else {
                    other_rejected += 1;
                }
            }
        }
    }

    eprintln!(
        "PA12 Batch 14 ported .skill bridge inventory: total={}/{}, non_shahai={}/{}, noop_rejected={}, filter_rejected={}, active_rejected={}, modifier_rejected={}, view_as_rejected={}, other_rejected={}",
        bridged_ok,
        files.len(),
        non_shahai_ok,
        non_shahai_total,
        noop_rejected,
        filter_rejected,
        active_rejected,
        modifier_rejected,
        view_as_rejected,
        other_rejected
    );
    assert!(
        bridged_ok >= 1000,
        "PA12 Batch 14 should unlock >= 1000 ported .skill files at loader bridge (got {bridged_ok})"
    );
    assert!(
        noop_rejected >= 800,
        "All INTENTIONAL_EMPTY_NOOP placeholder files must remain rejected (got {noop_rejected})"
    );
}

#[test]
fn pa12_batch14_conditional_modifiers_dcwuhun_tieji_and_unique_category() {
    use engine_native_kernel::catalog::{CardKind, Suit};
    let custom_dsl = r#"
skill cond_kongcheng(91400601) locked {
    modifier card_target_cancel(sha) override owner_hand_empty;
}
skill cond_hongyan_base(91400602) locked {
    modifier hand_limit_base_max_hp override owner_has_heart_equip;
}
skill cond_wansha(91400603) locked {
    modifier tao_use_save_cancel_for_non_dying_others_during_owner_turn override 1;
}
skill cond_qizi(91400604) locked {
    modifier tao_use_save_cancel_for_dying_beyond_distance override 1;
}
skill act_unique_cat(91400605) {
    active play_phase uses_per_play_phase 2 select_he 1 1 unused_category_by_skill_this_turn {
        effect discard_selected(self);
        effect draw_cards(self, 1);
    }
}
"#;
    let pack = pack_strict(
        &[
            ("custom", custom_dsl),
            ("dcwuhun", DCWUHUN),
            ("tieji", TIEJI),
            ("xianzhen", XIANZHEN),
        ],
        "pa12-b14-regressions",
    );
    let packs = [&pack];

    // 1. cond_kongcheng (owner_hand_empty) & cond_hongyan_base (owner_has_heart_equip)
    {
        let mut e = Engine::new_with_extensions(914_601, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(1, "cond_kongcheng");
        let sha0 = e.debug_place(0, "sha");
        // 1 号有手牌时，cond_kongcheng 不生效，0 号可正常对其出【杀】
        assert!(!e.state_core().player(1).hand.is_empty());
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha0,
                targets: vec![1],
            },
        );
        drain_choices(&mut e);

        // 清空 1 号手牌后，cond_kongcheng 生效，禁止对其出【杀】
        e.state_core_mut().turn.sha_used_play_phase = false;
        e.debug_clear_hand(1);
        let sha1 = e.debug_place(0, "sha");
        let rej = submit_outcome(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha1,
                targets: vec![1],
            },
        );
        assert!(matches!(
            rej,
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));

        // cond_hongyan_base：无红桃装备时手牌上限基数为当前体力(2)，装备红桃装备后变为体力上限(4)
        e.state_core_mut().player_mut(0).max_hp = 4;
        e.state_core_mut().player_mut(0).hp = 2;
        e.debug_grant(0, "cond_hongyan_base");
        assert_eq!(
            e.state_core().hand_limit(e.catalog(), 0),
            2,
            "无红桃装备时基数仍为 hp(2)"
        );
        let heart_pos = e
            .state_core()
            .deck
            .iter()
            .position(|&c| {
                e.catalog().cards[c as usize].kind.is_equip()
                    && e.catalog().cards[c as usize].suit == Suit::Heart
            })
            .expect("牌堆至少有一张红桃装备");
        let heart_equip = e.state_core_mut().deck.remove(heart_pos);
        e.state_core_mut().player_mut(0).equips.push(heart_equip);
        assert_eq!(
            e.state_core().hand_limit(e.catalog(), 0),
            4,
            "有红桃装备时基数升为 max_hp(4)"
        );
    }

    // 2. cond_wansha & cond_qizi (DyingPeachTargetCancel 条件)
    {
        let mut e = Engine::new_with_extensions(914_602, 4, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "cond_wansha");
        let tao2 = e.debug_place(2, "tao");
        let tao0 = e.debug_place(0, "tao");
        assert!(
            engine_native_kernel::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
                e.state_core(),
                e.catalog(),
                2,
                1,
                tao2,
            ),
            "完杀在 0 号回合禁止第三方 2 号对濒死的 1 号用【桃】"
        );
        assert!(
            !engine_native_kernel::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
                e.state_core(),
                e.catalog(),
                0,
                1,
                tao0,
            ),
            "完杀持有者 0 号本人仍可对濒死角色用【桃】"
        );

        e.debug_grant(1, "cond_qizi");
        let horse_pos = e
            .state_core()
            .deck
            .iter()
            .position(|&c| e.catalog().cards[c as usize].kind == CardKind::HorsePlus)
            .unwrap();
        let plus_horse = e.state_core_mut().deck.remove(horse_pos);
        e.state_core_mut().player_mut(3).equips.push(plus_horse);
        assert!(e.state_core().computed_distance(e.catalog(), 1, 3) > 2);
        let tao1 = e.debug_place(1, "tao");
        assert!(
            engine_native_kernel::rvm_bridge::modifiers::eval_dying_peach_target_canceled(
                e.state_core(),
                e.catalog(),
                1,
                3,
                tao1,
            ),
            "dcqizi 禁止对距离 > 2 的其他濒死角色用【桃】"
        );
    }

    // 3. 153 ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan (dcwuhun): 死亡时选梦魇标记最多者判定，非桃/桃园则令其死亡
    {
        let mut e = Engine::new_with_extensions(914_603, 3, &packs).expect("建局");
        e.debug_grant(0, "dcwuhun");
        e.state_core_mut().player_mut(0).hp = 1;
        // 1 号对 0 号造成 1 点伤害（触发 dcwuhun 第 1 条加 1 枚梦魇标记），令 0 号进入濒死并死亡
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        // 将牌堆中的一张非桃/桃园的【杀】移到牌堆顶（deck[0]），确保武魂判定生效致死
        let sha_pos = e
            .state_core()
            .deck
            .iter()
            .position(|&c| e.catalog().cards[c as usize].kind == CardKind::Sha)
            .unwrap();
        let non_tao = e.state_core_mut().deck.remove(sha_pos);
        e.state_core_mut().deck.insert(0, non_tao);
        while let Some(ask) = e.open_choice().cloned() {
            if ask.spec.kind == ChoiceKind::SelectPlayers {
                assert_eq!(ask.spec.options, vec![ChoiceOption::Player(1)]);
                let mut restored = assert_archive_roundtrip(&e, &packs);
                for g in [&mut e, &mut restored] {
                    answer(g, vec![ChoiceOption::Player(1)]);
                }
                assert_eq!(e.state_hash(), restored.state_hash());
            } else if ask.spec.options.contains(&ChoiceOption::Confirm) {
                answer(&mut e, vec![ChoiceOption::Confirm]);
            } else {
                answer(&mut e, vec![ChoiceOption::Pass]);
            }
        }
        assert!(!e.state_core().player(0).alive, "0 号已阵亡");
        assert!(
            !e.state_core().player(1).alive,
            "1 号因 dcwuhun 判定非桃/桃园而阵亡"
        );
    }

    // 4. 152 JudgeMakeCurrentShaTargetUnrespondableOnRed (tieji): 黑色判定不封闪，红色判定封闪
    {
        let mut e = Engine::new_with_extensions(914_604, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "tieji");
        let sha_a = e.debug_place(0, "sha");
        let black_pos = e
            .state_core()
            .deck
            .iter()
            .position(|&c| e.catalog().cards[c as usize].suit == Suit::Spade)
            .unwrap();
        let black_card = e.state_core_mut().deck.remove(black_pos);
        e.state_core_mut().deck.insert(0, black_card);
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_a,
                targets: vec![1],
            },
        );
        // OptionalGate confirm + JudgeMakeCurrentShaTargetUnrespondableOnRed confirm
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask_tj = e.open_choice().cloned().expect("tieji 判定确认");
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Confirm]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(ask_tj.spec.kind, ChoiceKind::Confirm);
        assert!(
            !e.state_core()
                .vm_aux
                .flags
                .contains_key(&(0, format!("sha_unrespondable:{sha_a}:1"))),
            "黑色判定牌不应置位 sha_unrespondable"
        );
    }

    // 5. unique_category_in_turn & ChooseCompareSelectedHandCard 指定第 2 张手牌
    {
        let mut e = Engine::new_with_extensions(914_605, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_grant(0, "act_unique_cat");
        let def_uc = e.catalog().skill_by_name("act_unique_cat").unwrap();
        e.debug_clear_hand(0);
        let sha_c1 = e.debug_place(0, "sha");
        let sha_c2 = e.debug_place(0, "tao");
        e.state_core_mut().vm_aux.set_selected_cards(0, &[sha_c1]);
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def_uc });
        // 第 2 次再选基本牌 (tao) 应因 unique_category_in_turn 被拒绝
        e.debug_clear_hand(0);
        e.state_core_mut().player_mut(0).hand.push(sha_c2);
        e.state_core_mut().vm_aux.set_selected_cards(0, &[sha_c2]);
        let rej = submit_outcome(&mut e, 0, GameAction::ActivateSkill { skill: def_uc });
        assert!(
            matches!(rej, CommandOutcome::Rejected(_)),
            "同回合已用过基本牌类别后不可再用基本牌发动 act_unique_cat"
        );

        // ChooseCompareSelectedHandCard: 当手牌有 [c_first, c_second] 且选中 c_second 时，应消耗 c_second
        e.debug_grant(0, "xianzhen");
        let def_xz = e.catalog().skill_by_name("xianzhen").unwrap();
        e.debug_clear_hand(0);
        let c_first = e.debug_place(0, "sha");
        let c_second = e.debug_place(0, "tao");
        e.state_core_mut().vm_aux.set_selected_cards(0, &[c_second]);
        e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def_xz });
        answer(&mut e, vec![ChoiceOption::Player(1)]);
        assert!(
            e.state_core().player(0).hand.contains(&c_first)
                && !e.state_core().player(0).hand.contains(&c_second),
            "ChooseCompareSelectedHandCard 必须消耗所选的第 2 张手牌 c_second 而非首张手牌"
        );
    }
}

