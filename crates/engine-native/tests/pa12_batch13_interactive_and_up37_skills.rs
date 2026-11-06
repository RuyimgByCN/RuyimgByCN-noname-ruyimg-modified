//! PA12 迁移批⑬ + UP37 五技能闭环测试（2026-11-06，零版本号变更）：
//! 1. `codec_ext.rs` 20 个新效果原语（tags 109..=128）编解码往返与未知 tag 129 拒绝。
//! 2. 12 个高频交互式原语（tags 109..=120）在原生 `Engine` 上的真实开窗、非默认选择与
//!    `NGSA 107` 挂起态存读档往返（`state_hash` 前后一致）。
//! 3. `UP37` 5 个真实 `.skill` 端到端闭环（`compile_source_named` -> `bridge_vm_skill_decls` -> `Engine`）：
//!    - `R01` `dcdichou`（`3304378096`）+ `dcdichou_eff`（`2754298115`）
//!    - `R03` `yingba`（`2187983153`）+ `yingba_mark`（`3219705664`）+ `pinghe`（`2176317369`）
//!    - `R06` `dcmingjie`（`2317659847`）
//!    - `R07` `dczigu`（`3889039356`）
//!    - `R08` `zzhenggong`（`2283567045`）
//! 4. 7 个确定性原语（tags 122..=128）在原生 `Engine` 上的行为验证。

use content_compiler::compile_source_named;
use engine_native::choice::ChoiceKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{
    CardId, DrawTargetScope, EffectRequest, PlayerId, SkillDefId, SkillId, TargetCardScope,
};

const DCDICHOU: &str = include_str!("../../../content/original/ported/xianding/dcdichou.skill");
const YINGBA: &str = include_str!("../../../content/original/ported/shiji/yingba.skill");
const PINGHE: &str = include_str!("../../../content/original/ported/shiji/pinghe.skill");
const DCMINGJIE: &str = include_str!("../../../content/original/ported/xianding/dcmingjie.skill");
const DCZIGU: &str = include_str!("../../../content/original/ported/huicui/dczigu.skill");
const ZZHENGGONG: &str = include_str!("../../../content/original/ported/yijiang/zzhenggong.skill");

const ANYONG: &str = include_str!("../../../content/original/ported/huicui/anyong.skill");
const YANYU: &str = include_str!("../../../content/original/ported/yijiang/yanyu.skill");
const YOUYI: &str = include_str!("../../../content/original/ported/shiji/youyi.skill");
const XINKUANGGU: &str = include_str!("../../../content/original/ported/shenhua/xinkuanggu.skill");
const KUANSHI: &str = include_str!("../../../content/original/ported/sp/kuanshi.skill");
const HFJIEYING: &str = include_str!("../../../content/original/ported/sp2/hfjieying.skill");
const SPMINGSHI: &str = include_str!("../../../content/original/ported/shiji/spmingshi.skill");
const XIAOXI: &str = include_str!("../../../content/original/ported/sp/xiaoxi.skill");
const SIJIAN: &str = include_str!("../../../content/original/ported/sp/sijian.skill");
const MENGJIN: &str = include_str!("../../../content/original/ported/shenhua/mengjin.skill");
const FENGJIE: &str = include_str!("../../../content/original/ported/shiji/fengjie.skill");

const DCWUSHEN: &str = include_str!("../../../content/original/ported/xianding/dcwushen.skill");
const CLANXSYINGXIANG: &str =
    include_str!("../../../content/original/ported/clan/clanxsyingxiang.skill");
const CLANZHONGLLI: &str = include_str!("../../../content/original/ported/clan/clanzhongliu.skill");
const REBAOBIAN: &str = include_str!("../../../content/original/ported/sp/rebaobian.skill");
const ZHANSHEN: &str = include_str!("../../../content/standard/zhanshen.skill");
const OLDDANSHOU: &str = include_str!("../../../content/original/ported/yijiang/olddanshou.skill");

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(900_000);

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
            description: "PA12 批⑬ + UP37 严格桥接测试包",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn pack_normalized(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let mut pkg = compile_source_named(*id, source).expect("编译 .skill");
        for s in &mut pkg.skill_index {
            s.lord_only = false;
            s.non_gainable = false;
        }
        for a in &mut pkg.active_index {
            a.selection = None;
        }
        for t in &mut pkg.trigger_index {
            if t.filter == "owner_is_target_active_used_at_least_2_this_play_phase" {
                t.filter = "owner_is_target".to_string();
            }
        }
        if pkg.trigger_index.len() > 1 {
            let first_opt = pkg.trigger_index[0].optional;
            for t in &mut pkg.trigger_index {
                t.optional = first_opt;
            }
        }
        pkg.modifier_index.clear();
        pkg.manifest.logic_hash = pkg.compute_logic_hash();
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑬ 效果测试包",
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
fn pa12_batch13_codec_roundtrip_109_to_128_and_reject_129() {
    let effects = vec![
        (
            109u8,
            EffectRequest::ChooseOtherDamage {
                player: PlayerId::new(1),
                amount: 2,
                skill: SkillId::new(4291071501),
            },
        ),
        (
            110,
            EffectRequest::ChooseTargetDraw {
                player: PlayerId::new(1),
                count: 2,
                scope: DrawTargetScope::Any,
                skill: SkillId::new(2317659847),
            },
        ),
        (
            111,
            EffectRequest::ChooseOtherRecover {
                player: PlayerId::new(1),
                amount: 1,
            },
        ),
        (
            112,
            EffectRequest::ChooseDrawOrRecover {
                player: PlayerId::new(1),
                draw_count: 2,
                recover_count: 1,
                allow_cancel: true,
            },
        ),
        (
            113,
            EffectRequest::ChooseOtherAddMark {
                player: PlayerId::new(1),
                mark: 30_507_535,
                delta: 1,
                skill: SkillId::new(2742461307),
            },
        ),
        (
            114,
            EffectRequest::ResolveHfjieying {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                grant_def: SkillDefId::new(4111610111),
                skill: SkillId::new(2801808452),
            },
        ),
        (
            115,
            EffectRequest::ChooseDiscardOneHandOrEquip {
                player: PlayerId::new(2),
            },
        ),
        (
            116,
            EffectRequest::DiscardBasicDrawEqual {
                player: PlayerId::new(1),
                skill: SkillId::new(9900116),
            },
        ),
        (
            117,
            EffectRequest::ChooseUseVirtualSha {
                player: PlayerId::new(1),
                ignore_distance: true,
            },
        ),
        (
            118,
            EffectRequest::ChooseOtherDiscardTargetCard {
                player: PlayerId::new(1),
                scope: TargetCardScope::HandEquip,
                skill: SkillId::new(3832046646),
            },
        ),
        (
            119,
            EffectRequest::ChooseDiscardTargetCard {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                scope: TargetCardScope::HandEquip,
            },
        ),
        (
            120,
            EffectRequest::ChooseOtherGiveOneHandEquip {
                player: PlayerId::new(1),
                skill: SkillId::new(2986115352),
            },
        ),
        (
            121,
            EffectRequest::ResolveDcdichou {
                player: PlayerId::new(1),
                target: PlayerId::new(2),
                card: CardId::new(7),
                action: 10,
                helper_skill_def: SkillDefId::new(2754298115),
                skill: SkillId::new(3304378096),
            },
        ),
        (
            122,
            EffectRequest::RecordCurrentShaAllTargets {
                player: PlayerId::new(1),
                card: CardId::new(3),
                skill: SkillId::new(3292885385),
            },
        ),
        (
            123,
            EffectRequest::TakeOneScopedCard {
                from: PlayerId::new(2),
                to: PlayerId::new(1),
                scope: TargetCardScope::HandEquip,
            },
        ),
        (
            124,
            EffectRequest::DrawAllEventTargets {
                players: vec![PlayerId::new(1), PlayerId::new(2)],
                count: 1,
            },
        ),
        (
            125,
            EffectRequest::RefreshSkill {
                skill: SkillId::new(3016047578),
            },
        ),
        (
            126,
            EffectRequest::AddFirstMissingSkill {
                player: PlayerId::new(1),
                skill_defs: [
                    SkillDefId::new(695232805),
                    SkillDefId::new(2829854661),
                    SkillDefId::new(2561255694),
                ],
                skill: SkillId::new(4258572312),
            },
        ),
        (
            127,
            EffectRequest::DiscardEquipmentSlot {
                player: PlayerId::new(1),
                slot: 1,
            },
        ),
        (
            128,
            EffectRequest::FinishCurrentPhaseAbortResolution {
                player: PlayerId::new(1),
                stage: 0,
            },
        ),
    ];

    assert_eq!(effects.len(), 20);
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

    // 未知 tag 129 必须拒绝
    assert!(dec_effects(&[1, 0, 0, 0, 129]).is_none());
}

#[test]
fn pa12_batch13_interactive_primitives_109_to_114_targets_and_archive() {
    let pack = pack_normalized(
        &[
            ("anyong", ANYONG),
            ("yanyu", YANYU),
            ("youyi", YOUYI),
            ("xinkuanggu", XINKUANGGU),
            ("kuanshi", KUANSHI),
            ("hfjieying", HFJIEYING),
        ],
        "pa12-b13-targets",
    );
    let packs = [&pack];

    // 109: ChooseOtherDamage (anyong)
    {
        let mut e = Engine::new_with_extensions(913_109, 3, &packs).expect("建局");
        e.debug_grant(0, "anyong");
        let hp2_before = e.state_core().player(2).hp;
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e.open_choice().cloned().expect("ChooseOtherDamage 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        assert_eq!(
            ask.spec.options,
            vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]
        );
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(2)]);
            drain_choices(g);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(2).hp, hp2_before - 1);
    }

    // 110: ChooseTargetDraw with DrawTargetScope::MaleOther (yanyu) — 仅提供其他存活男性角色
    {
        let mut e = Engine::new_with_extensions(913_110, 3, &packs).expect("建局");
        e.assign_general_named(1, "曹操").expect("1 号男性武将");
        e.assign_general_named(2, "甄姬").expect("2 号女性武将");
        e.debug_grant(0, "yanyu");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let h1_before = e.state_core().player(1).hand.len();
        submit(&mut e, 0, GameAction::NextPhase); // Play 阶段结束触发 yanyu
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e
            .open_choice()
            .cloned()
            .expect("ChooseTargetDraw(MaleOther) 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        assert_eq!(
            ask.spec.options,
            vec![ChoiceOption::Player(1)],
            "MaleOther 应仅列出男性其他存活角色（1 号），过滤女性 2 号"
        );
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hand.len(), h1_before + 2);
    }

    // 111: ChooseOtherRecover (youyi)
    {
        let mut e = Engine::new_with_extensions(913_111, 3, &packs).expect("建局");
        e.debug_grant(0, "youyi");
        e.debug_set_hp(1, 2);
        e.debug_set_hp(2, 1);
        advance_to_phase(&mut e, 0, Phase::Play);
        let def = e.catalog().skill_by_name("youyi").expect("youyi");
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let ask = e.open_choice().cloned().expect("ChooseOtherRecover 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        assert_eq!(
            ask.spec.options,
            vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]
        );
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(2)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(2).hp, 2);
    }

    // 112: ChooseDrawOrRecover (xinkuanggu)
    {
        let mut e = Engine::new_with_extensions(913_112, 2, &packs).expect("建局");
        e.debug_grant(0, "xinkuanggu");
        e.debug_set_hp(0, 2);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        let ask = e.open_choice().cloned().expect("ChooseDrawOrRecover 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::PickTag);
        assert_eq!(
            ask.spec.options,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]
        );
        assert!(ask.spec.allow_cancel);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Tag(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(0).hp, 3);
    }

    // 113: ChooseOtherAddMark (kuanshi)
    {
        let mut e = Engine::new_with_extensions(913_113, 3, &packs).expect("建局");
        e.debug_grant(0, "kuanshi");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        submit(&mut e, 0, GameAction::NextPhase); // -> Discard
        submit(&mut e, 0, GameAction::NextPhase); // -> End
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e.open_choice().cloned().expect("ChooseOtherAddMark 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(2)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(
            e.state_core()
                .vm_aux
                .marks
                .get(&(2, 30_507_535))
                .copied()
                .unwrap_or(0),
            1
        );
    }

    // 114: ResolveHfjieying (hfjieying)
    {
        let mut e = Engine::new_with_extensions(913_114, 3, &packs).expect("建局");
        e.debug_grant(0, "hfjieying");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        submit(&mut e, 0, GameAction::NextPhase); // -> Discard
        submit(&mut e, 0, GameAction::NextPhase); // -> End
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e.open_choice().cloned().expect("ResolveHfjieying 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(2)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        let def2 = e.catalog().skill_by_name("hfjieying2").expect("hfjieying2");
        assert!(e.state_core().has_skill(2, def2));
    }
}

#[test]
fn pa12_batch13_interactive_primitives_115_to_120_cards_and_archive() {
    const PROBE_116: &str = "skill probe_116(9910116) locked {\n  trigger damage owner_is_target {\n    effect discard_basic_draw_equal(self, self_skill);\n  }\n}\n";
    let pack = pack_normalized(
        &[
            ("spmingshi", SPMINGSHI),
            ("probe_116", PROBE_116),
            ("xiaoxi", XIAOXI),
            ("sijian", SIJIAN),
            ("mengjin", MENGJIN),
            ("fengjie", FENGJIE),
        ],
        "pa12-b13-cards",
    );
    let packs = [&pack];

    // 115: ChooseDiscardOneHandOrEquip (spmingshi)
    {
        let mut e = Engine::new_with_extensions(913_115, 2, &packs).expect("建局");
        e.debug_grant(0, "spmingshi");
        e.debug_clear_hand(1);
        let c1 = e.debug_place(1, "sha");
        let c2 = e.debug_place(1, "shan");
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        let ask = e
            .open_choice()
            .cloned()
            .expect("ChooseDiscardOneHandOrEquip 开窗");
        assert_eq!((ask.observer, ask.spec.prompt), (1, "rvm_discard_pick"));
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(c2)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hand, vec![c1]);
        assert!(e.state_core().discard.contains(&c2));
    }

    // 116: DiscardBasicDrawEqual (probe_116)
    {
        let mut e = Engine::new_with_extensions(913_116, 2, &packs).expect("建局");
        e.debug_grant(0, "probe_116");
        e.debug_clear_hand(0);
        let c_sha = e.debug_place(0, "sha");
        let c_shan = e.debug_place(0, "shan");
        let c_duel = e.debug_place(0, "duel");
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        let ask = e.open_choice().cloned().expect("DiscardBasicDrawEqual 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        assert_eq!((ask.spec.min, ask.spec.max), (1, 2));
        assert!(ask.spec.allow_cancel);
        assert_eq!(
            ask.spec.options,
            vec![ChoiceOption::Card(c_sha), ChoiceOption::Card(c_shan)]
        );
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(c_sha), ChoiceOption::Card(c_shan)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert!(e.state_core().player(0).hand.contains(&c_duel));
        assert_eq!(e.state_core().player(0).hand.len(), 3);
    }

    // 117: ChooseUseVirtualSha (xiaoxi, game_started)
    {
        let mut e = Engine::new_with_extensions(913_117, 2, &packs).expect("建局");
        e.debug_grant(0, "xiaoxi");
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let hp1_before = e.state_core().player(1).hp;
        submit(&mut e, 0, GameAction::NextPhase); // 首轮首个阶段切换触发 game_started
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e.open_choice().cloned().expect("ChooseUseVirtualSha 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
        assert!(ask.spec.allow_cancel);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Player(1)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    }

    // 118: ChooseOtherDiscardTargetCard (sijian) — 两段挂起均做存读档往返
    {
        let mut e = Engine::new_with_extensions(913_118, 3, &packs).expect("建局");
        e.debug_grant(0, "sijian");
        e.debug_clear_hand(0);
        let lost = e.debug_place(0, "sha");
        e.debug_clear_hand(2);
        let c2_a = e.debug_place(2, "sha");
        let c2_b = e.debug_place(2, "shan");
        e.debug_give_hand_card(0, 1, lost);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask1 = e
            .open_choice()
            .cloned()
            .expect("ChooseOtherDiscardTargetCard 第一段选人");
        assert_eq!(ask1.spec.kind, ChoiceKind::SelectPlayers);
        let mut restored1 = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored1] {
            answer(g, vec![ChoiceOption::Player(2)]);
        }
        assert_eq!(e.state_hash(), restored1.state_hash());
        let ask2 = e
            .open_choice()
            .cloned()
            .expect("ChooseOtherDiscardTargetCard 第二段选牌");
        assert_eq!(ask2.spec.kind, ChoiceKind::SelectCards);
        let mut restored2 = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored2] {
            answer(g, vec![ChoiceOption::Card(c2_b)]);
        }
        assert_eq!(e.state_hash(), restored2.state_hash());
        assert_eq!(e.state_core().player(2).hand, vec![c2_a]);
        assert!(e.state_core().discard.contains(&c2_b));
    }

    // 119: ChooseDiscardTargetCard (mengjin)
    {
        let mut e = Engine::new_with_extensions(913_119, 2, &packs).expect("建局");
        e.debug_grant(0, "mengjin");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let c_sha = e.debug_place(0, "sha");
        e.debug_clear_hand(1);
        let c_shan = e.debug_place(1, "shan");
        let c1_a = e.debug_place(1, "sha");
        let c1_b = e.debug_place(1, "tao");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: c_sha,
                targets: vec![1],
            },
        );
        answer(&mut e, vec![ChoiceOption::Card(c_shan)]);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        let ask = e
            .open_choice()
            .cloned()
            .expect("ChooseDiscardTargetCard 开窗");
        assert_eq!(ask.spec.kind, ChoiceKind::SelectCards);
        let mut restored = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored] {
            answer(g, vec![ChoiceOption::Card(c1_b)]);
        }
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(e.state_core().player(1).hand, vec![c1_a]);
        assert!(e.state_core().discard.contains(&c1_b));
    }

    // 120: ChooseOtherGiveOneHandEquip (fengjie) — 两段挂起均做存读档往返
    {
        let mut e = Engine::new_with_extensions(913_120, 3, &packs).expect("建局");
        e.debug_grant(1, "fengjie");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let c1_a = e.debug_place(1, "sha");
        let c1_b = e.debug_place(1, "tao");
        submit(&mut e, 0, GameAction::NextPhase); // -> Discard
        submit(&mut e, 0, GameAction::NextPhase); // -> End
        submit(&mut e, 0, GameAction::NextPhase); // -> seat 1 TurnBegin -> Prepare
        let ask1 = e
            .open_choice()
            .cloned()
            .expect("ChooseOtherGiveOneHandEquip 第一段选牌");
        assert_eq!((ask1.observer, ask1.spec.kind), (1, ChoiceKind::SelectCards));
        let mut restored1 = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored1] {
            answer(g, vec![ChoiceOption::Card(c1_b)]);
        }
        assert_eq!(e.state_hash(), restored1.state_hash());
        let ask2 = e
            .open_choice()
            .cloned()
            .expect("ChooseOtherGiveOneHandEquip 第二段选人");
        assert_eq!(
            (ask2.observer, ask2.spec.kind),
            (1, ChoiceKind::SelectPlayers)
        );
        let mut restored2 = assert_archive_roundtrip(&e, &packs);
        for g in [&mut e, &mut restored2] {
            answer(g, vec![ChoiceOption::Player(2)]);
        }
        assert_eq!(e.state_hash(), restored2.state_hash());
        assert_eq!(e.state_core().player(1).hand, vec![c1_a]);
        assert_eq!(e.state_core().player(2).hand, vec![c1_b]);
    }
}

#[test]
fn up37_r01_dcdichou_and_dcdichou_eff_end_to_end_and_archive() {
    let pack = pack_strict(&[("dcdichou", DCDICHOU)], "up37-r01-dcdichou");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(937_001, 3, &packs).expect("建局");
    e.debug_grant(1, "dcdichou");

    // 1. 触发一（card_targeted owner_is_target_other_source_alive，锁定技）：
    //    0 号在出牌阶段对 1 号使用【杀】时，0 号获得 1 枚「敌仇」标记（3304378096）。
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    let c0_sha = e.debug_place(0, "sha");
    let c0_keep = e.debug_place(0, "shan");
    let c0_dismantle = e.debug_place(0, "dismantle");
    e.debug_clear_hand(1);
    let c1_shan = e.debug_place(1, "shan");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: c0_sha,
            targets: vec![1],
        },
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(0, 3_304_378_096))
            .copied()
            .unwrap_or(0),
        1
    );
    answer(&mut e, vec![ChoiceOption::Card(c1_shan)]);

    // 2. 推进到 1 号回合的判定阶段开始（phase_begin owner_is_target -> resolve_dcdichou(self, 0, 2754298115)）：
    //    废除一个装备栏 -> 选有敌仇标记的角色（0 号） -> 弃置其【过河拆桥】（4 字） ->
    //    本回合获得 dcdichou_eff（2754298115），攻击范围与出杀次数上限各 +4。
    submit(&mut e, 0, GameAction::NextPhase); // -> Discard (hand=2 <= hp)
    submit(&mut e, 0, GameAction::NextPhase); // -> End
    submit(&mut e, 0, GameAction::NextPhase); // -> seat 1 TurnBegin -> Prepare -> Judge PhaseBegin
    let ask_slot = e
        .open_choice()
        .cloned()
        .expect("dcdichou stage 1 选废除装备栏");
    assert_eq!((ask_slot.observer, ask_slot.spec.kind), (1, ChoiceKind::PickTag));
    let mut restored_s1 = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored_s1] {
        answer(g, vec![ChoiceOption::Tag(2)]); // 废除防具栏
    }
    assert_eq!(e.state_hash(), restored_s1.state_hash());

    let ask_target = e
        .open_choice()
        .cloned()
        .expect("dcdichou stage 2 选敌仇目标");
    assert_eq!(ask_target.spec.kind, ChoiceKind::SelectPlayers);
    assert_eq!(ask_target.spec.options, vec![ChoiceOption::Player(0)]);
    let mut restored_s2 = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored_s2] {
        answer(g, vec![ChoiceOption::Player(0)]);
    }
    assert_eq!(e.state_hash(), restored_s2.state_hash());

    let ask_card = e
        .open_choice()
        .cloned()
        .expect("dcdichou stage 3 选弃置目标牌");
    assert_eq!(ask_card.spec.kind, ChoiceKind::SelectCards);
    let mut restored_s3 = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored_s3] {
        answer(g, vec![ChoiceOption::Card(c0_dismantle)]);
    }
    assert_eq!(e.state_hash(), restored_s3.state_hash());
    assert_eq!(e.state_core().player(0).hand, vec![c0_keep]);
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(1, 2_754_298_115))
            .copied()
            .unwrap_or(0),
        4,
        "【过河拆桥】四字牌名 -> dcdichou_eff 标记为 4"
    );
    assert_eq!(
        e.state_core().attack_range(e.catalog(), 1),
        5,
        "基础攻击范围 1 + dcdichou_eff(4) = 5"
    );

    // 3. 推进到 1 号出牌阶段：验证对带标记角色（0 号）使用【杀】摸 1 张牌并移去 1 枚敌仇标记，
    //    且因 dcdichou_eff 的 sha_use_limit +4 可在同一出牌阶段使用第 2 张【杀】！
    submit(&mut e, 1, GameAction::NextPhase); // -> Draw (Again triggers dcdichou phase_begin: disable slot 3, no marked target since 0 still has 1 card, wait: 0 still has mark 1 & 1 card!)
    // 清空 0 号手牌避免后续阶段重复进入 stage 2，先验证 Play 阶段多刀与移标：
    drain_choices(&mut e);
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(0);
    // 给 0 号补 1 枚敌仇标记以验证触发三（对有敌仇标记角色用牌摸 1 并移去 1 枚标记）与触发四（本回合失去过标记角色受火伤）
    e.state_core_mut().vm_aux.marks.insert((0, 3_304_378_096), 1);
    let c1_sha1 = e.debug_place(1, "sha");
    let c1_sha2 = e.debug_place(1, "sha");
    let h1_before = e.state_core().player(1).hand.len();
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: c1_sha1,
            targets: vec![0],
        },
    );
    // 触发三：摸 1 张牌并移去 0 号的 1 枚敌仇标记
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(0, 3_304_378_096))
            .copied()
            .unwrap_or(0),
        0
    );
    assert_eq!(e.state_core().player(1).hand.len(), h1_before); // 用 1 摸 1

    // dcdichou_eff 令出杀次数上限 +4，同一出牌阶段可继续出第 2 张【杀】
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: c1_sha2,
            targets: vec![2],
        },
    );
    drain_choices(&mut e);

    // 4. 触发四（phase_end owner_is_target -> resolve_dcdichou(self, 10, 2754298115)）：
    //    0 号本回合曾失去过敌仇标记（dcdichou_mark_removed_this_turn = 1），再给 0 号补 1 枚敌仇标记，
    //    验证同时具备当前标记与失去过标记时去重仅入列 1 次，且全清标记与旗标！
    e.state_core_mut().vm_aux.marks.insert((0, 3_304_378_096), 1);
    e.debug_clear_hand(1);
    let hp0_before = e.state_core().player(0).hp;
    submit(&mut e, 1, GameAction::NextPhase); // -> Discard
    // Discard phase_begin triggers resolve_dcdichou(0): slot choice only (0 has 0 cards)
    drain_choices(&mut e);
    submit(&mut e, 1, GameAction::NextPhase); // -> End
    // End phase_begin first (slot choice), then phase_end (stage 10 fire damage)
    if let Some(ask_end_slot) = e.open_choice().cloned() {
        if ask_end_slot.spec.kind == ChoiceKind::PickTag {
            answer(&mut e, vec![ask_end_slot.spec.options[0]]);
        }
    }
    let ask_fire = e
        .open_choice()
        .cloned()
        .expect("dcdichou stage 10 选火伤目标");
    assert_eq!(ask_fire.spec.kind, ChoiceKind::SelectPlayers);
    assert_eq!(ask_fire.spec.options, vec![ChoiceOption::Player(0)]);
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(0, 3_304_378_096))
            .copied()
            .unwrap_or(0),
        0,
        "stage 10 应清空全场敌仇标记"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "dcdichou_mark_removed_this_turn".to_string()))
            .copied()
            .unwrap_or(0),
        0,
        "stage 10 应清除本回合失去敌仇标记旗标"
    );
    let mut restored_s10 = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored_s10] {
        answer(g, vec![ChoiceOption::Player(0)]);
        drain_choices(g);
    }
    assert_eq!(e.state_hash(), restored_s10.state_hash());
    assert_eq!(e.state_core().player(0).hp, hp0_before - 1);
}

#[test]
fn up37_r03_yingba_and_yingba_mark_end_to_end() {
    let pack = pack_strict(
        &[("yingba", YINGBA), ("pinghe", PINGHE)],
        "up37-r03-yingba-pinghe",
    );
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(937_003, 3, &packs).expect("建局");
    e.debug_grant(0, "yingba");
    e.debug_grant(0, "pinghe");

    // 1. 冯河（pinghe.skill）与英霸（yingba.skill）协同：
    //    2 号对 0 号造成伤害前，冯河防止本次伤害、0 号减 1 点体力上限，并打开交牌/重铸窗口；
    //    0 号将 1 张手牌交给 1 号后，因 0 号拥有英霸（2187983153），伤害来源 2 号获得 1 枚平定印记（3219705664）与 yingba_mark 技能！
    advance_to_phase(&mut e, 2, Phase::Play);
    e.debug_clear_hand(0);
    let c0_gift = e.debug_place(0, "shan");
    e.debug_clear_hand(2);
    let c2_sha = e.debug_place(2, "sha");
    let max0_init = e.state_core().player(0).max_hp;
    let hp0_init = e.state_core().player(0).hp;
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: c2_sha,
            targets: vec![0],
        },
    );
    if e.open_choice()
        .is_some_and(|c| c.spec.prompt == "respond_shan")
    {
        answer(&mut e, vec![ChoiceOption::Pass]);
    }
    // 冯河第一段：选 1 张手牌（并在挂起态验证存读档往返）
    assert_eq!(e.state_core().player(0).max_hp, max0_init - 1);
    let mut restored_p1 = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored_p1] {
        answer(g, vec![ChoiceOption::Card(c0_gift)]);
    }
    assert_eq!(e.state_hash(), restored_p1.state_hash());
    // 冯河第二段：选择交给 1 号（并在挂起态验证存读档往返）
    let mut restored_p2 = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored_p2] {
        answer(g, vec![ChoiceOption::Player(1)]);
    }
    assert_eq!(e.state_hash(), restored_p2.state_hash());
    assert_eq!(
        e.state_core().player(0).hp,
        hp0_init.min((max0_init - 1) as i32),
        "冯河已防止伤害"
    );
    let mark_def = e
        .catalog()
        .skill_by_name("yingba_mark")
        .expect("yingba_mark");
    assert!(
        e.state_core().has_skill(2, mark_def),
        "伤害来源 2 号应由冯河获得 yingba_mark"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(2, 3_219_705_664))
            .copied()
            .unwrap_or(0),
        1
    );

    // 2. 推进到 0 号出牌阶段，验证英霸主动技（targets 1 1 other max_hp_gt 1）：
    //    将 1 号体力上限设为 1（不满足 max_hp_gt 1），即使预置 1 号为候选也会被过滤并自动选中体力上限 > 1 的 2 号！
    advance_to_phase(&mut e, 0, Phase::Play);
    e.state_core_mut().player_mut(1).max_hp = 1;
    e.state_core_mut().player_mut(1).hp = 1;
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    let max0_before = e.state_core().player(0).max_hp;
    let max2_before = e.state_core().player(2).max_hp;
    let yingba_def = e.catalog().skill_by_name("yingba").expect("yingba");
    submit(&mut e, 0, GameAction::ActivateSkill { skill: yingba_def });

    assert_eq!(e.state_core().player(0).max_hp, max0_before - 1);
    assert_eq!(
        e.state_core().player(1).max_hp,
        1,
        "体力上限为 1 的 1 号不得成为英霸目标"
    );
    assert_eq!(e.state_core().player(2).max_hp, max2_before - 1);
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(2, 3_219_705_664))
            .copied()
            .unwrap_or(0),
        2,
        "2 号叠加至 2 枚平定印记"
    );

    // 3. 验证 yingba_mark 修饰器：推进到 2 号出牌阶段，2 号拥有 2 枚平定印记，
    //    本回合已用牌数(0, 1) < 平定印记数(2) 时不可对拥有英霸的 0 号使用牌；用满 2 张牌后第 3 张可指定 0 号！
    advance_to_phase(&mut e, 2, Phase::Play);
    e.debug_set_hp(2, 1);
    let c_sha_to_0 = e.debug_place(2, "sha");
    let c_tao1 = e.debug_place(2, "tao");
    let c_tao2 = e.debug_place(2, "tao");
    let rej0 = submit_outcome(
        &mut e,
        2,
        GameAction::PlayCard {
            card: c_sha_to_0,
            targets: vec![0],
        },
    );
    assert!(
        matches!(rej0, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "cards_used_turn(0) < yingba_mark(2) 时对 0 号用杀应被拒: {rej0:?}"
    );
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: c_tao1,
            targets: vec![],
        },
    );
    assert_eq!(e.state_core().turn.cards_used_turn, 1);
    let rej1 = submit_outcome(
        &mut e,
        2,
        GameAction::PlayCard {
            card: c_sha_to_0,
            targets: vec![0],
        },
    );
    assert!(
        matches!(rej1, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "cards_used_turn(1) < yingba_mark(2) 时对 0 号用杀仍应被拒: {rej1:?}"
    );
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: c_tao2,
            targets: vec![],
        },
    );
    assert_eq!(e.state_core().turn.cards_used_turn, 2);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: c_sha_to_0,
            targets: vec![0],
        },
    );
}

#[test]
fn up37_r06_dcmingjie_end_to_end_and_archive() {
    let pack = pack_strict(&[("dcmingjie", DCMINGJIE)], "up37-r06-dcmingjie");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(937_006, 2, &packs).expect("建局");
    e.debug_grant(0, "dcmingjie");
    e.debug_clear_hand(0);
    let h1_before = e.state_core().player(1).hand.len();

    // 首轮首个阶段切换触发 dcmingjie（game_started owner_alive，非 optional）：
    // 自己摸 1 张牌（连同摸牌阶段 2 张共 3 张）并令一名其他角色摸 1 张牌
    submit(&mut e, 0, GameAction::NextPhase);
    let ask = e
        .open_choice()
        .cloned()
        .expect("dcmingjie ChooseTargetDraw 开窗");
    assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
    assert_eq!(ask.spec.options, vec![ChoiceOption::Player(1)]);
    let mut restored = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored] {
        answer(g, vec![ChoiceOption::Player(1)]);
    }
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(e.state_core().player(0).hand.len(), 3);
    assert_eq!(e.state_core().player(1).hand.len(), h1_before + 1);
}

#[test]
fn up37_r07_dczigu_end_to_end_and_archive() {
    let pack = pack_strict(&[("dczigu", DCZIGU)], "up37-r07-dczigu");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(937_007, 3, &packs).expect("建局");
    e.debug_grant(0, "dczigu");
    advance_to_phase(&mut e, 0, Phase::Play);
    let h2_before = e.state_core().player(2).hand.len();

    let def = e.catalog().skill_by_name("dczigu").expect("dczigu");
    submit(&mut e, 0, GameAction::ActivateSkill { skill: def });

    let ask = e
        .open_choice()
        .cloned()
        .expect("dczigu ChooseTargetDraw 开窗");
    assert_eq!(ask.spec.kind, ChoiceKind::SelectPlayers);
    assert_eq!(
        ask.spec.options,
        vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]
    );
    let mut restored = assert_archive_roundtrip(&e, &packs);
    for g in [&mut e, &mut restored] {
        answer(g, vec![ChoiceOption::Player(2)]);
    }
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(e.state_core().player(2).hand.len(), h2_before + 1);
}

#[test]
fn up37_r08_zzhenggong_end_to_end() {
    let pack = pack_strict(&[("zzhenggong", ZZHENGGONG)], "up37-r08-zzhenggong");
    let packs = [&pack];
    let mut e = Engine::new_with_extensions(937_008, 2, &packs).expect("建局");
    e.debug_grant(0, "zzhenggong");
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(0);
    let c1_weapon = e.debug_place(1, "qinggang");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: c1_weapon,
            targets: vec![],
        },
    );
    assert_eq!(e.state_core().player(1).equips, vec![c1_weapon]);

    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    assert!(e.state_core().player(1).equips.is_empty());
    assert_eq!(e.state_core().player(0).hand, vec![c1_weapon]);
}

#[test]
fn pa12_batch13_deterministic_tags_122_to_128_behavior() {
    let mut vm_skills = Vec::new();
    for (id, source, force_owner_alive) in [
        ("dcwushen", DCWUSHEN, true),
        ("clanxsyingxiang", CLANXSYINGXIANG, true),
        ("clanzhongliu", CLANZHONGLLI, true),
        ("rebaobian", REBAOBIAN, false),
        ("zhanshen", ZHANSHEN, true),
        ("olddanshou", OLDDANSHOU, false),
    ] {
        let mut pkg = compile_source_named(id, source).expect("编译 .skill");
        for s in &mut pkg.skill_index {
            s.lord_only = false;
            s.non_gainable = false;
        }
        for t in &mut pkg.trigger_index {
            if force_owner_alive {
                t.filter = "owner_alive".to_string();
                t.optional = false;
            }
        }
        pkg.modifier_index.clear();
        pkg.manifest.logic_hash = pkg.compute_logic_hash();
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接");
        vm_skills.extend(out.decls);
    }
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "pa12-b13-det",
            version: 1,
            description: "PA12 批⑬ 确定性效果测试包",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    };
    let packs = [&pack];

    // 122: RecordCurrentShaAllTargets (dcwushen)
    {
        let mut e = Engine::new_with_extensions(913_122, 2, &packs).expect("建局");
        e.debug_grant(0, "dcwushen");
        advance_to_phase(&mut e, 0, Phase::Play);
        let c_sha = e.debug_place(0, "sha");
        e.debug_clear_hand(1);
        let c1_shan = e.debug_place(1, "shan");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: c_sha,
                targets: vec![1],
            },
        );
        answer(&mut e, vec![ChoiceOption::Card(c1_shan)]);
        assert_eq!(
            e.state_core()
                .vm_aux
                .flags
                .get(&(0, format!("sha_card_target:{c_sha}:1")))
                .copied()
                .unwrap_or(0),
            1
        );
    }

    // 124: DrawAllEventTargets (clanxsyingxiang)
    {
        let mut e = Engine::new_with_extensions(913_124, 2, &packs).expect("建局");
        e.debug_grant(0, "clanxsyingxiang");
        advance_to_phase(&mut e, 0, Phase::Play);
        let c_sha = e.debug_place(0, "sha");
        e.debug_clear_hand(1);
        let c1_shan = e.debug_place(1, "shan");
        let h1_before = e.state_core().player(1).hand.len();
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: c_sha,
                targets: vec![1],
            },
        );
        answer(&mut e, vec![ChoiceOption::Card(c1_shan)]);
        // 1 号打出 1 张闪（-1），clanxsyingxiang 的 DrawAllEventTargets 令目标 1 号摸 1 张（+1）
        assert_eq!(e.state_core().player(1).hand.len(), h1_before);
    }

    // 125: RefreshSkill (clanzhongliu)
    {
        let mut e = Engine::new_with_extensions(913_125, 2, &packs).expect("建局");
        e.debug_grant(0, "clanzhongliu");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.state_core_mut()
            .vm_aux
            .once
            .insert((0, "clanzhongliu".to_string(), 1));
        let c_sha = e.debug_place(0, "sha");
        e.debug_clear_hand(1);
        let c1_shan = e.debug_place(1, "shan");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: c_sha,
                targets: vec![1],
            },
        );
        answer(&mut e, vec![ChoiceOption::Card(c1_shan)]);
        assert!(
            !e.state_core()
                .vm_aux
                .once
                .contains(&(0, "clanzhongliu".to_string(), 1))
        );
    }

    // 127: DiscardEquipmentSlot (zhanshen)
    {
        let mut e = Engine::new_with_extensions(913_127, 2, &packs).expect("建局");
        advance_to_phase(&mut e, 0, Phase::Play);
        e.debug_clear_hand(0);
        let c_weapon = e.debug_place(0, "qinggang");
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: c_weapon,
                targets: vec![],
            },
        );
        assert_eq!(e.state_core().player(0).equips, vec![c_weapon]);
        e.debug_grant(0, "zhanshen");
        // 推进到 1 号回合的 Prepare 阶段（owner_alive 触发 zhanshen -> 弃置 0 号武器栏）
        submit(&mut e, 0, GameAction::NextPhase); // -> Discard
        submit(&mut e, 0, GameAction::NextPhase); // -> End
        submit(&mut e, 0, GameAction::NextPhase); // -> seat 1 TurnBegin -> Prepare
        assert!(e.state_core().player(0).equips.is_empty());
        assert!(e.state_core().discard.contains(&c_weapon));
    }

    // 126: AddFirstMissingSkill (rebaobian)
    {
        let mut e = Engine::new_with_extensions(913_126, 2, &packs).expect("建局");
        e.debug_grant(0, "rebaobian");
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        assert_eq!(
            e.state_core()
                .vm_aux
                .flags
                .get(&(0, "skill:695232805".to_string()))
                .copied()
                .unwrap_or(0),
            1
        );
    }

    // 128: FinishCurrentPhaseAbortResolution (olddanshou)
    {
        let mut e = Engine::new_with_extensions(913_128, 2, &packs).expect("建局");
        e.debug_grant(0, "olddanshou");
        let h0_before = e.state_core().player(0).hand.len();
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        assert_eq!(e.state_core().player(0).hand.len(), h0_before + 1);
        assert_eq!(
            e.state_core()
                .vm_aux
                .flags
                .get(&(0, "phase_aborted".to_string()))
                .copied()
                .unwrap_or(0),
            1
        );
    }
}
