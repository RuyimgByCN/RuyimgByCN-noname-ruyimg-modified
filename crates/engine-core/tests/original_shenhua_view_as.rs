//! Goldens for 神话再临「酒池」与「火计」的视为技结算。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, Engine};
use rule_api::EffectRequest;

const JIUCHI: u32 = 2_316_672_021;
const HUOJI: u32 = 4_120_560_565;
const KANPO: u32 = 2_834_927_312;
const DUANLIANG: u32 = 2_757_928_964;
const DUANLIANG1: u32 = 3_934_371_847;
const GZDUANLIANG: u32 = 2_148_140_249;
const LIANHUAN: u32 = 2_921_824_505;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn package_engine(seed: u64) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(seed)),
        CommandResult::Accepted { .. }
    ));
    engine
}

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|item| item.def.raw() == def)
        })
        .unwrap()
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    inject_to(tx, owner, ZoneKind::Hand, kind, suit, rank)
}

fn inject_to(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(97_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    };
    let card_id = tx.state_mut().cards.alloc(card);
    tx.state_mut()
        .zones
        .get_mut(zone)
        .unwrap()
        .cards
        .push(card_id);
    card_id
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let startup_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, startup_skills).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn activate(
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    card: engine_api::CardId,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: vec![card],
        },
    }
}

fn pass_wuxie(
    mut state: engine_core::state::GameState,
    mut pipeline: engine_core::command::PipelineResult,
    mut key: u64,
) -> engine_core::state::GameState {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipeline.choice.expect("铁索连环应打开无懈窗口");
        key += 1;
        let (next, next_pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(
            next_pipeline.result,
            CommandResult::Accepted { .. }
        ));
        state = next;
        pipeline = next_pipeline;
    }
    state
}

#[test]
fn package_exposes_jiuchi_huoji_and_kanpo_view_as_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == JIUCHI
                && view.as_kind == "jiu"
                && view.window == window
                && view.filter == "spade"
        }));
    }
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == HUOJI && view.as_kind == "huogong" && view.filter == "red"
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == KANPO
            && view.as_kind == "wuxie"
            && view.window == "response"
            && view.filter == "black"
    }));
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == DUANLIANG
            && view.as_kind == "bingliang"
            && view.window == "play_phase"
            && view.filter == "black_basic_equip"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DUANLIANG
            && modifier.target == "card_distance_unlimited_if_target_hand_gte_owner"
            && modifier.selector.as_deref() == Some("bingliang")
            && modifier.value == 1
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == DUANLIANG1
            && view.as_kind == "bingliang"
            && view.window == "play_phase"
            && view.filter == "black_basic_equip"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == GZDUANLIANG
            && view.as_kind == "bingliang"
            && view.window == "play_phase"
            && view.filter == "black_basic_equip"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == LIANHUAN
            && view.as_kind == "tiesuo"
            && view.window == "play_phase"
            && view.filter == "club"
            && view.source_zone.is_none()
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == 3_455_386_297
            && character.name_key == "re_xuhuang"
            && character.max_hp == 4
            && character.skills == [DUANLIANG, 2_177_948_990]
    }));
}

#[test]
fn lianhuan_club_hand_card_can_recast_or_use_tiesuo() {
    let engine = package_engine(20_260_720);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = add_skill(&mut tx, actor, LIANHUAN);
    let wrong = inject(&mut tx, actor, CardKind::Shan, Suit::Heart, 6);
    let recast = inject(&mut tx, actor, CardKind::Shan, Suit::Club, 7);

    let before = canonical_state_hash(tx.state());
    let (rejected, pipeline) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], wrong, 20),
    );
    assert!(matches!(pipeline.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&rejected), before);

    let hand_before = tx.state().hand_count(actor);
    let (recast_state, pipeline) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], recast, 21),
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    assert!(!recast_state.hand_cards(actor).contains(&recast));
    assert_eq!(recast_state.hand_count(actor), hand_before);

    let mut tx = Transaction::begin(recast_state);
    let source = inject(&mut tx, actor, CardKind::Sha, Suit::Club, 8);
    let (state, pipeline) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![target], source, 22),
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    let state = pass_wuxie(state, pipeline, 22);
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}

#[test]
fn gzduanliang_accepts_only_black_basic_or_equipment_from_hand_or_equip() {
    let engine = package_engine(20_260_719);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let skill = add_skill(&mut tx, actor, GZDUANLIANG);
    let red_basic = inject(&mut tx, actor, CardKind::Sha, Suit::Heart, 2);
    let black_trick = inject(&mut tx, actor, CardKind::TrickGuohe, Suit::Spade, 3);
    let black_basic = inject(&mut tx, actor, CardKind::Shan, Suit::Club, 4);
    let black_equip = inject_to(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        5,
    );

    for (key, rejected_card) in [(8, red_basic), (9, black_trick)] {
        let before = canonical_state_hash(tx.state());
        let (state, pipe) = apply_command(
            tx.state().clone(),
            activate(actor, skill, vec![target], rejected_card, key),
        );
        assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
        assert_eq!(canonical_state_hash(&state), before);
    }

    for (key, source) in [(10, black_basic), (11, black_equip)] {
        let (state, pipe) = apply_command(
            tx.state().clone(),
            activate(actor, skill, vec![target], source, key),
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        let judge = state.zones.find(ZoneKind::Judge, target).unwrap();
        let card = state.cards.get(source).unwrap();
        assert_eq!(card.kind, CardKind::TrickBingliang);
        assert_eq!(card.zone, judge);
    }
}

#[test]
fn kanpo_black_hand_card_only_works_in_a_wuxie_response_window() {
    let engine = package_engine(20_260_718);
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    let skill = add_skill(&mut tx, other, KANPO);
    let wuzhong = inject(&mut tx, actor, CardKind::TrickWuzhong, Suit::Heart, 7);
    let black = inject(&mut tx, other, CardKind::Sha, Suit::Club, 8);

    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let before = canonical_state_hash(tx.state());
    let (rejected, pipe) =
        apply_command(tx.state().clone(), activate(other, skill, vec![], black, 5));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&rejected), before);

    tx.state_mut().turn.current = actor;
    let hand_before = tx.state().hand_count(actor);
    let (state, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(6),
            kind: CommandKind::PlayCard {
                card: wuzhong,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("看破应进入无懈响应窗口");
    assert_eq!(choice.observer, other);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == black.raw()));

    let (resolved, pipe) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(7),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![black.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(actor), hand_before - 1);
    assert!(!resolved.hand_cards(other).contains(&black));
}

#[test]
fn jiuchi_uses_only_a_spade_hand_card_as_jiu() {
    let engine = package_engine(20_260_715);
    let actor = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    let skill = add_skill(&mut tx, actor, JIUCHI);
    let wrong = inject(&mut tx, actor, CardKind::Shan, Suit::Heart, 2);
    let source = inject(&mut tx, actor, CardKind::Shan, Suit::Spade, 3);
    tx.state_mut().players.get_mut(actor).unwrap().hp -= 1;

    let before = canonical_state_hash(tx.state());
    let (rejected, pipe) =
        apply_command(tx.state().clone(), activate(actor, skill, vec![], wrong, 1));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&rejected), before);

    let hp = tx.state().players.get(actor).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], source, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(actor).unwrap().hp, hp + 1);
    assert!(!state.hand_cards(actor).contains(&source));
}

#[test]
fn jiuchi_spade_card_can_self_save_from_dying() {
    let engine = package_engine(20_260_716);
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_hand(&mut tx, actor);
    add_skill(&mut tx, actor, JIUCHI);
    let source = inject(&mut tx, actor, CardKind::Shan, Suit::Spade, 4);
    tx.state_mut().players.get_mut(actor).unwrap().hp = 1;

    let error = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: actor,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap_err();
    assert_eq!(error, EffectError::ChoiceRequested);
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), actor),
        Some(source)
    );

    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, pipe) = apply_command(
        committed.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let player = state.players.get(actor).unwrap();
    assert!(player.alive && !player.dying);
    assert_eq!(player.hp, 1);
    assert!(!state.hand_cards(actor).contains(&source));
}

#[test]
fn huoji_red_card_runs_the_full_huogong_resolution() {
    let engine = package_engine(20_260_717);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = add_skill(&mut tx, actor, HUOJI);
    let wrong = inject(&mut tx, actor, CardKind::Shan, Suit::Club, 2);
    let source = inject(&mut tx, actor, CardKind::Shan, Suit::Heart, 3);
    let matching = inject(&mut tx, actor, CardKind::Sha, Suit::Spade, 7);
    let shown = inject(&mut tx, target, CardKind::Tao, Suit::Spade, 9);

    let before = canonical_state_hash(tx.state());
    let (rejected, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![target], wrong, 4),
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&rejected), before);

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![target], source, 5),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let reveal = pipe.choice.unwrap();
    assert_eq!(reveal.observer, target);

    let (state, pipe) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(6),
            kind: CommandKind::ResolveChoice {
                choice_id: reveal.choice_id,
                selections: vec![shown.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let discard = pipe.choice.unwrap();
    assert_eq!(discard.observer, actor);
    assert!(discard
        .options
        .iter()
        .any(|option| option.token == matching.raw()));

    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(7),
            kind: CommandKind::ResolveChoice {
                choice_id: discard.choice_id,
                selections: vec![matching.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.hand_cards(actor).contains(&source));
    assert!(!state.hand_cards(actor).contains(&matching));
}
