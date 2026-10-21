//! Golden coverage for OFFLINE 樊稠「扬武」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSYANGWU: u32 = 3_480_637_807;
const NS_FANCHOU: u32 = 3_362_795_240;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_exact_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 494);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == NS_FANCHOU)
            .unwrap()
            .skills,
        [NSYANGWU]
    );
}

#[test]
fn heart_cost_is_paid_before_rounded_up_hand_difference_is_gained() {
    let package = content_compiler::compile_source_named(
        "nsyangwu-test",
        include_str!("../../../../content/original/ported/offline/nsyangwu.skill"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_202, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(NSYANGWU),
    });
    run_effects(&mut tx, effects).unwrap();
    let cost = inject(&mut tx, owner, Suit::Heart, 98_100);
    inject(&mut tx, owner, Suit::Club, 98_101);
    let target_cards: Vec<_> = (0..5)
        .map(|index| inject(&mut tx, target, Suit::Spade, 98_200 + index))
        .collect();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let (state, activated) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![cost],
        },
        1,
    );
    let choice = activated.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(state.hand_count(owner), 1);
    let (state, resolved) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: target_cards[..2].iter().map(|card| card.raw()).collect(),
            cancelled: false,
        },
        2,
    );
    assert!(resolved.events.iter().any(|event| matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.len() == 2 && to.is_empty())));
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(state.hand_count(target), 3);
}
