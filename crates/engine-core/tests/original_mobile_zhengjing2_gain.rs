//! 第二百零五批：gain_whole 消费池散批——zhengjing2（正争·终段）行为测试。
//! 实现：gain_whole_skill_pile(self, self_skill, 自身堆) + skip_next_judge_phase
//! (self) + remove_skill_by_def(self)。「争」堆供给依赖 zhengjing 置入面落地
//! （dormant 契约）；跳过摸牌阶段为登记债（无 draw-skip 原语）。

use engine_api::{
    CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHENGJING2: u32 = 4_211_882_480;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/zhengjing2.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("zhengjing2-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHENGJING2),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand_card(state: GameState, owner: PlayerId, def: u32) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, card)
}

fn fill_pile(state: GameState, owner: PlayerId, n: usize) -> GameState {
    let mut state = state;
    for i in 0..n {
        let (s2, card) = inject_hand_card(state, owner, 9_600 + i as u32);
        state = s2;
        let mut tx = Transaction::begin(state);
        let existing = tx
            .state()
            .skill_piles
            .iter()
            .find(|p| p.owner == owner && p.skill_def == ZHENGJING2)
            .map(|p| p.zone);
        let zone = match existing {
            Some(zone) => zone,
            None => {
                let zone = tx.state_mut().zones.alloc(Zone {
                    id: engine_api::ZoneId::NONE,
                    kind: ZoneKind::Expansion,
                    owner,
                    cards: Vec::new(),
                });
                tx.state_mut()
                    .skill_piles
                    .push(engine_core::state::SkillPileRef {
                        owner,
                        skill_def: ZHENGJING2,
                        zone,
                        usable_as_hand: false,
                        wants_lost_count: false,
                    });
                zone
            }
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = zone;
        }
        state = tx.commit().state;
    }
    state
}

fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let outcome = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Ok(effects) => engine_core::effect::run_effects(&mut tx, effects).map(|_| ()),
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, outcome)
}

#[test]
fn zhengjing2_gains_pile_and_removes_itself() {
    let (state, owner, _other) = setup(20_260_917_01);
    let state = fill_pile(state, owner, 3);
    let hand_before = state.hand_count(owner);
    let max_before = state.players.get(owner).unwrap().max_hp;

    let (state, outcome) = fire_prepare(state, owner);
    assert!(outcome.is_ok(), "forced effects, got {outcome:?}");
    assert_eq!(
        state.hand_count(owner),
        hand_before + 3,
        "pile cards moved into hand"
    );
    assert_eq!(
        state.skill_pile_cards(owner, ZHENGJING2).len(),
        0,
        "pile drained"
    );
    assert!(
        !state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == owner && skill.def.raw() == ZHENGJING2),
        "skill removed itself"
    );
    let _ = max_before;
}
