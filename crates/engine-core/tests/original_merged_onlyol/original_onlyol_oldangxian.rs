//! Golden for ONLYOL 廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OL_LIAOHUA: u32 = 2_195_044_091;
const OLDANGXIAN: u32 = 4_090_178_594;
const OLFULI: u32 = 3_137_508_676;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    previous: PlayerId,
    owner: PlayerId,
    target: PlayerId,
    draw_sha: CardId,
    discard_sha: CardId,
    other_sha: CardId,
}

fn inject_card(
    state: &mut GameState,
    zone_kind: ZoneKind,
    owner: PlayerId,
    def: u32,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: matches!(zone_kind, ZoneKind::DiscardPile),
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let previous = tx.state().turn.current;
    let previous_index = seats.iter().position(|player| *player == previous).unwrap();
    let owner = seats[(previous_index + 1) % seats.len()];
    let target = seats[(previous_index + 2) % seats.len()];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDANGXIAN),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let draw_sha = inject_card(
        tx.state_mut(),
        ZoneKind::DrawPile,
        PlayerId::NONE,
        149_001,
        Suit::Spade,
    );
    let discard_sha = inject_card(
        tx.state_mut(),
        ZoneKind::DiscardPile,
        PlayerId::NONE,
        149_002,
        Suit::Heart,
    );
    let other_sha = inject_card(tx.state_mut(), ZoneKind::Hand, owner, 149_003, Suit::Club);
    Setup {
        state: tx.commit().state,
        previous,
        owner,
        target,
        draw_sha,
        discard_sha,
        other_sha,
    }
}

fn command(
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

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, actor, CommandKind::EndPhase, key)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
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

fn open_gain_choice(setup: Setup, key: u64) -> (Setup, GameState, PipelineResult) {
    let (state, offered) = end_phase(setup.state.clone(), setup.previous, key);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainShaForOpeningExtraPlay
    );
    (setup, state, offered)
}

#[test]
fn package_publishes_complete_ol_liaohua_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // N01「胆守」新增 host effect tag 771，包 ABI 提升到 757。
    assert_eq!(package.manifest.rule_vm_abi_version, 757);
    // N01 新增 ol_jie_danshou 后 193 个顶层技能。
    assert_eq!(package.skill_index.len(), 193);
    // N01 新增 ol_jie_zhuran 后 86 个角色。
    assert_eq!(package.character_index.len(), 86);
    assert_eq!(package.trigger_index.len(), 32); // 批162 olrenxin +1；N01 胆守 +1
    assert_eq!(package.modifier_index.len(), 171); // 批162 olrenxin noop modifier -1
    assert_eq!(package.modules.len(), 36); // 批162 olrenxin +1；N01 胆守 trigger+active +2
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_LIAOHUA)
        .unwrap();
    assert_eq!(character.name_key, "ol_liaohua");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLDANGXIAN, OLFULI]);
}

#[test]
fn gain_is_optional_and_cancelling_does_not_arm_the_penalty() {
    let setup = setup(2_026_072_501);
    let owner = setup.owner;
    let draw_sha = setup.draw_sha;
    let (setup, state, offered) = open_gain_choice(setup, 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (0, 1, true)
    );
    assert_eq!(
        choice
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let hp = state.players.get(owner).unwrap().hp;
    let (state, result) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&draw_sha));
    let (state, result) = end_phase(state, owner, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(state
        .zones
        .get(
            state
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap()
        )
        .unwrap()
        .cards
        .contains(&setup.discard_sha));
}

#[test]
fn gained_draw_pile_sha_alone_ignores_distance_and_no_damage_causes_self_damage() {
    let setup = setup(2_026_072_502);
    let owner = setup.owner;
    let draw_sha = setup.draw_sha;
    let discard_sha = setup.discard_sha;
    let other_sha = setup.other_sha;
    let (_, state, _) = open_gain_choice(setup, 44);
    let hp = state.players.get(owner).unwrap().hp;
    let (state, result) = resolve(state, owner, vec![1], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&draw_sha));
    assert!(!state.hand_cards(owner).contains(&discard_sha));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        &state, owner, draw_sha
    ));
    assert!(!engine_core::query::card_distance_unlimited_for_card(
        &state, owner, other_sha
    ));
    let (state, result) = end_phase(state, owner, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(!engine_core::query::card_distance_unlimited_for_card(
        &state, owner, draw_sha
    ));
}

#[test]
fn damage_dealt_after_accepting_suppresses_the_self_damage() {
    let setup = setup(2_026_072_503);
    let owner = setup.owner;
    let target = setup.target;
    let (_, state, _) = open_gain_choice(setup, 100);
    let (state, _) = resolve(state, owner, vec![1], false, 21);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let (state, result) = end_phase(state, owner, 119);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn opening_phase_origin_prevents_oldangxian_from_firing_in_another_skills_segment() {
    let setup = setup(2_026_072_504);
    let owner = setup.owner;
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLFULI),
        }],
    )
    .unwrap();
    let other_skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == OLFULI)
        })
        .unwrap();
    run_effects(&mut tx, vec![EffectRequest::BeginTurn { player: owner }]).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::QueueOpeningExtraPlayPhase {
            player: owner,
            skill: other_skill,
        }],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::FinishTurnBegin { player: owner }]
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, Vec::new(), true, 30);
    let (state, result) = end_phase(state, owner, 31);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.opening_extra_play_active_skill, other_skill);
}
