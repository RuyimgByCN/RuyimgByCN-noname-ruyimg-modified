//! 第一百八十三批：sp 法箓族花色化落地（xinfu_falu 收集 + xinfu_dianhua 点化，
//! ABI 683）。falu：game_started 全 add 四花色标记；cards_moved 弃花色 X 且无
//! X 标记 → add_mark。dianhua：guanxing_top_by_falu_marks 数非零种数观星。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FALU: u32 = 3_540_560_358;
const DIANHUA: u32 = 3_399_270_449;
const FALU_SPADE: u32 = 3_540_560_359;
const FALU_HEART: u32 = 3_540_560_360;
const FALU_CLUB: u32 = 3_540_560_361;
const FALU_DIAMOND: u32 = 3_540_560_362;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(FALU),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DIANHUA),
            },
        ],
    )
    .unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_701),
        kind: CardKind::Sha,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn discard_to(tx: &mut Transaction, player: PlayerId, card: CardId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    tx.state_mut().cards.get_mut(card).unwrap().zone = discard;
    tx.state_mut().zones.get_mut(hand).unwrap().cards.retain(|c| *c != card);
    tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
}

fn mark_count(state: &GameState, player: PlayerId, def: u32) -> u32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(found, _)| *found == def)
                .map(|(_, count)| *count)
        })
        .unwrap_or(0)
        .max(0) as u32
}

#[test]
fn package_publishes_falu_family_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 683);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FALU
            && trigger.event == "game_started"
            && trigger.filter == "owner_alive"
    }));
    for filter in [
        "owner_discard_suit_no_own_mark_spade_falu",
        "owner_discard_suit_no_own_mark_heart_falu",
        "owner_discard_suit_no_own_mark_club_falu",
        "owner_discard_suit_no_own_mark_diamond_falu",
    ] {
        assert!(
            package.trigger_index.iter().any(|trigger| {
                trigger.skill_def_id == FALU && trigger.filter == filter
            }),
            "falu must publish {filter}"
        );
    }
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DIANHUA
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn falu_collects_suit_marks_on_discard() {
    // owner 弃黑桃与红桃各一张：获得 spade/heart 两枚法箓标记。
    let (mut state, owner, _other) = started(2_026_092_301);
    let c1 = {
        let mut tx = Transaction::begin(state.clone());
        let c = inject_hand(&mut tx, owner, Suit::Spade, 6);
        state = tx.commit().state;
        c
    };
    let c2 = {
        let mut tx = Transaction::begin(state.clone());
        let c = inject_hand(&mut tx, owner, Suit::Heart, 7);
        state = tx.commit().state;
        c
    };
    // game_started 已初始补齐四花色标记各 1（法箓①效果）。
    assert_eq!(mark_count(&state, owner, FALU_SPADE), 1);
    assert_eq!(mark_count(&state, owner, FALU_HEART), 1);

    // 弃黑桃牌 → cards_moved 触发 spade 法箓标记。
    {
        let mut tx = Transaction::begin(state.clone());
        discard_to(&mut tx, owner, c1);
        let dispatched = engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardsMoved {
                cards: vec![c1],
                from: state.zones.find(ZoneKind::Hand, owner).unwrap(),
                to: state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap(),
                reason: engine_api::CardMoveReason::Discard,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        );
        match dispatched {
            Ok(requests) => {
                let _ = engine_core::effect::run_effects(&mut tx, requests);
            }
            Err(e) => panic!("dispatch err: {e:?}"),
        }
        state = tx.commit().state;
    }
    assert_eq!(
        mark_count(&state, owner, FALU_SPADE),
        1,
        "spade discard keeps spade mark at 1 (no duplicate)"
    );

    // 弃红桃牌 → heart 标记。
    {
        let mut tx = Transaction::begin(state.clone());
        discard_to(&mut tx, owner, c2);
        let dispatched = engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardsMoved {
                cards: vec![c2],
                from: state.zones.find(ZoneKind::Hand, owner).unwrap(),
                to: state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap(),
                reason: engine_api::CardMoveReason::Discard,
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        );
        match dispatched {
            Ok(requests) => {
                let _ = engine_core::effect::run_effects(&mut tx, requests);
            }
            Err(e) => panic!("dispatch err: {e:?}"),
        }
        state = tx.commit().state;
    }
    assert_eq!(
        mark_count(&state, owner, FALU_HEART),
        1,
        "heart discard grants heart falu mark"
    );
}
