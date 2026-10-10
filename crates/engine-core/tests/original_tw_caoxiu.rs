//! Golden coverage for TW 曹休「前驱乐/倾袭」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWQIANJU: u32 = 2_546_046_607;
const TWQINGXI: u32 = 2_467_573_737;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twcaoxiu.skill");

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
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
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
fn qianju_equips_an_empty_slot_and_qingxi_discards_equipment_for_damage() {
    let package = content_compiler::compile_source_named("tw-caoxiu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 379);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_120, "identity-standard").unwrap();
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
        players
            .iter()
            .copied()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend(
        [TWQIANJU, TWQINGXI].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();

    let gained = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Armor,
        93_790,
    );
    let qianju = skill(&tx, owner, TWQIANJU);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwqianju {
            player: owner,
            target,
            skill: qianju,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .zones
        .find(ZoneKind::Equip, owner)
        .and_then(|zone| tx.state().zones.get(zone))
        .unwrap()
        .cards
        .contains(&gained));

    let source_weapon = inject(&mut tx, ZoneKind::Equip, owner, CardKind::Weapon, 93_791);
    inject(&mut tx, ZoneKind::Equip, target, CardKind::HorseDef, 93_792);
    let sha = inject(&mut tx, ZoneKind::Hand, owner, CardKind::Sha, 93_793);
    tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    tx.state_mut().resolution.resume_payload = vec![owner.raw(), target.raw(), 1, 0, 1, sha.raw()];
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let qingxi = skill(&tx, owner, TWQINGXI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwqingxi {
                player: owner,
                target,
                card: sha,
                stage: 0,
                count: 0,
                cards: Vec::new(),
                skill: qingxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, target, vec![1], 1);
    let state = choose(state, target, vec![source_weapon.raw()], 2);
    let target_equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(state.zones.get(target_equip).unwrap().cards.is_empty());
    assert!(!state
        .zones
        .get(owner_equip)
        .unwrap()
        .cards
        .contains(&source_weapon));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
}
