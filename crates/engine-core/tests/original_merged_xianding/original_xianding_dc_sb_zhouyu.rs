//! Golden for XIANDING 新杀谋周瑜【融火/英谋】（ABI 602）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBRONGHUO: u32 = 2_240_860_827;
const DCSBYINGMOU: u32 = 4_166_950_825;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/xianding/dc_sb_zhouyu.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dc-sb-zhouyu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
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
        player: players[0],
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn alloc(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_602 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(602),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn ronghuo_permission_is_non_stacking_and_yingmou_can_toggle_at_game_start() {
    let (state, [owner, _, _], skill) = setup(DCSBRONGHUO);
    let mut tx = Transaction::begin(state);
    for _ in 0..2 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhouyu {
                player: owner,
                source: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                event_targets: Vec::new(),
                damaged_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().skills.get(skill).unwrap().content_flags & 1, 1);

    let (state, [owner, _, _], skill) = setup(DCSBYINGMOU);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDcSbZhouyu {
            player: owner,
            source: owner,
            card: CardId::NONE,
            action: 10,
            event_targets: Vec::new(),
            damaged_targets: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skills.get(skill).unwrap().view_as_branch, 1);
}

#[test]
fn yingmou_yin_without_damage_cards_discards_to_owner_then_deals_fire_damage() {
    let (state, [owner, target, source], skill) = setup(DCSBYINGMOU);
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, 1);
    let source_cards = [
        alloc(&mut tx, source, 2),
        alloc(&mut tx, source, 3),
        alloc(&mut tx, source, 4),
    ];
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhouyu {
                player: owner,
                source,
                card: CardId::NONE,
                action: 13,
                event_targets: vec![target],
                damaged_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(
        tx.commit().state,
        source,
        source_cards[..2].iter().map(|card| card.raw()).collect(),
    );
    assert_eq!(state.hand_count(source), state.hand_count(owner));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 0);
    assert!(instance.state_blob.is_empty());
}
