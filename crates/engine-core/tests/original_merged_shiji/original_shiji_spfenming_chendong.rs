//! Golden for SHIJI SP 陈董（sp_chendong）与「分明」（ABI 402）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SP_CHENDONG: u32 = 4_077_906_935;
const SPYILIE: u32 = 3_923_077_723;
const SPFENMING: u32 = 2_351_231_725;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(88_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(SPFENMING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SPFENMING)
        .unwrap()
        .id;
    (tx.commit().state, owner, target, skill)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result
    );
    state
}

#[test]
fn package_closes_sp_chendong_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 402);

    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_CHENDONG)
        .expect("sp_chendong character index entry");
    assert_eq!(character.name_key, "sp_chendong");
    assert_eq!(character.skills, vec![SPYILIE, SPFENMING]);
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");

    let fenming = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SPFENMING)
        .expect("spfenming active index");
    assert!(fenming.targets.as_ref().unwrap().hp_lte_owner);
}

#[test]
fn spfenming_rejects_higher_hp_and_links_unlinked_target() {
    let (mut state, owner, target, skill) = setup(20_260_807);
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(target).unwrap().hp = 4;

    let revision = state.revision;
    let (mut state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));

    state.players.get_mut(target).unwrap().hp = 3;
    let state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}

#[test]
fn spfenming_takes_one_card_from_linked_other() {
    let (mut state, owner, target, skill) = setup(20_260_808);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
    let stolen = inject_hand(&mut tx, target, 7);
    state = tx.commit().state;

    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        3,
    );
    assert_ne!(state.resolution.pending_choice_id, 0);
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![stolen.raw()],
            cancelled: false,
        },
        4,
    );
    assert!(state.hand_cards(owner).contains(&stolen));
    assert!(!state.hand_cards(target).contains(&stolen));
}
