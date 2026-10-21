//! Golden coverage for TW 孙翊「躁厉」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZAOLI: u32 = 2_591_520_200;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twsunyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

fn inject(
    tx: &mut Transaction,
    zone_kind: ZoneKind,
    owner: PlayerId,
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
        weapon_range: 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn zaoli_allows_only_new_hand_cards_and_refills_three_equipment_slots() {
    let package = content_compiler::compile_source_named("tw-sunyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 380);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| { character.id == 3_013_394_897 && character.skills == [TWZAOLI] }));
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_121, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let initial_hand = tx.state().hand_cards(owner);
    if !initial_hand.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: initial_hand,
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: Some(tx.state().zones.find(ZoneKind::Hand, owner).unwrap()),
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWZAOLI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let old = inject(&mut tx, ZoneKind::Hand, owner, CardKind::Sha, 93_800);
    let gained = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Shan,
        93_801,
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![gained],
            to: hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().playable_hand_cards(owner), vec![gained]);

    for (kind, def) in [
        (CardKind::Weapon, 93_802),
        (CardKind::Armor, 93_803),
        (CardKind::HorseDef, 93_804),
    ] {
        inject(&mut tx, ZoneKind::Equip, owner, kind, def);
        inject(&mut tx, ZoneKind::DrawPile, PlayerId::NONE, kind, def + 10);
    }
    let zaoli = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWZAOLI)
        .unwrap()
        .id;
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzaoli {
                player: owner,
                stage: 0,
                cards: Vec::new(),
                slots: Vec::new(),
                count: 0,
                skill: zaoli,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![old.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        state
            .zones
            .find(ZoneKind::Equip, owner)
            .and_then(|zone| state.zones.get(zone))
            .unwrap()
            .cards
            .len(),
        3
    );
}
