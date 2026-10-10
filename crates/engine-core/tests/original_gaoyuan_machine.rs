//! 第二百七十一批：gaoyuan 告援状态机行为测试（ABI 720 / tag 744 /
//! Cont 319）。链路：成为【杀】目标（存在持「诤」标的合法转移候选）→
//! 选弃牌（0..1 可取消）→ 选转移目标（source 射程内持诤标者）→ 弃牌 +
//! 转移（弃牌先于转移，与上游一致）。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GAOYUAN: u32 = 3_635_381_901;
const GAOYUAN_SRC: &str =
    include_str!("../../../content/original/ported/mobile/gaoyuan.skill");
const MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const ZHENGJIAN_MARK: u32 = 2_446_053_157;

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    candidate: PlayerId,
    far: PlayerId,
    sha: CardId,
    cost: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone_kind = if kind == CardKind::Weapon {
        ZoneKind::Equip
    } else {
        ZoneKind::Hand
    };
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_100 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let compiled = content_compiler::compile_source_named("gaoyuan-test", GAOYUAN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_913_301, "identity-standard").unwrap();
    let [source, owner, candidate, far]: [PlayerId; 4] =
        tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([source, owner, candidate, far].into_iter().map(|player| {
        EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GAOYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [source, owner, candidate, far] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(&mut tx, owner, CardKind::Other, Suit::Club, 2);
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 3);
    // 候选需在 source 射程内：给 source 一把范围 2 的武器（4 人环座，
    // candidate 与 source 相隔 2）。
    inject(&mut tx, source, CardKind::Weapon, Suit::Diamond, 6);
    Setup {
        state: tx.commit().state,
        source,
        owner,
        candidate,
        far,
        sha,
        cost,
    }
}

fn mark_zhengjian(state: GameState, player: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player,
            mark: MarkDefId::new(ZHENGJIAN_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    tx.commit().state
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

fn play_sha_at_owner(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    submit(
        setup.state.clone(),
        setup.source,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        key,
    )
}

#[test]
fn package_publishes_gaoyuan_zhengjian_marked_filter() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == GAOYUAN)
        .expect("gaoyuan trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_sha_target_can_discard_redirect_to_zhengjian_marked"
    );
}

#[test]
fn discard_then_redirects_sha_to_marked_candidate_in_source_range() {
    let base = setup();
    let marked = mark_zhengjian(base.state, base.candidate);
    let setup = Setup {
        state: marked,
        ..base
    };
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let candidate_hp = setup.state.players.get(setup.candidate).unwrap().hp;
    let far_hp = setup.state.players.get(setup.far).unwrap().hp;

    let (state, cost_choice) = play_sha_at_owner(&setup, 1);
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
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.candidate.raw()],
        "only the marked candidate is offered"
    );

    let (state, redirected) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: target_choice.choice_id,
            selections: vec![setup.candidate.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(
        matches!(redirected.result, CommandResult::Accepted { .. }),
        "redirect rejected: {:?}",
        redirected.result
    );
    // 转移成功：owner 免伤、candidate 掉血、far 不受波及。
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(
        state.players.get(setup.candidate).unwrap().hp,
        candidate_hp - 1
    );
    assert_eq!(state.players.get(setup.far).unwrap().hp, far_hp);
    // 弃牌入弃牌堆。
    assert_eq!(
        state.cards.get(setup.cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(redirected.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            ..
        } if *player == setup.source && *card == setup.sha && *target == setup.candidate
    )));
}

#[test]
fn declining_the_discard_keeps_the_original_sha_target() {
    let base = setup();
    let marked = mark_zhengjian(base.state, base.candidate);
    let setup = Setup {
        state: marked,
        ..base
    };
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let candidate_hp = setup.state.players.get(setup.candidate).unwrap().hp;

    let (mut state, mut outcome) = play_sha_at_owner(&setup, 4);
    // 取消告援（引擎可能对同一 card_targeted 事件重复交付触发，逐个取消）。
    for key in 5..9 {
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
    assert!(matches!(outcome.result, CommandResult::Accepted { .. }));
    // 拒绝告援：owner 掉血，candidate 免伤，cost 仍在手。
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(setup.candidate).unwrap().hp, candidate_hp);
    assert!(state.hand_cards(setup.owner).contains(&setup.cost));
}
