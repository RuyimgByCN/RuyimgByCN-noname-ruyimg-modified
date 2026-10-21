//! Golden coverage for MOBILE 杨阜「借兵」(ABI 499).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIEBING: u32 = 3_832_378_886;
const HANNAN: u32 = 3_756_118_415;
const YANGFU: u32 = 2_919_026_286;
const SOURCE: &str = include_str!("../../../../content/original/ported/mobile/jiebing.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

#[test]
fn damage_excludes_source_and_equips_random_gain() {
    let packaged = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        packaged
            .character_index
            .iter()
            .find(|character| character.id == YANGFU)
            .unwrap()
            .skills,
        [JIEBING, HANNAN]
    );

    let package = content_compiler::compile_source_named("jiebing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 499);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, source, donor) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIEBING),
    });
    run_effects(&mut tx, effects).unwrap();

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        let zone = tx.state().zones.find(kind, donor).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        run_effects(
            &mut tx,
            cards
                .into_iter()
                .map(|card| EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(zone),
                })
                .collect(),
        )
        .unwrap();
    }
    let donor_equip = tx.state().zones.find(ZoneKind::Equip, donor).unwrap();
    let (weapon, from) = tx
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .find(|card| card.kind == CardKind::Weapon)
        .map(|card| (card.id, card.zone))
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: donor_equip,
            from: Some(from),
        }],
    )
    .unwrap();

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    assert_eq!(tx.state().resolution.continuation_payload[2], source.raw());
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![donor.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert_eq!(
        state.cards.get(weapon).unwrap().owner,
        owner,
        "{:?}",
        state.cards.get(weapon).unwrap()
    );
    assert_eq!(state.cards.get(weapon).unwrap().zone, owner_equip);
}
