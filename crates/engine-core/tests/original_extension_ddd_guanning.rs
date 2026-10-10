//! Golden for 3D精选 管宁【游学/承经】（ABI 634）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YOUXUE: u32 = 2_449_657_606;
const CHENGJING: u32 = 4_114_032_104;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/guanning.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-guanning", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 634);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend(
        [YOUXUE, CHENGJING].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    serial: u32,
    kind: CardKind,
    zone_kind: ZoneKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_634_000 + serial),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn youxue_starts_on_round_one_moves_unique_mark_and_draws_distance() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[2];
    let skill = skill_id(&state, owner, YOUXUE);
    state.turn.round = 1;
    state.turn.current = owner;
    state.turn.phase = Phase::Prepare;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddGuanning {
                player: owner,
                actor: owner,
                action: 0,
                card: CardId::NONE,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![target.raw()], false, 1);
    assert!(result.choice.is_none());
    assert_eq!(state.hand_count(owner), 2);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, _)| *mark == YOUXUE));
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == YOUXUE && *count == 1 }));
}

#[test]
fn chengjing_keeps_last_eligible_use_then_uses_equipment_and_offers_gift() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let recipient = players[2];
    let skill = skill_id(&state, owner, CHENGJING);
    let first = inject(&mut state, actor, 1, CardKind::TrickWuzhong, ZoneKind::Hand);
    let last = inject(&mut state, actor, 2, CardKind::TrickWuzhong, ZoneKind::Hand);
    state.turn.current = actor;
    state.turn.phase = Phase::Play;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: actor,
                mark: engine_api::MarkDefId::new(YOUXUE),
                delta: 1,
            },
            EffectRequest::UseCard {
                player: actor,
                card: first,
                targets: Vec::new(),
            },
            EffectRequest::UseCard {
                player: actor,
                card: last,
                targets: Vec::new(),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.ddd_chengjing_eligible_uses,
        vec![first, last]
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddGuanning {
            player: owner,
            actor,
            action: 1,
            card: CardId::NONE,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, CHENGJING), vec![last]);

    let material = inject(tx.state_mut(), owner, 3, CardKind::Weapon, ZoneKind::Equip);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, opened) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(opened.choice.is_some());
    let (state, gift) = choose(state, owner, vec![material.raw()], false, 3);
    assert!(gift.choice.is_some(), "{gift:?}");
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(material).unwrap().zone, discard);
    let (state, result) = choose(state, owner, vec![1], false, 4);
    assert!(result.choice.is_none());
    assert!(state.hand_cards(recipient).contains(&material));
}
