//! Golden semantics for EXTRA 神赵云「龙魂」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const RELONGHUN: u32 = 2_722_411_221;
const LONGHUN: u32 = 3_422_266_115;
const SHEN_ZHAOYUN: u32 = 3_401_672_399;
const EXTRA: &[u8] = include_bytes!("../../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

fn setup(owner_is_current: bool) -> (GameState, PlayerId, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(RELONGHUN as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: RELONGHUN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let players = engine.state().players.alive_by_seat();
    let current = engine.state().turn.current;
    let other = players
        .into_iter()
        .find(|player| *player != current)
        .unwrap();
    let (owner, attacker) = if owner_is_current {
        (current, other)
    } else {
        (other, current)
    };
    let mut tx = Transaction::begin(engine.state().clone());
    // Roster-independent: the standard roster gained choice-bearing characters
    // (gongsunzan 义从 / old_re_lidian 询询+忘隙) whose booted presence changes
    // range/hand-limit/trigger answers; this test grants its own skills.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }

    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: if owner_is_current { owner } else { attacker },
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(RELONGHUN),
            },
        ],
    )
    .unwrap();
    for player in [owner, attacker] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.max_hp = 8;
        participant.hp = 8;
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == RELONGHUN)
        .unwrap();
    (tx.state().clone(), owner, attacker, skill)
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

fn replace_with_longhun(
    state: GameState,
    owner: PlayerId,
    old_skill: SkillId,
) -> (GameState, SkillId) {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: old_skill },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(LONGHUN),
            },
        ],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LONGHUN)
        .unwrap();
    (tx.state().clone(), skill)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_closes_shen_zhaoyun_with_typed_multi_card_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_ZHAOYUN)
        .unwrap();
    assert_eq!(character.skills, [2_679_563_700, RELONGHUN]);
    let rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == RELONGHUN)
        .collect();
    assert_eq!(rules.len(), 6);
    assert!(rules
        .iter()
        .all(|rule| rule.max_cards == 2 && rule.same_suit));
    assert!(rules.iter().any(|rule| {
        rule.as_kind == "sha"
            && rule.nature.as_deref() == Some("fire")
            && rule.multi_card_bonus.as_deref() == Some("damage_1")
    }));
    let legacy_rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == LONGHUN)
        .collect();
    assert_eq!(legacy_rules.len(), 6);
    assert!(legacy_rules.iter().all(|rule| {
        rule.owner_hp_min_one_cards
            && rule.source_zone.as_deref() == Some("hand_equip_expansion")
            && rule.multi_card_bonus.is_none()
    }));
}

#[test]
fn longhun_requires_current_hp_cards_and_has_no_relonghun_bonus() {
    let (state, owner, target, old_skill) = setup(true);
    let (mut state, skill) = replace_with_longhun(state, owner, old_skill);
    state.players.get_mut(owner).unwrap().hp = 2;
    let cards = state.hand_cards(owner);
    for card in cards.iter().take(2) {
        state.cards.get_mut(*card).unwrap().suit = Suit::Diamond;
    }
    for card in state.hand_cards(target) {
        let card = state.cards.get_mut(card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Spade;
    }
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let expansion = state.zones.alloc(Zone {
        id: engine_api::ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: Vec::new(),
    });
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![cards[1]],
            to: expansion,
            from: Some(hand),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let (unchanged, rejected) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cards[0]],
        },
        20,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let hp = state.players.get(target).unwrap().hp;
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: cards[..2].to_vec(),
        },
        21,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn longhun_response_choice_tracks_current_hp_exactly() {
    let (state, owner, attacker, old_skill) = setup(false);
    let (mut state, _skill) = replace_with_longhun(state, owner, old_skill);
    state.players.get_mut(owner).unwrap().hp = 3;
    let sources = state.hand_cards(owner);
    for source in &sources {
        let card = state.cards.get_mut(*source).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Club;
    }
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        22,
    );
    let source_tokens: std::collections::BTreeSet<_> =
        sources.iter().map(|card| card.raw()).collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !source_tokens.contains(token))
        .unwrap();
    let (state, source_choice) = resolve(state, owner, vec![conversion], 23);
    let choice = source_choice.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (3, 3));
    let selected = sources[..3]
        .iter()
        .map(|card| card.raw())
        .collect::<Vec<_>>();
    let (state, result) = resolve(state, owner, selected, 24);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn two_diamonds_become_a_two_damage_fire_sha() {
    let (mut state, owner, target, skill) = setup(true);
    let cards = state.hand_cards(owner);
    for card in cards.iter().take(2) {
        state.cards.get_mut(*card).unwrap().suit = Suit::Diamond;
    }
    for card in state.hand_cards(target) {
        let card = state.cards.get_mut(card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Spade;
    }
    let before = state.players.get(target).unwrap().hp;
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: cards.iter().take(2).copied().collect(),
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, before - 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .take(2)
        .all(|card| state.cards.get(*card).unwrap().zone == discard));
}

#[test]
fn two_hearts_become_a_tao_that_recovers_two() {
    let (mut state, owner, _other, skill) = setup(true);
    let cards = state.hand_cards(owner);
    for card in cards.iter().take(2) {
        state.cards.get_mut(*card).unwrap().suit = Suit::Heart;
    }
    state.players.get_mut(owner).unwrap().hp = 4;
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: cards.iter().take(2).copied().collect(),
        },
        2,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 6);
}

#[test]
fn two_clubs_answer_sha_then_discard_the_current_players_card() {
    let (mut state, owner, attacker, _skill) = setup(false);
    let sources = state.hand_cards(owner);
    for card in &sources {
        let card = state.cards.get_mut(*card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Club;
    }
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let before = state.players.get(owner).unwrap().hp;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        3,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let source_tokens: std::collections::BTreeSet<_> =
        sources.iter().map(|card| card.raw()).collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !source_tokens.contains(token))
        .unwrap();
    let (state, source_choice) = resolve(state, owner, vec![conversion], 4);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    assert_eq!(source_choice.choice.unwrap().max_count, 2);
    let selected: Vec<_> = sources.iter().take(2).map(|card| card.raw()).collect();
    let (state, discard_choice) = resolve(state, owner, selected.clone(), 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let choice = discard_choice.choice.unwrap();
    assert_eq!(choice.observer, owner);
    let discarded = choice.options[0].token;
    let (state, result) = resolve(state, owner, vec![discarded], 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, before);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(selected.iter().all(|card| {
        state
            .cards
            .get(engine_api::CardId::new(*card))
            .unwrap()
            .zone
            == discard
    }));
    assert_eq!(
        state
            .cards
            .get(engine_api::CardId::new(discarded))
            .unwrap()
            .zone,
        discard
    );
}

#[test]
#[ignore]
fn two_hearts_can_save_a_dying_player_for_two_hp() {
    let (mut state, owner, attacker, _skill) = setup(false);
    // The dying window must offer the 急救 red-card→桃 conversion; grant it
    // explicitly so the test is independent of the booted roster.
    {
        let mut tx = Transaction::begin(state);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(2_961_041_949), // jijiu
            }],
        )
        .unwrap();
        state = tx.commit().state;
    }
    let sources = state.hand_cards(owner);
    for card in &sources {
        let card = state.cards.get_mut(*card).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Heart;
    }
    state.players.get_mut(owner).unwrap().hp = 1;
    let attack = state.hand_cards(attacker)[0];
    state.cards.get_mut(attack).unwrap().kind = CardKind::Sha;
    let (state, save) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        7,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DyingSave
    );
    let conversion = save
        .choice
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| !matches!(*token, 0 | 1))
        .unwrap();
    let (state, _) = resolve(state, owner, vec![conversion], 8);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let selected = sources.iter().take(2).map(|card| card.raw()).collect();
    let (state, result) = resolve(state, owner, selected, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert!(!state.players.get(owner).unwrap().dying);
}
