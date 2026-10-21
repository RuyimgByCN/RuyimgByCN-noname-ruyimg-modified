//! Golden for 3D精选 刘宠【精彀】【末焱】（ABI 619）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDJINGGOU: u32 = 3_419_786_472;
const DDDMOYAN: u32 = 4_038_206_557;
const STATE_MAGIC: u32 = 0xDDD0_6190;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/liuchong.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuchong", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
        [DDDJINGGOU, DDDMOYAN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().turn.round = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_200 + serial),
        kind,
        suit: Suit::Club,
        rank: 8,
        weapon_range: u8::from(kind == CardKind::Weapon) * 5,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_exposes_exact_abi_and_trigger_contracts() {
    let package = content_compiler::compile_source_named("ddd-liuchong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 619);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 3);
}

#[test]
fn jinggou_weapon_with_maximum_range_damages_another_player() {
    let (mut state, players) = setup(20_260_809);
    let owner = players[0];
    let target = players[1];
    let weapon = inject(&mut state, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    let instance = skill(&state, owner, DDDJINGGOU);
    let hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuchong {
                player: owner,
                actor: owner,
                card: weapon,
                action: 2,
                cards: Vec::new(),
                skill: instance,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let (state, result) = choose(state, owner, vec![target.raw()], 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn moyan_gives_selected_cards_and_increases_damage_at_its_threshold() {
    let (state, players) = setup(20_260_810);
    let owner = players[0];
    let recipient = players[1];
    let source = players[2];
    let instance = skill(&state, owner, DDDMOYAN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuchong {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                skill: instance,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![recipient.raw()], 20);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, recipient);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    let selected: Vec<_> = choice
        .options
        .iter()
        .take(2)
        .map(|option| option.token)
        .collect();
    let (state, result) = choose(state, recipient, selected, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(recipient), 2);
    let blob = &state.skills.get(instance).unwrap().state_blob;
    assert_eq!(
        u32::from_le_bytes(blob[0..4].try_into().unwrap()),
        STATE_MAGIC
    );
    assert_eq!(
        u32::from_le_bytes(blob[8..12].try_into().unwrap()),
        recipient.raw()
    );
    assert_eq!(u32::from_le_bytes(blob[12..16].try_into().unwrap()), 2);

    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 2);

    inject(&mut state, recipient, ZoneKind::Hand, CardKind::Shan, 2);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}
