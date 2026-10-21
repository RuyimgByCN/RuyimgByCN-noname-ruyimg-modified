//! Golden for archived extension 玩点论杀 费诗【率言】（ABI 608）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSSHUAIYAN: u32 = 2_978_279_193;
const SOURCE: &str = include_str!("../../../../content/original/ported/wandianlunsha/feishi.skill");

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
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

fn open_shuaiyan(
    tx: &mut Transaction,
    owner: PlayerId,
    recovered: PlayerId,
    skill: engine_api::SkillId,
) {
    assert_eq!(
        run_effects(
            tx,
            vec![EffectRequest::ResolveNsshuaiyan {
                player: owner,
                recovered_player: recovered,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
}

#[test]
fn recovered_player_chooses_owner_draw_or_owner_discards_their_card() {
    let package = content_compiler::compile_source_named("wandianlunsha-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let recovered = players[1];
    tx.state_mut().turn.current = players[2];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSSHUAIYAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == NSSHUAIYAN)
        .unwrap()
        .id;

    let hand_before = tx.state().hand_count(owner);
    open_shuaiyan(&mut tx, owner, recovered, skill);
    let state = choose(tx.commit().state, recovered, vec![0], 6081);
    assert_eq!(state.hand_count(owner), hand_before + 1);

    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, recovered).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_081),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner: recovered,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    open_shuaiyan(&mut tx, owner, recovered, skill);
    let state = choose(tx.commit().state, recovered, vec![1], 6082);
    assert!(state.has_pending_choice());
    let state = choose(state, owner, vec![card.raw()], 6083);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&card));
}
