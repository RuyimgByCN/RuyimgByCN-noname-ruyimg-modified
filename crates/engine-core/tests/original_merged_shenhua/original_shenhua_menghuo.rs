//! Golden coverage for 祸首/再起/求首（shenhua 孟获，ABI 540-542）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HUOSHOU: u32 = 2_566_702_627;
const ZAIQI: u32 = 3_013_017_329;
const TWQIUSHOU: u32 = 1_145_148_285;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/menghuo.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("menghuo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
        skill_def: SkillDefId::new(HUOSHOU),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZAIQI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWQIUSHOU),
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

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_menghuo_contract() {
    let compiled = content_compiler::compile_source_named("menghuo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 542);
    let zaiqi = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZAIQI)
        .unwrap();
    assert_eq!(zaiqi.event, "phase_draw");
    assert_eq!(zaiqi.filter, "owner_is_target_draw_not_replaced");
    assert!(zaiqi.optional);
    let qiushou = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TWQIUSHOU)
        .unwrap();
    assert_eq!(qiushou.event, "card_resolved");
    assert_eq!(
        qiushou.filter,
        "owner_alive_nanman_resolved_target_dead_or_damaged_gt_three"
    );
    let qiushou_skill = compiled
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == TWQIUSHOU)
        .unwrap();
    assert!(qiushou_skill.lord_only);
    let huoshou_mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == HUOSHOU)
        .map(|m| m.target.as_str())
        .collect();
    assert!(huoshou_mods.contains(&"card_target_cancel"));
    assert!(huoshou_mods.contains(&"nanman_damage_source_as_owner"));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
}

#[test]
fn huoshou_redirects_nanman_damage_source_to_the_owner() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let nanman = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::TrickNanman,
        96_300,
        Suit::Spade,
        7,
    );
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.other,
            target: setup.far,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: nanman,
            card_kind: 24,
            card_flags: 0,
        }],
    )
    .unwrap();
    let dealt = events
        .iter()
        .find_map(|event| match event {
            DomainEvent::DamageDealt { source, .. } => Some(*source),
            _ => None,
        })
        .expect("damage dealt event");
    assert_eq!(
        dealt, setup.owner,
        "nanman damage source redirected to 孟获"
    );
    let far = tx.state().players.get(setup.far).unwrap();
    assert_eq!(
        far.hp,
        far.max_hp as i32 - 1,
        "far player took exactly one damage"
    );
}

#[test]
fn zaiqi_replaces_draw_and_recovers_per_heart() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // 主公 owner: max hp 5. Wound the owner once (5 -> 4): lost HP = 1 -> flip 2 cards.
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
    let mut state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    // Stack a heart then a club on top of the draw pile.
    let mut tx = Transaction::begin(state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let heart = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        96_310,
        Suit::Heart,
        5,
    );
    let club = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_311,
        Suit::Club,
        8,
    );
    // Enter the draw phase (owner's turn).
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let state = tx.commit().state;
    let mut dispatch_tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        fired,
        Err(EffectError::ChoiceRequested),
        "optional 再起 confirm opened"
    );
    let mut state = dispatch_tx.commit().state;
    assert!(state.has_pending_choice(), "optional zaiqi confirm");
    state = resolve_choice(state, setup.owner, vec![1], 2);
    // Heart healed 1 HP; the club went to the owner's hand; the draw was replaced.
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        5,
        "heart recovered one HP"
    );
    assert!(
        state.hand_cards(setup.owner).contains(&club),
        "non-heart revealed card gained"
    );
    assert!(!state.hand_cards(setup.owner).contains(&heart));
    assert_eq!(
        state.cards.get(heart).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap(),
        "heart revealed card to discard"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn qiushou_draws_for_shu_qun_after_nanman_death() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Kill the far player so the Nanman resolution has a dead target.
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: setup.far,
            killer: setup.other,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert!(!state.players.get(setup.far).unwrap().alive);
    let hands_before: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| (player, state.hand_count(player)))
        .collect();
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: setup.other,
            card: CardId::NONE,
            kind: 24,
            targets: vec![setup.owner, setup.other, setup.far],
            damaged_targets: vec![],
            source_cards: vec![],
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    for (player, before) in hands_before {
        let group = state.player_group(player).map(|g| g.to_string());
        let grew = state.hand_count(player) > before;
        assert_eq!(
            grew,
            matches!(group.as_deref(), Some("shu") | Some("qun")),
            "player {player:?} group {group:?} draw behavior"
        );
    }
}

#[test]
fn qiushou_does_not_fire_without_death_or_many_damage() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let hands_before: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| (player, tx.state().hand_count(player)))
        .collect();
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: setup.other,
            card: CardId::NONE,
            kind: 24,
            targets: vec![setup.owner, setup.other],
            damaged_targets: vec![setup.other],
            source_cards: vec![],
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    assert!(effects.is_empty(), "no 求首 trigger for 1 damaged target");
    let state = tx.commit().state;
    for (player, before) in hands_before {
        assert_eq!(state.hand_count(player), before);
    }
}
