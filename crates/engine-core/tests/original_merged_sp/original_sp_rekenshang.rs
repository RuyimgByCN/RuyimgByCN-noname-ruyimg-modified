//! Golden coverage for SP 马休马铁「垦伤」(rekenshang).
//! Play-phase view-as: at least two hand/equip cards as Sha (uncapped,
//! at_least syntax with max 255 sentinel, 批 298).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REKENSHANG: u32 = 3_074_298_633;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/rekenshang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, Vec<CardId>) {
    let compiled = content_compiler::compile_source_named("rekenshang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_41, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REKENSHANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut cards = Vec::new();
    for (index, kind) in [CardKind::Tao, CardKind::Shan, CardKind::Other]
        .into_iter()
        .enumerate()
    {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(170_800 + index as u32),
            kind,
            suit: Suit::Club,
            rank: 4,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        cards.push(card);
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == REKENSHANG)
        .unwrap();
    (tx.commit().state, owner, other, skill, cards)
}

#[test]
fn compiled_source_and_npack_publish_sha_at_least_two_uncapped() {
    let compiled = content_compiler::compile_source_named("rekenshang", SOURCE).unwrap();
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == REKENSHANG)
        .expect("rekenshang view_as");
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.min_cards, 2);
    assert_eq!(view.max_cards, u8::MAX);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == REKENSHANG)
        .expect("npack rekenshang view_as");
    assert_eq!(pack.as_kind, "sha");
    assert_eq!(pack.min_cards, 2);
    assert_eq!(pack.max_cards, u8::MAX);
}

#[test]
fn conversion_requires_two_cards_minimum() {
    let (state, owner, other, skill, cards) = setup();
    let hp = state.players.get(other).unwrap().hp;
    // 1 张素材 → 低于 at_least 2 下限，拒绝。
    let (_, rejected) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![cards[0]],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    // 2 张素材 → 接受，转换【杀】结算掉血。
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![cards[0], cards[1]],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
}
