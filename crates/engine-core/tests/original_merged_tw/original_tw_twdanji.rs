//! Golden coverage for TW JSP 关羽「单骑」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWDANJI: u32 = 2_211_821_528;
const TWDANJI_EFFECT: u32 = 2_360_461_242;
const TW_JSP_GUANYU: u32 = 4_246_243_945;
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/tw/twdanji.skill"),
    "\nskill test_wusheng(16) { view_as sha play_phase red; }\n",
);
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_357_000 + u32::from(rank)),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_and_first_transformed_sha_draw_are_exact() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 357);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_JSP_GUANYU)
            .unwrap()
            .skills,
        [16, TWDANJI]
    );

    let package = content_compiler::compile_source_named("twdanji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
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
    effects.extend(
        [16, TWDANJI_EFFECT].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == 16)
        .unwrap()
        .id;
    let first = inject(&mut tx, owner, 3);
    let second = inject(&mut tx, owner, 4);
    let before = tx.state().hand_count(owner);

    let use_sha = |state: GameState, card, key| {
        let (state, result) = apply_command(
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
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state
    };

    let mut state = use_sha(tx.commit().state, first, 1);
    assert_eq!(state.hand_count(owner), before);
    state.turn.set_flag(turn_flag::SHA_PLAYED, 0);
    state.turn.set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let state = use_sha(state, second, 2);
    assert_eq!(state.hand_count(owner), before - 1);
}
