//! Golden coverage for TW 旧全琮「振赡」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ZHENSHAN: u32 = 2_898_394_319;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/zhenshan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

fn command(actor: PlayerId, key: u64, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind,
    }
}

#[test]
fn zhenshan_swaps_with_lower_hand_target_then_obeys_turn_limit() {
    let package = content_compiler::compile_source_named("tw-zhenshan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 383);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| { character.id == 3_552_851_534 && character.skills == [ZHENSHAN] }));
    let mut engine = Engine::new();
    engine.submit(command(
        PlayerId::NONE,
        1,
        CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_080_124,
            ruleset: "identity-standard".into(),
        },
    ));
    engine.load_package(&package).unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHENSHAN),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let extra = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_840),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(extra);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZHENSHAN)
        .unwrap();
    let owner_cards = tx.state().hand_cards(owner);
    let target_cards = tx.state().hand_cards(target);
    let state = tx.commit().state;

    let (state, result) = apply_command(
        state,
        command(
            owner,
            2,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(state.has_pending_choice());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        command(
            owner,
            3,
            CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        ),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(owner), target_cards);
    assert_eq!(state.hand_cards(target), owner_cards);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);

    let (_, result) = apply_command(
        state,
        command(
            owner,
            4,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(!matches!(result.result, CommandResult::Accepted { .. }));
}
