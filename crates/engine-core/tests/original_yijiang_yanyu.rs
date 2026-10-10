//! Golden coverage for 夏侯氏「燕语」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YANYU: u32 = 2_263_835_600;
const QIAOSHI: u32 = 2_607_312_044;
const XIAHOUSHI: u32 = 4_278_886_536;
const PANZHANGMAZHONG: u32 = 3_150_630_034;
const ZHANGCHUNHUA: u32 = 2_950_326_494;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/yanyu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    male: PlayerId,
    female: PlayerId,
    skill: SkillId,
    sha: [CardId; 3],
    non_sha: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner);
    let male = others.next().unwrap();
    let female = others.next().unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = XIAHOUSHI;
    tx.state_mut().players.get_mut(male).unwrap().character_def = PANZHANGMAZHONG;
    tx.state_mut()
        .players
        .get_mut(female)
        .unwrap()
        .character_def = ZHANGCHUNHUA;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YANYU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YANYU)
        .unwrap();
    let sha = [
        inject(tx.state_mut(), owner, 91_001, CardKind::Sha),
        inject(tx.state_mut(), owner, 91_002, CardKind::ShaFire),
        inject(tx.state_mut(), owner, 91_003, CardKind::ShaThunder),
    ];
    let non_sha = inject(tx.state_mut(), owner, 91_004, CardKind::TrickWuzhong);
    Setup {
        state: tx.commit().state,
        owner,
        male,
        female,
        skill,
        sha,
        non_sha,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: vec![card],
            },
        },
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
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn end_play(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn package_publishes_complete_xiahoushi_in_abi53_package() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIAHOUSHI)
        .unwrap();
    assert_eq!(character.skills, [QIAOSHI, YANYU]);
    let compiled = content_compiler::compile_source_named("yanyu", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn selection_only_offers_logical_sha_and_recast_counts_the_current_play_phase() {
    let setup = setup(2_026_071_801);
    let (state, offered) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let offered_tokens: Vec<_> = offered
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered_tokens, setup.sha.map(CardId::raw));
    assert!(!offered_tokens.contains(&setup.non_sha.raw()));
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_use_count,
        0
    );

    let hand = state.hand_count(setup.owner);
    let (state, recast) = resolve(state, setup.owner, vec![setup.sha[0].raw()], false, 2);
    assert!(matches!(recast.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.sha[0]).unwrap().zone, discard);
    let instance = state.skills.get(setup.skill).unwrap();
    assert_eq!((instance.use_count, instance.play_phase_use_count), (1, 1));
}

#[test]
fn two_recasts_offer_only_a_male_other_player_then_draw_two() {
    let setup = setup(2_026_071_802);
    let (state, first) = activate(setup.state, setup.owner, setup.skill, setup.sha[0], 10);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (state, second) = activate(state, setup.owner, setup.skill, setup.sha[1], 11);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.skills.get(setup.skill).unwrap().play_phase_use_count,
        2
    );

    let (state, optional) = end_play(state, setup.owner, 12);
    assert!(matches!(optional.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, target_choice) = resolve(state, setup.owner, vec![1], false, 13);
    assert!(matches!(
        target_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let target_tokens: Vec<_> = target_choice
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(target_tokens, vec![setup.male.raw()]);

    let before = canonical_state_hash(&state);
    let (same, forged) = resolve(state, setup.owner, vec![setup.female.raw()], false, 14);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(same.has_pending_choice());

    let hand = same.hand_count(setup.male);
    let (state, accepted) = resolve(same, setup.owner, vec![setup.male.raw()], false, 15);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.male), hand + 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn one_recast_is_insufficient_and_entering_a_new_play_phase_resets_the_counter() {
    let setup = setup(2_026_071_803);
    let (state, used) = activate(setup.state, setup.owner, setup.skill, setup.sha[0], 20);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let (state, ended) = end_play(state, setup.owner, 21);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(ended.choice.is_none());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state()
            .skills
            .get(setup.skill)
            .unwrap()
            .play_phase_use_count,
        0
    );
    let (state, used) = activate(
        tx.commit().state,
        setup.owner,
        setup.skill,
        setup.sha[1],
        22,
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    let (state, ended) = end_play(state, setup.owner, 23);
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(ended.choice.is_none());
    assert!(!state.has_pending_choice());
}
