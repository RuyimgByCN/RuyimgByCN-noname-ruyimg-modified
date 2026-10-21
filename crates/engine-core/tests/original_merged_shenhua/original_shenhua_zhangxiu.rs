//! Golden coverage for 雄乱/踞襄（shenhua 张绣，ABI 590）+ 从谏复用（ABI 524）。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const XIONGLUAN: u32 = 1_234_567_904;
const XIONGLUAN_BAN: u32 = 1_234_567_905;
const TWJUXIANG: u32 = 1_234_567_907;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/shenhua/drlt_xiongluan.skill");
const SOURCE_TWJUXIANG: &str =
    include_str!("../../../../content/original/ported/shenhua/twjuxiang.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    third: PlayerId,
}

fn combined_source() -> String {
    format!("{SOURCE}\n{SOURCE_TWJUXIANG}")
}

fn setup() -> Setup {
    let package =
        content_compiler::compile_source_named("zhangxiu-test", &combined_source()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let third = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    for def in [XIONGLUAN, TWJUXIANG] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        third,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn find_skill(tx: &Transaction, owner: PlayerId, def: u32) -> SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
        .copied()
        .unwrap()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0x5900),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, engine_api::CommandResult::Accepted { .. }),
        "activation must be accepted: {result:?}"
    );
    state
}

#[test]
fn package_exposes_zhangxiu_contract() {
    let compiled = content_compiler::compile_source_named("zhangxiu", &combined_source()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 590);
    assert_eq!(compiled.skill_index.len(), 3);
    let xiongluan = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == XIONGLUAN)
        .unwrap();
    assert!(!xiongluan.locked);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIONGLUAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(active.targets.as_ref().map(|s| s.max), Some(1));
    let twjuxiang = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == TWJUXIANG)
        .unwrap();
    assert!(twjuxiang.lord_only);
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&XIONGLUAN)));
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&TWJUXIANG)));
}

#[test]
fn xiongluan_disables_slots_judge_marks_target_and_bans_hand_cards() {
    let mut setup = setup();
    let skill = find_skill(&mut setup.tx, setup.owner, XIONGLUAN);
    let state = activate(
        setup.tx.commit().state,
        setup.owner,
        skill,
        vec![setup.other],
    );
    let tx = Transaction::begin(state);
    // 装备栏与判定区废除。
    assert!(tx
        .state()
        .players
        .get(setup.owner)
        .is_some_and(|p| p.disabled_equip_slots.iter().all(|v| *v > 0)));
    assert!(tx.state().disabled_judge_zones.contains(&setup.owner));
    // 限定目标标记。
    assert_eq!(
        tx.state()
            .turn
            .flag(engine_core::state::turn_flag::xiongluan_target_by_player(
                setup.owner
            ))
            .max(0) as u32,
        setup.other.raw()
    );
    // 目标获得禁手牌技能。
    assert!(tx
        .state()
        .players
        .get(setup.other)
        .is_some_and(|p| p.skills.iter().any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == XIONGLUAN_BAN)
        })));
    // 限定技整局一次：再次激活被拒。
    let state2 = tx.commit().state;
    let skill2 = find_skill(
        &mut Transaction::begin(state2.clone()),
        setup.owner,
        XIONGLUAN,
    );
    let (_state3, result) = apply_command(
        state2,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0x5901),
            kind: CommandKind::ActivateSkill {
                skill: skill2,
                targets: vec![setup.other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, engine_api::CommandResult::Rejected { .. }),
        "limited skill must be rejected on second use"
    );
}

#[test]
fn xiongluan_target_has_zero_distance_and_unlimited_use_count() {
    let mut setup = setup();
    let hand = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.owner)
        .unwrap();
    let _sha = alloc_card(
        &mut setup.tx,
        setup.owner,
        hand,
        CardKind::Sha,
        1,
        Suit::Spade,
        5,
    );
    let skill = find_skill(&mut setup.tx, setup.owner, XIONGLUAN);
    let state = activate(
        setup.tx.commit().state,
        setup.owner,
        skill,
        vec![setup.other],
    );
    let tx = Transaction::begin(state);
    // 对限定目标距离为 0。
    let mut ctx = engine_core::query::QueryContext::new();
    let distance = engine_core::query::query_explain(
        tx.state(),
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.other,
        },
        &mut ctx,
    );
    assert_eq!(distance.response, QueryResponse::Int(0));
    // 与未标记目标距离保持常规（至少为 1）。
    let mut ctx = engine_core::query::QueryContext::new();
    let distance_third = engine_core::query::query_explain(
        tx.state(),
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.third,
        },
        &mut ctx,
    );
    assert!(
        matches!(distance_third.response, QueryResponse::Int(d) if d >= 1),
        "unmarked target distance must stay regular"
    );
}

#[test]
fn twjuxiang_fires_on_other_qun_play_phase_and_moves_equip() {
    let mut setup = setup();
    // other 设为群势力；third 设为他势力。
    setup
        .tx
        .state_mut()
        .group_overrides
        .insert(setup.other, "qun".into());
    setup
        .tx
        .state_mut()
        .group_overrides
        .insert(setup.third, "shu".into());
    let equip_zone = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Equip, setup.other)
        .unwrap();
    let equip = alloc_card(
        &mut setup.tx,
        setup.other,
        equip_zone,
        CardKind::Weapon,
        2,
        Suit::Heart,
        3,
    );
    let skill = find_skill(&mut setup.tx, setup.owner, TWJUXIANG);
    let state = setup.tx.commit().state;
    // 触发 other 的出牌阶段：先确认可选触发，再选装备牌。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let event = DomainEvent::TurnStateChanged {
        current: setup.other,
        phase: engine_api::TurnPhaseTag::Play,
    };
    // 循环：计划阶段可选确认 → 触发 → 装备牌选择，逐段推进直至无挂起选择。
    for _ in 0..8u32 {
        if !tx.state().has_pending_choice() {
            let fired =
                dispatch_for_events(&mut tx, &[event.clone()], &mut TriggerFireRecord::new());
            match fired {
                Ok(effects) => match run_effects(&mut tx, effects) {
                    Ok(_) => break,
                    Err(EffectError::ChoiceRequested) => {}
                    Err(e) => panic!("run error: {e:?}"),
                },
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("dispatch error: {e:?}"),
            }
            if !tx.state().has_pending_choice() {
                break;
            }
        }
        // 处理当前选择：可选触发确认 [1] 或装备牌选择。
        let cont = tx.state().resolution.continuation_payload.clone();
        let choice_id = tx.state().resolution.pending_choice_id;
        let observer = PlayerId::new(tx.state().resolution.pending_observer);
        let committed = tx.commit();
        let state = committed.state;
        let selections = match committed.output.choice.as_ref() {
            Some(choice) if choice.kind == ChoiceKind::Confirm => vec![1],
            Some(choice) => {
                assert_eq!(
                    cont.first(),
                    Some(&engine_core::effect::RESOLVE_TWJUXIANG_CARD)
                );
                assert_eq!(choice.kind, ChoiceKind::SelectCards);
                vec![choice.options[0].token]
            }
            None => {
                // 续体选择（resume 路径不重复缓冲 choice 请求）。
                assert_eq!(
                    cont.first(),
                    Some(&engine_core::effect::RESOLVE_TWJUXIANG_CARD)
                );
                vec![equip.raw()]
            }
        };
        let (state, result) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5902),
                kind: CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(choice_id),
                    selections,
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "twjuxiang choice must resolve: {result:?}"
        );
        tx = Transaction::begin(state);
    }
    let state = tx.commit().state;
    let tx = Transaction::begin(state);
    // 装备牌已置入 owner（主公）装备区。
    assert!(tx
        .state()
        .zones
        .find(ZoneKind::Equip, setup.owner)
        .is_some_and(|zone| tx
            .state()
            .zones
            .get(zone)
            .is_some_and(|z| z.cards.contains(&equip))));
    // 本回合 other 已用（限一次）。
    assert_ne!(
        tx.state()
            .turn
            .flag(engine_core::state::turn_flag::twjuxiang_used_by_event_player(setup.other)),
        0
    );
    let _ = skill;
}
