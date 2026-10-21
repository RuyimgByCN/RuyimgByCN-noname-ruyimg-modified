//! Golden coverage for TW 孙鲁班「骄矜」and package closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TW_SUNLUBAN: u32 = 2_766_500_142;
const TW_FENGXI: u32 = 3_675_106_908;
const TWZENHUI: u32 = 3_053_684_104;
const XINJIAOJIN: u32 = 2_479_658_413;
const TW: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");
const MOBILE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn inherited_jiaojin_discards_equipment_and_prevents_the_whole_damage() {
    let tw = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(tw.manifest.rule_vm_abi_version, 733);
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);
    assert_eq!(
        tw.character_index
            .iter()
            .find(|character| character.id == TW_SUNLUBAN)
            .unwrap()
            .skills,
        [TWZENHUI, XINJIAOJIN]
    );

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_204, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&mobile)
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&tw)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINJIAOJIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = TW_SUNLUBAN;
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = TW_FENGXI;
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_801),
        kind: CardKind::Armor,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(equipment);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 1);
    let state = resolve(state, owner, vec![equipment.raw()], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
}
