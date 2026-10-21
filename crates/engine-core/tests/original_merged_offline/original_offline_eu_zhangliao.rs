//! Golden coverage for 欧陆张辽「突袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const EU_ZHANGLIAO: u32 = 3_318_164_381;
const EU_TUXI: u32 = 3_876_759_954;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/eu_tuxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: Option<PlayerId>,
    skill: engine_api::SkillId,
}

fn setup(player_count: u8, seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, player_count, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players.get(2).copied();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(EU_TUXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = EU_ZHANGLIAO;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == owner && instance.def.raw() == EU_TUXI)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, def: u32) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let kind = if zone == ZoneKind::Equip {
        CardKind::Weapon
    } else {
        CardKind::Other
    };
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: zone_id,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn open_effect(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                    player: setup.owner,
                    target: setup.target,
                    skill: setup.skill,
                    ready_to_choose: false,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_publishes_complete_eu_zhangliao_and_exact_abi63_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == EU_ZHANGLIAO)
        .unwrap();
    assert_eq!(character.name_key, "eu_zhangliao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [EU_TUXI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == EU_TUXI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source");
    assert!(trigger.optional);
    let compiled = content_compiler::compile_source_named("eu_tuxi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 63);
    assert_eq!(
        compiled.modules[0].logic_hash,
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );
}

#[test]
fn draw_finishes_before_ordered_target_owner_and_next_choices() {
    let mut setup = setup(3, 2_026_072_101);
    let other = setup.other.unwrap();
    let target_card = inject(&mut setup.state, setup.target, ZoneKind::Hand, 99_401);
    let owner_hand = inject(&mut setup.state, setup.owner, ZoneKind::Hand, 99_402);
    let owner_equip = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 99_403);
    let other_card = inject(&mut setup.state, other, ZoneKind::Hand, 99_404);
    let state = open_effect(&setup);
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors
    );

    let (state, result) = resolve(state, setup.owner, vec![target_card.raw()], 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let self_options: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(self_options, [owner_equip.raw()]);
    assert!(!self_options.contains(&owner_hand.raw()));

    let (state, result) = resolve(state, setup.owner, vec![owner_equip.raw()], 11);
    let next_options: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(next_options, [other_card.raw()]);
    let (state, result) = resolve(state, setup.owner, vec![other_card.raw()], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.target), 0);
    assert_eq!(state.hand_count(other), 0);
    assert_eq!(state.hand_count(setup.owner), 5);
}

#[test]
fn two_player_previous_and_next_duplicate_is_preserved() {
    let mut setup = setup(2, 2_026_072_102);
    let target_card = inject(&mut setup.state, setup.target, ZoneKind::Hand, 99_411);
    let first = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 99_412);
    let second = inject(&mut setup.state, setup.owner, ZoneKind::Equip, 99_413);
    let state = open_effect(&setup);
    let (state, _) = resolve(state, setup.owner, vec![target_card.raw()], 20);
    let (state, result) = resolve(state, setup.owner, vec![first.raw()], 21);
    let options: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, [second.raw()]);
    let (state, _) = resolve(state, setup.owner, vec![second.raw()], 22);
    assert!(!state.has_pending_choice());
}

#[test]
fn empty_or_dead_entries_are_skipped() {
    let mut setup = setup(3, 2_026_072_103);
    let other = setup.other.unwrap();
    setup.state.players.get_mut(setup.target).unwrap().alive = false;
    let other_card = inject(&mut setup.state, other, ZoneKind::Hand, 99_421);
    let state = open_effect(&setup);
    let options = &state.resolution.continuation_payload;
    assert!(options.contains(&other_card.raw()));
}

#[test]
fn forged_card_selection_is_rejected_atomically() {
    let mut setup = setup(3, 2_026_072_104);
    inject(&mut setup.state, setup.target, ZoneKind::Hand, 99_431);
    let state = open_effect(&setup);
    let before = canonical_state_hash(&state);
    let (state, result) = resolve(state, setup.owner, vec![u32::MAX], 40);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}
