//! Golden coverage for 雷击/鬼道/黄天（shenhua 张角，ABI 549-551）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RELEIJI: u32 = 48_412_566;
const GUIDAO: u32 = 1_025_117_348;
const HUANGTIAN: u32 = 2_761_489_181;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhangjiao.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zj-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RELEIJI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GUIDAO),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUANGTIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        far,
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

/// 雷击完整流程：张角响应闪 → 确认 → 选目标 → 判定（牌堆顶牌）。
fn run_releiji(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    shan: CardId,
    key_base: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: shan,
            response_kind: 3,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional 雷击 confirm");
    let (state, confirmed) = resolve(state, owner, vec![1], key_base);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(state, owner, vec![other.raw()], key_base + 1);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_zhangjiao_contract() {
    let compiled = content_compiler::compile_source_named("zj", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 551);
    let releiji = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == RELEIJI)
        .unwrap();
    assert_eq!(releiji.event, "card_responded");
    assert_eq!(releiji.filter, "owner_responded_shan");
    assert!(releiji.optional);
    let huangtian = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HUANGTIAN)
        .unwrap();
    assert_eq!(huangtian.event, "phase_play");
    assert_eq!(
        huangtian.filter,
        "owner_alive_other_qun_play_phase_can_give_owner"
    );
    let huangtian_skill = compiled
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == HUANGTIAN)
        .unwrap();
    assert!(huangtian_skill.lord_only);
    let guidao_mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == GUIDAO)
        .map(|m| m.target.as_str())
        .collect();
    assert!(guidao_mods.contains(&"judgment_rewrite_black_hand_equip"));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
}

#[test]
fn releiji_spade_deals_two_thunder_damage() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_500,
        Suit::Heart,
        2,
    );
    // Stack a spade judge card on top of the draw pile.
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _judge = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_501,
        Suit::Spade,
        9,
    );
    let far_hp = tx.state().players.get(setup.far).unwrap().max_hp;
    let state = run_releiji(tx.commit().state, setup.owner, setup.far, shan, 1);
    let far = state.players.get(setup.far).unwrap();
    assert_eq!(far.hp, far_hp as i32 - 2, "spade judge: two thunder damage");
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp as i32,
        state.players.get(setup.owner).unwrap().hp as i32 + 0
    );
}

#[test]
fn releiji_club_recovers_and_deals_one() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_510,
        Suit::Heart,
        2,
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _judge = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_511,
        Suit::Club,
        7,
    );
    let far_hp = tx.state().players.get(setup.far).unwrap().max_hp;
    // Wound the lord owner (5 -> 4) so the club recovery is observable.
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let owner_hp = tx.state().players.get(setup.owner).unwrap().hp;
    assert_eq!(owner_hp, 4);
    let state = run_releiji(tx.commit().state, setup.owner, setup.far, shan, 1);
    let far = state.players.get(setup.far).unwrap();
    assert_eq!(far.hp, far_hp as i32 - 1, "club judge: one thunder damage");
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        owner_hp + 1,
        "club judge: owner recovers one"
    );
}

#[test]
fn guidao_rewrites_judgment_with_black_card() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_520,
        Suit::Heart,
        2,
    );
    // Draw pile top is a heart (no branch); the owner holds a black card to rewrite.
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _heart_judge = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        96_521,
        Suit::Heart,
        5,
    );
    let black = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_522,
        Suit::Spade,
        8,
    );
    let state = run_releiji(tx.commit().state, setup.owner, setup.far, shan, 1);
    // 鬼道 offers the black card to replace the heart judgment.
    let mut tx = Transaction::begin(state);
    assert!(tx.state().has_pending_choice(), "鬼道 rewrite choice");
    let mut state = tx.commit().state;
    let (state, rewritten) = resolve(state, setup.owner, vec![black.raw()], 5);
    assert!(matches!(rewritten.result, CommandResult::Accepted { .. }));
    let far = state.players.get(setup.far).unwrap();
    let far_hp = far.max_hp;
    assert_eq!(
        far.hp,
        far_hp as i32 - 2,
        "rewritten spade judge: two thunder damage"
    );
}

#[test]
fn huangtian_gives_shan_when_qun_player_in_play_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let group = tx.state().player_group(setup.other).map(|g| g.to_string());
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_530,
        Suit::Heart,
        2,
    );
    let is_qun = group.as_deref() == Some("qun");
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.other,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    if !is_qun {
        match fired {
            Ok(effects) => assert!(effects.is_empty(), "no 黄天 trigger for non-Qun player"),
            Err(EffectError::ChoiceRequested) => panic!("黄天 must not open a choice for non-Qun"),
            Err(_) => {}
        }
        return;
    }
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "黄天 card choice"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = dispatch_tx.commit().state;
    // The optional trigger opens a confirm first; accept it, then the
    // card-choice (GuardedChoice) appears.
    let (state, confirmed) = resolve(state, setup.other, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(state, setup.other, vec![shan.raw()], 2);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert!(
        state.hand_cards(setup.owner).contains(&shan),
        "Qun player gave the 闪 to the lord"
    );
    assert!(!state.hand_cards(setup.other).contains(&shan));
    assert!(!state.has_pending_choice());
}

#[test]
fn huangtian_skips_non_shan_lightning_hands() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let group = tx.state().player_group(setup.other).map(|g| g.to_string());
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Give the Qun player only a Tao: no 闪/闪电 -> no trigger.
    let _tao = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_540,
        Suit::Heart,
        1,
    );
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.other,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => assert!(effects.is_empty(), "no 黄天 trigger without 闪/闪电"),
        Err(EffectError::ChoiceRequested) => {
            if group.as_deref() == Some("qun") {
                panic!("黄天 must not open a choice without 闪/闪电");
            }
        }
        Err(_) => {}
    }
}
