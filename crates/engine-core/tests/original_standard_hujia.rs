//! Golden coverage for Standard 护驾 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HUJIA: u32 = 3_353_250_727;
const SOURCE: &str = include_str!("../../../content/standard/hujia.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first_wei: PlayerId,
    second_wei: PlayerId,
    attacker: PlayerId,
    first_shan: CardId,
    second_shan: CardId,
    sha: CardId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_184, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let [owner, first_wei, second_wei, attacker] = players.as_slice() else {
        panic!("expected four players");
    };
    let (owner, first_wei, second_wei, attacker) = (*owner, *first_wei, *second_wei, *attacker);
    for (player, character) in [(owner, 1), (first_wei, 2), (second_wei, 3), (attacker, 8)] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Lord;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUJIA),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, first_wei, second_wei, attacker] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let first_shan = tx.state().hand_cards(first_wei)[0];
    let second_shan = tx.state().hand_cards(second_wei)[0];
    let sha = tx.state().hand_cards(attacker)[0];
    tx.state_mut().cards.get_mut(first_shan).unwrap().kind = CardKind::Shan;
    tx.state_mut().cards.get_mut(second_shan).unwrap().kind = CardKind::Shan;
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == HUJIA)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        first_wei,
        second_wei,
        attacker,
        first_shan,
        second_shan,
        sha,
        skill,
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

#[test]
fn package_closes_caocao_with_lord_only_group_response() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let caocao = package
        .character_index
        .iter()
        .find(|character| character.id == 1)
        .unwrap();
    assert_eq!(caocao.skills, [1, HUJIA]);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == HUJIA)
        .unwrap();
    assert!(skill.lord_only);
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == HUJIA)
        .unwrap();
    assert_eq!(view.source_zone.as_deref(), Some("group_wei"));
    let compiled = content_compiler::compile_source_named("standard-hujia", SOURCE).unwrap();
    assert_eq!(compiled.view_as_index, vec![view.clone()]);
}

#[test]
fn wei_players_are_asked_in_seat_order_until_one_provides_shan() {
    let setup = setup();
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let (state, offered) = command(
        setup.state,
        setup.attacker,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        1,
    );
    let hujia_token = offered.choice.as_ref().unwrap().options[0].token;
    assert_eq!(offered.choice.as_ref().unwrap().observer, setup.owner);
    let (state, first) = resolve(state, setup.owner, vec![hujia_token], false, 2);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.first_wei);
    let (state, second) = resolve(state, setup.first_wei, vec![], true, 3);
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.second_wei);
    let (state, saved) = resolve(
        state,
        setup.second_wei,
        vec![setup.second_shan.raw()],
        false,
        4,
    );
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert!(saved.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
    assert!(state
        .hand_cards(setup.first_wei)
        .contains(&setup.first_shan));
    assert_eq!(
        state.cards.get(setup.second_shan).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(saved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardResponded {
            player,
            respond_to,
            card,
            ..
        } if *player == setup.second_wei
            && *respond_to == setup.owner
            && *card == setup.second_shan
    )));
}

#[test]
fn hujia_is_unavailable_when_the_owner_is_not_the_lord() {
    let mut setup = setup();
    setup.state.players.get_mut(setup.owner).unwrap().identity = Identity::Rebel;
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let (state, result) = command(
        setup.state,
        setup.attacker,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        5,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp - 1);
    assert_eq!(state.skills.get(setup.skill).unwrap().use_count, 0);
}
