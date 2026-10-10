//! Golden coverage for COLLAB「卯兔」and the complete ol_maotu closure.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OLMAOTU: u32 = 2_889_501_326;
const OLMAOTU_EFFECT: u32 = 3_563_166_419;
const OL_MAOTU: u32 = 4_111_734_089;
const SOURCE: &str = include_str!("../../../content/original/ported/collab/olmaotu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn inject_juedou(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::TrickJuedou,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("olmaotu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, OLMAOTU as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let higher = players[1];
    let lower = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLMAOTU),
    });
    effects.push(EffectRequest::SetTurnState {
        current: higher,
        phase: Phase::Play.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    tx.state_mut().players.get_mut(higher).unwrap().hp = 3;
    tx.state_mut().players.get_mut(lower).unwrap().hp = 1;
    let high_card = inject_juedou(&mut tx, higher, 107_001);
    let low_card = inject_juedou(&mut tx, lower, 107_002);
    (tx, owner, higher, lower, high_card, low_card)
}

#[test]
fn package_publishes_abi107_helper_modifier_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLMAOTU && skill.name == "olmaotu"));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == OLMAOTU_EFFECT && skill.name == "olmaotu_effect" }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLMAOTU_EFFECT
            && modifier.target == "card_target_cancel_if_other_user_hp_gte_target"
            && modifier.value == 1
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_MAOTU
            && character.skills == vec![OLMAOTU]
            && character.max_hp == 3
            && character.sex == "female"
            && character.group == "qun"
    }));
}

#[test]
fn leaving_dying_grants_dynamic_hp_based_target_protection() {
    let (mut tx, owner, higher, lower, high_card, low_card) = isolated();
    assert!(target_legal(&tx, higher, high_card, owner));
    assert!(target_legal(&tx, lower, low_card, owner));

    {
        let player = tx.state_mut().players.get_mut(lower).unwrap();
        player.hp = 0;
        player.dying = true;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: lower,
            player: lower,
            amount: 1,
        }],
    )
    .unwrap();

    assert!(has_skill(tx.state(), owner, OLMAOTU_EFFECT));
    assert!(!target_legal(&tx, higher, high_card, owner));
    assert!(target_legal(&tx, lower, low_card, owner));

    tx.state_mut().players.get_mut(higher).unwrap().hp = 1;
    assert!(target_legal(&tx, higher, high_card, owner));
    tx.state_mut().players.get_mut(lower).unwrap().hp = 2;
    assert!(!target_legal(&tx, lower, low_card, owner));
}

#[test]
fn protection_survives_other_turns_and_expires_at_the_owners_next_turn_start() {
    let (mut tx, owner, higher, lower, high_card, _) = isolated();
    {
        let player = tx.state_mut().players.get_mut(lower).unwrap();
        player.hp = 0;
        player.dying = true;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: lower,
            player: lower,
            amount: 1,
        }],
    )
    .unwrap();
    assert!(!target_legal(&tx, higher, high_card, owner));

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: higher,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();

    assert!(!has_skill(tx.state(), owner, OLMAOTU_EFFECT));
    assert!(target_legal(&tx, higher, high_card, owner));
}
