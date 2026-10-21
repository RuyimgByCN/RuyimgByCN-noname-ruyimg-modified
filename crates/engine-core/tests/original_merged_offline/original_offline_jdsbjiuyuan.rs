//! 第一百七十三批：offline「救援」（jd_sb_sunquan jdsbjiuyuan）零新原语重写。
//! 上游：出牌阶段限一次，获得一名吴势力角色装备区里的所有牌，然后你回复 1 点体力。
//! 实现：active targets 1 1 other has_gainable_equip → gain_target_equipment 五槽位
//! + recover_hp(self, 1)。
//! 近似（登记债）：吴势力限定与 zhuSkill 主公技面未建模（active 声明无势力过滤）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JDSBJIUYUAN: u32 = 2_733_343_101;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

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
            skill_def: SkillDefId::new(JDSBJIUYUAN),
        }],
    )
    .unwrap();
    // owner 先受 1 伤，使回复可观察。
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp - 1;
    (tx.commit().state, owner, other)
}

fn inject_equip(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_201),
        kind,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: if kind == CardKind::Weapon { 1 } else { 0 },
        zone,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn equip_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
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
fn package_publishes_jdsbjiuyuan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == JDSBJIUYUAN));
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == JDSBJIUYUAN)
        .expect("jdsbjiuyuan must publish an active entry");
    let targets = active.targets.as_ref().expect("targets block");
    assert!(targets.has_gainable_equip, "must filter gainable equipment");
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
}

#[test]
fn jdsbjiuyuan_gains_all_target_equipment_and_self_recovers() {
    let (state, owner, other) = started(2_026_091_801);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let _e1 = inject_equip(&mut tx, other, CardKind::Weapon);
    let _e2 = inject_equip(&mut tx, other, CardKind::Armor);
    let state = tx.commit().state;

    let hp_before = hp_of(&state, owner);
    let owner_hand = hand_count(&state, owner);

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(JDSBJIUYUAN))
        .unwrap()
        .id;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );

    assert_eq!(equip_count(&state, other), 0, "target equipment stripped");
    assert_eq!(
        hand_count(&state, owner),
        owner_hand + 2,
        "owner gains both equipment cards into hand"
    );
    assert_eq!(hp_of(&state, owner), hp_before + 1, "owner recovers one");
}

#[test]
fn jdsbjiuyuan_requires_gainable_equipment() {
    let (state, owner, other) = started(2_026_091_802);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(JDSBJIUYUAN))
        .unwrap()
        .id;
    let revision = state.revision;
    let (_state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![other],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        !matches!(result.result, CommandResult::Accepted { .. }),
        "target without gainable equipment must be rejected"
    );
}
