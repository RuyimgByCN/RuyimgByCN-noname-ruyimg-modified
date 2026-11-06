//! PA12 迁移批⑩（2026-11-06，零版本号变更）：
//! 1. `rvm_ops` 拆分 `commit_combat.rs` 子模块并接入 13 个既有 ABI（<= 759）确定性效果原语（tags 58..=70）：
//!    - `DiscardAllHandAndEquip`（tag 58）：弃置目标角色全部手牌与装备牌；
//!    - `DiscardAllHandEquipJudge`（tag 59）：弃置目标角色手牌、装备区与判定区全部牌；
//!    - `RestoreCharacterState`（tag 60）：复原武将牌（解除横置并翻转背面朝上至正面）；
//!    - `RemoveAllSkills`（tag 61）：移除目标角色全部技能与 VM 临时技能旗标；
//!    - `DrawLinkedPlayers`（tag 62）：按当前回合角色起的逆时针座次令所有存活横置角色各摸 N 张牌；
//!    - `RecoverIfMinCurrentHp`（tag 63）：若目标角色体力值为全场存活最低（含并列），回复 N 点体力；
//!    - `EquipWeaponFromDrawPile`（tag 64）：若目标角色武器栏为空，从牌堆检索一张武器牌直接置入其装备区并授予武器技能；
//!    - `RemoveCardsByKind`（tag 65）：从牌堆与弃牌堆移出指定 `kind_tag` 的全部卡牌至 `"removed"` 区（`CardZone == 7`）；
//!    - `QueueOpeningExtraPlayPhase`（tag 66）：在回合开始（`TurnBegin`）入队先手额外出牌阶段（`Play` 后接 `Judge` 恢复正常阶段序列）；
//!    - `IncreaseCurrentShaBaseDamage`（tag 67）：令当前正在使用的【杀】伤害基数 +N；
//!    - `IncreaseCurrentCardBaseDamage`（tag 68）：令当前正在使用的伤害牌（【杀】、【决斗】、【南蛮入侵】、【万箭齐发】）伤害基数 +N；
//!    - `IncreaseCurrentShaTargetDamage`（tag 69）：令当前【杀】对指定目标造成的伤害 +N；
//!    - `MakeCurrentShaTargetUnrespondable`（tag 70）：令指定目标不可响应当前【杀】（直接跳过八卦阵判定与出【闪】询问）。
//! 2. `rvm_named` 拆分 `eval.rs` 子模块并补齐 `"owner_source_or_targeted_"` 域前缀、`"linked_exists"`（`Atom::AnyLinked`）、
//!    `"damage_card"`（`Atom::DamageCard`）、`"wounded_nanman_wanjian_or_normal_spade_sha"`、
//!    `"target_hand_gte_hp_or_lte_attack_range"`、`"target_hand_lte_owner_or_hp_gte_owner"`、
//!    `"target_alive_owner_outside_target_attack_range"`。

use content_compiler::compile_source_named;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use engine_native_kernel::rvm_ops::{accept_effect, dec_effects, enc_effects, run_query};
use rule_api::{CardId, EffectRequest, PlayerId, QueryRequest, QueryResponse, SkillId};

const HUILEI: &str = include_str!("../../../content/original/ported/yijiang/huilei.skill");
const QIBIE: &str = include_str!("../../../content/original/ported/sp2/qibie.skill");
const DUANCHANG: &str = include_str!("../../../content/original/ported/shenhua/duanchang.skill");
const OLDNIEPAN: &str = include_str!("../../../content/original/ported/shenhua/oldniepan.skill");
const JYSUOZHOU: &str = include_str!("../../../content/original/ported/offline/jysuozhou.skill");
const DANGXIAN: &str = include_str!("../../../content/original/ported/yijiang/dangxian.skill");
const REDANGXIAN: &str = include_str!("../../../content/original/ported/mobile/redangxian.skill");
const STDKENGQIANG: &str =
    include_str!("../../../content/original/ported/sixiang/stdkengqiang.skill");
const OLJUEJUE: &str = include_str!("../../../content/original/ported/collab/oljuejue.skill");
const HANYONG: &str = include_str!("../../../content/original/ported/sp/hanyong.skill");
const LIEGONG: &str = include_str!("../../../content/original/ported/shenhua/liegong.skill");
const XINANJIAN: &str = include_str!("../../../content/original/ported/mobile/xinanjian.skill");
const TWFUJIAN: &str = include_str!("../../../content/original/ported/tw/twfujian.skill");

fn pack_for(sources: &[(&str, &str)], pack_name: &'static str) -> ExtensionPack {
    let mut vm_skills = Vec::new();
    for (id, source) in sources {
        let pkg = compile_source_named(*id, source).expect("编译真实 ported .skill");
        assert!(
            pkg.manifest.rule_vm_abi_version <= rule_vm::RULE_VM_ABI_VERSION,
            "批⑩不引入新 ABI：{} > {}",
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
            description: "PA12 批⑩ 战斗与状态确定性效果原语",
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
fn pa12_batch10_codec_roundtrip_58_to_70() {
    let effects = vec![
        EffectRequest::DiscardAllHandAndEquip {
            player: PlayerId::new(1),
        },
        EffectRequest::DiscardAllHandEquipJudge {
            player: PlayerId::new(2),
        },
        EffectRequest::RestoreCharacterState {
            player: PlayerId::new(3),
        },
        EffectRequest::RemoveAllSkills {
            player: PlayerId::new(4),
        },
        EffectRequest::DrawLinkedPlayers { count: 2 },
        EffectRequest::RecoverIfMinCurrentHp {
            player: PlayerId::new(1),
            amount: 1,
        },
        EffectRequest::EquipWeaponFromDrawPile {
            player: PlayerId::new(2),
        },
        EffectRequest::RemoveCardsByKind { kind: 10 },
        EffectRequest::QueueOpeningExtraPlayPhase {
            player: PlayerId::new(1),
            skill: SkillId::new(7),
        },
        EffectRequest::IncreaseCurrentShaBaseDamage {
            player: PlayerId::new(1),
            card: CardId::new(11),
            amount: 1,
            skill: SkillId::new(8),
        },
        EffectRequest::IncreaseCurrentCardBaseDamage {
            player: PlayerId::new(2),
            card: CardId::new(12),
            amount: 2,
            skill: SkillId::new(9),
        },
        EffectRequest::IncreaseCurrentShaTargetDamage {
            source: PlayerId::new(1),
            card: CardId::new(13),
            target: PlayerId::new(3),
            amount: 1,
        },
        EffectRequest::MakeCurrentShaTargetUnrespondable {
            source: PlayerId::new(1),
            card: CardId::new(14),
            target: PlayerId::new(2),
        },
    ];
    for req in &effects {
        accept_effect(req, 4).expect("批⑩ 13 项新变体均应通过 accept_effect");
    }
    let mut buf = Vec::new();
    enc_effects(&mut buf, &effects);
    let decoded = dec_effects(&buf).expect("dec_effects 应成功解码 tags 58..=70");
    assert_eq!(decoded, effects, "tags 58..=70 编解码必须 100% 往返一致");

    assert!(accept_effect(&EffectRequest::DrawLinkedPlayers { count: 0 }, 4).is_err());
    assert!(accept_effect(
        &EffectRequest::RecoverIfMinCurrentHp {
            player: PlayerId::new(1),
            amount: 0,
        },
        4
    )
    .is_err());
    assert!(accept_effect(&EffectRequest::RemoveCardsByKind { kind: 0 }, 4).is_err());
    assert!(accept_effect(
        &EffectRequest::IncreaseCurrentShaBaseDamage {
            player: PlayerId::new(1),
            card: CardId::new(1),
            amount: 0,
            skill: SkillId::new(1),
        },
        4
    )
    .is_err());
    assert!(accept_effect(
        &EffectRequest::IncreaseCurrentShaTargetDamage {
            source: PlayerId::new(1),
            card: CardId::new(1),
            target: PlayerId::new(2),
            amount: 0,
        },
        4
    )
    .is_err());
}

#[test]
fn pa12_batch10_huilei_qibie_duanchang_death_effects() {
    let pack = pack_for(
        &[
            ("huilei", HUILEI),
            ("qibie", QIBIE),
            ("duanchang", DUANCHANG),
        ],
        "pa12-batch10-death",
    );
    let mut e = Engine::new_with_extensions(910_001, 4, &[&pack]).expect("建局");
    // 0 号持一个技能以便验证断肠移除全部技能；1 号持挥泪+断肠；2 号持泣别。
    e.debug_grant(0, "qibie");
    e.debug_grant(1, "huilei");
    e.debug_grant(1, "duanchang");
    e.debug_grant(2, "qibie");

    submit_to_phase(&mut e, 0, Phase::Play);
    // 清空全桌桃/酒，令 1 号 1 血且无闪，0 号有手牌+装备，2 号受伤且有手牌+装备。
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).equips.clear();
    }
    e.state_core_mut().player_mut(1).hp = 1;
    e.state_core_mut().player_mut(2).max_hp = 4;
    e.state_core_mut().player_mut(2).hp = 2;

    let p0_hand = e.debug_place(0, "shan");
    let p0_weap = e.debug_place(0, "qinggang");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: p0_weap,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert!(e.state_core().player(0).hand.contains(&p0_hand));
    assert_eq!(e.state_core().player(0).equips, vec![p0_weap]);

    let p2_hand = e.debug_place(2, "shan");
    let p2_equip = e.debug_place(2, "renwang");
    e.state_core_mut().player_mut(2).hand.retain(|c| *c != p2_equip);
    e.state_core_mut().player_mut(2).equips.push(p2_equip);
    assert_eq!(e.state_core().player(2).hand, vec![p2_hand]);

    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    assert!(!e.state_core().player(1).alive, "1 号应阵亡");
    assert!(
        e.state_core().player(0).hand.is_empty() && e.state_core().player(0).equips.is_empty(),
        "挥泪：击杀者 0 号应弃置全部手牌与装备牌"
    );
    let p0_has_any_skill = e.state_core().skills.keys().any(|(o, _)| *o == 0);
    assert!(!p0_has_any_skill, "断肠：击杀者 0 号应失去全部技能");

    assert_eq!(
        e.state_core().player(2).hp,
        3,
        "泣别：其他角色死亡时 2 号弃置全部手牌与装备后回复 1 点体力"
    );
    assert!(
        e.state_core().player(2).equips.is_empty() && e.state_core().player(2).hand.len() == 2,
        "泣别：2 号旧手牌与装备已弃空并新摸 2 张牌"
    );
}

#[test]
fn pa12_batch10_oldniepan_dying_restore_and_discard_three_zones() {
    let pack = pack_for(&[("oldniepan", OLDNIEPAN)], "pa12-batch10-oldniepan");
    let mut e = Engine::new_with_extensions(910_002, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "oldniepan");

    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).equips.clear();
        e.state_core_mut().player_mut(s).judge.clear();
    }
    e.state_core_mut().player_mut(1).max_hp = 3;
    e.state_core_mut().player_mut(1).hp = 1;
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(1).face_down = true;

    let h1 = e.debug_place(1, "duel");
    let eq1 = e.debug_place(1, "renwang");
    e.state_core_mut().player_mut(1).hand.retain(|c| *c != eq1);
    e.state_core_mut().player_mut(1).equips.push(eq1);
    let j1 = e.debug_place(1, "le");
    e.state_core_mut().player_mut(1).hand.retain(|c| *c != j1);
    e.state_core_mut().player_mut(1).judge.push(j1);
    assert_eq!(e.state_core().player(1).hand, vec![h1]);

    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    let p1 = e.state_core().player(1);
    assert!(p1.alive, "旧版涅槃：1 号应脱离濒死存活");
    assert_eq!(p1.hp, 3, "旧版涅槃：1 号体力应回复至 3 点");
    assert!(!p1.linked, "旧版涅槃：restore_character_state 应解除横置");
    assert!(!p1.face_down, "旧版涅槃：restore_character_state 应翻回正面");
    assert!(
        p1.equips.is_empty() && p1.judge.is_empty(),
        "旧版涅槃：discard_all_hand_equip_judge 应清空装备区与判定区"
    );
    assert_eq!(p1.hand.len(), 3, "旧版涅槃：弃空三区后应摸 3 张牌");
    assert!(
        !p1.hand.contains(&h1) && e.state_core().discard.contains(&j1),
        "旧版涅槃：旧手牌与判定牌均已进入弃牌堆"
    );
}

#[test]
fn pa12_batch10_jysuozhou_draw_linked_players() {
    let pack = pack_for(&[("jysuozhou", JYSUOZHOU)], "pa12-batch10-jysuozhou");
    let mut e = Engine::new_with_extensions(910_003, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "jysuozhou");

    submit_to_phase(&mut e, 0, Phase::Play);
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    e.state_core_mut().player_mut(3).linked = false;
    e.state_core_mut().player_mut(1).hand.clear();
    let h1_before = e.state_core().player(1).hand.len();
    let h2_before = e.state_core().player(2).hand.len();
    let h3_before = e.state_core().player(3).hand.len();

    // 从牌堆/弃牌堆找一张梅花杀置入 0 号手牌。
    let club_sha = (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let def = &e.catalog().cards[*c as usize];
            def.kind == CardKind::Sha && def.suit == Suit::Club
        })
        .expect("标准牌堆必有梅花杀");
    e.state_core_mut().deck.retain(|c| *c != club_sha);
    e.state_core_mut().discard.retain(|c| *c != club_sha);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.retain(|c| *c != club_sha);
    }
    e.state_core_mut().player_mut(0).hand.push(club_sha);

    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: club_sha,
            targets: vec![1],
        },
    );
    drain_confirm(&mut e);

    assert_eq!(
        e.state_core().player(1).hand.len(),
        h1_before + 1,
        "索舟：横置角色 1 号应摸 1 张牌"
    );
    assert_eq!(
        e.state_core().player(2).hand.len(),
        h2_before + 1,
        "索舟：横置角色 2 号应摸 1 张牌"
    );
    assert_eq!(
        e.state_core().player(3).hand.len(),
        h3_before,
        "索舟：未横置角色 3 号不摸牌"
    );
}

#[test]
fn pa12_batch10_dangxian_and_redangxian_opening_extra_play_phase() {
    let pack = pack_for(
        &[("dangxian", DANGXIAN), ("redangxian", REDANGXIAN)],
        "pa12-batch10-dangxian",
    );
    let mut e = Engine::new_with_extensions(910_004, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "redangxian");

    submit_to_phase(&mut e, 0, Phase::Play);
    // 在弃牌堆放入一张【杀】供解烦当先回合开始时回收。
    let discard_sha = e.debug_place(0, "sha");
    e.state_core_mut()
        .player_mut(0)
        .hand
        .retain(|c| *c != discard_sha);
    e.state_core_mut().discard.push(discard_sha);
    e.state_core_mut().player_mut(0).hand.clear();

    // 结束 0 号回合 → 1 号回合开始触发 redangxian：获得弃牌堆【杀】并先进入额外出牌阶段！
    while e.state_core().turn.current == 0 {
        submit(&mut e, 0, GameAction::NextPhase);
        drain_confirm(&mut e);
    }

    assert_eq!(e.state_core().turn.current, 1, "应进入 1 号回合");
    assert_eq!(
        e.state_core().turn.phase,
        Phase::Play,
        "当先：准备阶段结束后应先进入额外出牌阶段（尚未经过摸牌阶段）"
    );
    assert!(
        e.state_core().player(1).hand.contains(&discard_sha),
        "解烦当先：回合开始应从弃牌堆获得一张【杀】"
    );
    assert_eq!(
        e.state_core().turn.extra_phases,
        vec![0],
        "额外出牌阶段结束后待恢复 Judge(0) 正常阶段序列"
    );

    // 结束先手额外出牌阶段 → 进入 Judge(0)，再推进 Judge -> Draw -> 正常 Play 阶段！
    let hand_before_normal_draw = e.state_core().player(1).hand.len();
    submit(&mut e, 1, GameAction::NextPhase);
    drain_confirm(&mut e);
    assert_eq!(e.state_core().turn.phase, Phase::Judge);
    submit_to_phase(&mut e, 1, Phase::Play);

    assert_eq!(e.state_core().turn.current, 1);
    assert_eq!(
        e.state_core().turn.phase,
        Phase::Play,
        "当先：先手额外出牌阶段结束后应经判定、摸牌阶段进入正常出牌阶段"
    );
    assert_eq!(
        e.state_core().player(1).hand.len(),
        hand_before_normal_draw + 2,
        "正常摸牌阶段应再摸 2 张牌"
    );
}

#[test]
fn pa12_batch10_stdkengqiang_oljuejue_hanyong_card_and_sha_damage_bonus() {
    let pack = pack_for(
        &[
            ("stdkengqiang", STDKENGQIANG),
            ("oljuejue", OLJUEJUE),
            ("hanyong", HANYONG),
        ],
        "pa12-batch10-damage-bonus",
    );
    let mut e = Engine::new_with_extensions(910_005, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "stdkengqiang");
    e.debug_grant(1, "oljuejue");
    e.debug_grant(2, "hanyong");

    // 1) 0 号（铿锵）：使用【决斗】，选 2（失去 1 点体力并令此牌伤害基数 +1）→ 1 号无杀受 2 点伤害。
    submit_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).hp = 4;
        e.state_core_mut().player_mut(s).max_hp = 4;
    }
    let duel = e.debug_place(0, "duel");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: duel,
            targets: vec![1],
        },
    );
    // 确认发动铿锵
    let ask1 = e.open_choice().cloned().expect("铿锵确认窗");
    assert!(ask1.spec.options.contains(&ChoiceOption::Confirm));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ask1.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    // 选择项 2（Tag(2)：失去 1 点体力且此牌伤害 +1）
    let ask2 = e.open_choice().cloned().expect("铿锵二选一");
    assert!(ask2.spec.options.contains(&ChoiceOption::Tag(2)));
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ask2.id,
            selection: vec![ChoiceOption::Tag(2)],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3, "铿锵选项 2：0 号失去 1 点体力");
    assert_eq!(
        e.state_core().player(1).hp,
        2,
        "铿锵选项 2：【决斗】伤害基数 +1，1 号受 2 点伤害（4 -> 2）"
    );

    // 2) 1 号（绝决）：每回合首次使用【杀】伤害基数 +1 → 2 号无闪受 2 点伤害。
    submit_to_phase(&mut e, 1, Phase::Play);
    e.state_core_mut().player_mut(2).hand.clear();
    e.state_core_mut().player_mut(2).hp = 4;
    let sha = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(2).hp,
        2,
        "绝决：首次使用【杀】伤害基数 +1，2 号受 2 点伤害（4 -> 2）"
    );

    // 3) 2 号（悍勇）：已受伤（hp=2 < max_hp=4）使用【南蛮入侵】→ 伤害基数 +1 且轮数 <= hp 获得标记 638947751。
    submit_to_phase(&mut e, 2, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
    }
    e.state_core_mut().player_mut(3).hp = 4;
    let nanman = e.debug_place(2, "savage");
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: nanman,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(3).hp,
        2,
        "悍勇：受伤时使用【南蛮入侵】伤害基数 +1，3 号受 2 点伤害（4 -> 2）"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(2, 638947751))
            .copied()
            .unwrap_or(0),
        1,
        "悍勇：当前轮数 <= 体力值时获得标记 638947751"
    );
}

#[test]
fn pa12_batch10_liegong_and_xinanjian_sha_target_unrespondable_and_damage() {
    let pack = pack_for(
        &[("liegong", LIEGONG), ("xinanjian", XINANJIAN)],
        "pa12-batch10-liegong-xinanjian",
    );
    let mut e = Engine::new_with_extensions(910_006, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "liegong");
    e.debug_grant(1, "xinanjian");

    // 1) 0 号（烈弓，hp=2）对 1 号（手牌数=2 >= 0 号 hp，且手里有【闪】）出【杀】→ 1 号不可响应直接受伤。
    submit_to_phase(&mut e, 0, Phase::Play);
    e.state_core_mut().player_mut(0).hp = 2;
    e.state_core_mut().player_mut(1).hand.clear();
    e.state_core_mut().player_mut(1).hp = 4;
    let _s1 = e.debug_place(1, "shan");
    let _s2 = e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    // 确认发动烈弓（此后不应出现 1 号出【闪】询问！）
    let ask = e.open_choice().cloned().expect("烈弓确认窗");
    assert_eq!(ask.observer, 0);
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ask.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(
        e.open_choice().is_none(),
        "烈弓：目标不可响应此【杀】，不应弹出 1 号出【闪】窗口"
    );
    assert_eq!(
        e.state_core().player(1).hp,
        3,
        "烈弓：1 号虽有两张【闪】仍直接受 1 点伤害"
    );

    // 2) 1 号（暗箭，装备诸葛连弩/麒麟弓或对无武器且距离 >1 的目标出杀；给 1 号装 -1 马，对 3 号（距离 2->1，但 3 号看 1 号距离=2 > 攻击范围 1）出【杀】）
    submit_to_phase(&mut e, 1, Phase::Play);
    for s in 0..4u8 {
        e.state_core_mut().player_mut(s).hand.clear();
        e.state_core_mut().player_mut(s).equips.clear();
    }
    e.state_core_mut().player_mut(3).hp = 4;
    let horse_minus = e.debug_place(1, "horse_minus");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: horse_minus,
            targets: vec![],
        },
    );
    drain_confirm(&mut e);
    // 此时 1->3 距离为 1（可出杀），而 3->1 距离为 2 > 3 号攻击范围 1（滿足 owner_outside_target_attack_range）！
    let sha2 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![3],
        },
    );
    let ask_anjian = e.open_choice().cloned().expect("暗箭二选一窗");
    assert_eq!(ask_anjian.observer, 1);
    assert!(ask_anjian.spec.options.contains(&ChoiceOption::Tag(2)));
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ask_anjian.id,
            selection: vec![ChoiceOption::Tag(2)],
        },
    );
    drain_confirm(&mut e);
    assert_eq!(
        e.state_core().player(3).hp,
        2,
        "暗箭选项 2：此【杀】对 3 号伤害 +1，3 号受 2 点伤害（4 -> 2）"
    );
}

#[test]
fn pa12_batch10_twfujian_equip_weapon_recover_min_hp_and_remove_cards_by_kind() {
    let mut pkg_fujian = compile_source_named("twfujian", TWFUJIAN).expect("编译 twfujian");
    // 保留已接入的 phase_prepare 与 cards_moved 触发段（game_started 属 §2 C 开局帧待办）。
    pkg_fujian
        .trigger_index
        .retain(|t| t.event != "game_started");
    pkg_fujian.manifest.logic_hash = pkg_fujian.compute_logic_hash();
    let fujian_out = content_native_loader::bridge_vm_skill_decls(&pkg_fujian).expect("桥接 twfujian");

    // 同时编译含 recover_if_min_current_hp 与 remove_cards_by_kind 的触发技能以端到端验证两原语。
    let aux_dsl = r#"
skill aux_min_hp_and_remove(9910010) locked {
  trigger phase_prepare owner_is_target {
    effect recover_if_min_current_hp(self, 1);
    effect remove_cards_by_kind(sha);
  }
}
"#;
    let pkg_aux = compile_source_named("aux_min_hp_and_remove", aux_dsl).expect("编译 aux");
    let aux_out = content_native_loader::bridge_vm_skill_decls(&pkg_aux).expect("桥接 aux");

    let mut vm_skills = fujian_out.decls;
    vm_skills.extend(aux_out.decls);
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "pa12-batch10-fujian-aux",
            version: 1,
            description: "PA12 批⑩ 负剑、最低体力回复与牌种移出",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(910_007, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "twfujian");
    e.debug_grant(1, "aux_min_hp_and_remove");

    submit_to_phase(&mut e, 0, Phase::Play);
    e.state_core_mut().player_mut(0).hand.clear();
    e.state_core_mut().player_mut(1).equips.clear();
    e.state_core_mut().player_mut(1).max_hp = 4;
    e.state_core_mut().player_mut(1).hp = 1; // 全场最低体力

    // 确认推进前牌堆/弃牌堆中存在普通【杀】（kind == 1）。
    let sha_before = run_query(
        e.state_core(),
        e.catalog(),
        4,
        1,
        QueryRequest::PublicCardKindCount { kind: 1 },
    )
    .expect("查询公共区杀数量");
    assert!(matches!(sha_before, QueryResponse::Int(n) if n > 0));

    // 结束 0 号回合 → 1 号进入准备阶段触发 twfujian 与 aux_min_hp_and_remove！
    while e.state_core().turn.current == 0 {
        submit(&mut e, 0, GameAction::NextPhase);
        drain_confirm(&mut e);
    }

    let has_weapon = e.state_core().player(1).equips.iter().any(|c| {
        e.catalog().card_slot_of(e.state_core(), *c) == Some(EquipSlot::Weapon)
    });
    assert!(
        has_weapon,
        "负剑（phase_prepare）：1 号武器栏为空时应从牌堆自动装备一把武器"
    );
    assert_eq!(
        e.state_core().player(1).hp,
        2,
        "recover_if_min_current_hp：1 号为全场最低体力（1），准备阶段应回复 1 点体力至 2"
    );

    let sha_after = run_query(
        e.state_core(),
        e.catalog(),
        4,
        1,
        QueryRequest::PublicCardKindCount { kind: 1 },
    )
    .expect("查询移出后公共区杀数量");
    assert_eq!(
        sha_after,
        QueryResponse::Int(0),
        "remove_cards_by_kind(sha)：牌堆与弃牌堆中的普通【杀】应全部移出至 removed 区"
    );
    let removed_sha = e
        .state_core()
        .vm_aux
        .piles
        .get(&(1, "removed".to_string()))
        .and_then(|v| v.first().copied())
        .expect("removed 区至少有一张被移出的杀");
    let zone = run_query(
        e.state_core(),
        e.catalog(),
        4,
        1,
        QueryRequest::CardZone {
            card: CardId::new(removed_sha),
        },
    )
    .expect("查询被移出卡牌区域");
    assert_eq!(zone, QueryResponse::Int(7), "被移出卡牌的 CardZone 应为 7（Removed）");
}
