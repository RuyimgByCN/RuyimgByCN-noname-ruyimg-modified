//! Golden coverage for 欧陆乐进/骁果 rewritten from the legacy JavaScript.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const EU_XIAOGUO: u32 = 2_461_210_777;
const EU_YUEJIN: u32 = 3_372_050_370;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
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
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(EU_XIAOGUO),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == EU_XIAOGUO)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
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
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_publishes_exact_unlimited_basic_hand_contract_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == EU_XIAOGUO && skill.name == "eu_xiaoguo"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == EU_YUEJIN)
        .unwrap();
    assert_eq!(character.name_key, "eu_yuejin");
    assert_eq!(character.skills, [EU_XIAOGUO]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == EU_XIAOGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (
            selection.zone.as_str(),
            selection.min,
            selection.max,
            selection.filter.as_deref(),
        ),
        ("hand", 1, 1, Some("card_basic"))
    );
}

#[test]
fn non_basic_cost_is_atomic_and_non_dying_damage_can_repeat() {
    let mut setup = setup(2_026_072_302);
    let trick = inject_hand(
        &mut setup.state,
        setup.owner,
        98_001,
        CardKind::TrickWuzhong,
    );
    let first = inject_hand(&mut setup.state, setup.owner, 98_002, CardKind::Sha);
    let second = inject_hand(&mut setup.state, setup.owner, 98_003, CardKind::Shan);
    let target = setup.targets[0];

    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(setup.state, setup.owner, target, setup.skill, trick, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hp = same.players.get(target).unwrap().hp;
    let (state, accepted) = activate(same, setup.owner, target, setup.skill, first, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.skills.get(setup.skill).unwrap().active_turn_banned);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == setup.owner && *damaged == target
    )));

    let (state, repeated) = activate(state, setup.owner, target, setup.skill, second, 3);
    assert!(matches!(repeated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert!(!state.skills.get(setup.skill).unwrap().active_turn_banned);
}

#[test]
fn exact_dying_damage_bans_until_the_next_turn_starts() {
    let mut setup = setup(2_026_072_303);
    let lethal = inject_hand(&mut setup.state, setup.owner, 98_011, CardKind::Sha);
    let retry = inject_hand(&mut setup.state, setup.owner, 98_012, CardKind::Shan);
    let dying_target = setup.targets[0];
    let surviving_target = setup.targets[1];
    let tao = inject_hand(&mut setup.state, surviving_target, 98_013, CardKind::Tao);
    setup.state.players.get_mut(dying_target).unwrap().hp = 1;

    let (dying, accepted) = activate(
        setup.state,
        setup.owner,
        dying_target,
        setup.skill,
        lethal,
        10,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::EnteredDying { player, source }
            if *player == dying_target && *source == setup.owner
    )));
    assert!(dying.skills.get(setup.skill).unwrap().active_turn_banned);
    let choice = accepted.choice.expect("dying save choice");
    assert!(choice.options.iter().any(|option| option.token == 1));
    let (state, saved) = apply_command(
        dying,
        Command {
            actor: choice.observer,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(saved.result, CommandResult::Accepted { .. }),
        "{:?}",
        saved.result
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(dying_target).unwrap().hp, 1);
    assert_eq!(
        state.cards.get(tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(state.skills.get(setup.skill).unwrap().active_turn_banned);

    let before = canonical_state_hash(&state);
    let (mut same, rejected) =
        activate(state, setup.owner, surviving_target, setup.skill, retry, 12);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(same.hand_cards(setup.owner).contains(&retry));

    same.turn.phase = Phase::End;
    let (next_turn, advanced) = apply_command(
        same,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(
        matches!(advanced.result, CommandResult::Accepted { .. }),
        "{:?}",
        advanced.result
    );
    assert_ne!(next_turn.turn.current, setup.owner);
    assert!(
        !next_turn
            .skills
            .get(setup.skill)
            .unwrap()
            .active_turn_banned
    );
}
