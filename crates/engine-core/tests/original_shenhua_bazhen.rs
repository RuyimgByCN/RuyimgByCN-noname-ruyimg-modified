//! Golden coverage for 神话「八阵」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, EquipSlot, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content, Engine};
use rule_api::EffectRequest;

const BAZHEN: u32 = 3_741_926_095;
const SP_ZHUGELIANG: u32 = 2_869_965_581;
const HUOJI: u32 = 4_120_560_565;
const KANPO: u32 = 2_834_927_312;
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

fn engine(seed: u64) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(seed)),
        CommandResult::Accepted { .. }
    ));
    engine
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
    def: u32,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_equip(),
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
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

fn replace_skills_with_bazhen(tx: &mut Transaction, player: PlayerId) {
    let effects: Vec<_> = tx
        .state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .chain([EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(BAZHEN),
        }])
        .collect();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_publishes_complete_sp_zhugeliang_and_virtual_bagua_modifier() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == BAZHEN
            && modifier.target == "virtual_bagua_if_armor_slot_empty"
            && modifier.selector.is_none()
            && modifier.value == 1
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == SP_ZHUGELIANG
            && character.max_hp == 3
            && character.skills == [BAZHEN, HUOJI, KANPO]
    }));
}

#[test]
fn virtual_bagua_requires_an_enabled_empty_armor_slot() {
    let engine = engine(20_260_722_87);
    let player = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    replace_skills_with_bazhen(&mut tx, player);
    assert!(content::player_has_bagua(tx.state(), player));
    assert_eq!(
        content::player_bagua_source(tx.state(), player),
        Some(CardId::NONE)
    );

    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let armor = inject(
        &mut tx,
        player,
        equip,
        CardKind::Armor,
        Suit::Club,
        2,
        9_287_001,
    );
    assert!(!content::player_has_bagua(tx.state(), player));

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![armor],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert!(content::player_has_bagua(tx.state(), player));

    tx.state_mut()
        .players
        .get_mut(player)
        .unwrap()
        .disabled_equip_slots[EquipSlot::Armor.index()] = 1;
    assert!(!content::player_has_bagua(tx.state(), player));
}

#[test]
fn virtual_bagua_red_judgment_answers_sha_without_a_fake_cause_card() {
    let engine = engine(20_260_723_87);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    let source_skills = tx.state().players.get(source).unwrap().skills.clone();
    run_effects(
        &mut tx,
        source_skills
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill })
            .collect(),
    )
    .unwrap();
    replace_skills_with_bazhen(&mut tx, target);

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let judgment = inject(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        Suit::Heart,
        5,
        9_287_002,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = inject(
        &mut tx,
        source,
        hand,
        CardKind::Sha,
        Suit::Spade,
        7,
        9_287_003,
    );
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, pipeline) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    let choice = pipeline.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == content::BAGUA_RESPONSE_TOKEN));

    let (state, pipeline) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![content::BAGUA_RESPONSE_TOKEN],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved {
            player,
            result_card,
            cause_card,
            ..
        } if *player == target && *result_card == judgment && cause_card.is_none()
    )));
}
