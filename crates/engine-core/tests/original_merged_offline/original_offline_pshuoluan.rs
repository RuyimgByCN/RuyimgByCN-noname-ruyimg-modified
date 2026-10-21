//! Golden for OFFLINE「惑乱」的双向群/蜀伤害修正。

use content_schema::PackageCharacterEntry;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PSHUOLUAN: u32 = 3_905_746_106;
const QUN_CHARACTER: u32 = 9_081_001;
const SHU_CHARACTER: u32 = 9_081_002;
const WEI_CHARACTER: u32 = 9_081_003;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("pshuoluan_{group}"),
        max_hp: 10,
        initial_hp: 10,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    package.character_index.extend([
        character(QUN_CHARACTER, "qun"),
        character(SHU_CHARACTER, "shu"),
        character(WEI_CHARACTER, "wei"),
    ]);
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_722),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_722,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let (owner, shu, wei) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PSHUOLUAN),
    });
    run_effects(&mut tx, effects).unwrap();

    for (player, character) in [
        (owner, QUN_CHARACTER),
        (shu, SHU_CHARACTER),
        (wei, WEI_CHARACTER),
    ] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.character_def = character;
        participant.hp = 10;
        participant.max_hp = 10;
    }
    (tx, owner, shu, wei)
}

fn damage(tx: &mut Transaction, source: PlayerId, target: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
}

fn hp(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state().players.get(player).unwrap().hp
}

#[test]
fn package_publishes_exact_bidirectional_modifier_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == PSHUOLUAN)
        .expect("pshuoluan must be published");
    assert_eq!(skill.name, "pshuoluan");
    assert!(skill.locked);

    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == PSHUOLUAN)
        .collect();
    assert_eq!(modifiers.len(), 2);
    assert!(modifiers.iter().any(|modifier| {
        modifier.target == "damage_amount"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_qun_other_shu")
    }));
    assert!(modifiers.iter().any(|modifier| {
        modifier.target == "damage_taken_amount"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_qun_other_shu")
    }));
}

#[test]
fn qun_owner_and_shu_other_increase_damage_in_both_directions() {
    let (mut outgoing, owner, shu, _) = setup();
    damage(&mut outgoing, owner, shu);
    assert_eq!(hp(&outgoing, shu), 8);

    let (mut incoming, owner, shu, _) = setup();
    damage(&mut incoming, shu, owner);
    assert_eq!(hp(&incoming, owner), 8);
}

#[test]
fn other_groups_and_unrelated_damage_are_not_increased() {
    let (mut qun_to_wei, owner, _, wei) = setup();
    damage(&mut qun_to_wei, owner, wei);
    assert_eq!(hp(&qun_to_wei, wei), 9);

    let (mut wei_to_qun, owner, _, wei) = setup();
    damage(&mut wei_to_qun, wei, owner);
    assert_eq!(hp(&wei_to_qun, owner), 9);

    let (mut unrelated, _, shu, wei) = setup();
    damage(&mut unrelated, shu, wei);
    assert_eq!(hp(&unrelated, wei), 9);
}
