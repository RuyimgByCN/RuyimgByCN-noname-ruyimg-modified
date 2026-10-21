//! Golden coverage for 陈群「品第」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHENQUN: u32 = 2_960_571_502;
const PINDI: u32 = 4_057_378_399;
const FAEN: u32 = 2_817_547_947;
const PINDI_SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/pindi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PINDI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
        second: players[2],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn pindi_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == PINDI)
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

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = pindi_skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_complete_chenqun_and_exact_abi52_module_in_abi53_pack() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHENQUN)
        .unwrap();
    assert_eq!(character.skills, [PINDI, FAEN]);
    let compiled = content_compiler::compile_source_named("pindi", PINDI_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 52);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn first_use_draws_one_and_links_owner_when_target_is_wounded() {
    let mut setup = setup(2_026_073_201);
    let cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_201);
    inject(&mut setup.state, setup.first, CardKind::Shan, 99_202);
    {
        let target = setup.state.players.get_mut(setup.first).unwrap();
        target.max_hp = 4;
        target.hp = 3;
    }
    let before = setup.state.hand_count(setup.first);
    let (state, opened) = activate(setup.state, setup.owner, setup.first, cost, 1);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded
    );
    let (state, resolved) = resolve(state, setup.owner, vec![1], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), before + 1);
    assert!(state
        .players
        .get(setup.owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count == 1));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    let skill = state.skills.get(pindi_skill(&state, setup.owner)).unwrap();
    assert_eq!(skill.use_count, 1);
    assert_eq!(skill.state_blob.get(..4), Some(b"AUC1".as_slice()));
}

#[test]
fn second_use_discards_two_target_cards_and_counts_from_the_whole_turn() {
    let mut setup = setup(2_026_073_202);
    let first_cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_211);
    inject(&mut setup.state, setup.first, CardKind::Shan, 99_212);
    let (state, _) = activate(setup.state, setup.owner, setup.first, first_cost, 10);
    let (mut state, _) = resolve(state, setup.owner, vec![1], 11);

    let second_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_213);
    let target_card_a = inject(&mut state, setup.second, CardKind::Shan, 99_214);
    let target_card_b = inject(&mut state, setup.second, CardKind::Tao, 99_215);
    let target_card_c = inject(&mut state, setup.second, CardKind::Jiu, 99_216);
    let (state, opened) = activate(state, setup.owner, setup.second, second_cost, 12);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    let (state, branch) = resolve(state, setup.owner, vec![2], 13);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(branch.choice.as_ref().unwrap().observer, setup.second);
    assert_eq!(branch.choice.as_ref().unwrap().min_count, 2);
    let (state, discarded) = resolve(
        state,
        setup.second,
        vec![target_card_a.raw(), target_card_b.raw()],
        14,
    );
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(target_card_a).unwrap().zone, discard);
    assert_eq!(state.cards.get(target_card_b).unwrap().zone, discard);
    assert_ne!(state.cards.get(target_card_c).unwrap().zone, discard);
    assert_eq!(
        state
            .skills
            .get(pindi_skill(&state, setup.owner))
            .unwrap()
            .use_count,
        2
    );
}

#[test]
fn repeated_target_or_logical_category_and_forged_branch_are_atomic() {
    let mut setup = setup(2_026_073_203);
    let first_cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_221);
    inject(&mut setup.state, setup.first, CardKind::Shan, 99_222);
    let (state, _) = activate(setup.state, setup.owner, setup.first, first_cost, 20);
    let (mut state, _) = resolve(state, setup.owner, vec![1], 21);

    let repeated_target_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_223);
    let before_target = canonical_state_hash(&state);
    let (state, rejected_target) =
        activate(state, setup.owner, setup.first, repeated_target_cost, 22);
    assert!(!matches!(
        rejected_target.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before_target);

    let mut state = state;
    let repeated_category_cost = inject(&mut state, setup.owner, CardKind::Shan, 99_224);
    let before_category = canonical_state_hash(&state);
    let (state, rejected_category) =
        activate(state, setup.owner, setup.second, repeated_category_cost, 23);
    assert!(!matches!(
        rejected_category.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before_category);

    let mut state = state;
    let valid_cost = inject(&mut state, setup.owner, CardKind::Weapon, 99_226);
    inject(&mut state, setup.second, CardKind::Tao, 99_227);
    let (state, _) = activate(state, setup.owner, setup.second, valid_cost, 24);
    let before_forge = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![3], 25);
    assert!(!matches!(forged.result, CommandResult::Accepted { .. }));
    assert_eq!(canonical_state_hash(&state), before_forge);
}
