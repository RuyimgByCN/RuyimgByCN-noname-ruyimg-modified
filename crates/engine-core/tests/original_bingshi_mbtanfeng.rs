//! Golden tests for BINGSHI「探锋」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{move_one_card, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{turn_flag, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBTANFENG: u32 = 4_191_284_417;
const XIAHOUSHANG: u32 = 2_359_253_130;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/bingshi/guoyuan.skill"),
    include_str!("../../../content/original/ported/bingshi/kunfen_pot_weiyan.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_chenzhi.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_huangzu.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_luyusheng.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_sunjun.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_tianfeng.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_yanghong.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_zhangyan.skill"),
    include_str!("../../../content/original/ported/bingshi/mbtanfeng.skill"),
    include_str!("../../../content/original/ported/bingshi/pangxi.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_caozhen.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chendao.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chengpu.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chenjiao.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chenqun.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_dengai.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_dongzhao.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_huanjie.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_lougui.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_lusu.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_sunchen.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_xiahouba.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_zanghong.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_zhangren.skill"),
    include_str!("../../../content/original/ported/bingshi/sunshao.skill")
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn started_state(seed: u64) -> GameState {
    let mut engine = Engine::new();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine.state().clone()
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

fn start_effect(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player: owner,
            max_discard: 2,
            damage: 1,
            selected: 0,
            step: 0,
            target: PlayerId::NONE,
        }],
    )
    .map(|_| ())
}

#[test]
fn package_contains_complete_xiahoushang() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBTANFENG));
    // 33 名上游武将 + ABI 521 孙韶（敢决/筑墼，上游 bingshi/skill.js 定义、角色条目
    // 随 ABI 521 落地）＝ 34。
    assert_eq!(committed.character_index.len(), 36);
    let xiahoushang = committed
        .character_index
        .iter()
        .find(|character| character.id == XIAHOUSHANG)
        .unwrap();
    assert_eq!(xiahoushang.skills, vec![MBTANFENG]);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.name_key == "sunshao"));
}

#[test]
fn both_branches_discard_hand_and_equip_deal_fire_damage_and_skip_phases() {
    let state = started_state(2_026_071_504);
    let owner = state.turn.current;
    let targets: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let discard_target = targets[0];
    let damage_target = targets[1];
    let mut tx = Transaction::begin(state);

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, discard_target)
        .unwrap();
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, discard_target)
        .unwrap();
    let equip_card = tx.state().hand_cards(discard_target)[0];
    tx.state_mut().cards.get_mut(equip_card).unwrap().kind = CardKind::Weapon;
    move_one_card(&mut tx, equip_card, hand, equip, discard_target, true).unwrap();
    let hand_card = tx.state().hand_cards(discard_target)[0];

    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOrFireDamageSkipPhases
    );
    let state = tx.commit().state;

    let (cancelled_state, cancelled) = resolve(state.clone(), Vec::new(), true);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!cancelled_state.has_pending_choice());
    assert_eq!(cancelled_state.turn.flag(turn_flag::SKIP_DRAW), 0);
    assert_eq!(cancelled_state.turn.flag(turn_flag::SKIP_PLAY), 0);

    let before_forgery = canonical_state_hash(&state);
    let (unchanged, rejected) = resolve(state.clone(), vec![3], false);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forgery);

    let (state, _) = resolve(state, vec![1, 2], false);
    let (state, _) = resolve(state, vec![discard_target.raw()], false);
    let (state, discarded) = resolve(state, vec![hand_card.raw(), equip_card.raw()], false);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SKIP_DRAW), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardOrFireDamageSkipPhases
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(hand_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(equip_card).unwrap().zone, discard);

    let hp = state.players.get(damage_target).unwrap().hp;
    let (state, damaged) = resolve(state, vec![damage_target.raw()], false);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(damage_target).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
    assert!(damaged.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            nature: 1,
            ..
        } if *source == owner && *target == damage_target
    )));
}
