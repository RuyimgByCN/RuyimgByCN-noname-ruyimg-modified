//! 第一百八十二批：offline「千军」（hm_guigu hm_qianjun）零新原语近似重写。
//! 上游：限定技，出牌阶段，将装备区所有牌交给一名其他角色，然后你与其交换座位，
//! 你回复 1 点体力，并获得「乱击」。
//! 实现：active once_per_game targets 1 1 other has_equip →
//! give_all_equipment_choose_recover_or_damage(self, target)（xianzhou 同款：
//! 给光装备→owner 选回血或受伤，选回血即上游语义超集）+ add_skill(olluanji)。
//! 近似（登记债）：swapSeat 座次交换未建模（引擎级）；olluanji DSL 占位待连带。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HM_QIANJUN: u32 = 3_418_823_181;
const OLLUANJI_HMQ: u32 = 3_418_823_182;
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
            skill_def: SkillDefId::new(HM_QIANJUN),
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
        def: CardDefId::new(99_601),
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

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == player && s.def == SkillDefId::new(def))
}

#[test]
fn package_publishes_hm_qianjun_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    assert!(package.skill_index.iter().any(|s| s.skill_def_id == HM_QIANJUN));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == OLLUANJI_HMQ));
    let active = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == HM_QIANJUN)
        .expect("hm_qianjun must publish an active entry");
    assert!(active.once_per_game, "limited skill must be once_per_game");
    assert!(active.once_per_game, "limited skill must be once_per_game");
    // 约束面：equip_nonempty 描述 owner 侧（自己装备区非空），与上游
    // filter player.countCards("e") 对齐。
    let _ = active;
}

#[test]
fn hm_qianjun_gives_all_equipment_recovers_and_gains_olluanji() {
    let (state, owner, other) = started(2_026_092_301);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let _e1 = inject_equip(&mut tx, owner, CardKind::Weapon);
    let _e2 = inject_equip(&mut tx, owner, CardKind::Armor);
    let state = tx.commit().state;

    let hp_before = hp_of(&state, owner);
    assert_eq!(equip_count(&state, owner), 2);
    let other_hand = hand_count(&state, other);

    let skill_id = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def == SkillDefId::new(HM_QIANJUN))
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

    // 收牌者（other）选择：cancel = owner 回血（ChooseRecoverOwnerOrDamage 语义）。
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "recipient choice rejected: {:?}",
        result.result
    );

    // 全装备移交 + 自回 1 + 获得「乱击」占位技能。
    assert_eq!(equip_count(&state, owner), 0, "equipment stripped");
    assert_eq!(hand_count(&state, other), other_hand + 2, "target receives both");
    assert_eq!(hp_of(&state, owner), hp_before + 1, "owner recovers one");
    assert!(
        has_skill(&state, owner, OLLUANJI_HMQ),
        "owner must gain olluanji placeholder"
    );
}

