//! Golden for DIY「远筹」：远距普通锦囊不能以拥有者为目标。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const NSYUANCHOU: u32 = 2_147_508_736;
const SOURCE: &str = include_str!("../../../../content/original/ported/diy/nsyuanchou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/diy/diy.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: u32,
    weapon_range: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn target_legal(tx: &Transaction, user: PlayerId, card: CardId, target: PlayerId) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn prepared() -> (
    Transaction,
    PlayerId,
    PlayerId,
    PlayerId,
    CardId,
    CardId,
    CardId,
    CardId,
) {
    let package = content_compiler::compile_source_named("nsyuanchou-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, NSYUANCHOU as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let near = players[1];
    let far = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSYUANCHOU),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    inject(&mut tx, far, ZoneKind::Equip, CardKind::Weapon, 166_001, 3);
    let far_juedou = inject(
        &mut tx,
        far,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        166_002,
        0,
    );
    let near_juedou = inject(
        &mut tx,
        near,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        166_003,
        0,
    );
    let far_le = inject(&mut tx, far, ZoneKind::Hand, CardKind::TrickLe, 166_004, 0);
    let far_sha = inject(&mut tx, far, ZoneKind::Hand, CardKind::Sha, 166_005, 0);
    (
        tx,
        owner,
        near,
        far,
        far_juedou,
        near_juedou,
        far_le,
        far_sha,
    )
}

#[test]
fn package_publishes_abi204_distance_conditional_modifier() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 684);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == NSYUANCHOU
            && modifier.target == "card_target_cancel_if_other_user_distance_gt_one"
            && modifier.selector.as_deref() == Some("ordinary_trick")
            && modifier.value == 1
    }));
}

#[test]
fn only_far_ordinary_tricks_are_cancelled_using_final_distance() {
    let (mut tx, owner, near, far, far_juedou, near_juedou, far_le, far_sha) = prepared();
    assert!(!target_legal(&tx, far, far_juedou, owner));
    assert!(target_legal(&tx, near, near_juedou, owner));
    assert!(target_legal(&tx, far, far_le, owner));
    assert!(target_legal(&tx, far, far_sha, owner));

    inject(
        &mut tx,
        far,
        ZoneKind::Equip,
        CardKind::HorseOff,
        166_006,
        0,
    );
    assert!(target_legal(&tx, far, far_juedou, owner));
}
