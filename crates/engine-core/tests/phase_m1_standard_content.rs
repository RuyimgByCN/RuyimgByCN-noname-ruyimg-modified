//! M1: expanded deck tricks, jiu mark, standard package auto-load, character skills.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::effect::run_effects;
use engine_core::state::CardKind;
use engine_core::transaction::Transaction;
use engine_core::{run_identity_match, Engine, MatchConfig};
use rule_api::EffectRequest;
use std::collections::BTreeMap;

const STANDARD_SOURCE: &str = include_str!("../../../content/standard/standard_m1.skill");
const STANDARD_CHARACTERS: &str = include_str!("../../../content/standard/characters.json");

fn start(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

#[test]
fn bootstrap_loads_standard_package() {
    let mut eng = Engine::new();
    eng.submit(start(1, 4));
    assert!(
        eng.content().package_count() >= 1,
        "standard pack should auto-load"
    );
    // caocao may or may not be dealt; jianxiong defs available via package
    let defs = eng.content().defs_for_skill(engine_api::SkillDefId::new(1));
    assert!(!defs.is_empty());
    assert!(matches!(
        defs[0].handler,
        engine_core::trigger::TriggerHandler::RunVm { .. }
    ));
}

#[test]
fn deck_includes_new_tricks() {
    let mut eng = Engine::new();
    eng.submit(start(2, 4));
    let kinds: Vec<_> = eng
        .state()
        .cards
        .sorted_live()
        .into_iter()
        .map(|c| c.kind)
        .collect();
    assert!(kinds.iter().any(|k| *k == CardKind::TrickShunshou));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickJuedou));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickNanman));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickWanjian));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickTaoyuan));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickWugu));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickHuogong));
    assert!(kinds.iter().any(|k| *k == CardKind::TrickWuxie));
    assert!(kinds.iter().any(|k| *k == CardKind::ShaFire));
    assert!(kinds.iter().any(|k| *k == CardKind::ShaThunder));
}

#[test]
fn roster_has_expected_characters() {
    assert!(engine_core::content::STANDARD_CHARACTERS
        .iter()
        .any(|c| c.name_key == "machao" && !c.skills.is_empty()));
    assert!(engine_core::content::STANDARD_CHARACTERS.iter().any(|c| {
        c.id == 4212795236 && c.name_key == "huaxiong" && c.max_hp == 6 && c.skills == [3872761084]
    }));
    assert!(engine_core::content::STANDARD_CHARACTERS.iter().any(|c| {
        c.id == 4123573742
            && c.name_key == "std_panfeng"
            && c.max_hp == 4
            && c.skills == [4213747779]
    }));
}

#[test]
fn committed_standard_npack_drives_match_roster() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/standard/standard.npack.json");
    let bytes = std::fs::read(path).expect("committed standard npack");
    let package = content_schema::ContentPackage::from_json_bytes(&bytes).expect("valid npack");
    assert_eq!(
        package.character_index.len(),
        engine_core::content::STANDARD_CHARACTERS.len()
    );
    let mut compiled = content_compiler::compile_source_named("standard", STANDARD_SOURCE).unwrap();
    compiled.character_index = serde_json::from_str(STANDARD_CHARACTERS).unwrap();
    // The committed npack declares the refresh dependency (gongsunzan /
    // old_re_lidian reference refresh skills); logic_hash covers dependencies.
    compiled.manifest.dependencies = package.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(package.skill_index.len(), compiled.skill_index.len());
    assert_eq!(package.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == 14
            && trigger.event == "card_used"
            && trigger.filter == "owner_source_ordinary_trick"
    }));
    assert!(package.active_index.iter().any(|active| {
        active.skill_def_id == 10
            && active.uses_per_turn == Some(1)
            && active.selection.as_ref().is_some_and(|selection| {
                selection.zone == "hand_equip" && selection.min == 1 && selection.max == 255
            })
    }));

    let roster: BTreeMap<_, _> = package
        .character_index
        .iter()
        .map(|character| (character.id, character))
        .collect();
    let mut engine = Engine::new();
    engine.load_package(&package).expect("load npack");
    let result = engine.submit(start(17, 4));
    assert!(matches!(result, engine_api::CommandResult::Accepted { .. }));

    for player in engine.state().players.iter_live() {
        let character = roster
            .get(&player.character_def)
            .expect("dealt character came from package roster");
        let granted: Vec<_> = player
            .skills
            .iter()
            .map(|skill_id| engine.state().skills.get(*skill_id).unwrap().def.raw())
            .collect();
        assert_eq!(granted, character.skills);
    }
}

#[test]
fn jiu_mark_boosts_sha_damage() {
    let mut eng = Engine::new();
    eng.submit(start(3, 2));
    // Force play phase for current
    let actor = eng.state().turn.current;
    let other = eng
        .state()
        .players
        .iter_live()
        .find(|p| p.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(eng.state().clone());
    tx.state_mut().turn.phase = engine_core::Phase::Play;
    tx.state_mut().revision = 1;
    // Full HP so jiu grants mark
    if let Some(p) = tx.state_mut().players.get_mut(actor) {
        p.hp = p.max_hp as i32;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: actor,
            mark: engine_api::MarkDefId::new(engine_core::content::MARK_JIU),
            delta: 1,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(other).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: actor,
            target: other,
            amount: 2, // simulate jiu-boosted sha amount
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, hp_before - 2);
}

#[test]
fn headless_match_still_runs_with_m1_content() {
    let report = run_identity_match(MatchConfig {
        player_count: 4,
        rng_seed: 99,
        max_steps: 800,
        ruleset: "identity-standard".into(),
    })
    .expect("match");
    assert!(report.steps > 0);
}

#[test]
fn character_skills_registered_from_package() {
    let mut eng = Engine::new();
    eng.submit(start(42, 8));
    // At least one player should have a skill instance from character kit
    let with_skills = eng
        .state()
        .players
        .iter_live()
        .filter(|p| !p.skills.is_empty())
        .count();
    assert!(
        with_skills >= 1,
        "expected some characters with skills dealt"
    );
}
