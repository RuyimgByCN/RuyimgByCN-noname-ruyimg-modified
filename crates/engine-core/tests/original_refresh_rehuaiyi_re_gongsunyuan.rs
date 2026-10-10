//! Golden coverage for 界公孙渊「怀异」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_GONGSUNYUAN: u32 = 3_935_634_515;
const REHUAIYI: u32 = 4_054_411_167;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/rehuaiyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("rehuaiyi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, u64::from(REHUAIYI), "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REHUAIYI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
        second: players[2],
    }
}

fn inject(
    state: &mut GameState,
    zone_kind: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone_owner = if matches!(
        zone_kind,
        ZoneKind::DrawPile | ZoneKind::DiscardPile | ZoneKind::Processing
    ) {
        PlayerId::NONE
    } else {
        owner
    };
    let zone = state.zones.find(zone_kind, zone_owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner: zone_owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REHUAIYI)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let skill = skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
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
fn package_publishes_complete_re_gongsunyuan_and_abi320_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 320);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GONGSUNYUAN)
        .unwrap();
    assert_eq!(character.name_key, "re_gongsunyuan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [REHUAIYI]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REHUAIYI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_play_phase, Some(2));
    assert!(active.hand_nonempty);
    let compiled = content_compiler::compile_source_named("rehuaiyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 320);
    assert_eq!(
        compiled.modules[0].logic_hash,
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );
}

#[test]
fn uniform_hand_draws_and_permits_exactly_one_more_use_this_play_phase() {
    let mut setup = setup();
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        Suit::Heart,
        99_501,
    );
    inject(
        &mut setup.state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Shan,
        Suit::Diamond,
        99_502,
    );
    let (mut state, first) = activate(setup.state, setup.owner, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(
        state
            .skills
            .get(skill(&state, setup.owner))
            .unwrap()
            .play_phase_use_count,
        1
    );

    inject(
        &mut state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Tao,
        Suit::Heart,
        99_503,
    );
    let (state, second) = activate(state, setup.owner, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(
        state
            .skills
            .get(skill(&state, setup.owner))
            .unwrap()
            .play_phase_use_count,
        2
    );

    let before = canonical_state_hash(&state);
    let (state, third) = activate(state, setup.owner, 3);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}

#[test]
fn mixed_hand_discards_one_color_gains_by_seat_and_loses_hp_after_two_cards() {
    let mut setup = setup();
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        Suit::Heart,
        99_510,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Tao,
        Suit::Diamond,
        99_514,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Shan,
        Suit::Spade,
        99_511,
    );
    let first_card = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.first,
        CardKind::Tao,
        Suit::Club,
        99_512,
    );
    let second_card = inject(
        &mut setup.state,
        ZoneKind::Equip,
        setup.second,
        CardKind::Weapon,
        Suit::Diamond,
        99_513,
    );
    let initial_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, color_choice) = activate(setup.state, setup.owner, 10);
    assert_eq!(
        color_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::Custom
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveRehuaiyi
    );
    let (state, target_choice) = resolve(state, setup.owner, vec![0b01], false, 11);
    assert_eq!(
        target_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(state.hand_count(setup.owner), 1);

    let (state, first_gain) = resolve(
        state,
        setup.owner,
        vec![setup.second.raw(), setup.first.raw()],
        false,
        12,
    );
    assert_eq!(
        first_gain.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        first_gain.choice.as_ref().unwrap().options[0].token,
        first_card.raw()
    );

    let before_forgery = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![second_card.raw()], false, 13);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before_forgery);

    let (state, second_gain) = resolve(state, setup.owner, vec![first_card.raw()], false, 14);
    assert_eq!(
        second_gain.choice.as_ref().unwrap().options[0].token,
        second_card.raw()
    );
    let (state, completed) = resolve(state, setup.owner, vec![second_card.raw()], false, 15);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, initial_hp - 1);
    assert_eq!(state.cards.get(first_card).unwrap().owner, setup.owner);
    assert_eq!(state.cards.get(second_card).unwrap().owner, setup.owner);

    let before = canonical_state_hash(&state);
    let (state, second_use) = activate(state, setup.owner, 16);
    assert!(matches!(second_use.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}

#[test]
fn cancelling_target_choice_gains_nothing_and_does_not_lose_hp() {
    let mut setup = setup();
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        Suit::Heart,
        99_520,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Shan,
        Suit::Spade,
        99_521,
    );
    let target_card = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.first,
        CardKind::Tao,
        Suit::Club,
        99_522,
    );
    let initial_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, _) = activate(setup.state, setup.owner, 20);
    let (state, _) = resolve(state, setup.owner, vec![0b01], false, 21);
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 22);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, initial_hp);
    assert_eq!(state.cards.get(target_card).unwrap().owner, setup.first);
}
