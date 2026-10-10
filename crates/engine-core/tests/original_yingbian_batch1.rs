//! Golden coverage for yingbian 包批次一（ABI 567 门控）：才媛/崇信/新婉仪 + 全包契约。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CAIYUAN: u32 = 4_141_410_001;
const CHONGXIN: u32 = 4_141_410_002;
const XINWANYI: u32 = 4_141_410_003;
const QIANGSHOU: u32 = 3_061_185_460;
const YBZHUIJI: u32 = 3_463_543_744;

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .map(|zone| {
            state
                .zones
                .get(zone)
                .map(|z| z.cards.len() as u32)
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
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
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(source: &str, defs: &[u32]) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("yingbian-batch1", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
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
    for def in defs {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, other)
}

fn find_skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
        .copied()
        .unwrap()
}

#[test]
fn package_exposes_yingbian_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/yingbian/yingbian.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.character_index.len(), 23);
    for def in [CAIYUAN, CHONGXIN, XINWANYI, QIANGSHOU, YBZHUIJI] {
        assert!(
            committed
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == def),
            "skill {def} must exist"
        );
    }
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&CAIYUAN)));
    // 晋羊祜 lives in the SP package; its skills (huaiyuan/chongxin/dezhang)
    // are ported into yingbian and referenced cross-package.
    let sp = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/sp/sp.npack.json"
    ))
    .unwrap();
    assert!(sp
        .character_index
        .iter()
        .any(|character| character.skills.contains(&CHONGXIN)));
    assert!(committed.character_index.iter().any(|character| {
        character.name_key == "cheliji" && character.skills.contains(&QIANGSHOU)
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.name_key == "xuangongzhu" && character.skills.contains(&YBZHUIJI)
    }));
}

#[test]
fn caiyuan_draws_two_at_end_phase() {
    let source = include_str!("../../../content/original/ported/yingbian/caiyuan.skill");
    let (mut tx, owner, _other) = setup(source, &[CAIYUAN]);
    let before = hand_count(&tx.state(), owner);
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut state = match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx.commit().state,
            Err(EffectError::ChoiceRequested) => tx.commit().state,
            Err(e) => panic!("run error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    if state.has_pending_choice() {
        let pending = state.resolution.pending_choice_id;
        let observer = PlayerId::new(state.resolution.pending_observer);
        let (state2, result) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5670),
                kind: CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(pending),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "caiyuan confirm: {result:?}"
        );
        state = state2;
    }
    assert_eq!(hand_count(&state, owner), before + 2, "caiyuan draws two");
}

#[test]
fn chongxin_activates_and_recasts_selected() {
    let source = include_str!("../../../content/original/ported/yingbian/chongxin.skill");
    let (mut tx, owner, _other) = setup(source, &[CHONGXIN]);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = alloc_card(&mut tx, owner, hand, CardKind::Sha, 1, Suit::Club, 4);
    let skill = find_skill(&mut tx, owner, CHONGXIN);
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0x5671),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(result.result, engine_api::CommandResult::Accepted { .. }),
        "chongxin must activate: {result:?}"
    );
    // 重铸：牌进入弃牌堆（替换为摸一张）。
    assert!(
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .is_some_and(|zone| state
                .zones
                .get(zone)
                .is_some_and(|z| z.cards.contains(&card))),
        "recasted card goes to the discard pile"
    );
    assert_eq!(
        hand_count(&state, owner),
        1,
        "recast draws a replacement card"
    );
}

#[test]
fn xinwanyi_stores_card_from_other() {
    let source = include_str!("../../../content/original/ported/yingbian/xinwanyi.skill");
    let (mut tx, owner, other) = setup(source, &[XINWANYI]);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let card = alloc_card(&mut tx, other, other_hand, CardKind::Sha, 1, Suit::Spade, 5);
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: other,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut state = match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx.commit().state,
            Err(EffectError::ChoiceRequested) => tx.commit().state,
            Err(e) => panic!("run error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    // 确认后进入选牌选择。
    if state.has_pending_choice() {
        let pending = state.resolution.pending_choice_id;
        let observer = PlayerId::new(state.resolution.pending_observer);
        let (state2, result) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5672),
                kind: CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(pending),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "xinwanyi confirm: {result:?}"
        );
        state = state2;
    }
    // 再处理选牌选择（储入技能牌堆）。
    if state.has_pending_choice() {
        let pending = state.resolution.pending_choice_id;
        let observer = PlayerId::new(state.resolution.pending_observer);
        let (state2, result) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5673),
                kind: CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(pending),
                    selections: vec![card.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "xinwanyi card choice: {result:?}"
        );
        state = state2;
    }
    // 牌已从 other 手牌移出并进入拥有者的技能牌堆。
    assert!(
        !state
            .zones
            .find(ZoneKind::Hand, other)
            .is_some_and(|zone| state
                .zones
                .get(zone)
                .is_some_and(|z| z.cards.contains(&card))),
        "card leaves the other's hand"
    );
    assert!(
        state.skill_piles.iter().any(|pile| pile.owner == owner
            && state
                .zones
                .get(pile.zone)
                .is_some_and(|z| z.cards.contains(&card))),
        "card enters the owner's skill pile"
    );
}
