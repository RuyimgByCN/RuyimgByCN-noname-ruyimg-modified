//! Golden coverage for TW 卞夫人「挽危/约俭」.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWWANWEI: u32 = 3_733_856_593;
const TWWANWEI_EFFECT: u32 = 3_469_371_143;
const TWYUEJIAN: u32 = 3_308_823_251;
const TWYUEJIAN_EFFECT: u32 = 3_469_371_145;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twbianfuren.skill");

fn inject(
    tx: &mut Transaction,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_376_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn wanwei_prevents_and_uses_bottom_while_yuejian_grants_tiered_rewards() {
    let package = content_compiler::compile_source_named("tw-bianfuren-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 376);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_117, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [TWWANWEI, TWYUEJIAN].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };

    let wanwei = skill(&tx, TWWANWEI);
    tx.state_mut().resolution.current_damage_id = 77;
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwbianfuren {
            player: owner,
            target,
            helper_skill_def: TWWANWEI_EFFECT,
            action: 0,
            stage: 0,
            cards: Vec::new(),
            skill: wanwei,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert!(tx.state().resolution.prevented_damage_ids.contains(&77));

    let yuejian = skill(&tx, TWYUEJIAN);
    let cards = [
        inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 3),
        inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 4),
        inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 5),
    ];
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwbianfuren {
            player: owner,
            target: owner,
            helper_skill_def: TWYUEJIAN_EFFECT,
            action: 2,
            stage: 1,
            cards: cards.to_vec(),
            skill: yuejian,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .contains(&(TWYUEJIAN_EFFECT, 1)));

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWWANWEI_EFFECT),
        }],
    )
    .unwrap();
    let helper = skill(&tx, TWWANWEI_EFFECT);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    tx.state_mut().zones.get_mut(draw).unwrap().cards.clear();
    let bottom = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Tao,
        1,
    );
    let top = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Sha,
        2,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwbianfuren {
            player: owner,
            target: PlayerId::NONE,
            helper_skill_def: 0,
            action: 1,
            stage: 0,
            cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&top));
    assert_eq!(
        tx.state().cards.get(bottom).unwrap().zone,
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
