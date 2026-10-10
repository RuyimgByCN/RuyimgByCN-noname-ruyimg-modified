//! 第一百八十批：offline「踏鹤」（psdaohe）——choose_other_give_then_recover
//! 原语（ABI 682）首件。上游：出牌阶段限一次，选一名有手牌的其他角色，其将
//! 任意张手牌交给你，然后其回复 1 点体力。近似：固定交 1 张（登记债）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PSDAOHE: u32 = 2_073_952_815;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

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
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PSDAOHE),
        }],
    )
    .unwrap();
    (tx.commit().state, owner, other)
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_501),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_psdaohe_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == PSDAOHE)
        .expect("psdaohe must publish an active entry");
    let targets = active.targets.as_ref().expect("targets block");
    assert!(targets.hand_nonempty, "must filter hand-nonempty givers");
    assert_eq!((targets.min, targets.max, targets.allow_self), (1, 1, false));
}

#[test]
fn psdaohe_giver_stashes_one_card_and_self_recovers() {
    let (state, owner, giver) = started(2_026_092_301);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let gift = inject_hand(&mut tx, giver, 8);
    let _ = inject_hand(&mut tx, giver, 9);
    // giver 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(giver).unwrap().hp;
    tx.state_mut().players.get_mut(giver).unwrap().hp = hp - 1;
    let state = tx.commit().state;

    let giver_hp_before = hp_of(&state, giver);
    let giver_hand = hand_count(&state, giver);
    let owner_hand = hand_count(&state, owner);

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(PSDAOHE))
        .unwrap()
        .id;
    let revision = state.revision;
    let (mut state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![giver],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        output.result
    );

    // 三段：owner 选 giver（ActivateSkill targets）→ 原语 open_choice owner 选 giver
    // → giver 选牌（选注入牌）。
    assert!(state.has_pending_choice(), "giver choice must open");
    let state = {
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![giver.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "giver pick rejected: {:?}",
            result.result
        );
        next
    };
    assert!(state.has_pending_choice(), "giver card choice must open");
    let state = {
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: giver,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(2),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![gift.raw()],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "giver stash rejected: {:?}",
            result.result
        );
        next
    };

    assert_eq!(hand_count(&state, giver), giver_hand - 1, "giver loses one card");
    assert_eq!(hand_count(&state, owner), owner_hand + 1, "owner receives gift");
    assert_eq!(
        hp_of(&state, giver),
        giver_hp_before + 1,
        "giver recovers one"
    );
}
