//! Golden coverage for MOBILE「暗箭」rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINANJIAN: u32 = 2_652_033_709;
const TEST_MULTI_SHA: u32 = 2_652_033_710;
const XINANJIAN_SOURCE: &str =
    include_str!("../../../content/original/ported/mobile/xinanjian.skill");
/// Test-only helper (def 2652033710): grants the owner +1 sha target and makes
/// every multi-target sha target unable to respond, so the multi-target
/// scenario exercises only xinanjian picks without response choices.
const TEST_MULTI_SHA_SOURCE: &str = "\
// Test-only helper skill for multi-target sha isolation.
skill test_multi_sha(2652033710) {
  trigger card_targeted owner_source_sha_target_alive_owner_outside_target_attack_range {
    effect make_current_sha_target_unrespondable(self, event_card, event_player);
  }
  modifier sha_target_limit add 1;
}
";
const XIN_PANZHANGMAZHONG: u32 = 3_389_482_484;
/// Concatenate every committed mobile source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn mobile_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/mobile");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(player_count: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("xinanjian-test", &format!("{XINANJIAN_SOURCE}\n{TEST_MULTI_SHA_SOURCE}")).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 2_026_072_409, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
        skill_def: SkillDefId::new(XINANJIAN),
    });
    if player_count > 4 {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TEST_MULTI_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, seats)
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
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult, CardId) {
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        XINANJIAN + key as u32,
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets,
                card_targets: vec![],
            },
        },
    );
    (state, result, sha)
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

fn equip_range_two(tx: &mut Transaction, owner: PlayerId, def: u32) {
    inject(tx, owner, ZoneKind::Equip, CardKind::Weapon, def);
}

#[test]
fn package_contains_abi120_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(package.skill_index.len(), 440);
    assert_eq!(package.character_index.len(), 204);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINANJIAN && skill.name == "xinanjian"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == XIN_PANZHANGMAZHONG)
        .unwrap();
    assert_eq!(character.name_key, "xin_panzhangmazhong");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [3_614_910_583, XINANJIAN]);

    let compiled = content_compiler::compile_source_named("xinanjian", &mobile_sources()).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn target_includes_owner_in_attack_range_so_skill_does_not_fire() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        XINANJIAN + 100,
    );
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![target], 1);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    assert!(state.resolution.vm_cont_module.is_empty());
}

#[test]
fn unrespondable_branch_skips_target_shan() {
    let (state, owner, seats) = setup(4);
    let target = seats[2];
    let mut tx = Transaction::begin(state);
    equip_range_two(&mut tx, owner, XINANJIAN + 200);
    inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        XINANJIAN + 201,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![target], 2);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, result) = resolve(state, owner, vec![1], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(state.hand_count(target), 1);
}

#[test]
fn damage_branch_keeps_response_and_adds_one_if_sha_hits() {
    let (state, owner, seats) = setup(4);
    let target = seats[2];
    let mut tx = Transaction::begin(state);
    equip_range_two(&mut tx, owner, XINANJIAN + 300);
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        XINANJIAN + 301,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![target], 4);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![2], false, 5);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    let (state, result) = resolve(state, target, vec![], true, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}

#[test]
fn multi_target_damage_bonus_changes_only_selected_target() {
    let (state, owner, seats) = setup(5);
    let first = seats[2];
    let second = seats[3];
    let mut tx = Transaction::begin(state);
    equip_range_two(&mut tx, owner, XINANJIAN + 400);
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let second_hp = tx.state().players.get(second).unwrap().hp;
    let (state, result, _) = play_sha(tx.commit().state, owner, vec![first, second], 7);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![2], false, 8);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![1], false, 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 2);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
}

#[test]
fn intent_rejects_a_stale_or_mismatched_sha_tuple() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let card = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        XINANJIAN + 500,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::IncreaseCurrentShaTargetDamage {
                source: owner,
                card,
                target,
                amount: 1,
            }],
        ),
        Err(EffectError::ContentFault(
            "Sha use refund requested outside a saved proactive Sha".into()
        ))
    );
}
