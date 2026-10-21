//! Golden coverage for mobile 侠怀（部分）+ 破降 upgrades (2026-08-21 placeholder
//! batch follow-up). Zero new ABI (mobile 500).

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBXIAHUI: u32 = 3_915_506_793;
const POXIANG: u32 = 3_023_488_106;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

const XIAHUI_SRC: &str = include_str!("../../../../content/original/ported/mobile/mbxiahui.skill");
const POXIANG_SRC: &str = include_str!("../../../../content/original/ported/mobile/poxiang.skill");

fn bootstrap(source: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId, SkillId) {
    let compiled = content_compiler::compile_source_named("upgrade-src", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx, owner, other, skill)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn xiahui_publishes_black_ignored_hand_limit() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == MBXIAHUI
            && modifier.target == "card_ignored_hand_limit"
            && modifier.selector.as_deref() == Some("black")
            && modifier.value == 1
    }));
    let compiled = content_compiler::compile_source_named("mbxiahui", XIAHUI_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == MBXIAHUI && modifier.target == "card_ignored_hand_limit"
    }));
}

#[test]
fn poxiang_gives_draws_and_loses_hp_in_order() {
    let (mut tx, owner, other, skill) = bootstrap(POXIANG_SRC, POXIANG);
    let gift = inject_hand(&mut tx, owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let revision = tx.state().revision;
    let (state, pipe) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(7),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    if !matches!(pipe.result, CommandResult::Accepted { .. }) {
        panic!("activate rejected: {:?}", pipe.result);
    }
    let mut state = state;
    // 选择链固定两步：选目标（other）→ 选要交出的牌（gift）。
    let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
    state = resolve(state, actor, vec![u32::from(other.raw())], 10);
    let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
    state = resolve(state, actor, vec![u32::from(gift.raw())], 11);
    assert_eq!(
        state.resolution.pending_choice_id, 0,
        "no further choices expected"
    );
    let hand_other = state.zones.find(ZoneKind::Hand, other).unwrap();
    assert!(
        state.zones.get(hand_other).unwrap().cards.contains(&gift),
        "gift card should have moved to other's hand"
    );
    assert_eq!(state.hand_count(owner), 3, "should draw exactly three");
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
}
