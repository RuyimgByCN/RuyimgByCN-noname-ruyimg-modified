//! Golden coverage for KEY 弄幻 / 义父 / 振刃 / 海德洛 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MIDORI_NONGHUAN: u32 = 4_093_922_167;
const AKANE_YIFU2: u32 = 3_457_576_296;
const LUCIA_ZHENREN: u32 = 3_777_419_907;
const MIKI_HYDRO: u32 = 3_782_739_325;

const NONGHUAN_SRC: &str =
    include_str!("../../../../content/original/ported/key/midori_nonghuan.skill");
const YIFU_SRC: &str = include_str!("../../../../content/original/ported/key/akane_yifu2.skill");
const ZHENREN_SRC: &str =
    include_str!("../../../../content/original/ported/key/lucia_zhenren.skill");
const HYDRO_SRC: &str =
    include_str!("../../../../content/original/ported/key/miki_hydrogladiator_skill.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 6,
        weapon_range: u8::from(zone_kind == ZoneKind::Equip) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(tx: &Transaction, owner: PlayerId, skill_def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card_targets: Vec<CardId>,
    key: u64,
) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn nonghuan_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("midori_nonghuan", NONGHUAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MIDORI_NONGHUAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MIDORI_NONGHUAN));
}

#[test]
fn yifu_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("akane_yifu2", YIFU_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_YIFU2));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_YIFU2));
}

#[test]
fn yifu_discards_one_and_lets_other_draw() {
    let (mut tx, owner, other) = bootstrap(YIFU_SRC, "yifu-test", AKANE_YIFU2);
    let card = inject_card(&mut tx, owner, ZoneKind::Hand, CardKind::Shan, 345_101);
    let skill = skill_id(&tx, owner, AKANE_YIFU2);
    let other_hand = tx.state().hand_count(other);
    let state = activate(tx.commit().state, owner, skill, vec![other], vec![card], 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn zhenren_source_and_npack_publish_phase_end() {
    let compiled = content_compiler::compile_source_named("lucia_zhenren", ZHENREN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LUCIA_ZHENREN)
        .expect("zhenren trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == LUCIA_ZHENREN && trigger.event == "phase_end" }));
}

#[test]
fn zhenren_end_phase_discards_own_equipment() {
    let (mut tx, owner, other) = bootstrap(ZHENREN_SRC, "zhenren-test", LUCIA_ZHENREN);
    inject_card(&mut tx, owner, ZoneKind::Equip, CardKind::Weapon, 377_101);
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    assert_eq!(tx.state().zones.get(equip).unwrap().cards.len(), 1);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("phase-end dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert!(tx.state().zones.get(equip).unwrap().cards.is_empty());
}

#[test]
fn hydro_source_and_npack_publish_sha_damage() {
    let compiled =
        content_compiler::compile_source_named("miki_hydrogladiator_skill", HYDRO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIKI_HYDRO)
        .expect("hydro trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == MIKI_HYDRO && trigger.event == "damage" }));
}
