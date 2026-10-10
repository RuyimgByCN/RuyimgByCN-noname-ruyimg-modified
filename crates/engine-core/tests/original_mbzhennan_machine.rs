//! 第二百七十六批：mbzhennan 镇南状态机行为测试（ABI 725 / tag 749 /
//! Cont 323）。链路：成为锦囊目标且目标数>max(1,使用者体力) → 选弃牌
//! （可取消）→ 选任意角色（强制）→ 弃牌 + 对其造成 1 点伤害。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBZHENNANN: u32 = 3_738_117_601;
const MBZHENNANN_SRC: &str =
    include_str!("../../../content/original/ported/mobile/rezhennan.skill");
const MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    third: PlayerId,
    trick: CardId,
    cost: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_600 + u32::from(rank)),
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

/// 三人局：source 使用 3 目标锦囊（c1/c2/c3），owner 是目标之一且体力 1，
/// 满足 目标数 3 > max(1, hp=1) 门槛。
fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let compiled = content_compiler::compile_source_named("mbzhennann-test", MBZHENNANN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_913_801, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (source, owner, third) = (players[0], players[1], players[2]);
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MBZHENNANN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [source, owner, third] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    // source 体力压到 1：任意 ≥2 目标锦囊即过门槛。
    tx.state_mut().players.get_mut(source).unwrap().hp = 1;
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let trick = inject(&mut tx, source, CardKind::TrickNanman, Suit::Spade, 3);
    let cost = inject(&mut tx, owner, CardKind::Other, Suit::Club, 2);
    Setup {
        state: tx.commit().state,
        source,
        owner,
        third,
        trick,
        cost,
    }
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_mbzhennann_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBZHENNANN)
        .expect("mbzhennan trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_is_target_trick_targets_gt_source_hp_hand_equip_nonempty"
    );
}

#[test]
fn discard_then_deals_damage_to_chosen_any_character() {
    let setup = setup();
    let owner_hp = hp_of(&setup.state, setup.owner);
    let third_hp = hp_of(&setup.state, setup.third);

    // source 用 3 目标锦囊（owner/c3/c3 自身外的 third…此处选 owner、third、source）。
    let (state, cost_choice) = submit(
        setup.state.clone(),
        setup.source,
        CommandKind::PlayCard {
            card: setup.trick,
            targets: vec![setup.owner, setup.third],
            card_targets: vec![],
        },
        1,
    );
    eprintln!(
        "DBG play: result={:?} choice={:?} events={:?}",
        cost_choice.result,
        cost_choice.choice.as_ref().map(|c| (c.observer, c.kind)),
        cost_choice.events.iter().map(|e| format!("{e:?}")).take(6).collect::<Vec<_>>()
    );
    let cost_choice = cost_choice.choice.unwrap();
    assert_eq!(cost_choice.observer, setup.owner);
    assert_eq!(cost_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        cost_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.cost.raw()]
    );

    // 选成本牌 → 进入目标选择（任意存活角色，强制）。
    let (state, target_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![setup.cost.raw()],
            cancelled: false,
        },
        2,
    );
    let target_choice = target_choice.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(target_choice.options.len(), 3, "any alive character");

    // 对 third 造成 1 点伤害。
    let (state, done) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: target_choice.choice_id,
            selections: vec![setup.third.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    // 南蛮自身对两目标各 1 伤害；镇南再加 1 点给 third。
    assert_eq!(hp_of(&state, setup.third), third_hp - 2);
    assert_eq!(hp_of(&state, setup.owner), owner_hp - 1);
    // 成本牌入弃牌堆。
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&setup.cost));
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_keeps_targets_unharmed() {
    let setup = setup();
    let owner_hp = hp_of(&setup.state, setup.owner);
    let third_hp = hp_of(&setup.state, setup.third);

    let (state, cost_choice) = submit(
        setup.state.clone(),
        setup.source,
        CommandKind::PlayCard {
            card: setup.trick,
            targets: vec![setup.owner, setup.third],
            card_targets: vec![],
        },
        1,
    );
    let cost_choice = cost_choice.choice.unwrap();
    let (mut state, mut outcome) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![],
            cancelled: true,
        },
        2,
    );
    // 取消后南蛮队列继续，逐个处理（取消）残留响应询问。
    for key in 3..8 {
        match outcome.choice.clone() {
            Some(choice) => {
                let (next, next_outcome) = submit(
                    state,
                    choice.observer,
                    CommandKind::ResolveChoice {
                        choice_id: choice.choice_id,
                        selections: vec![],
                        cancelled: true,
                    },
                    key,
                );
                state = next;
                outcome = next_outcome;
            }
            None => break,
        }
    }
    assert!(!state.has_pending_choice());
    assert_eq!(hp_of(&state, setup.third), third_hp - 1);
    assert_eq!(hp_of(&state, setup.owner), owner_hp - 1);
    // 成本仍在手。
    assert!(state.hand_cards(setup.owner).contains(&setup.cost));
}
