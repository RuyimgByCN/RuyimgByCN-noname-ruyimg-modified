//! Golden coverage for SP 糜夫人「存嗣」and its derived「勇决」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CUNSI: u32 = 3_184_006_504;
const YONGJUE: u32 = 3_901_337_090;
const MIFUREN: u32 = 2_211_264_628;
const SHAMOKE: u32 = 3_007_668_710;
const GANFUREN: u32 = 3_981_632_700;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
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

#[test]
fn package_publishes_complete_mifuren_and_current_abi() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let mifuren = package
        .character_index
        .iter()
        .find(|character| character.id == MIFUREN)
        .unwrap();
    assert_eq!(mifuren.skills, [2_533_816_770, CUNSI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YONGJUE));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == CUNSI)
        .unwrap();
    assert!(active.once_per_game && active.hand_nonempty && active.face_up);
    assert!(active.targets.as_ref().is_some_and(|targets| {
        !targets.allow_self && targets.male && targets.min == 1 && targets.max == 1
    }));
}

#[test]
fn cunsi_gives_all_hand_grants_yongjue_and_turns_over() {
    let (mut state, players) = started(2_026_072_501);
    let owner = players[0];
    let male = players[1];
    let female = players[2];
    state.players.get_mut(owner).unwrap().character_def = MIFUREN;
    state.players.get_mut(male).unwrap().character_def = SHAMOKE;
    state.players.get_mut(female).unwrap().character_def = GANFUREN;
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, CUNSI);
    let first = inject(&mut state, owner, CardKind::Sha, 91_001);
    let second = inject(&mut state, owner, CardKind::Shan, 91_002);

    let (state, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![female],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![male],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.cards.get(first).unwrap().owner, male);
    assert_eq!(state.cards.get(second).unwrap().owner, male);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert!(state
        .players
        .get(male)
        .unwrap()
        .skills
        .iter()
        .any(|skill| { state.skills.get(*skill).unwrap().def.raw() == YONGJUE }));
}

#[test]
fn yongjue_optionally_gains_another_players_resolved_sha() {
    let (mut state, players) = started(2_026_072_502);
    let owner = players[0];
    let source = players[1];
    let target = players[2];
    state.turn.current = source;
    state.turn.phase = Phase::Play;
    state.players.get_mut(target).unwrap().hp = 10;
    state.players.get_mut(target).unwrap().max_hp = 10;
    add_skill(&mut state, owner, YONGJUE);
    let sha = inject(&mut state, source, CardKind::Sha, 91_101);

    let (state, played) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = played.choice.unwrap();
    assert_eq!(choice.observer, owner);

    let (state, confirmed) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        11,
    );
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, owner);
    assert_eq!(
        state.cards.get(sha).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}
