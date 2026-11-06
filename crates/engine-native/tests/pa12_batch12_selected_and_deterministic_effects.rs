//! PA12 迁移批⑫（2026-11-06，零版本号变更）：
//! 1. `rvm_ops/codec_ext.rs` 与 `rvm_ops/commit_ext.rs` 模块拆分（保持单文件有效代码远低于 1,000 行门禁）。
//! 2. `EngineVmHost` 与 `VmAux` 接入 `selected_cards` / `selected_targets` 预选通道。
//! 3. `rvm_ops` 接入 25 个确定性效果原语（tags 84..=108）：
//!    - `DiscardCards`（tag 84）
//!    - `StoreSelectedInSkillPile`（tag 85）
//!    - `UseVirtualCardOn`（tag 86）
//!    - `RecastCards`（tag 87）
//!    - `AddCardUsePolicySkill`（tag 88）
//!    - `DrawSelectedTargets`（tag 89）
//!    - `QueueExtraTurn`（tag 90）
//!    - `DisableEquipSlots`（tag 91）
//!    - `EnableEquipSlot`（tag 92）
//!    - `GainShadowCards`（tag 93）
//!    - `GainTargetHandCard`（tag 94）
//!    - `AddBoundSkill`（tag 95）
//!    - `GiveSelectedThenRecoverTarget`（tag 96）
//!    - `DistributeShadowCards`（tag 97）
//!    - `PlaceHandCardToExpansion`（tag 98）
//!    - `RecordActiveTarget`（tag 99）
//!    - `ClearPhaseGiftTarget`（tag 100）
//!    - `DealDamageBanActiveIfDying`（tag 101）
//!    - `DiscardSelectedDrawAllHandBonus`（tag 102）
//!    - `DiscardSelectedRecordUniqueCategoryTarget`（tag 103）
//!    - `DiscardTargetCardGainMount`（tag 104）
//!    - `DrawForLifetimeCardActionDivisors`（tag 105）
//!    - `RecoverSelectedTargets`（tag 106）
//!    - `RevealTopGainIfColor`（tag 107）
//!    - `UseVirtualCardOnSelectedTargets`（tag 108）
//! 4. `rvm_named` 激活 `target_hand_nonempty`、`target_hand_empty`、`hand_or_equip_empty`、
//!    `card_equip`、`target_wounded`、`draw_phase_skipped`、`min_hp`、`hp_le_one` 谓词短语；
//!    `content-native-loader` 与 `rvm_bridge` 接入 `recover_before`、`judgment_resolved`、
//!    `phase_draw_skipped` 三类触发时机。

use content_compiler::compile_source_named;
use engine_native::catalog::{CardKind, QueryCtx, Suit};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{
    CardUseData, DamageData, DamageNature, DrawOutcome, DrawPhaseProgress, EventData, Phase,
};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects};
use rule_api::{CardId, EffectRequest, PlayerId, SkillDefId, SkillId, TargetCardScope};

const CHONGXIN: &str = include_str!("../../../content/original/ported/yingbian/chongxin.skill");
const REZHIHENG: &str = include_str!("../../../content/original/ported/refresh/rezhiheng.skill");
const OLWEIYANG: &str = include_str!("../../../content/original/ported/collab/olweiyang.skill");
const QUANJI: &str = include_str!("../../../content/original/ported/yijiang/quanji.skill");
const DCXINGWU: &str = include_str!("../../../content/original/ported/xianding/dcxingwu.skill");
const SHIZHAN: &str = include_str!("../../../content/original/ported/refresh/shizhan.skill");
const JYLIANHUAN: &str = include_str!("../../../content/original/ported/offline/jylianhuan.skill");
const DCWANGLIE: &str = include_str!("../../../content/original/ported/shenhua/dcwanglie.skill");
const SONGCI: &str = include_str!("../../../content/original/ported/sp/songci.skill");
const ZIYUAN: &str = include_str!("../../../content/original/ported/sp/ziyuan.skill");
const ZUICI: &str = include_str!("../../../content/original/ported/shiji/zuici.skill");
const KENGO_GUIDUI: &str =
    include_str!("../../../content/original/ported/key/kengo_guidui.skill");
const PXSIZHOU: &str = include_str!("../../../content/original/ported/sp/pxsizhou.skill");
const JSRGJIAOHAO: &str = include_str!("../../../content/original/ported/jsrg/jsrgjiaohao.skill");
const REFENXUN: &str = include_str!("../../../content/original/ported/sp/refenxun.skill");
const OL_WUQIAN: &str = include_str!("../../../content/original/ported/extra/ol_wuqian.skill");
const EU_XIAOGUO: &str = include_str!("../../../content/original/ported/offline/eu_xiaoguo.skill");
const QIAOMENG: &str = include_str!("../../../content/original/ported/refresh/qiaomeng.skill");
const SCLS_QINZHENG: &str =
    include_str!("../../../content/original/ported/offline/scls_qinzheng.skill");
const PSZHONGHUN: &str = include_str!("../../../content/original/ported/offline/pszhonghun.skill");
const JIEZI: &str = include_str!("../../../content/original/ported/shenhua/jiezi.skill");
const TIANDU: &str = include_str!("../../../content/standard/tiandu.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let mut pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑫不引入新 ABI：{} > {}",
            pkg.manifest.rule_vm_abi_version,
            rule_vm::RULE_VM_ABI_VERSION
        );
        for s in &mut pkg.skill_index {
            s.lord_only = false;
            s.non_gainable = false;
        }
        for a in &mut pkg.active_index {
            a.selection = None;
            a.targets = None;
            if a.uses_per_turn.is_some_and(|n| n > 1) {
                a.uses_per_turn = Some(1);
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
        let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
        vm_skills.extend(out.decls);
    }
    ExtensionPack {
        def: ExtensionDef {
            name: pack_name,
            version: 1,
            description: "PA12 批⑫ 预选通道、25 项确定性效果原语与触发时机测试包",
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
        !matches!(out, CommandOutcome::Rejected(_)),
        "submit({actor}, {action:?}) 失败: {out:?}"
    );
}

fn drain_confirm(e: &mut Engine) {
    for _ in 0..80 {
        let Some(ask) = e.open_choice().cloned() else {
            return;
        };
        if ask.spec.options.contains(&ChoiceOption::Confirm) {
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
        } else if ask.spec.options.contains(&ChoiceOption::Pass) {
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else if ask.spec.allow_cancel {
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection: vec![],
                },
            );
        } else {
            let n = ask.spec.min.max(1) as usize;
            let selection: Vec<ChoiceOption> = ask.spec.options.iter().take(n).cloned().collect();
            submit(
                e,
                ask.observer,
                GameAction::Answer {
                    choice_id: ask.id,
                    selection,
                },
            );
        }
    }
}

fn submit_to_phase(e: &mut Engine, seat: u8, want: Phase) {
    for _ in 0..80 {
        drain_confirm(e);
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

#[test]
fn pa12_batch12_codec_roundtrip_84_to_108() {
    let effects = vec![
        EffectRequest::DiscardCards {
            player: PlayerId::new(1),
            cards: vec![CardId::new(10), CardId::new(11)],
        },
        EffectRequest::StoreSelectedInSkillPile {
            player: PlayerId::new(1),
            cards: vec![CardId::new(12)],
            skill: SkillId::new(5),
        },
        EffectRequest::UseVirtualCardOn {
            player: PlayerId::new(1),
            kind: 20,
            target: PlayerId::new(2),
        },
        EffectRequest::RecastCards {
            player: PlayerId::new(1),
            cards: vec![CardId::new(13)],
        },
        EffectRequest::AddCardUsePolicySkill {
            player: PlayerId::new(1),
            card: CardId::new(14),
            policy_mask: 1,
            skill_def: SkillDefId::new(2388106243),
        },
        EffectRequest::DrawSelectedTargets {
            targets: vec![PlayerId::new(2), PlayerId::new(3)],
            count: 2,
            skill: SkillId::new(6),
        },
        EffectRequest::QueueExtraTurn {
            player: PlayerId::new(1),
            check: 0,
            skill: SkillId::new(7),
        },
        EffectRequest::DisableEquipSlots {
            player: PlayerId::new(1),
            slots: vec![1, 2, 3, 4, 5],
        },
        EffectRequest::EnableEquipSlot {
            player: PlayerId::new(2),
            slot: 1,
            source: PlayerId::new(1),
        },
        EffectRequest::GainShadowCards {
            player: PlayerId::new(1),
            count: 2,
            skill: SkillId::new(8),
        },
        EffectRequest::GainTargetHandCard {
            player: PlayerId::new(1),
            skill: SkillId::new(9),
        },
        EffectRequest::AddBoundSkill {
            player: PlayerId::new(1),
            skill_def: SkillDefId::new(3372440036),
            bound_target: PlayerId::new(2),
        },
        EffectRequest::GiveSelectedThenRecoverTarget {
            player: PlayerId::new(1),
            target: PlayerId::new(2),
            cards: vec![CardId::new(15)],
            amount: 1,
            skill: SkillId::new(10),
            ready_to_recover: true,
        },
        EffectRequest::DistributeShadowCards {
            player: PlayerId::new(1),
            count: 3,
            skill: SkillId::new(11),
        },
        EffectRequest::PlaceHandCardToExpansion {
            player: PlayerId::new(1),
            skill: SkillId::new(12),
        },
        EffectRequest::RecordActiveTarget {
            player: PlayerId::new(1),
            target: PlayerId::new(2),
            skill: SkillId::new(13),
        },
        EffectRequest::ClearPhaseGiftTarget {
            player: PlayerId::new(1),
            skill: SkillId::new(14),
        },
        EffectRequest::DealDamageBanActiveIfDying {
            source: PlayerId::new(1),
            target: PlayerId::new(2),
            amount: 1,
            skill: SkillId::new(15),
        },
        EffectRequest::DiscardSelectedDrawAllHandBonus {
            player: PlayerId::new(1),
            cards: vec![CardId::new(16), CardId::new(17)],
            skill: SkillId::new(16),
        },
        EffectRequest::DiscardSelectedRecordUniqueCategoryTarget {
            player: PlayerId::new(1),
            target: PlayerId::new(2),
            card: CardId::new(18),
            skill: SkillId::new(17),
        },
        EffectRequest::DiscardTargetCardGainMount {
            player: PlayerId::new(1),
            target: PlayerId::new(2),
            scope: TargetCardScope::Equip,
        },
        EffectRequest::DrawForLifetimeCardActionDivisors {
            player: PlayerId::new(1),
            divisors: [3, 5, 8],
        },
        EffectRequest::RecoverSelectedTargets {
            source: PlayerId::new(1),
            targets: vec![PlayerId::new(1), PlayerId::new(2)],
            amount: 1,
            skill: SkillId::new(18),
        },
        EffectRequest::RevealTopGainIfColor {
            player: PlayerId::new(1),
            color_mask: 1,
        },
        EffectRequest::UseVirtualCardOnSelectedTargets {
            player: PlayerId::new(1),
            kind: 34,
            targets: vec![PlayerId::new(2), PlayerId::new(3)],
        },
    ];
    assert_eq!(effects.len(), 25);
    for req in &effects {
        accept_effect(req, 4).expect("批⑫ 25 项新变体均应通过 accept_effect");
    }
    let mut buf = Vec::new();
    enc_effects(&mut buf, &effects);
    let decoded = dec_effects(&buf).expect("dec_effects 应成功解码 tags 84..=108");
    assert_eq!(decoded, effects, "tags 84..=108 编解码必须 100% 往返一致");

    // 边界与非法入参拒绝校验
    assert!(accept_effect(
        &EffectRequest::DiscardCards {
            player: PlayerId::new(1),
            cards: vec![CardId::NONE],
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::RecastCards {
            player: PlayerId::new(1),
            cards: vec![CardId::NONE],
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::DrawSelectedTargets {
            targets: vec![PlayerId::new(2)],
            count: 0,
            skill: SkillId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::DisableEquipSlots {
            player: PlayerId::new(1),
            slots: Vec::new(),
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::EnableEquipSlot {
            player: PlayerId::new(1),
            slot: 9,
            source: PlayerId::NONE,
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::RevealTopGainIfColor {
            player: PlayerId::new(1),
            color_mask: 0,
        },
        4
    )
    .is_err());
}

#[test]
fn pa12_batch12_discard_selected_recast_and_zhiheng_all_hand_bonus() {
    let pack = pack_for(
        &[
            ("chongxin", CHONGXIN),
            ("rezhiheng", REZHIHENG),
            ("olweiyang", OLWEIYANG),
        ],
        "pa12-b12-discard-recast-zhiheng",
    );
    let mut e = Engine::new_with_extensions(12001, 3, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);
    let chongxin_def = e.catalog().skill_by_name("chongxin").expect("chongxin");
    let rezhiheng_def = e.catalog().skill_by_name("rezhiheng").expect("rezhiheng");
    let olweiyang_def = e.catalog().skill_by_name("olweiyang").expect("olweiyang");

    // 1. chongxin (RecastCards): 预选 1 张手牌重铸 -> 弃置该牌并摸 1 张（手牌总数不变，原牌入弃牌堆）
    e.debug_grant(0, "chongxin");
    let hand_before = e.state_core().player(0).hand.clone();
    let recast_card = hand_before[0];
    e.state_core_mut()
        .vm_aux
        .set_selected_cards(0, &[recast_card]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: chongxin_def });
    drain_confirm(&mut e);
    assert!(e.state_core().discard.contains(&recast_card));
    assert_eq!(e.state_core().player(0).hand.len(), hand_before.len());

    // 2. rezhiheng (DiscardSelectedDrawAllHandBonus):
    // 2a) 部分手牌弃置（只选 1 张）：摸 1 张（无 +1 奖励）
    e.debug_grant(0, "rezhiheng");
    let hand_partial = e.state_core().player(0).hand.clone();
    assert!(hand_partial.len() >= 2);
    e.state_core_mut()
        .vm_aux
        .set_selected_cards(0, &[hand_partial[0]]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: rezhiheng_def });
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), hand_partial.len());

    // 2b) 全部手牌弃置：清除本回合 once 后再发动，弃全部 N 张手牌 -> 摸 N + 1 张！
    e.state_core_mut().vm_aux.once.clear();
    let all_hand = e.state_core().player(0).hand.clone();
    let n = all_hand.len();
    e.state_core_mut().vm_aux.set_selected_cards(0, &all_hand);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: rezhiheng_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        n + 1,
        "rezhiheng 弃置全部手牌应额外多摸 1 张"
    );

    // 3. olweiyang (DiscardCards + RecoverSelectedTargets):
    // 弃置 2 张预选牌，令已受伤的座位 1 与座位 2 各回复 1 点体力
    e.debug_grant(0, "olweiyang");
    e.state_core_mut().player_mut(1).hp = 2;
    e.state_core_mut().player_mut(2).hp = 1;
    let pick2: Vec<u32> = e.state_core().player(0).hand.iter().take(2).copied().collect();
    let pre_len = e.state_core().player(0).hand.len();
    e.state_core_mut().vm_aux.set_selected_cards(0, &pick2);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1, 2]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: olweiyang_def });
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), pre_len - 2);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(e.state_core().player(2).hp, 2);
}

#[test]
fn pa12_batch12_skill_pile_and_expansion_effects() {
    let pack = pack_for(
        &[("quanji", QUANJI), ("dcxingwu", DCXINGWU)],
        "pa12-b12-piles",
    );
    let mut e = Engine::new_with_extensions(12002, 2, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);

    // 1. quanji (DrawCards(1) + StoreSelectedInSkillPile): 受到伤害后摸 1 张并将 1 张手牌置入「权」牌堆
    e.debug_grant(0, "quanji");
    let hand_before = e.state_core().player(0).hand.len();
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain_confirm(&mut e);
    let pile = e
        .state_core()
        .vm_aux
        .piles
        .get(&(0, "quanji".to_string()))
        .cloned()
        .unwrap_or_default();
    assert_eq!(pile.len(), 1, "quanji 应将 1 张手牌置入技能牌堆");
    assert_eq!(e.state_core().player(0).hand.len(), hand_before);

    // 2. dcxingwu (PlaceHandCardToExpansion): 弃牌阶段开始时将首张手牌置入「星舞」牌堆
    e.debug_grant(0, "dcxingwu");
    let first_hand = e.state_core().player(0).hand[0];
    submit_to_phase(&mut e, 0, Phase::End);
    drain_confirm(&mut e);
    let xingwu_pile = e
        .state_core()
        .vm_aux
        .piles
        .get(&(0, "dcxingwu".to_string()))
        .cloned()
        .unwrap_or_default();
    assert_eq!(xingwu_pile, vec![first_hand]);
}

#[test]
fn pa12_batch12_virtual_card_and_policy_effects() {
    let wn_dutan = include_str!("../../../content/original/ported/offline/wn_dutan.skill");
    let aux_virtual_guohe = r#"
skill aux_virtual_guohe(9912009) {
  active play_phase targets 1 1 other {
    effect use_virtual_card_on(event_source, guohe, self);
  }
}
"#;
    let pack = pack_for(
        &[
            ("shizhan", SHIZHAN),
            ("jylianhuan", JYLIANHUAN),
            ("dcwanglie", DCWANGLIE),
            ("wn_dutan", wn_dutan),
            ("aux_virtual_guohe", aux_virtual_guohe),
        ],
        "pa12-b12-virtual-and-policy",
    );
    let mut e = Engine::new_with_extensions(12003, 3, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);
    let shizhan_def = e.catalog().skill_by_name("shizhan").expect("shizhan");
    let jylianhuan_def = e.catalog().skill_by_name("jylianhuan").expect("jylianhuan");
    let wn_dutan_def = e.catalog().skill_by_name("wn_dutan").expect("wn_dutan");
    let guohe_def = e
        .catalog()
        .skill_by_name("aux_virtual_guohe")
        .expect("aux_virtual_guohe");

    // 1. shizhan (UseVirtualCardOn): 目标（座位 1）对拥有者（座位 0）使用虚拟【决斗】
    // 清空座位 0 手牌令决斗直接对座位 0 造成 1 点伤害
    e.debug_grant(0, "shizhan");
    e.debug_clear_hand(0);
    let hp0_before = e.state_core().player(0).hp;
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: shizhan_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        hp0_before - 1,
        "shizhan 虚拟【决斗】应结算并令无杀的座位 0 受到 1 点伤害"
    );

    // 1b. UseVirtualCardOn(guohe): 目标（座位 1）对拥有者（座位 0）使用虚拟【过河拆桥】
    // 应弃置座位 0 的手牌，且不得将虚拟哨兵卡 ID 0 推入弃牌堆
    e.debug_grant(0, "aux_virtual_guohe");
    let victim_card = e.debug_place(0, "shan");
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: guohe_def });
    drain_confirm(&mut e);
    assert!(
        !e.state_core().player(0).hand.contains(&victim_card)
            && e.state_core().discard.contains(&victim_card),
        "虚拟【过河拆桥】应拆掉目标手牌入弃牌堆"
    );
    assert!(
        !e.state_core().discard.contains(&0),
        "虚拟【过河拆桥】不得将哨兵卡 ID 0 推入弃牌堆"
    );

    // 2. jylianhuan (LoseHp + UseVirtualCardOnSelectedTargets): 对预选目标使用虚拟【铁索连环】
    e.debug_grant(0, "jylianhuan");
    assert!(!e.state_core().player(1).linked);
    assert!(!e.state_core().player(2).linked);
    e.state_core_mut()
        .vm_aux
        .set_selected_target_seats(0, &[1, 2]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: jylianhuan_def });
    drain_confirm(&mut e);
    assert!(e.state_core().player(1).linked, "jylianhuan 应横置目标 1");
    assert!(e.state_core().player(2).linked, "jylianhuan 应横置目标 2");
    e.state_core_mut().player_mut(1).linked = false;
    e.state_core_mut().player_mut(2).linked = false;

    // 2b. wn_dutan (UseVirtualCardOnSelectedTargets 多目标虚拟【决斗】):
    // 对预选的 2 个目标 [1, 2] 依次结算虚拟【决斗】，无杀的座位 1 与座位 2 均受到 1 点决斗伤害
    e.debug_grant(0, "wn_dutan");
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let hp1_before_dutan = e.state_core().player(1).hp;
    let hp2_before_dutan = e.state_core().player(2).hp;
    e.state_core_mut()
        .vm_aux
        .set_selected_target_seats(0, &[1, 2]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: wn_dutan_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        hp1_before_dutan - 1,
        "wn_dutan 多目标虚拟【决斗】应对第 1 个目标（座位 1）造成 1 点伤害"
    );
    assert_eq!(
        e.state_core().player(2).hp,
        hp2_before_dutan - 1,
        "wn_dutan 多目标虚拟【决斗】应对第 2 个目标（座位 2）造成 1 点伤害"
    );

    // 3. dcwanglie (AddCardUsePolicySkill): 出牌阶段使用杀时令当前牌不可响应
    e.debug_grant(0, "dcwanglie");
    let sha = e.debug_place(0, "sha");
    let _shan = e.debug_place(1, "shan");
    let hp1_before = e.state_core().player(1).hp;
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, format!("card_policy:{sha}")))
            .copied(),
        Some(2)
    );
    assert_eq!(
        e.state_core().player(1).hp,
        hp1_before - 1,
        "dcwanglie 不可响应的【杀】应直接命中持闪的座位 1"
    );
}

#[test]
fn pa12_batch12_selected_targets_draw_give_recover_and_extra_turn() {
    let aux_extra_turn = r#"
skill aux_queue_extra_turn(9912001) {
  active play_phase {
    effect queue_extra_turn(self, 0, self_skill);
  }
}
"#;
    let pack = pack_for(
        &[
            ("songci", SONGCI),
            ("ziyuan", ZIYUAN),
            ("aux_extra_turn", aux_extra_turn),
        ],
        "pa12-b12-draw-give-recover",
    );
    let mut e = Engine::new_with_extensions(12004, 2, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);
    let songci_def = e.catalog().skill_by_name("songci").expect("songci");
    let ziyuan_def = e.catalog().skill_by_name("ziyuan").expect("ziyuan");
    let extra_turn_def = e
        .catalog()
        .skill_by_name("aux_queue_extra_turn")
        .expect("aux_queue_extra_turn");

    // 1. songci (DrawSelectedTargets): 令预选目标（座位 1）摸 2 张牌
    e.debug_grant(0, "songci");
    let hand1_before = e.state_core().player(1).hand.len();
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: songci_def });
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(1).hand.len(), hand1_before + 2);

    // 2. ziyuan (GiveSelectedThenRecoverTarget): 将预选牌交给座位 1 并令其回复 1 点体力
    e.debug_grant(0, "ziyuan");
    e.state_core_mut().player_mut(1).hp = 1;
    let gift_card = e.state_core().player(0).hand[0];
    e.state_core_mut().vm_aux.set_selected_cards(0, &[gift_card]);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: ziyuan_def });
    drain_confirm(&mut e);
    assert!(e.state_core().player(1).hand.contains(&gift_card));
    assert_eq!(e.state_core().player(1).hp, 2);

    // 3. QueueExtraTurn: 发动 aux_queue_extra_turn 记录额外回合标记
    e.debug_grant(0, "aux_queue_extra_turn");
    submit(
        &mut e,
        0,
        GameAction::ActivateSkill {
            skill: extra_turn_def,
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "extra_turn_queued".to_string()))
            .copied(),
        Some(1)
    );
}

#[test]
fn pa12_batch12_equip_slots_shadow_cards_and_gain_target_hand() {
    let aux_shadow_and_grant = r#"
skill aux_dist_shadow_and_grant_sizhou(9912002) {
  active play_phase {
    effect distribute_shadow_cards(self, 2, self_skill);
    effect add_skill(self, 3079253303);
  }
}
"#;
    let pack = pack_for(
        &[
            ("zuici", ZUICI),
            ("kengo_guidui", KENGO_GUIDUI),
            ("jsrgjiaohao", JSRGJIAOHAO),
            ("pxsizhou", PXSIZHOU),
            ("aux_shadow_and_grant", aux_shadow_and_grant),
        ],
        "pa12-b12-equip-shadow-gain",
    );
    let mut e = Engine::new_with_extensions(12005, 2, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);
    let aux_def = e
        .catalog()
        .skill_by_name("aux_dist_shadow_and_grant_sizhou")
        .expect("aux_dist_shadow_and_grant_sizhou");

    // 1. zuici (DisableEquipSlots): 座位 0 濒死时触发涍辞废除装备栏、弃置装备并回复体力至 2
    e.debug_grant(0, "zuici");
    let weapon = e.debug_place(0, "zhuge");
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != weapon);
    e.state_core_mut().player_mut(0).equips.push(weapon);
    e.state_core_mut().player_mut(0).hp = 1;
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain_confirm(&mut e);
    assert!(
        e.state_core().player(0).equips.is_empty(),
        "zuici 废除装备栏时应弃置已装备的武器"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "disabled_equip_slots".to_string()))
            .copied(),
        Some(0b11_1110)
    );

    // 2. DistributeShadowCards + pxsizhou (GainTargetHandCard via skill_added):
    e.debug_grant(0, "aux_dist_shadow_and_grant_sizhou");
    let target_card = e.state_core().player(1).hand[0];
    submit(&mut e, 0, GameAction::ActivateSkill { skill: aux_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "shadow_cards_distributed".to_string()))
            .copied(),
        Some(2)
    );
    assert!(
        e.state_core().player(0).hand.contains(&target_card),
        "pxsizhou 在获得技能时应从对手处获得 1 张手牌"
    );

    // 3. kengo_guidui (EnableEquipSlot) + jsrgjiaohao (GainShadowCards):
    // 推进到下一轮座位 0 准备阶段 -> 触发 kengo_guidui 恢复全部 5 个装备栏，并触发 jsrgjiaohao 获得【影】
    e.debug_grant(0, "kengo_guidui");
    e.debug_grant(0, "jsrgjiaohao");
    submit_to_phase(&mut e, 1, Phase::Play);
    submit_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "disabled_equip_slots".to_string()))
            .copied(),
        None,
        "kengo_guidui 准备阶段恢复全部 5 个装备栏后 disabled_equip_slots 应清除"
    );
    assert!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "shadow_cards_gained".to_string()))
            .copied()
            .unwrap_or(0)
            > 0,
        "jsrgjiaohao 准备阶段应获得影牌并记录 shadow_cards_gained"
    );
}

#[test]
fn pa12_batch12_bound_skill_active_target_gift_target_and_dying_ban() {
    let aux_clear_gift = r#"
skill aux_clear_gift_and_unique_cat(9912003) {
  active play_phase targets 1 1 other unused_by_skill_this_turn select_he 1 1 unused_category_by_skill_this_turn {
    effect clear_phase_gift_target(self);
    effect discard_selected_record_unique_category_target(self, event_source);
  }
}
"#;
    let pack = pack_for(
        &[
            ("refenxun", REFENXUN),
            ("ol_wuqian", OL_WUQIAN),
            ("eu_xiaoguo", EU_XIAOGUO),
            ("aux_clear_gift", aux_clear_gift),
        ],
        "pa12-b12-bound-active-dying",
    );
    let mut e = Engine::new_with_extensions(12006, 2, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);
    let refenxun_def = e.catalog().skill_by_name("refenxun").expect("refenxun");
    let wuqian_def = e.catalog().skill_by_name("ol_wuqian").expect("ol_wuqian");
    let xiaoguo_def = e.catalog().skill_by_name("eu_xiaoguo").expect("eu_xiaoguo");
    let aux_clear_def = e
        .catalog()
        .skill_by_name("aux_clear_gift_and_unique_cat")
        .expect("aux_clear_gift_and_unique_cat");

    // 1. refenxun (AddBoundSkill): 记录绑定目标
    e.debug_grant(0, "refenxun");
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: refenxun_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "bound_target".to_string()))
            .copied(),
        Some(2)
    );

    // 2. ol_wuqian (RecordActiveTarget) + ClearPhaseGiftTarget + DiscardSelectedRecordUniqueCategoryTarget:
    e.debug_grant(0, "ol_wuqian");
    e.state_core_mut().vm_aux.marks.insert((0, 1252226867), 2);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: wuqian_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "active_target:ol_wuqian".to_string()))
            .copied(),
        Some(2)
    );
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "phase_gift_target".to_string()), 2);
    e.debug_grant(0, "aux_clear_gift_and_unique_cat");
    let pindi_card = e.state_core().player(0).hand[0];
    e.state_core_mut().vm_aux.set_selected_cards(0, &[pindi_card]);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(
        &mut e,
        0,
        GameAction::ActivateSkill {
            skill: aux_clear_def,
        },
    );
    drain_confirm(&mut e);
    assert!(!e
        .state_core()
        .vm_aux
        .flags
        .contains_key(&(0, "phase_gift_target".to_string())));
    assert!(e.state_core().discard.contains(&pindi_card));
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "unique_cat_target:aux_clear_gift_and_unique_cat".to_string()))
            .copied(),
        Some(2)
    );

    // 3. eu_xiaoguo (DealDamageBanActiveIfDying):
    // 3a) 目标体力为 2 时造成 1 点伤害不濒死 -> 不封禁技能
    e.debug_grant(0, "eu_xiaoguo");
    e.state_core_mut().player_mut(1).hp = 2;
    let c1 = e.state_core().player(0).hand[0];
    e.state_core_mut().vm_aux.set_selected_cards(0, &[c1]);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: xiaoguo_def });
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(1).hp, 1);
    assert!(!e
        .state_core()
        .vm_aux
        .flags
        .contains_key(&(0, "skill_banned_turn:eu_xiaoguo".to_string())));

    // 3b) 目标体力为 1 时造成 1 点伤害进入濒死 -> 封禁本回合 eu_xiaoguo！
    e.state_core_mut().vm_aux.once.clear();
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let c2 = e.debug_place(0, "sha");
    e.state_core_mut().vm_aux.set_selected_cards(0, &[c2]);
    e.state_core_mut().vm_aux.set_selected_target_seats(0, &[1]);
    submit(&mut e, 0, GameAction::ActivateSkill { skill: xiaoguo_def });
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "skill_banned_turn:eu_xiaoguo".to_string()))
            .copied(),
        Some(1),
        "令目标进入濒死后应封禁本回合主动技"
    );
    e.state_core_mut().vm_aux.once.clear();
    let rev = e.revision();
    let rejected = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: 991200699,
        action: GameAction::ActivateSkill { skill: xiaoguo_def },
    });
    assert!(
        matches!(rejected, CommandOutcome::Rejected(_)),
        "DealDamageBanActiveIfDying 置位 skill_banned_turn 后本回合再次发动主动技应被拒绝"
    );
}

#[test]
fn pa12_batch12_mount_divisors_and_reveal_top_color() {
    let pack = pack_for(
        &[
            ("qiaomeng", QIAOMENG),
            ("scls_qinzheng", SCLS_QINZHENG),
            ("pszhonghun", PSZHONGHUN),
        ],
        "pa12-b12-mount-divisors-color",
    );
    let mut e = Engine::new_with_extensions(12007, 2, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);

    // 1. qiaomeng (DiscardTargetCardGainMount):
    // 1a) 目标装备坐骑（+1 马 / -1 马）时，黑色【杀】造成伤害后将其收入自己手牌
    e.debug_grant(0, "qiaomeng");
    let mount_card = (1..e.catalog().cards.len() as u32)
        .find(|&c| {
            matches!(
                e.catalog().card_slot_of(e.state_core(), c),
                Some(
                    engine_native::catalog::EquipSlot::HorseMinus
                        | engine_native::catalog::EquipSlot::HorsePlus
                )
            )
        })
        .expect("坐骑牌");
    e.state_core_mut().deck.retain(|&c| c != mount_card);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != mount_card);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != mount_card);
    e.state_core_mut().player_mut(1).equips.push(mount_card);
    let black_sha = (1..e.catalog().cards.len() as u32)
        .find(|&c| {
            e.catalog().card_kind(c) == CardKind::Sha
                && matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("黑色杀");
    let _ = e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, black_sha);
    drain_confirm(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&mount_card),
        "qiaomeng 弃置目标坐骑牌时应将其收入自己手牌"
    );

    // 1b) 目标装备非坐骑（武器）时，仅弃入弃牌堆，不收入手牌
    let weapon = e.debug_place(1, "zhuge");
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != weapon);
    e.state_core_mut().player_mut(1).equips.push(weapon);
    let _ = e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, black_sha);
    drain_confirm(&mut e);
    assert!(e.state_core().discard.contains(&weapon));
    assert!(!e.state_core().player(0).hand.contains(&weapon));

    // 2. scls_qinzheng (DrawForLifetimeCardActionDivisors [3, 5, 8]):
    // 累计次数 15 同时被 3 和 5 整除 -> 摸 2 张牌！
    e.debug_grant(0, "scls_qinzheng");
    e.state_core_mut().player_mut(0).hp = 1;
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "lifetime_card_actions".to_string()), 15);
    let tao = e.debug_place(0, "tao");
    let hand_before_qz = e.state_core().player(0).hand.len();
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_qz - 1 + 2,
        "累计 15 次用牌同时被 3 和 5 整除，scls_qinzheng 应摸 2 张牌"
    );

    // 3. pszhonghun (RevealTopGainIfColor):
    // 牌堆顶为红牌时收入手牌；牌堆顶为黑牌时置入弃牌堆
    e.debug_grant(0, "pszhonghun");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "lifetime_card_actions".to_string()), 1);
    e.state_core_mut().player_mut(0).hp = 1;
    let red_tao = (1..e.catalog().cards.len() as u32)
        .find(|&c| {
            e.catalog().card_kind(c) == CardKind::Tao
                && matches!(e.catalog().cards[c as usize].suit, Suit::Heart | Suit::Diamond)
        })
        .expect("红桃牌");
    e.state_core_mut().deck.retain(|&c| c != red_tao);
    e.state_core_mut().discard.retain(|&c| c != red_tao);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != red_tao);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != red_tao);
    e.state_core_mut().player_mut(0).hand.push(red_tao);

    let red_top = (1..e.catalog().cards.len() as u32)
        .find(|&c| {
            c != red_tao
                && e.state_core().deck.contains(&c)
                && matches!(e.catalog().cards[c as usize].suit, Suit::Heart | Suit::Diamond)
        })
        .expect("牌堆红牌");
    e.state_core_mut().deck.retain(|&c| c != red_top);
    e.state_core_mut().deck.push(red_top);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_tao,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&red_top),
        "pszhonghun 展示牌堆顶为红牌时应获得之"
    );

    e.state_core_mut().player_mut(0).hp = 1;
    e.state_core_mut().discard.retain(|&c| c != red_tao);
    e.state_core_mut().player_mut(0).hand.push(red_tao);
    let black_top = (1..e.catalog().cards.len() as u32)
        .find(|&c| {
            e.state_core().deck.contains(&c)
                && matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("牌堆黑牌");
    e.state_core_mut().deck.retain(|&c| c != black_top);
    e.state_core_mut().deck.push(black_top);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_tao,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert!(
        e.state_core().discard.contains(&black_top)
            && !e.state_core().player(0).hand.contains(&black_top),
        "pszhonghun 展示牌堆顶为黑牌时应置入弃牌堆"
    );
}

#[test]
fn pa12_batch12_trigger_timings_and_named_predicates() {
    let recover_before_src = r#"
skill test_recover_before(990012001) locked {
  trigger recover_before owner_is_target {
    effect draw_cards(self, 1);
  }
}
"#;
    let pack = pack_for(
        &[
            ("jiezi", JIEZI),
            ("tiandu", TIANDU),
            ("test_recover_before", recover_before_src),
        ],
        "pa12-b12-timings-and-predicates",
    );
    let mut e = Engine::new_with_extensions(12008, 2, &[&pack]).expect("加载扩展包");
    submit_to_phase(&mut e, 0, Phase::Play);

    // 1. recover_before:
    e.debug_grant(0, "test_recover_before");
    e.state_core_mut().player_mut(0).hp = 2;
    let hand0_before = e.state_core().player(0).hand.len();
    let _ = e.debug_recover(0, 1);
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), hand0_before + 1);

    // 2. phase_draw_skipped (jiezi.skill) & judgment_resolved (tiandu.skill):
    // 座位 0 持有 jiezi，座位 1 持有 tiandu；
    // 2a) 座位 1 无兵粮寸断正常经历摸牌阶段 -> 座位 0 的 jiezi 不触发！
    e.debug_grant(0, "jiezi");
    e.debug_grant(1, "tiandu");
    e.debug_clear_hand(0);
    submit_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        0,
        "座位 1 正常完成摸牌阶段时不得触发座位 0 的 jiezi (phase_draw_skipped)"
    );

    // 2b) 给座位 1 挂上【兵粮寸断】（bing），并将牌堆顶设为红桃牌（非梅花，兵粮寸断生效跳过摸牌阶段）：
    //     推进到座位 1 的回合：
    //     - 判定阶段结算兵粮寸断后触发座位 1 的 tiandu (judgment_resolved) 获得判定牌；
    //     - 跳过座位 1 的摸牌阶段后触发座位 0 的 jiezi (phase_draw_skipped) 摸 1 张牌！
    submit_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    let _bing = e.debug_place_judge(1, "bing");
    let heart_judge = e.debug_set_deck_top_by(|c| c.suit == Suit::Heart);
    submit_to_phase(&mut e, 1, Phase::Play);
    drain_confirm(&mut e);
    assert!(
        e.state_core().player(1).hand.contains(&heart_judge),
        "tiandu (judgment_resolved) 应在兵粮寸断判定生效后获得判定牌"
    );
    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "座位 1 因兵粮寸断跳过摸牌阶段后，座位 0 的 jiezi (phase_draw_skipped) 应触发并摸 1 张牌"
    );

    // 3. NamedPredicate 短语真假双态校验：
    // target_hand_nonempty / target_hand_empty / hand_or_equip_empty / card_equip /
    // target_wounded / draw_phase_skipped / min_hp / hp_le_one
    let f_nonempty = engine_native::rvm_bridge::map_vm_filter("owner_source_target_hand_nonempty")
        .expect("target_hand_nonempty");
    let f_empty = engine_native::rvm_bridge::map_vm_filter("owner_source_target_hand_empty")
        .expect("target_hand_empty");
    let f_he_empty = engine_native::rvm_bridge::map_vm_filter("owner_is_target_hand_or_equip_empty")
        .expect("hand_or_equip_empty");
    let f_equip =
        engine_native::rvm_bridge::map_vm_filter("owner_source_card_equip").expect("card_equip");
    let f_wounded = engine_native::rvm_bridge::map_vm_filter("owner_source_target_wounded")
        .expect("target_wounded");
    let f_skipped = engine_native::rvm_bridge::map_vm_filter("owner_alive_draw_phase_skipped")
        .expect("draw_phase_skipped");
    let f_min_hp =
        engine_native::rvm_bridge::map_vm_filter("owner_is_target_min_hp").expect("min_hp");
    let f_hp_le_one =
        engine_native::rvm_bridge::map_vm_filter("owner_is_target_hp_le_one").expect("hp_le_one");

    let eval = |e: &Engine, policy: &engine_native::rvm_bridge::VmFilterPolicy, ev: &EventData| {
        let engine_native::rvm_bridge::VmFilterPolicy::Named(prog) = policy else {
            panic!("expected Named program");
        };
        let mut view = engine_native_kernel::rvm_named::FilterView {
            event_player: Some(1),
            event_source: Some(0),
            event_card: None,
            event_amount: 1,
            phase: None,
            nature: None,
            targets: vec![1],
            use_as: None,
            from_chain: false,
            batch_len: 0,
            moved_from: None,
            begin: None,
            draw_phase_skipped: false,
            kind: 1,
        };
        match ev {
            EventData::CardUsed(u) => {
                view.event_player = Some(u.player);
                view.event_source = Some(u.player);
                view.event_card = u.card;
                view.kind = 2;
            }
            EventData::DrawPhase {
                player, outcome, ..
            } => {
                view.event_player = Some(*player);
                view.event_source = Some(*player);
                view.draw_phase_skipped = matches!(outcome, DrawOutcome::Skipped);
                view.kind = 3;
            }
            EventData::Phase { player, .. } => {
                view.event_player = Some(*player);
                view.event_source = Some(*player);
                view.kind = 3;
            }
            _ => {}
        }
        let qctx = QueryCtx {
            state: e.state_core(),
            catalog: e.catalog(),
        };
        engine_native_kernel::rvm_named::eval_program(prog, &qctx, 0, "test", &view)
    };

    let dummy_dmg = EventData::Damage(DamageData {
        source: Some(0),
        target: 1,
        card: None,
        amount: 1,
        nature: DamageNature::Normal,
        prevented: false,
        redirect_chain: Vec::new(),
        from_chain: false,
        settlement: Default::default(),
    });
    assert!(!e.state_core().player(1).hand.is_empty());
    assert!(eval(&e, &f_nonempty, &dummy_dmg));
    assert!(!eval(&e, &f_empty, &dummy_dmg));
    e.debug_clear_hand(1);
    assert!(!eval(&e, &f_nonempty, &dummy_dmg));
    assert!(eval(&e, &f_empty, &dummy_dmg));

    let phase0 = EventData::Phase {
        player: 0,
        phase: Phase::Play,
        begin: true,
    };
    assert!(!eval(&e, &f_he_empty, &phase0));
    e.debug_clear_hand(0);
    e.state_core_mut().player_mut(0).equips.clear();
    assert!(eval(&e, &f_he_empty, &phase0));

    let equip_card = e.debug_place(0, "zhuge");
    let sha_card = e.debug_place(0, "sha");
    let use_equip = EventData::CardUsed(CardUseData {
        player: 0,
        card: Some(equip_card),
        targets: vec![0],
        use_as: None,
        nature: DamageNature::Normal,
        cancelled: false,
    });
    let use_sha = EventData::CardUsed(CardUseData {
        player: 0,
        card: Some(sha_card),
        targets: vec![1],
        use_as: None,
        nature: DamageNature::Normal,
        cancelled: false,
    });
    assert!(eval(&e, &f_equip, &use_equip));
    assert!(!eval(&e, &f_equip, &use_sha));

    e.state_core_mut().player_mut(1).hp = e.state_core().player(1).max_hp as i32;
    assert!(!eval(&e, &f_wounded, &dummy_dmg));
    e.state_core_mut().player_mut(1).hp = 1;
    assert!(eval(&e, &f_wounded, &dummy_dmg));

    let draw_running = EventData::DrawPhase {
        player: 1,
        progress: DrawPhaseProgress::Before,
        pending: 2,
        num_fixed: false,
        actual_gain: 0,
        outcome: DrawOutcome::Running,
    };
    let draw_skipped = EventData::DrawPhase {
        player: 1,
        progress: DrawPhaseProgress::Before,
        pending: 0,
        num_fixed: false,
        actual_gain: 0,
        outcome: DrawOutcome::Skipped,
    };
    assert!(!eval(&e, &f_skipped, &draw_running));
    assert!(eval(&e, &f_skipped, &draw_skipped));

    e.state_core_mut().player_mut(0).hp = 3;
    e.state_core_mut().player_mut(1).hp = 2;
    assert!(!eval(&e, &f_min_hp, &phase0));
    assert!(!eval(&e, &f_hp_le_one, &phase0));
    e.state_core_mut().player_mut(0).hp = 1;
    assert!(eval(&e, &f_min_hp, &phase0));
    assert!(eval(&e, &f_hp_le_one, &phase0));
}
