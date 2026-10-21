//! Golden coverage for 旧不屈（shenhua 旧周泰，ABI 591）.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const GZBUQU: u32 = 1_234_567_908;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/gzbuqu.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("gzbuqu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GZBUQU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
}

fn alloc_card(
    tx: &mut Transaction,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn set_hp(tx: &mut Transaction, player: PlayerId, hp: i32) {
    tx.state_mut().players.get_mut(player).unwrap().hp = hp;
}

#[test]
fn package_exposes_gzbuqu_contract() {
    let compiled = content_compiler::compile_source_named("gzbuqu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 591);
    assert_eq!(compiled.skill_index.len(), 1);
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&GZBUQU)));
}

#[test]
fn gzbuqu_moves_top_cards_to_chuang_and_marks_nodying_when_ranks_unique() {
    let mut setup = setup();
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top1 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Spade, 5);
    let top2 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Heart, 7);
    set_hp(&mut setup.tx, setup.owner, 0);
    // 触发：体力降至 0 以下（流失 3 点）→ 置入 1 张创牌（牌堆顶 top1，点数 5）。
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 3,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    // 免死标记：唯一点数 → 置位。
    assert!(tx.state().resolution.gzbuqu_nodying);
    let state = tx.commit().state;
    let tx = Transaction::begin(state);
    // 「创」技能牌堆含顶牌。
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner)
        .expect("创 pile must exist");
    let cards = tx.state().zones.get(pile.zone).unwrap().cards.clone();
    // 牌堆顶 = 最后一个分配的牌（top2 在 top1 之后）。
    assert!(cards.contains(&top2), "top card must enter 创");
    assert_eq!(cards.len(), 1, "one 创 card for the loss");
    let _ = top2;
    // 伤害链：免死标记消费后不进入濒死。
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    set_hp(&mut tx, setup.owner, 1);
    tx.state_mut().resolution.gzbuqu_nodying = true;
    let _events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.other,
            target: setup.owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(
        !tx.state().players.get(setup.owner).unwrap().dying,
        "nodying must suppress the dying chain"
    );
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, -1);
}

#[test]
fn gzbuqu_duplicate_rank_does_not_mark_nodying() {
    let mut setup = setup();
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _top1 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Spade, 5);
    let _top2 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Heart, 5);
    set_hp(&mut setup.tx, setup.owner, -1);
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 3,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(
        !tx.state().resolution.gzbuqu_nodying,
        "duplicate ranks must not mark nodying"
    );
}

#[test]
fn gzbuqu_recover_removes_chuang_card() {
    let mut setup = setup();
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _top1 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Spade, 5);
    let _top2 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Heart, 7);
    set_hp(&mut setup.tx, setup.owner, 0);
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 3,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    // 回复 1 点 → 触发恢复移除一张创牌。
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: PlayerId::NONE,
            player: setup.owner,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("run error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = tx.commit().state;
    let tx = Transaction::begin(state);
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner)
        .expect("创 pile must exist");
    assert_eq!(
        tx.state().zones.get(pile.zone).unwrap().cards.len(),
        0,
        "one 创 removed after recovery"
    );
}

#[test]
fn gzbuqu_skips_when_hp_still_positive() {
    let mut setup = setup();
    set_hp(&mut setup.tx, setup.owner, 2);
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(!tx.state().resolution.gzbuqu_nodying);
    assert!(
        tx.state().skill_piles.is_empty(),
        "no 创 cards while hp stays positive"
    );
}

#[test]
fn gzbuqu_distance_query_unaffected() {
    let mut setup = setup();
    let state = setup.tx.commit().state;
    let tx = Transaction::begin(state);
    let mut ctx = engine_core::query::QueryContext::new();
    let distance = engine_core::query::query_explain(
        tx.state(),
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.other,
        },
        &mut ctx,
    );
    assert!(matches!(distance.response, QueryResponse::Int(_)));
}
