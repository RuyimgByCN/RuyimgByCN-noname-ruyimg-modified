//! Golden for SHIJI SP 花鬘（sp_huaman）与「嬉战」（ABI 403）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SP_HUAMAN: u32 = 2_409_866_600;
const SPXIANGZHEN: u32 = 2_216_375_930;
const SPFANGZONG: u32 = 2_304_282_956;
const SPXIZHAN: u32 = 3_032_288_469;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

fn inject_hand(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(89_000 + u32::from(rank)),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, phase_player) = (players[0], players[1]);
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
        skill_def: SkillDefId::new(SPFANGZONG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SPXIZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = phase_player;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx.commit().state, owner, phase_player)
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
fn package_closes_sp_huaman_with_exact_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 502);

    let character = package
        .character_index
        .iter()
        .find(|c| c.id == SP_HUAMAN)
        .expect("sp_huaman character index entry");
    assert_eq!(character.name_key, "sp_huaman");
    assert_eq!(character.skills, vec![SPXIANGZHEN, SPFANGZONG, SPXIZHAN]);
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SPXIZHAN)
        .expect("spxizhan trigger");
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(!trigger.optional);
}

#[test]
fn spxizhan_empty_selection_loses_one_hp() {
    let (mut state, owner, phase_player) = setup(20_260_809);
    let mut tx = Transaction::begin(state);
    let _card = inject_hand(&mut tx, owner, Suit::Spade, 3);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpxizhan {
            player: owner,
            phase_player,
            fangzong_skill_def: SkillDefId::new(SPFANGZONG),
        }],
    );
    assert!(matches!(
        err,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseSpxizhanDiscardOrLose
    );
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![],
            cancelled: false,
        },
        1,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    let fangzong = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SPFANGZONG)
        .unwrap();
    assert!(!fangzong.turn_disabled);
}

#[test]
fn spxizhan_discard_disables_fangzong() {
    let (mut state, owner, phase_player) = setup(20_260_810);
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner, Suit::Heart, 5);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpxizhan {
            player: owner,
            phase_player,
            fangzong_skill_def: SkillDefId::new(SPFANGZONG),
        }],
    );
    assert!(matches!(
        err,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![card.raw()],
            cancelled: false,
        },
        2,
    );
    let fangzong = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == SPFANGZONG)
        .unwrap();
    assert!(fangzong.turn_disabled);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, discard);
}
