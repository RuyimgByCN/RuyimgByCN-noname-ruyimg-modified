//! Golden coverage for 界庞德「鞬出」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIANCHU: u32 = 2_342_849_364;
const RE_PANGDE: u32 = 2_815_087_846;
const MASHU: u32 = 2;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/jianchu.skill");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("jianchu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_723, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIANCHU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(
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
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_jianchu_re_pangde_and_standard_dependency() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "standard"
            && dependency.version == "0.1.0"
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIANCHU)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "card_targeted",
            "owner_source_sha_target_alive_has_discardable_hand_equip",
            true
        )
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_PANGDE
            && character.name_key == "re_pangde"
            && character.max_hp == 4
            && character.skills == [MASHU, JIANCHU]
    }));
}

#[test]
fn equipment_branch_discards_the_card_and_makes_only_this_target_unrespondable() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 94_001);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 94_002);
    let weapon = inject(&mut tx, target, ZoneKind::Equip, CardKind::Weapon, 94_003);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (offered, result) = play_sha(tx.commit().state, owner, target, sha, 1);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (selecting, result) = resolve(offered, owner, vec![1], false, 2);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == weapon.raw()));
    let (resolved, result) = resolve(selecting, owner, vec![weapon.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);
    assert!(resolved.hand_cards(target).contains(&shan));
    let discard = resolved
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(resolved.cards.get(weapon).unwrap().zone, discard);
}

#[test]
fn non_equipment_branch_gives_the_physical_sha_then_preserves_normal_response() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 94_011);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 94_012);
    let hand = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 94_013);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (offered, _) = play_sha(tx.commit().state, owner, target, sha, 11);
    let (selecting, _) = resolve(offered, owner, vec![1], false, 12);
    let (responding, result) = resolve(selecting, owner, vec![hand.raw()], false, 13);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    assert!(responding.hand_cards(target).contains(&sha));
    let (resolved, result) = resolve(responding, target, vec![shan.raw()], false, 14);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp);
    assert!(resolved.hand_cards(target).contains(&sha));
}
