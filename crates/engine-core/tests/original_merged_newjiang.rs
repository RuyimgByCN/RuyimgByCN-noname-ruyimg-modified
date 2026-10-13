//! O2 全量合并第二批（2026-10-14）：家族 newjiang 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_newjiang_yizu {
//! Golden for NEWJIANG「轶祖」的目标、体力比较与每回合限一次语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const YIZU: u32 = 3_120_129_590;
const LKZHONGZHUANG: u32 = 3_477_978_221;
const DCYIJU: u32 = 3_976_872_382;
const BIANCAI: u32 = 2_285_861_837;
const SPQISHE: u32 = 2_801_887_664;
const KANGLI: u32 = 2_295_716_313;
const SUNLI: u32 = 3_384_580_445;
const FENYUE2: u32 = 2_246_618_728;
const XINXHZHIYAN_FALSE: u32 = 3_265_620_898;
const WEIFENG2: u32 = 2_853_992_972;
/// Concatenate every committed newjiang source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn newjiang_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/newjiang");
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

const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/newjiang/newjiang.npack.json");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started_engine(package: &content_schema::ContentPackage) -> Engine {
    let mut engine = Engine::new();
    engine.load_package(package).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    engine
}

#[test]
fn committed_package_matches_source() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("newjiang", &newjiang_sources()).unwrap();
    compiled.character_index = serde_json::from_str(include_str!(
        "../../../content/original/ported/newjiang/characters.json"
    ))
    .unwrap();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert_eq!(committed.skill_index.len(), 103);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == YIZU && skill.locked));
    assert_eq!(committed.manifest.rule_vm_abi_version, 732);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.id == SUNLI && character.skills == vec![KANGLI]));
    assert_eq!(
        committed
            .view_as_index
            .iter()
            .filter(|view| view.skill_def_id == SPQISHE)
            .count(),
        2
    );
    for window in ["play_phase", "response"] {
        assert!(committed.view_as_index.iter().any(|view| {
            view.skill_def_id == SPQISHE
                && view.as_kind == "jiu"
                && view.window == window
                && view.filter == "card_equip"
                && view.source_zone.as_deref() == Some("hand_equip")
        }));
    }
    assert!(committed.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SPQISHE
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_equip_count")
    }));
    assert!(committed.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINXHZHIYAN_FALSE
            && modifier.target == "card_targets_self_only"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
}

#[test]
fn weifeng2_publishes_mark_display_and_phase_end_draw() {
    // The 1.11.5 delta round gave every catalog skill a non-empty body:
    // weifeng2 gained an optional phase_end draw (the 惧-mark display is
    // presentation-only; the old JS never entered the rule runtime).
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WEIFENG2));
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == WEIFENG2));
    assert!(package
        .modifier_index
        .iter()
        .all(|modifier| modifier.skill_def_id != WEIFENG2));
    assert!(package
        .active_index
        .iter()
        .all(|active| active.skill_def_id != WEIFENG2));
    assert!(package
        .view_as_index
        .iter()
        .all(|view_as| view_as.skill_def_id != WEIFENG2));
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

fn add_only_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

#[test]
fn xinxhzhiyan_false_rejects_cards_targeting_other_players() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&package);
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    add_only_skill(&mut tx, owner, XINXHZHIYAN_FALSE);
    assert!(engine_core::query::card_targets_self_only_for(
        tx.state(),
        owner
    ));
    tx.state_mut().turn.phase = Phase::Play;
    let guohe = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(guohe).unwrap().kind = CardKind::TrickGuohe;

    let (_, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(XINXHZHIYAN_FALSE as u64),
            kind: CommandKind::PlayCard {
                card: guohe,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn spqishe_counts_equipment_and_uses_only_equipment_as_jiu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&package);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_only_skill(&mut tx, owner, SPQISHE);
    let cards = tx.state().hand_cards(owner);
    let equip_cards = [cards[0], cards[1]];
    let wrong = cards[2];
    tx.state_mut().cards.get_mut(equip_cards[0]).unwrap().kind = CardKind::Armor;
    tx.state_mut().cards.get_mut(equip_cards[1]).unwrap().kind = CardKind::Weapon;
    tx.state_mut().cards.get_mut(wrong).unwrap().kind = CardKind::Sha;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equip_cards.to_vec(),
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(hand_limit(&tx, owner), hp + 2);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    tx.state_mut().turn.phase = Phase::Play;

    let (_, rejected) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(40),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![wrong],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, accepted) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(41),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![equip_cards[0]],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.zones.get(equip).unwrap().cards, vec![equip_cards[1]]);
}

#[test]
fn spqishe_equipment_can_self_save_while_hand_cards_are_disabled() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.load_package(&sp).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(42),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_720,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    add_only_skill(&mut tx, owner, SPQISHE);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENYUE2),
        }],
    )
    .unwrap();
    let source = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(source).unwrap().kind = CardKind::Armor;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![source],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), owner),
        Some(source)
    );

    let committed = tx.commit();
    let choice_id = engine_api::ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(43),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(source).unwrap().zone, discard);
    let owner = state.players.get(owner).unwrap();
    assert!(owner.alive && !owner.dying);
    assert_eq!(owner.hp, 1);
}

fn isolated_biancai(package: &content_schema::ContentPackage) -> (Transaction, PlayerId, PlayerId) {
    let engine = started_engine(package);
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(BIANCAI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn offer_biancai(
    tx: &mut Transaction,
    current: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve_biancai(
    state: engine_core::state::GameState,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn biancai_is_optional_and_gains_equipment_from_the_color_selected_pile() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    let (mut red, owner, other) = isolated_biancai(&package);
    let draw = red
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = red.state().zones.get(draw).unwrap().cards.clone();
    for card in &draw_cards {
        red.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let judged_red = *draw_cards.last().unwrap();
    let deck_equipment = draw_cards[draw_cards.len() - 2];
    red.state_mut().cards.get_mut(judged_red).unwrap().suit = Suit::Heart;
    red.state_mut().cards.get_mut(deck_equipment).unwrap().kind = CardKind::Weapon;

    assert_eq!(
        offer_biancai(&mut red, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve_biancai(red.state().clone(), owner, false, 30);
    assert_eq!(declined.cards.get(judged_red).unwrap().zone, draw);
    assert_eq!(declined.cards.get(deck_equipment).unwrap().zone, draw);

    let mut red = Transaction::begin(declined);
    assert_eq!(
        offer_biancai(&mut red, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let red = resolve_biancai(red.state().clone(), owner, true, 31);
    let discard = red
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = red.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(red.cards.get(judged_red).unwrap().zone, discard);
    assert_eq!(red.cards.get(deck_equipment).unwrap().zone, hand);

    let (mut black, owner, other) = isolated_biancai(&package);
    let draw = black
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = black.state().zones.get(draw).unwrap().cards.clone();
    for card in &draw_cards {
        black.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let judged_black = *draw_cards.last().unwrap();
    black.state_mut().cards.get_mut(judged_black).unwrap().suit = Suit::Spade;
    let discard_equipment = black.state().hand_cards(owner)[0];
    black
        .state_mut()
        .cards
        .get_mut(discard_equipment)
        .unwrap()
        .kind = CardKind::Armor;
    let discard = black
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut black,
        vec![EffectRequest::MoveCards {
            cards: vec![discard_equipment],
            to: discard,
            from: None,
        }],
    )
    .unwrap();

    assert_eq!(
        offer_biancai(&mut black, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let black = resolve_biancai(black.state().clone(), owner, true, 32);
    let hand = black.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(black.cards.get(judged_black).unwrap().zone, discard);
    assert_eq!(black.cards.get(discard_equipment).unwrap().zone, hand);
}

#[test]
fn dcyiju_forces_one_owner_hand_or_equip_discard_for_another_single_target() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCYIJU),
    });
    run_effects(&mut tx, effects).unwrap();

    let equip_card = tx.state().hand_cards(owner)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    tx.state_mut().cards.get_mut(equip_card).unwrap().kind = CardKind::Armor;
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip_card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let source_cards = tx.state().hand_cards(source);
    let used_multi = source_cards[0];
    let used_single = source_cards[1];

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: used_multi,
            targets: vec![owner, source],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    let self_used = tx.state().hand_cards(owner)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: self_used,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: used_single,
            targets: vec![owner],
        }],
    )
    .unwrap_err();
    assert_eq!(err, engine_core::effect::EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::DiscardOneHandOrEquip
    );

    let committed = tx.commit();
    let choice_id = engine_api::ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![equip_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equip_card).unwrap().zone, discard);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let used_no_cards = tx.state().hand_cards(source)[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: used_no_cards,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn dcyiju_choice_resumes_the_played_sha_response() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCYIJU),
    });
    run_effects(&mut tx, effects).unwrap();

    let owner_cards = tx.state().hand_cards(owner);
    let equip_card = owner_cards[0];
    let shan = owner_cards[1];
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    tx.state_mut().cards.get_mut(equip_card).unwrap().kind = CardKind::Armor;
    tx.state_mut().cards.get_mut(shan).unwrap().kind = CardKind::Shan;
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip_card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let sha = tx.state().hand_cards(source)[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let hp_before = tx.state().players.get(owner).unwrap().hp;

    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::DiscardOneHandOrEquip
    );
    assert_eq!(
        state.resolution.resume_kind,
        engine_core::state::ContinuationKind::ResumePlayedSha
    );

    let discard_choice = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: discard_choice,
                selections: vec![equip_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::RespondToSha
    );
    assert!(state.has_pending_choice());

    let sha_choice = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id: sha_choice,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn dcyiju_choice_resumes_the_played_trick_wuxie_pipeline() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCYIJU),
    });
    run_effects(&mut tx, effects).unwrap();

    let owner_cards = tx.state().hand_cards(owner);
    let equip_card = owner_cards[0];
    let guohe_card = owner_cards[1];
    for card in &owner_cards {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let source_cards = tx.state().hand_cards(source);
    for card in &source_cards {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let guohe = source_cards[0];
    tx.state_mut().cards.get_mut(guohe).unwrap().kind = CardKind::TrickGuohe;
    tx.state_mut().cards.get_mut(equip_card).unwrap().kind = CardKind::Armor;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip_card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;

    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20),
            kind: CommandKind::PlayCard {
                card: guohe,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::DiscardOneHandOrEquip
    );
    assert_eq!(
        state.resolution.resume_kind,
        engine_core::state::ContinuationKind::WuxieWindow
    );

    let discard_choice = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(21),
            kind: CommandKind::ResolveChoice {
                choice_id: discard_choice,
                selections: vec![equip_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::PickTargetCard
    );

    let guohe_choice = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(22),
            kind: CommandKind::ResolveChoice {
                choice_id: guohe_choice,
                selections: vec![guohe_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(guohe_card).unwrap().zone, discard);
    assert!(!state.has_pending_choice());
}

#[test]
fn yizu_checks_source_hp_then_recovers_only_once_in_the_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YIZU),
    });
    run_effects(&mut tx, effects).unwrap();

    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 4;
        owner_state.hp = 3;
    }
    {
        let source_state = tx.state_mut().players.get_mut(source).unwrap();
        source_state.max_hp = 4;
        source_state.hp = 2;
    }
    let cards = tx.state().hand_cards(source);
    assert!(cards.len() >= 3);
    tx.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::Sha;
    tx.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::TrickJuedou;
    tx.state_mut().cards.get_mut(cards[2]).unwrap().kind = CardKind::Sha;

    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[0],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);

    tx.state_mut().players.get_mut(source).unwrap().hp = 4;
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[1],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: source,
            card: cards[2],
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
}

#[test]
fn lkzhongzhuang_uses_source_range_only_for_logical_sha_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LKZHONGZHUANG),
    });
    effects.push(EffectRequest::DiscardFromHand {
        player: target,
        count: u32::MAX,
    });
    run_effects(&mut tx, effects).unwrap();
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 30;
    target_state.max_hp = 30;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, 28);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target,
            card: CardId::NONE,
            amount: 2,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, 27);

    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(LKZHONGZHUANG + 1),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(weapon);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target,
            card: CardId::NONE,
            amount: 2,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, 25);
    tx.state_mut().cards.get_mut(weapon).unwrap().weapon_range = 4;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target,
            card: CardId::NONE,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, 23);

    tx.state_mut().cards.get_mut(weapon).unwrap().weapon_range = 1;
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(engine_core::content::MARK_JIU),
            delta: 1,
        }],
    )
    .unwrap();
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 22);
}

#[test]
fn kangli_discards_only_drawn_cards_that_never_left_hand() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let engine = started_engine(&committed);
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    add_only_skill(&mut tx, owner, KANGLI);
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 20;
    tx.state_mut().players.get_mut(other).unwrap().hp = 20;

    let before_receive: std::collections::BTreeSet<_> =
        tx.state().hand_cards(owner).into_iter().collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let remembered: Vec<_> = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|card| !before_receive.contains(card))
        .collect();
    assert_eq!(remembered.len(), 2);

    let escaped = remembered[0];
    let continuous = remembered[1];
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![escaped],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![escaped],
            to: hand,
            from: Some(discard),
        }],
    )
    .unwrap();

    let before_deal: std::collections::BTreeSet<_> =
        tx.state().hand_cards(owner).into_iter().collect();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().cards.get(escaped).unwrap().zone, hand);
    assert_eq!(tx.state().cards.get(continuous).unwrap().zone, discard);
    assert_eq!(
        tx.state()
            .hand_cards(owner)
            .into_iter()
            .filter(|card| !before_deal.contains(card))
            .count(),
        2
    );
}
}
