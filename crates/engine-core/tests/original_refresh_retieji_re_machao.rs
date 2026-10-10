//! Golden coverage for 界马超「铁骑」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_MACHAO: u32 = 4_268_725_118;
const RETIEJI: u32 = 3_731_623_167;
const TARGET_OPEN: u32 = 90_100;
const TARGET_LOCKED: u32 = 90_101;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/retieji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    shan: CardId,
    matching: Option<CardId>,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, with_matching_card: bool) -> Setup {
    let source = format!(
        "{SOURCE}\n\
         skill target_open({TARGET_OPEN}) {{ modifier hand_limit add 1; }}\n\
         skill target_locked({TARGET_LOCKED}) locked {{ modifier hand_limit add 1; }}"
    );
    let package = content_compiler::compile_source_named("retieji-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RETIEJI),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(TARGET_OPEN),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(TARGET_LOCKED),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        2,
    );
    let matching = with_matching_card.then(|| {
        inject(
            &mut tx,
            target,
            ZoneKind::Equip,
            CardKind::Weapon,
            Suit::Heart,
            6,
        )
    });
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Heart,
        7,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    Setup {
        state: tx.commit().state,
        owner,
        target,
        sha,
        shan,
        matching,
    }
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

fn play_and_accept(setup: Setup) -> (GameState, PipelineResult, Setup) {
    let (state, offered) = submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    let (state, result) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        2,
    );
    (state, result, setup)
}

fn skill_enabled(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state.skills.get(*skill).is_some_and(|instance| {
                instance.def.raw() == def && state.skill_effectively_enabled(instance)
            })
        })
}

#[test]
fn package_publishes_complete_re_machao_and_abi316_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 316);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_MACHAO)
        .unwrap();
    assert_eq!(character.skills, [2, RETIEJI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RETIEJI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("retieji", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 316);
    let expected = compiled.modules.first().unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn matching_suit_card_is_given_and_avoids_the_sha_penalty() {
    let setup = setup(2_026_073_201, true);
    let owner = setup.owner;
    let target = setup.target;
    let matching = setup.matching.unwrap();
    let (state, result, setup) = play_and_accept(setup);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![matching.raw()]
    );
    assert!(!skill_enabled(&state, target, TARGET_OPEN));
    assert!(skill_enabled(&state, target, TARGET_LOCKED));

    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![matching.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(state.hand_cards(owner).contains(&matching));
    let shan_choice = result.choice.unwrap();
    assert!(shan_choice
        .options
        .iter()
        .any(|option| option.token == setup.shan.raw()));
    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: shan_choice.choice_id,
            selections: vec![setup.shan.raw()],
            cancelled: false,
        },
        4,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);

    let mut state = state;
    for key in 5..8 {
        (state, _) = submit(state, owner, CommandKind::EndPhase, key);
    }
    assert!(skill_enabled(&state, target, TARGET_OPEN));
}

#[test]
fn declining_the_gift_makes_only_this_sha_unrespondable_and_adds_one_damage() {
    let setup = setup(2_026_073_202, true);
    let target = setup.target;
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, result, _) = play_and_accept(setup);
    let choice = result.choice.unwrap();
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: Vec::new(),
            cancelled: true,
        },
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}

#[test]
fn missing_same_suit_card_applies_the_penalty_without_opening_a_gift_choice() {
    let setup = setup(2_026_073_203, false);
    let target = setup.target;
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, result, setup) = play_and_accept(setup);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert!(state.hand_cards(target).contains(&setup.shan));
}
