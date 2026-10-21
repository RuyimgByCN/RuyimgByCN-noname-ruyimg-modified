//! Golden coverage for MOBILE 李遗「教化」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIAOHUA: u32 = 4_264_685_853;
const LIWEI: u32 = 3_400_062_521;
const SOURCE: &str = include_str!("../../../../content/original/ported/mobile/jiaohua.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    skill: engine_api::SkillId,
}

fn inject(tx: &mut Transaction, zone_kind: ZoneKind, kind: CardKind, def: u32) -> CardId {
    let owner = PlayerId::NONE;
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jiaohua-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_073_301, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
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
        player: owner,
        skill_def: SkillDefId::new(JIAOHUA),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == JIAOHUA)
                .then_some(*skill)
        })
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        other,
        skill,
    }
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

fn activate(state: GameState, setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    submit(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![selection],
            cancelled: false,
        },
        key,
    )
}

fn tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_liwei_and_jiaohua() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 328);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == LIWEI)
            .unwrap()
            .skills,
        [JIAOHUA]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == JIAOHUA)
        .unwrap();
    assert_eq!(active.uses_per_play_phase, Some(2));
    assert!(active.targets.is_none());
    assert!(active.selection.is_none());
}

#[test]
fn category_cycle_persists_gains_from_draw_only_and_resets_after_all_three() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state.clone());
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let existing = tx.state().zones.get(draw).unwrap().cards.clone();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: existing,
            to: discard,
            from: Some(draw),
        }],
    )
    .unwrap();
    let lower_basic = inject(&mut tx, ZoneKind::DrawPile, CardKind::Sha, 99_701);
    inject(&mut tx, ZoneKind::DrawPile, CardKind::TrickWuzhong, 99_702);
    let top_basic = inject(&mut tx, ZoneKind::DrawPile, CardKind::Shan, 99_703);
    let discard_equipment = inject(&mut tx, ZoneKind::DiscardPile, CardKind::Armor, 99_704);
    let mut state = tx.commit().state;

    let (next, offered) = activate(state, &setup, 1);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(tokens(&offered), [1, 2, 4]);
    assert_eq!(
        next.resolution.continuation_kind,
        ContinuationKind::ResolveJiaohua
    );
    let (next, targets) = choose(next, setup.owner, 1, 2);
    assert_eq!(
        tokens(&targets),
        next.players
            .alive_by_seat()
            .iter()
            .map(|p| p.raw())
            .collect::<Vec<_>>()
    );
    let (next, gained) = choose(next, setup.owner, setup.other.raw(), 3);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    let other_hand = next.zones.find(ZoneKind::Hand, setup.other).unwrap();
    assert_eq!(next.cards.get(top_basic).unwrap().zone, other_hand);
    assert_ne!(next.cards.get(lower_basic).unwrap().zone, other_hand);

    let (next, offered) = activate(next, &setup, 4);
    assert_eq!(tokens(&offered), [2, 4]);
    let (next, targets) = choose(next, setup.owner, 4, 5);
    assert!(tokens(&targets).contains(&setup.owner.raw()));
    let owner_hand_before = next.hand_count(setup.owner);
    let (next, empty_gain) = choose(next, setup.owner, setup.owner.raw(), 6);
    assert!(matches!(empty_gain.result, CommandResult::Accepted { .. }));
    assert_eq!(next.hand_count(setup.owner), owner_hand_before);
    assert_eq!(
        next.cards.get(discard_equipment).unwrap().zone,
        next.zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (next, rejected) = activate(next, &setup, 7);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    state = next;
    state.skills.reset_play_phase_use_counts();

    let (next, offered) = activate(state, &setup, 8);
    assert_eq!(tokens(&offered), [2]);
    let (next, _) = choose(next, setup.owner, 2, 9);
    let (mut next, gained) = choose(next, setup.owner, setup.owner.raw(), 10);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    next.skills.reset_play_phase_use_counts();
    let (_, offered) = activate(next, &setup, 11);
    assert_eq!(tokens(&offered), [1, 2, 4]);
}
