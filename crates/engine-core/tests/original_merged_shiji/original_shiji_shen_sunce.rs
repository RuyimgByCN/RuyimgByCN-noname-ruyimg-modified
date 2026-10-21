//! Package closure for SHIJI 神孙策, rewritten from legacy JavaScript data.

const SHEN_SUNCE: u32 = 2_662_599_370;
const YINGBA: u32 = 2_187_983_153;
const SCFUHAI: u32 = 3_159_082_317;
const PINGHE: u32 = 2_176_317_369;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

#[test]
fn package_preserves_character_stats_and_skill_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_SUNCE)
        .unwrap();
    assert_eq!((character.initial_hp, character.max_hp), (1, 6));
    assert_eq!(character.skills, [YINGBA, SCFUHAI, PINGHE]);
}
// 第一百四十二批：覆海晋升覆盖——通用 `make_current_card_unrespondable`
// （natuxing 同款）使锦囊目标直击生效；此前 sha 专用近似只覆盖【杀】。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YINGBA_MARK: u32 = 3_219_705_664;

fn started() -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shiji-scfuhai", SCFUHAI_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_842, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let clear: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, clear).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCFUHAI),
        }],
    )
    .unwrap();
    tx.state_mut()
        .players
        .get_mut(other)
        .unwrap()
        .marks
        .push((YINGBA_MARK, 1));
    (tx.commit().state, owner, other)
}

const SCFUHAI_SOURCE: &str =
    include_str!("../../../../content/original/ported/shiji/scfuhai.skill");

fn inject(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: engine_api::Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn drive(tx: &mut Transaction, events: &[engine_api::DomainEvent]) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

#[test]
fn scfuhai_direct_hit_applies_to_trick_targeting_marked_player() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    let juedou = inject(
        &mut tx,
        owner,
        CardKind::TrickJuedou,
        engine_api::Suit::Club,
    );
    let before = tx.state().hand_count(owner);
    // optional 触发开出 confirm 选择：先驱动出选择，再以 ResolveChoice 确认。
    let _ = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: juedou,
            targets: vec![other],
        }],
    );
    assert!(tx.state().has_pending_choice(), "optional scfuhai asks");
    let committed = tx.commit();
    let (settled, _pipeline) = apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(9_001),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(committed.state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    // 覆海摸 1（每回合限次的近似：不限次）。
    assert_eq!(settled.hand_count(owner), before + 1);
    let sid = settled
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SCFUHAI)
        .unwrap()
        .id;
    let policy = settled
        .skills
        .get(sid)
        .map(|s| s.card_use_policy(juedou))
        .unwrap_or(0);
    assert_ne!(
        policy & 2,
        0,
        "trick targeting marked player is unrespondable"
    );
}

#[test]
fn scfuhai_stays_quiet_without_mark() {
    let (state, owner, _other) = started();
    let mut tx = Transaction::begin(state);
    // 清掉目标标记：换用无标记第三场景——直接驱一张无目标牌。
    let sha = inject(&mut tx, owner, CardKind::Sha, engine_api::Suit::Club);
    let before = tx.state().hand_count(owner);
    drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before, "no mark: no draw");
}
