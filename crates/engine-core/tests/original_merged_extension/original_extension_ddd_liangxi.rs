//! Golden for 3D精选 梁习【仝御】（ABI 640）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TONGYU: u32 = 2_302_761_270;
const TONGYU_BLOCKER: u32 = 3_861_572_400;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/liangxi.skill");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liangxi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 640);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap())
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_804, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TONGYU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_640_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
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

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

#[test]
fn tongyu_rejects_duplicate_suits() {
    let (mut state, players) = setup();
    let owner = players[0];
    let first = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        1,
    );
    let second = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Spade,
        2,
    );
    let tongyu = skill(&state, owner, TONGYU);
    let (_state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: tongyu,
            targets: Vec::new(),
            card_targets: vec![first, second],
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn tongyu_adds_material_count_rejects_target_and_can_give_leftovers() {
    let (mut state, players) = setup();
    let owner = players[0];
    let rejector = players[1];
    let material_a = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        10,
    );
    let material_b = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        11,
    );
    let revealed: Vec<_> = (0..5)
        .map(|index| {
            inject(
                &mut state,
                PlayerId::NONE,
                ZoneKind::DrawPile,
                CardKind::Other,
                Suit::Club,
                20 + index,
            )
        })
        .collect();
    let tongyu = skill(&state, owner, TONGYU);
    let (mut state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: tongyu,
            targets: Vec::new(),
            card_targets: vec![material_a, material_b],
        },
        10,
    );
    let mut next_choice = result.choice.unwrap();
    assert_eq!(next_choice.observer, owner);

    // Counterclockwise direction.
    let (next, result) = choose(state, owner, vec![1], false, 11);
    state = next;
    next_choice = result.choice.unwrap();
    // Every target chooses: owner accepts, rejector rejects, last target accepts.
    for (index, target) in [owner, players[2], rejector].into_iter().enumerate() {
        assert_eq!(next_choice.observer, target);
        let selection = u32::from(target == rejector);
        let (next, result) = choose(state, target, vec![selection], false, 12 + index as u64);
        state = next;
        next_choice = result.choice.unwrap();
    }
    assert!(state.players.get(owner).unwrap().skills.iter().any(|id| {
        state
            .skills
            .get(*id)
            .is_some_and(|skill| skill.def.raw() == TONGYU_BLOCKER)
    }));

    // Two effective targets each take one card from the five-card pool.
    for (index, target) in [owner, players[2]].into_iter().enumerate() {
        assert_eq!(next_choice.observer, target);
        let card = next_choice.options[0].token;
        let (next, result) = choose(state, target, vec![card], false, 20 + index as u64);
        state = next;
        next_choice = result.choice.unwrap();
    }
    // Optional remained trigger, then choose the rejecting target.
    assert_eq!(next_choice.observer, owner);
    let (next, result) = choose(state, owner, vec![1], false, 30);
    state = next;
    next_choice = result.choice.unwrap();
    assert_eq!(next_choice.observer, owner);
    (state, _) = choose(state, owner, vec![rejector.raw()], false, 31);

    assert_eq!(state.hand_count(rejector), 3);
    assert!(
        revealed
            .iter()
            .filter(|card| state.hand_cards(rejector).contains(card))
            .count()
            == 3
    );
}
