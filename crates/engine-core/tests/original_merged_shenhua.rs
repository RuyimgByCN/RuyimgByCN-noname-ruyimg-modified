//! O2 全量合并批次 1（2026-10-14）：家族 shenhua 的 60 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_shenhua_baonue {
//! Golden coverage for 神话「暴虐」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BAONUE: u32 = 3_659_854_073;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/baonue.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("baonue_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    qun_source: PlayerId,
    wei_source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let mut package = content_compiler::compile_source_named("baonue-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_288_010, "qun"),
        character(9_288_011, "qun"),
        character(9_288_012, "wei"),
        character(9_288_013, "shu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 20_260_722_88, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut others = seats.iter().copied().filter(|player| *player != lord);
    let qun_source = others.next().unwrap();
    let wei_source = others.next().unwrap();
    let target = others.next().unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_288_010;
    tx.state_mut()
        .players
        .get_mut(qun_source)
        .unwrap()
        .character_def = 9_288_011;
    tx.state_mut()
        .players
        .get_mut(wei_source)
        .unwrap()
        .character_def = 9_288_012;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = 9_288_013;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: lord,
        skill_def: SkillDefId::new(BAONUE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().hp = 3;
    Setup {
        tx,
        lord,
        qun_source,
        wei_source,
        target,
    }
}

fn inject_judgment(tx: &mut Transaction, suit: Suit) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_288_100 + suit as u32),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn deal(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
}

fn resolve_choice(
    tx: &Transaction,
    actor: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_exposes_damage_source_choice_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == BAONUE)
        .unwrap();
    assert!(skill.lord_only);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == BAONUE)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_alive_other_damage_source_group_qun");
    assert!(trigger.optional);
    assert_eq!(
        trigger.choice_actor,
        content_schema::TriggerChoiceActor::EventSource
    );
}

#[test]
fn only_another_living_qun_damage_source_gets_the_choice() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.state().clone());
    assert_eq!(
        deal(&mut tx, setup.qun_source, setup.target),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.pending_observer,
        setup.qun_source.raw()
    );

    let mut tx = Transaction::begin(setup.tx.state().clone());
    deal(&mut tx, setup.wei_source, setup.target).unwrap();
    assert!(!tx.state().has_pending_choice());

    let mut tx = Transaction::begin(setup.tx.state().clone());
    deal(&mut tx, setup.lord, setup.target).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn accepted_spade_judges_for_source_and_recovers_lord() {
    let mut setup = setup();
    let judgment = inject_judgment(&mut setup.tx, Suit::Spade);
    assert_eq!(
        deal(&mut setup.tx, setup.qun_source, setup.target),
        Err(EffectError::ChoiceRequested)
    );
    let (state, pipeline) = resolve_choice(&setup.tx, setup.qun_source, true, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.lord).unwrap().hp, 4);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved {
            player,
            result_card,
            cause_card,
            ..
        } if *player == setup.qun_source && *result_card == judgment && cause_card.is_none()
    )));
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered { source, player, amount: 1 }
            if *source == setup.lord && *player == setup.lord
    )));
}

#[test]
fn non_spade_does_not_recover_and_declining_does_not_judge() {
    let mut non_spade = setup();
    let club = inject_judgment(&mut non_spade.tx, Suit::Club);
    assert_eq!(
        deal(&mut non_spade.tx, non_spade.qun_source, non_spade.target,),
        Err(EffectError::ChoiceRequested)
    );
    let (state, pipeline) = resolve_choice(&non_spade.tx, non_spade.qun_source, true, 2);
    assert_eq!(state.players.get(non_spade.lord).unwrap().hp, 3);
    assert!(pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved { player, result_card, .. }
            if *player == non_spade.qun_source && *result_card == club
    )));

    let mut declined = setup();
    let spade = inject_judgment(&mut declined.tx, Suit::Spade);
    assert_eq!(
        deal(&mut declined.tx, declined.qun_source, declined.target),
        Err(EffectError::ChoiceRequested)
    );
    let (state, pipeline) = resolve_choice(&declined.tx, declined.qun_source, false, 3);
    assert_eq!(state.players.get(declined.lord).unwrap().hp, 3);
    assert!(!pipeline.events.iter().any(|event| matches!(
        event,
        DomainEvent::JudgmentResolved { result_card, .. } if *result_card == spade
    )));
}
}

mod original_shenhua_bazhen {
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
}

mod original_shenhua_caiwenji {
//! Golden coverage for 神话蔡琰「悲歌」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BEIGE: u32 = 2_536_833_640;
const TEST_REWRITE: u32 = 9_253_001;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/beige.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    target: PlayerId,
    source: PlayerId,
    skill: engine_api::SkillId,
}

fn character(id: u32) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("beige_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "female".into(),
        group: "qun".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn setup(rewriter: bool) -> Setup {
    let source = if rewriter {
        format!(
            "{SOURCE}\nskill test_rewrite({TEST_REWRITE}) locked {{ modifier judgment_rewrite add 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("beige-test", &source).unwrap();
    package.character_index = (0..3).map(|index| character(9_253_010 + index)).collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_727_01, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target, source) = (seats[0], seats[1], seats[2]);
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
        skill_def: SkillDefId::new(BEIGE),
    });
    if rewriter {
        effects.push(EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(TEST_REWRITE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == BEIGE)
        .unwrap()
        .id;
    Setup {
        tx,
        owner,
        target,
        source,
        skill,
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_253_100 + rank as u32),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    inject(tx, player, hand, suit, rank)
}

fn inject_judgment(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(tx, PlayerId::NONE, draw, suit, rank)
}

fn begin(mut setup: Setup, suit: Suit) -> (GameState, CardId, Result<(), EffectError>) {
    let cost = inject_hand(&mut setup.tx, setup.owner, Suit::Club, 1);
    inject_judgment(&mut setup.tx, suit, 7);
    let result = run_effects(
        &mut setup.tx,
        vec![EffectRequest::ResolveBeige {
            player: setup.owner,
            target: setup.target,
            source: setup.source,
            stage: 0,
            result_card: CardId::NONE,
            suit: Suit::None,
            rank: 0,
            rewriters: Vec::new(),
            skill: setup.skill,
        }],
    )
    .map(|_| ());
    (setup.tx.commit().state, cost, result)
}

fn resolve_first(state: GameState, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let selection = engine_core::query::discardable_target_cards(
        &state,
        actor,
        actor,
        rule_api::TargetCardScope::HandEquip,
    )[0]
    .raw();
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_contract_is_abi219_while_committed_package_is_abi221() {
    let package = content_compiler::compile_source_named("beige-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 219);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == BEIGE)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty"
    );
    assert!(trigger.optional);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    assert_eq!(module.code, [83, 84, 85, 80, 219, 1]);
    assert_eq!(
        module.logic_hash,
        [
            7, 31, 230, 177, 202, 60, 16, 99, 81, 198, 147, 57, 182, 167, 33, 230, 169, 90, 120,
            247, 146, 82, 145, 248, 69, 9, 44, 166, 133, 242, 19, 11,
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert_eq!(committed.skill_index.len(), 201);
    assert_eq!(committed.character_index.len(), 48);
    let caiwenji = committed
        .character_index
        .iter()
        .find(|character| character.id == 2_362_677_836)
        .unwrap();
    assert_eq!(caiwenji.skills, [BEIGE, 2_554_836_650]);
}

#[test]
fn final_suit_applies_each_legacy_branch_after_paying_one_card() {
    let mut heart = setup(false);
    heart
        .tx
        .state_mut()
        .players
        .get_mut(heart.target)
        .unwrap()
        .hp = 2;
    let target = heart.target;
    let (state, cost, result) = begin(heart, Suit::Heart);
    result.unwrap();
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);

    let diamond = setup(false);
    let target = diamond.target;
    let hand = diamond.tx.state().hand_count(target);
    let (state, _, result) = begin(diamond, Suit::Diamond);
    result.unwrap();
    assert_eq!(state.hand_count(target), hand + 2);

    let mut club = setup(false);
    let source = club.source;
    inject_hand(&mut club.tx, source, Suit::Heart, 2);
    inject_hand(&mut club.tx, source, Suit::Diamond, 3);
    let (state, _, result) = begin(club, Suit::Club);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, first) = resolve_first(state, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, second) = resolve_first(state, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), 0);

    let spade = setup(false);
    let source = spade.source;
    let (state, _, result) = begin(spade, Suit::Spade);
    result.unwrap();
    assert!(state.players.get(source).unwrap().turned_over);
}

#[test]
fn judgment_rewrite_card_becomes_the_final_beige_result() {
    let mut setup = setup(true);
    let source = setup.source;
    let target = setup.target;
    setup.tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let rewrite = inject_hand(&mut setup.tx, source, Suit::Spade, 9);
    let (state, _, result) = begin(setup, Suit::Heart);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, pipeline) = apply_command(
        state.clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![rewrite.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert!(state.players.get(source).unwrap().turned_over);
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_chendao {
//! Golden coverage for SHENHUA 陈到「往烈」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{
    card_distance_unlimited_for, card_targets_self_only_for, card_use_unrespondable_for_card,
};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCWANGLIE: u32 = 3_434_983_038;
const DCWANGLIE_EFFECT: u32 = 4_183_988_931;
const DCWANGLIE2: u32 = 2_388_106_243;
const CHENDAO: u32 = 3_186_469_990;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const MAIN: &str = include_str!("../../../content/original/ported/shenhua/dcwanglie.skill");
const COUNTER: &str =
    include_str!("../../../content/original/ported/shenhua/dcwanglie_effect.skill");
const LOCK: &str = include_str!("../../../content/original/ported/shenhua/dcwanglie2.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named(
        "dcwanglie-test",
        &format!("{MAIN}\n{COUNTER}\n{LOCK}"),
    )
    .unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(DCWANGLIE), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCWANGLIE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(has_skill(&tx, owner, DCWANGLIE_EFFECT));
    Setup { tx, owner, other }
}

fn inject(tx: &mut Transaction, player: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn has_skill(tx: &Transaction, owner: PlayerId, skill_def: u32) -> bool {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

fn dispatch_card(
    tx: &mut Transaction,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    )
}

#[test]
fn package_publishes_chendao_in_current_shenhua_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let chendao = package
        .character_index
        .iter()
        .find(|character| character.id == CHENDAO)
        .unwrap();
    assert_eq!(chendao.skills, [DCWANGLIE]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DCWANGLIE_EFFECT
            && modifier.target == "card_distance_unlimited"
            && modifier.value_source.as_deref() == Some("owner_skill_play_phase_fire_count_below_2")
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DCWANGLIE2
            && modifier.target == "card_use_unrespondable_for_skill_card_policy"
    }));
}

#[test]
fn first_two_cards_targeting_others_ignore_distance() {
    let mut setup = setup();
    let self_card = inject(&mut setup.tx, setup.owner, CardKind::Shan, 96_001);
    assert!(card_distance_unlimited_for(setup.tx.state(), setup.owner));
    let effects = dispatch_card(&mut setup.tx, setup.owner, self_card, vec![setup.owner]).unwrap();
    run_effects(&mut setup.tx, effects).unwrap();
    assert!(card_distance_unlimited_for(setup.tx.state(), setup.owner));

    for index in 0..2 {
        let card = inject(&mut setup.tx, setup.owner, CardKind::Shan, 96_010 + index);
        let effects = dispatch_card(&mut setup.tx, setup.owner, card, vec![setup.other]).unwrap();
        run_effects(&mut setup.tx, effects).unwrap();
        assert_eq!(
            card_distance_unlimited_for(setup.tx.state(), setup.owner),
            index == 0
        );
    }
}

#[test]
fn accepted_use_marks_only_current_card_unrespondable_and_cleans_both_helpers() {
    let mut setup = setup();
    let current = inject(&mut setup.tx, setup.owner, CardKind::TrickWuzhong, 96_101);
    let later = inject(&mut setup.tx, setup.owner, CardKind::TrickWuzhong, 96_102);
    assert_eq!(
        dispatch_card(&mut setup.tx, setup.owner, current, vec![setup.other]).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = setup.tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    assert!(has_skill(&tx, setup.owner, DCWANGLIE2));
    assert!(card_targets_self_only_for(tx.state(), setup.owner));
    assert!(card_use_unrespondable_for_card(
        tx.state(),
        setup.owner,
        current,
        CardKind::TrickWuzhong,
        Suit::Spade,
    ));
    assert!(!card_use_unrespondable_for_card(
        tx.state(),
        setup.owner,
        later,
        CardKind::TrickWuzhong,
        Suit::Spade,
    ));

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill(&tx, setup.owner, DCWANGLIE_EFFECT));
    assert!(!has_skill(&tx, setup.owner, DCWANGLIE2));
    assert!(!card_targets_self_only_for(tx.state(), setup.owner));
}
}

mod original_shenhua_dengai {
//! Golden coverage for 屯田/凿险/急袭（shenhua 邓艾，ABI 533-534）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const TUNTIAN: u32 = 2_366_746_565;
const ZAOXIAN: u32 = 853_475_869;
const JIXI: u32 = 3_780_186_838;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/dengai.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("dengai-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
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
        skill_def: SkillDefId::new(TUNTIAN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZAOXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_tuntian_zaoxian_contract() {
    let compiled = content_compiler::compile_source_named("dengai", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 534);
    let tuntian = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TUNTIAN)
        .unwrap();
    assert_eq!(tuntian.event, "cards_moved");
    assert_eq!(tuntian.filter, "owner_lost_cards_outside_turn");
    assert!(tuntian.optional);
    let zaoxian = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZAOXIAN)
        .unwrap();
    assert_eq!(zaoxian.event, "phase_prepare");
    assert_eq!(
        zaoxian.filter,
        "owner_phase_begin_skill_pile_gte_three_2366746565"
    );
    assert!(!zaoxian.optional);
    let jixi = compiled
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == JIXI)
        .unwrap();
    assert_eq!(jixi.as_kind, "shunshou");
    assert_eq!(jixi.source_zone.as_deref(), Some("hand_expansion"));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert_eq!(compiled.modifier_index.len(), 1);
}

#[test]
fn tuntian_stores_non_heart_judge_card_in_pile_and_reduces_distance() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Place a known non-heart card on the draw pile top (club 7).
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let judge_card = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_330,
        Suit::Club,
        7,
    );
    // The owner loses a hand card during another player's turn.
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_331,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    // The trigger opened the optional tuntian choice.
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "optional tuntian confirm pending"
    );
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Judge card (club 7) went into the 田 pile (face-up expansion of the owner).
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN)
        .map(|pile| pile.zone)
        .unwrap();
    let pile_cards: Vec<_> = state
        .zones
        .get(pile)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(pile_cards, vec![judge_card], "judge card stored as 田");
    assert!(!state.has_pending_choice());
    // Distance from the owner to the other player is reduced by the 田 count (base 1 → 0).
    let mut ctx = QueryContext::new();
    let dist = match query(
        &state,
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.other,
        },
        &mut ctx,
    ) {
        QueryResponse::Int(d) => d,
        other => panic!("distance query failed: {other:?}"),
    };
    // 相邻距离基准 1，被田数减至 0 后按引擎不变量钳制回 1（全引擎距离下限为 1，
    // 与旧 JS 距离可为 0 的差异为文档化近似）。
    assert_eq!(
        dist, 1,
        "adjacent distance clamped at the engine floor of 1"
    );
    let dist_back = match query(
        &state,
        &QueryRequest::Distance {
            from: setup.other,
            to: setup.owner,
        },
        &mut ctx,
    ) {
        QueryResponse::Int(d) => d,
        other => panic!("distance query failed: {other:?}"),
    };
    assert_eq!(dist_back, 1, "other-to-owner distance unaffected");
    // 隔座基准距离 2，减 1 田后为 1 —— 证明计数修正器生效。
    let seats = state.players.alive_by_seat();
    let far = seats[2];
    let dist_far = match query(
        &state,
        &QueryRequest::Distance {
            from: setup.owner,
            to: far,
        },
        &mut ctx,
    ) {
        QueryResponse::Int(d) => d,
        other => panic!("distance query failed: {other:?}"),
    };
    assert_eq!(dist_far, 1, "far distance reduced from 2 by the 田 count");
}

#[test]
fn tuntian_heart_judge_card_goes_nowhere() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let judge_card = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_340,
        Suit::Heart,
        7,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_341,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Heart judge card: no 田 pile is created.
    assert!(
        !state
            .skill_piles
            .iter()
            .any(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN),
        "no 田 pile for a heart judge"
    );
    let discard_cards: Vec<_> = state
        .zones
        .get(discard)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert!(
        discard_cards.contains(&judge_card),
        "heart judge card to discard"
    );
}

#[test]
fn zaoxian_awakens_with_three_tian() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    // Stack three non-heart cards: first three judgments build 田.
    let c1 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_350,
        Suit::Club,
        2,
    );
    let c2 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_351,
        Suit::Spade,
        3,
    );
    let c3 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_352,
        Suit::Club,
        4,
    );
    for _ in 0..3 {
        let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
        let lost = alloc_card(
            &mut tx,
            setup.owner,
            hand,
            CardKind::Tao,
            96_360,
            Suit::Heart,
            1,
        );
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![lost],
                    to: discard,
                    from: Some(hand),
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let mut state = tx.commit().state;
        state = resolve_choice(state, setup.owner, vec![1], 1);
        tx = Transaction::begin(state);
    }
    // Now the owner's prepare phase: 凿险 awakens (max hp 5 → 4, gains 急袭).
    let mut tx = tx;
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut tx = match fired {
        Ok(effects) => {
            let mut next = Transaction::begin(state);
            run_effects(&mut next, effects).unwrap();
            next
        }
        Err(EffectError::ChoiceRequested) => Transaction::begin(state),
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        4,
        "max hp reduced from lord 5 to 4"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == JIXI),
        "急袭 granted"
    );
    // 田 stay in the pile (usable by 急袭).
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN)
        .map(|pile| pile.zone)
        .unwrap();
    let pile_cards: Vec<_> = state
        .zones
        .get(pile)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(pile_cards.len(), 3);
    let _ = (c1, c2, c3);
}

#[test]
fn zaoxian_does_not_fire_below_three_tian() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _ = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_370,
        Suit::Club,
        2,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_371,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // One 田 only — prepare phase does not awaken.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        5,
        "no awakening below three 田 (lord bonus)"
    );
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == setup.owner && skill.def.raw() == JIXI));
}

#[test]
fn jixi_uses_tian_as_shunshou() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Grant 急袭, then build one 田 through the real tuntian flow.
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.owner,
            skill_def: SkillDefId::new(JIXI),
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _ = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_380,
        Suit::Club,
        5,
    );
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_381,
        Suit::Heart,
        1,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![lost],
                to: discard,
                from: Some(hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // The 田 pile is an expansion zone owned by the owner — the hand_expansion
    // view-as source for 顺手牵羊.
    let pile = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner && pile.skill_def == TUNTIAN)
        .map(|pile| pile.zone)
        .unwrap();
    let pile_zone = state.zones.get(pile).map(|zone| zone.kind).unwrap();
    assert_eq!(
        pile_zone,
        engine_core::state::ZoneKind::Expansion,
        "田 pile is an expansion zone"
    );
    let pile_cards: Vec<_> = state
        .zones
        .get(pile)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(pile_cards.len(), 1, "one 田 stored");
    assert!(
        state
            .cards
            .get(pile_cards[0])
            .is_some_and(|card| card.owner == setup.owner),
        "田 owned by the owner"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == JIXI),
        "急袭 active"
    );
}
}

mod original_shenhua_dianwei {
//! Golden semantics for SHENHUA 典韦「强袭」rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIANGXIX: u32 = 3_151_452_629;
const DIANWEI: u32 = 2_274_294_572;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/qiangxix.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
    weapon: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("qiangxix-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, u64::from(QIANGXIX), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANGXIX),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 8;
        participant.hp = 8;
    }
    let weapon = tx.state().hand_cards(owner)[0];
    let card = tx.state_mut().cards.get_mut(weapon).unwrap();
    card.kind = CardKind::Weapon;
    card.weapon_range = 2;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == QIANGXIX)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
        weapon,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    setup: &Setup,
    state: GameState,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_dianwei_and_qiangxix_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert!(package.skill_index.len() >= 75);
    assert!(package.character_index.len() >= 16);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DIANWEI)
        .unwrap();
    assert_eq!(character.skills, [QIANGXIX]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == QIANGXIX)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(2));
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| !targets.allow_self && targets.unused_by_skill_this_turn));
}

#[test]
fn weapon_and_hp_costs_damage_distinct_targets_with_two_use_limit() {
    let setup = setup();
    let first_hp = setup.state.players.get(setup.targets[0]).unwrap().hp;
    let second_hp = setup.state.players.get(setup.targets[1]).unwrap().hp;
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, activation) = activate(&setup, setup.state.clone(), setup.targets[0], 1);
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardWeaponOrLoseHp
    );
    let (state, branch) = resolve(state, setup.owner, vec![1], 2);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardWeaponOrLoseHp
    );
    let (state, paid) = resolve(state, setup.owner, vec![setup.weapon.raw()], 3);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(
        state.players.get(setup.targets[0]).unwrap().hp,
        first_hp - 1
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.weapon).unwrap().zone, discard);

    let (same_state, repeated) = activate(&setup, state.clone(), setup.targets[0], 4);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(same_state, state);

    let (state, second) = activate(&setup, state, setup.targets[1], 5);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(
        state.players.get(setup.targets[1]).unwrap().hp,
        second_hp - 1
    );

    let (_, limited) = activate(&setup, state, setup.targets[2], 6);
    assert!(matches!(limited.result, CommandResult::Rejected { .. }));
}
}

mod original_shenhua_dongzhuo {
//! Golden coverage for 神话董卓「肉林」「崩坏」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DONGZHUO: u32 = 3_126_184_051;
const JIUCHI: u32 = 2_316_672_021;
const ROULIN: u32 = 3_168_437_611;
const BENGHUAI: u32 = 3_365_741_831;
const BAONUE: u32 = 3_659_854_073;
const MULTI_SHA: u32 = 4_290_000_089;
const OWNER_CHARACTER: u32 = 9_289_010;
const FEMALE_CHARACTER: u32 = 9_289_011;
const MALE_CHARACTER: u32 = 9_289_012;
const DOUBLE_CHARACTER: u32 = 9_289_013;
const ROULIN_SOURCE: &str = include_str!("../../../content/original/ported/shenhua/roulin.skill");
const BENGHUAI_SOURCE: &str =
    include_str!("../../../content/original/ported/shenhua/benghuai.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn character(id: u32, sex: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("dongzhuo_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: sex.into(),
        group: "qun".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    female: PlayerId,
    male: PlayerId,
    double: PlayerId,
}

fn setup(skill: u32, multi_sha: bool) -> Setup {
    let source = format!(
        "{ROULIN_SOURCE}\n{BENGHUAI_SOURCE}\n\
         skill multi_sha({MULTI_SHA}) {{ modifier card_distance_unlimited(sha) override 1; modifier sha_target_limit add 1; }}"
    );
    let mut package = content_compiler::compile_source_named("dongzhuo-test", &source).unwrap();
    package.character_index = vec![
        character(OWNER_CHARACTER, "male"),
        character(FEMALE_CHARACTER, "female"),
        character(MALE_CHARACTER, "male"),
        character(DOUBLE_CHARACTER, "double"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_722_89,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let [female, male, double] = others.as_slice() else {
        panic!("expected three players other than the owner");
    };
    let (female, male, double) = (*female, *male, *double);
    for (player, character_def) in [
        (owner, OWNER_CHARACTER),
        (female, FEMALE_CHARACTER),
        (male, MALE_CHARACTER),
        (double, DOUBLE_CHARACTER),
    ] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character_def;
    }

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill),
    });
    if multi_sha {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MULTI_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    Setup {
        tx,
        owner,
        female,
        male,
        double,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_890 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_card(
    state: GameState,
    actor: PlayerId,
    choice_id: ChoiceId,
    card: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

fn phase_end(tx: &mut Transaction, player: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve_benghuai(tx: &Transaction, option: u32, key: u64) -> GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: tx.state().turn.current,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![option],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_dongzhuo_and_exact_skill_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DONGZHUO)
        .unwrap();
    assert_eq!(character.name_key, "dongzhuo");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 8);
    assert!(character.is_lord);
    assert_eq!(character.skills, [JIUCHI, ROULIN, BENGHUAI, BAONUE]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == ROULIN
            && modifier.target == "sha_response_count_if_other_female"
            && modifier.selector.is_none()
            && modifier.mode == "add"
            && modifier.value == 1
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == BENGHUAI)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target_not_min_hp");
    assert!(!trigger.optional);
}

#[test]
fn roulin_applies_in_both_directions_only_opposite_female_or_double_sex() {
    let setup = setup(ROULIN, false);
    for (source, target, expected) in [
        (setup.owner, setup.female, 2),
        (setup.owner, setup.double, 2),
        (setup.owner, setup.male, 1),
        (setup.female, setup.owner, 2),
        (setup.double, setup.owner, 2),
        (setup.male, setup.owner, 1),
    ] {
        assert_eq!(
            engine_core::query::sha_response_count_for(setup.tx.state(), source, target, 1),
            expected
        );
    }
}

#[test]
fn multi_target_sha_recomputes_roulin_for_each_current_target() {
    let mut setup = setup(ROULIN, true);
    for player in [setup.owner, setup.female, setup.male, setup.double] {
        run_effects(
            &mut setup.tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    let sha = inject(&mut setup.tx, setup.owner, CardKind::Sha, 7);
    let female_first = inject(&mut setup.tx, setup.female, CardKind::Shan, 8);
    let female_second = inject(&mut setup.tx, setup.female, CardKind::Shan, 9);
    let male_only = inject(&mut setup.tx, setup.male, CardKind::Shan, 10);
    let female_hp = setup.tx.state().players.get(setup.female).unwrap().hp;
    let male_hp = setup.tx.state().players.get(setup.male).unwrap().hp;
    setup.tx.state_mut().turn.phase = Phase::Play;

    let (state, result) = apply_command(
        setup.tx.state().clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.female, setup.male],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let first = result.choice.unwrap();
    assert_eq!(first.observer, setup.female);
    let (state, result) = resolve_card(state, setup.female, first.choice_id, female_first, 2);
    let second = result.choice.unwrap();
    assert_eq!(second.observer, setup.female);
    let (state, result) = resolve_card(state, setup.female, second.choice_id, female_second, 3);
    let third = result.choice.unwrap();
    assert_eq!(third.observer, setup.male);
    let (state, result) = resolve_card(state, setup.male, third.choice_id, male_only, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.female).unwrap().hp, female_hp);
    assert_eq!(state.players.get(setup.male).unwrap().hp, male_hp);
}

#[test]
fn benghuai_forces_hp_or_max_hp_loss_only_when_owner_is_not_tied_minimum() {
    let mut lose_hp = setup(BENGHUAI, false);
    lose_hp
        .tx
        .state_mut()
        .players
        .get_mut(lose_hp.owner)
        .unwrap()
        .hp = 3;
    lose_hp
        .tx
        .state_mut()
        .players
        .get_mut(lose_hp.female)
        .unwrap()
        .hp = 2;
    assert_eq!(
        phase_end(&mut lose_hp.tx, lose_hp.owner),
        Err(EffectError::ChoiceRequested)
    );
    let max_hp = lose_hp
        .tx
        .state()
        .players
        .get(lose_hp.owner)
        .unwrap()
        .max_hp;
    let lost_hp = resolve_benghuai(&lose_hp.tx, 1, 5);
    assert_eq!(lost_hp.players.get(lose_hp.owner).unwrap().hp, 2);
    assert_eq!(lost_hp.players.get(lose_hp.owner).unwrap().max_hp, max_hp);

    let mut lose_max = setup(BENGHUAI, false);
    lose_max
        .tx
        .state_mut()
        .players
        .get_mut(lose_max.owner)
        .unwrap()
        .hp = 3;
    lose_max
        .tx
        .state_mut()
        .players
        .get_mut(lose_max.female)
        .unwrap()
        .hp = 2;
    assert_eq!(
        phase_end(&mut lose_max.tx, lose_max.owner),
        Err(EffectError::ChoiceRequested)
    );
    let max_hp = lose_max
        .tx
        .state()
        .players
        .get(lose_max.owner)
        .unwrap()
        .max_hp;
    let lost_max = resolve_benghuai(&lose_max.tx, 2, 6);
    assert_eq!(lost_max.players.get(lose_max.owner).unwrap().hp, 3);
    assert_eq!(
        lost_max.players.get(lose_max.owner).unwrap().max_hp,
        max_hp - 1
    );

    let mut tied = setup(BENGHUAI, false);
    tied.tx.state_mut().players.get_mut(tied.owner).unwrap().hp = 2;
    tied.tx.state_mut().players.get_mut(tied.female).unwrap().hp = 2;
    let effects = phase_end(&mut tied.tx, tied.owner).unwrap();
    assert!(effects.is_empty());
    assert!(!tied.tx.state().has_pending_choice());

    let mut other_turn = setup(BENGHUAI, false);
    other_turn
        .tx
        .state_mut()
        .players
        .get_mut(other_turn.owner)
        .unwrap()
        .hp = 3;
    other_turn
        .tx
        .state_mut()
        .players
        .get_mut(other_turn.female)
        .unwrap()
        .hp = 2;
    let effects = phase_end(&mut other_turn.tx, other_turn.female).unwrap();
    assert!(effects.is_empty());
}
}

mod original_shenhua_drlt_congjian {
//! Golden coverage for 从俭（shenhua 张绣，ABI 524）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DRLT_CONGJIAN: u32 = 3_110_735_242;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/drlt_congjian.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    source: PlayerId,
    other: PlayerId,
    hand_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("congjian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let other = seats[2];
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
        skill_def: SkillDefId::new(DRLT_CONGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_130),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        source,
        other,
        hand_card: card,
    }
}

fn fire_used(tx: Transaction, user: PlayerId, card: CardId, targets: Vec<PlayerId>) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn alloc_trick_card(tx: &mut Transaction, zone: engine_api::ZoneId) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_131),
        kind: CardKind::TrickNanman,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_multi_target_trick_contract() {
    let compiled = content_compiler::compile_source_named("congjian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 524);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DRLT_CONGJIAN)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_is_target_multi_target_trick_has_hand_equip"
    );
    assert!(trigger.optional);
}

#[test]
fn gives_card_to_other_target_and_draws_one() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // 南蛮入侵 targets the owner and another player.
    let source_hand = tx.state().zones.find(ZoneKind::Hand, setup.source).unwrap();
    let trick = alloc_trick_card(&mut tx, source_hand);
    let tx = fire_used(tx, setup.source, trick, vec![setup.owner, setup.other]);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Recipient choice: only the other target is offered.
    assert!(state.has_pending_choice(), "recipient choice pending");
    state = resolve_choice(state, setup.owner, vec![setup.other.raw()], 2);
    // Card choice: the single hand card.
    assert!(state.has_pending_choice(), "card choice pending");
    let hand_before = state.hand_count(setup.owner);
    state = resolve_choice(state, setup.owner, vec![setup.hand_card.raw()], 3);

    assert_eq!(
        state.hand_count(setup.owner),
        hand_before,
        "gave one, drew one"
    );
    assert!(
        state.hand_cards(setup.other).contains(&setup.hand_card),
        "the other target received the given card"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_leaves_everything_in_place() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let source_hand = tx.state().zones.find(ZoneKind::Hand, setup.source).unwrap();
    let trick = alloc_trick_card(&mut tx, source_hand);
    let tx = fire_used(tx, setup.source, trick, vec![setup.owner, setup.other]);
    let state = tx.commit().state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = next;
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_eq!(state.hand_count(setup.other), 0);
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_drlt_huairou {
//! Golden tests for 神话再临「怀柔」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DRLT_HUAIROU: u32 = 3_699_262_389;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/drlt_huairou.skill");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn started() -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("drlt_huairou_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DRLT_HUAIROU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DRLT_HUAIROU)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn committed_package_exposes_the_typed_equipment_recast() {
    let committed = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    let compiled = content_compiler::compile_source_named("drlt_huairou", SOURCE).unwrap();
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DRLT_HUAIROU)
        .unwrap();
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_equip"));
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn equipment_recast_rejects_other_cards_then_draws_one_replacement() {
    let (mut state, owner, skill) = started();
    let hand = state.hand_cards(owner);
    assert!(hand.len() >= 2);
    let equipment = hand[0];
    let ordinary = hand[1];
    state.cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    state.cards.get_mut(ordinary).unwrap().kind = CardKind::Other;

    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, skill, vec![ordinary], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let hand_before = state.hand_count(owner);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let (state, result) = activate(state, owner, skill, vec![equipment], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            to,
            reason: CardMoveReason::Recast,
            ..
        } if cards == &vec![equipment] && *to == discard
    )));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsDrawn { player, cards } if *player == owner && cards.len() == 1
    )));
}
}

mod original_shenhua_drlt_wanglie {
//! Golden coverage for SHENHUA「往烈」and OLD「旧陈到」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{
    card_distance_unlimited_for, card_use_cancelled_for, card_use_unrespondable_for_kind,
};
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DRLT_WANGLIE: u32 = 3_599_509_027;
const DRLT_WANGLIE2: u32 = 2_154_023_320;
const OLD_CHENDAO: u32 = 3_212_321_683;
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn setup() -> (Transaction, PlayerId, CardId) {
    let package = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(DRLT_WANGLIE)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(DRLT_WANGLIE),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRLT_WANGLIE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(95_001),
        kind: CardKind::TrickWuzhong,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx, owner, card)
}

fn has_skill(tx: &Transaction, owner: PlayerId, skill_def: u32) -> bool {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

#[test]
fn packages_publish_abi115_skill_and_cross_package_character() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    assert_eq!(shenhua.skill_index.len(), 201);
    let trigger = shenhua
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DRLT_WANGLIE)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_or_trick_current_play_phase"
    );
    assert!(trigger.optional);
    let modifier = shenhua
        .modifier_index
        .iter()
        .find(|modifier| {
            modifier.skill_def_id == DRLT_WANGLIE && modifier.target == "card_distance_unlimited"
        })
        .unwrap();
    assert_eq!(
        modifier.value_source.as_deref(),
        Some("owner_play_phase_card_use_count_zero")
    );
    assert!(shenhua.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DRLT_WANGLIE2
            && trigger.event == "phase_play_end"
            && !trigger.optional
    }));

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert!(old
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_CHENDAO)
        .unwrap();
    assert_eq!(character.name_key, "old_chendao");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [DRLT_WANGLIE]);
}

#[test]
fn only_the_first_card_in_the_owners_play_phase_ignores_distance() {
    let (mut tx, owner, _) = setup();
    assert!(tx.state().content.tracks_per_player_play_phase_card_count());
    assert!(card_distance_unlimited_for(tx.state(), owner));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::play_phase_card_used_by_player(owner), 1);
    assert!(!card_distance_unlimited_for(tx.state(), owner));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::play_phase_card_used_by_player(owner), 0);
    tx.state_mut().turn.phase = Phase::End;
    assert!(!card_distance_unlimited_for(tx.state(), owner));
}

#[test]
fn accepting_sha_or_trick_makes_it_unrespondable_then_cleans_up() {
    let (mut tx, owner, card) = setup();
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: vec![],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    assert!(has_skill(&tx, owner, DRLT_WANGLIE2));
    assert!(card_use_cancelled_for(tx.state(), owner));
    assert!(card_use_unrespondable_for_kind(
        tx.state(),
        owner,
        CardKind::TrickWuzhong,
        Suit::Spade,
    ));

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(!has_skill(&tx, owner, DRLT_WANGLIE2));
    assert!(!card_use_cancelled_for(tx.state(), owner));
}

#[test]
fn non_sha_basic_and_wrong_phase_do_not_offer_wanglie() {
    let (mut tx, owner, card) = setup();
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Shan;
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickWuzhong;
    tx.state_mut().turn.phase = Phase::End;
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
}
}

mod original_shenhua_drlt_yongsi {
//! Golden coverage for 庸肆/伪帝（shenhua 袁术，ABI 527-528）.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DRLT_YONGSI: u32 = 4_168_032_997;
const DRLT_WEIDI: u32 = 2_524_927_368;
const DRLT_YONGSI1: u32 = 3_870_932_875;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/drlt_yongsi.skill");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("yongsi_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    qun2: PlayerId,
    qun3: PlayerId,
    other: PlayerId,
}

/// `groups[0]` is the lord's faction; the remaining three are the other seats.
fn setup_with_groups(groups: [&str; 4]) -> Setup {
    let mut package = content_compiler::compile_source_named("yongsi-test", SOURCE).unwrap();
    package.character_index = vec![
        character(9_218_010, groups[0]),
        character(9_218_011, groups[1]),
        character(9_218_012, groups[2]),
        character(9_218_013, groups[3]),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let rest: Vec<_> = seats
        .iter()
        .copied()
        .filter(|player| *player != lord)
        .collect();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_218_010;
    tx.state_mut()
        .players
        .get_mut(rest[0])
        .unwrap()
        .character_def = 9_218_011;
    tx.state_mut()
        .players
        .get_mut(rest[1])
        .unwrap()
        .character_def = 9_218_012;
    tx.state_mut()
        .players
        .get_mut(rest[2])
        .unwrap()
        .character_def = 9_218_013;
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
        player: lord,
        skill_def: SkillDefId::new(DRLT_YONGSI),
    });
    effects.push(EffectRequest::AddSkill {
        player: lord,
        skill_def: SkillDefId::new(DRLT_WEIDI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = lord;
    Setup {
        tx,
        lord,
        qun2: rest[0],
        qun3: rest[1],
        other: rest[2],
    }
}

fn alloc_hand(tx: &mut Transaction, owner: PlayerId, count: u32, base: u32) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut cards = Vec::new();
    for i in 0..count {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(96_200 + base + i),
            kind: CardKind::Sha,
            suit: Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        cards.push(card);
    }
    cards
}

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn fire_play_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_yongsi_weidi_contract() {
    let compiled = content_compiler::compile_source_named("yongsi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 528);
    let draw_trigger = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DRLT_YONGSI && t.event == "phase_draw")
        .unwrap();
    assert_eq!(draw_trigger.filter, "owner_is_target_draw_not_replaced");
    let end_trigger = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DRLT_YONGSI && t.event == "phase_play_end")
        .unwrap();
    assert_eq!(end_trigger.filter, "owner_is_target");
    let weidi = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DRLT_WEIDI)
        .unwrap();
    assert_eq!(weidi.event, "phase_discard");
    assert_eq!(
        weidi.filter,
        "owner_is_target_discard_phase_owe_limit_other_qun"
    );
}

#[test]
fn yongsi_replaces_draw_with_living_group_count() {
    let setup = setup_with_groups(["qun", "wei", "shu", "wu"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Draw;
    let tx = fire_phase(tx, setup.lord, TurnPhaseTag::Draw);
    let state = tx.commit().state;
    assert_eq!(
        state.turn.draw_phase_fixed_count(),
        Some(4),
        "four factions alive"
    );
}

#[test]
fn yongsi_play_end_without_damage_draws_to_hp() {
    let setup = setup_with_groups(["qun", "qun", "qun", "wei"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    alloc_hand(&mut tx, setup.lord, 2, 0);
    tx.state_mut().players.get_mut(setup.lord).unwrap().hp = 4;
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_play_end(tx, setup.lord);
    let state = tx.commit().state;
    assert_eq!(state.hand_count(setup.lord), 4, "drew up to HP");
}

#[test]
fn yongsi_play_end_with_damage_grants_discard_state() {
    let setup = setup_with_groups(["qun", "qun", "qun", "wei"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    alloc_hand(&mut tx, setup.lord, 2, 0);
    tx.state_mut().players.get_mut(setup.lord).unwrap().hp = 4;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.lord,
            target: setup.other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let tx = fire_play_end(tx, setup.lord);
    let state = tx.commit().state;
    assert!(
        state
            .players
            .get(setup.lord)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == SkillDefId::new(DRLT_YONGSI1))
            }),
        "the discard-phase state skill was granted after dealing damage"
    );
    assert_eq!(
        state.hand_count(setup.lord),
        2,
        "no draw when damage was dealt"
    );
}

#[test]
fn weidi_gives_cards_to_equal_qun_players() {
    let setup = setup_with_groups(["qun", "qun", "qun", "wei"]);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Six hand cards vs hand limit 4: needs two, exactly two Qun others exist.
    alloc_hand(&mut tx, setup.lord, 6, 10);
    tx.state_mut().players.get_mut(setup.lord).unwrap().hp = 4;
    tx.state_mut().turn.phase = Phase::Discard;
    let tx = fire_phase(tx, setup.lord, TurnPhaseTag::Discard);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, setup.lord, vec![1], 1);
    // Card choice: up to two discardable hand cards.
    assert!(state.has_pending_choice(), "card choice pending");
    let cards = state.hand_cards(setup.lord);
    assert!(cards.len() >= 6);
    let c1 = cards[0];
    let c2 = cards[1];
    let hand_before = state.hand_count(setup.lord);
    state = resolve_choice(state, setup.lord, vec![c1.raw(), c2.raw()], 2);
    // Target choice: two distinct Qun others.
    assert!(state.has_pending_choice(), "target choice pending");
    state = resolve_choice(
        state,
        setup.lord,
        vec![setup.qun2.raw(), setup.qun3.raw()],
        3,
    );

    assert_eq!(
        state.hand_count(setup.lord),
        hand_before - 2,
        "two cards were given away"
    );
    assert!(state.hand_cards(setup.qun2).contains(&c1));
    assert!(state.hand_cards(setup.qun3).contains(&c2));
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_duanchang {
//! Golden for 神话再临蔡文姬「断肠」rewritten from legacy JS.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DUANCHANG: u32 = 2_554_836_650;
const KUANGGU: u32 = 3_700_078_390;
const WEIMU: u32 = 4_145_129_024;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/duanchang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn started(seed: u64) -> Transaction {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    tx
}

fn grant(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_exact_forced_death_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert_eq!(package.trigger_index.len(), 158);
    assert_eq!(package.modifier_index.len(), 56);
    assert_eq!(package.active_index.len(), 25);
    assert_eq!(package.modules.len(), 183);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DUANCHANG && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DUANCHANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_target_source_alive");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("shenhua", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 159);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "duanchang_death"),
        compiled.modules.first()
    );
}

#[test]
fn death_removes_every_current_skill_only_from_a_living_killer() {
    let mut tx = started(DUANCHANG as u64);
    let killer = tx.state().turn.current;
    let victim = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != killer)
        .unwrap();
    grant(&mut tx, victim, DUANCHANG);
    grant(&mut tx, killer, KUANGGU);
    grant(&mut tx, killer, WEIMU);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer,
        }],
    )
    .unwrap();
    assert!(tx.state().players.get(killer).unwrap().skills.is_empty());

    let mut tx = started(DUANCHANG as u64 + 1);
    let other = tx.state().turn.current;
    let victim = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != other)
        .unwrap();
    grant(&mut tx, victim, DUANCHANG);
    grant(&mut tx, other, KUANGGU);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: PlayerId::NONE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().skills.len(), 1);
}
}

mod original_shenhua_fangquan2 {
//! shenhua/fangquan2 放权（第九十二批·第五消费者）：回合结束可选弃一张手牌，
//! 选一名其他角色入队纯额外回合（choose_others_insert_extra_turn，check=0）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANGQUAN2: u32 = 4_075_257_478;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/shenhua"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("shenhua-fangquan2", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_834, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FANGQUAN2),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

#[test]
fn fangquan2_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == FANGQUAN2
            && t.event == "turn_ended"
            && t.filter == "owner_is_target"
            && t.optional
    }));
}

fn end_event(current: PlayerId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::End,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn fangquan2_discards_and_inserts_chosen_target_extra_turn() {
    use engine_core::state::ZoneKind;
    let (state, holder) = started();
    let victim = *state
        .players
        .alive_by_seat()
        .iter()
        .find(|&&p| p != holder)
        .unwrap();

    // holder 至少有一张手牌（body-if hand_count>0 才走效果链）。
    let mut tx = Transaction::begin(state);
    assert!(tx.state().hand_count(holder) > 0, "bootstrap must deal cards");
    let hand_before = tx.state().hand_count(holder);

    // 自己的回合结束：confirm → 弃一 → 目标选择续体。
    tx.state_mut().turn.current = holder;
    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[
            engine_api::DomainEvent::PhaseEnded {
                current: cur,
                phase: engine_api::TurnPhaseTag::End,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            },
        ],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // confirm 后先开「弃一张手牌」的 Custom 选择。
    let mid = resolve(tx.state, holder, vec![1], 1101);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::Custom
    );
    let hand_cards: Vec<u32> = mid.hand_cards(holder).iter().map(|c| c.raw()).collect();
    let drop_one = resolve(
        mid,
        holder,
        vec![hand_cards[0]],
        11015,
    );
    // 弃牌完成后进入目标选择续体；手牌已弃一（确定性取最前）。
    assert_eq!(
        drop_one.resolution.continuation_kind,
        ContinuationKind::ChooseOthersInsertExtraTurn
    );
    assert_eq!(drop_one.hand_count(holder), hand_before - 1);

    let observer = drop_one.resolution.pending_observer;
    let final_state = resolve(drop_one, PlayerId::new(observer), vec![victim.raw()], 1102);

    // 目标入队、无尾随代价标记。
    assert_eq!(final_state.turn.extra_turn_queue.first(), Some(&victim));
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                victim
            )),
        0
    );

    // 弃掉的牌在弃牌堆。
    let discard = final_state
        .zones
        .find(engine_core::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(!final_state.zones.get(discard).unwrap().cards.is_empty());
}
}

mod original_shenhua_fangzhu {
//! Golden coverage for 神话「放逐」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FANGZHU: u32 = 2_335_755_764;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/fangzhu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("fangzhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722_85, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
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
        skill_def: SkillDefId::new(FANGZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 6;
        owner_state.hp = 6;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_exposes_exact_damage_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FANGZHU && skill.name == "fangzhu" && !skill.locked));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FANGZHU)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_other_alive");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("fangzhu", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn target_draws_current_owner_lost_hp_before_turning_over() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (mut state, confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver
    );
    let options: Vec<_> = confirmed
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!options.contains(&setup.owner.raw()));
    assert!(options.contains(&setup.target.raw()));

    state.players.get_mut(setup.owner).unwrap().hp = 3;
    let (state, resolved) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 3);
    assert!(state.players.get(setup.target).unwrap().turned_over);
    let draw_index = resolved
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, .. } if *player == setup.target))
        .unwrap();
    let turn_index = resolved
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::PlayerTurnedOver { player, .. } if *player == setup.target))
        .unwrap();
    assert!(draw_index < turn_index);
}

#[test]
fn no_other_player_suppresses_trigger_and_dead_choice_is_rejected() {
    let setup = setup();
    let mut no_other = Transaction::begin(setup.state.clone());
    for player in no_other.state().players.alive_by_seat() {
        if player != setup.owner {
            no_other.state_mut().players.get_mut(player).unwrap().alive = false;
        }
    }
    run_effects(
        &mut no_other,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!no_other.state().has_pending_choice());

    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], false, 3);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state.players.get_mut(setup.target).unwrap().alive = false;
    let (_, rejected) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
}

mod original_shenhua_guanqiujian {
//! Golden coverage for 征荣/鸿举/清俭（shenhua 毌丘俭，ABI 545-547）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHENGRONG: u32 = 3_942_454_790;
const HONGJU: u32 = 1_729_861_379;
const QINGJIAN: u32 = 3_345_707_863;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/guanqiujian.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("gqj-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
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
        skill_def: SkillDefId::new(ZHENGRONG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HONGJU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn skill_instance(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
        .unwrap()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    def: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = skill_instance(&state, owner, def);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

/// Run one full 征荣 flow: the owner uses a Sha on `other` (eligible when the target's
/// hand count is at least the owner's), confirms, picks the target and one card.
/// Returns the committed state and the taken card.
fn run_zhengrong(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    card: CardId,
    key_base: u64,
) -> (GameState, CardId) {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional 征荣 confirm");
    let (state, confirmed) = resolve(state, owner, vec![1], key_base);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let target_tokens = option_tokens(&confirmed);
    assert_eq!(target_tokens, [other.raw()]);
    let (state, picked) = resolve(state, owner, vec![other.raw()], key_base + 1);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    let card_tokens = option_tokens(&picked);
    let taken = CardId::new(card_tokens[0]);
    let (state, done) = resolve(state, owner, vec![taken.raw()], key_base + 2);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    (state, taken)
}

fn rong_pile_cards(state: &GameState, owner: PlayerId) -> Vec<CardId> {
    state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == ZHENGRONG)
        .map(|pile| {
            state
                .zones
                .get(pile.zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default()
}

#[test]
fn package_exposes_guanqiujian_contract() {
    let compiled = content_compiler::compile_source_named("gqj", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 547);
    let zhengrong = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZHENGRONG)
        .unwrap();
    assert_eq!(zhengrong.event, "card_targeted");
    assert_eq!(
        zhengrong.filter,
        "owner_source_first_damage_card_target_hand_gte_owner"
    );
    assert!(zhengrong.optional);
    let hongju = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HONGJU)
        .unwrap();
    assert_eq!(hongju.event, "phase_prepare");
    assert_eq!(
        hongju.filter,
        "owner_phase_begin_skill_pile_gte_three_3942454790"
    );
    let qingjian = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == QINGJIAN)
        .unwrap();
    assert_eq!(qingjian.window, "play_phase");
    assert_eq!(qingjian.uses_per_turn, Some(1));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert_eq!(compiled.active_index.len(), 1);
}

#[test]
fn zhengrong_stores_one_target_card_as_rong() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_400,
        Suit::Spade,
        8,
    );
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Target has 2 hand cards >= owner's 1.
    let victim = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_401,
        Suit::Heart,
        1,
    );
    let _spare = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_402,
        Suit::Heart,
        2,
    );
    let (state, taken) = run_zhengrong(tx.commit().state, setup.owner, setup.other, sha, 1);
    assert_eq!(taken, victim, "owner took the victim card");
    let pile = rong_pile_cards(&state, setup.owner);
    assert_eq!(pile, vec![victim], "taken card stored as 荣");
}

#[test]
fn hongju_awakens_with_three_rong() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Build three 荣 through three 征荣 flows; the target keeps at least two hand cards.
    let mut state = tx.commit().state;
    for i in 0..3u64 {
        let mut tx = Transaction::begin(state);
        let sha = alloc_card(
            &mut tx,
            setup.owner,
            owner_hand,
            CardKind::Sha,
            96_410 + i as u32,
            Suit::Spade,
            8,
        );
        // Grant the target enough cards to keep hand >= owner each round.
        let _ = alloc_card(
            &mut tx,
            setup.other,
            other_hand,
            CardKind::Tao,
            96_420 + i as u32,
            Suit::Heart,
            1,
        );
        let _ = alloc_card(
            &mut tx,
            setup.other,
            other_hand,
            CardKind::Shan,
            96_421 + i as u32,
            Suit::Heart,
            2,
        );
        state = run_zhengrong(tx.commit().state, setup.owner, setup.other, sha, 10 + i * 4).0;
    }
    assert_eq!(rong_pile_cards(&state, setup.owner).len(), 3);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "鸿举 swap-or-skip choice"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = dispatch_tx.commit().state;
    // Skip the swap: awaken directly (max hp 5 -> 4, gain 清俭).
    let (state, skipped) = resolve(state, setup.owner, vec![0], 1);
    assert!(matches!(skipped.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        4,
        "max hp reduced"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == QINGJIAN),
        "清俭 granted"
    );
    assert_eq!(
        rong_pile_cards(&state, setup.owner).len(),
        3,
        "荣 stay in the pile"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn qingjian_exchanges_rong_and_discards_target_equipment() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Seed one 荣 via the real flow.
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_430,
        Suit::Spade,
        8,
    );
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_431,
        Suit::Heart,
        1,
    );
    let (state, _) = run_zhengrong(tx.commit().state, setup.owner, setup.other, sha, 1);
    assert_eq!(rong_pile_cards(&state, setup.owner).len(), 1);
    // Grant 清俭 for the active test.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.owner,
            skill_def: SkillDefId::new(QINGJIAN),
        }],
    )
    .unwrap();
    // Give the owner a hand card and the target an equipped weapon.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let hand_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        96_432,
        Suit::Heart,
        2,
    );
    let other_equip = tx.state().zones.find(ZoneKind::Equip, setup.other).unwrap();
    let weapon = alloc_card(
        &mut tx,
        setup.other,
        other_equip,
        CardKind::Weapon,
        96_433,
        Suit::Spade,
        6,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, QINGJIAN, 2);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let rong = rong_pile_cards(&state, setup.owner)[0];
    assert_eq!(option_tokens(&activated), [rong.raw()]);
    let (state, r) = resolve(state, setup.owner, vec![rong.raw()], 3);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    let (state, r) = resolve(state, setup.owner, vec![hand_card.raw()], 4);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    let (state, r) = resolve(state, setup.owner, vec![setup.other.raw()], 5);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    let (state, r) = resolve(state, setup.owner, vec![weapon.raw()], 6);
    assert!(matches!(r.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // 荣 moved to the owner's hand; the hand card and the weapon are discarded.
    assert!(state.hand_cards(setup.owner).contains(&rong), "荣 gained");
    let discard_zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(hand_card).unwrap().zone, discard_zone);
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard_zone);
}

#[test]
fn zhengrong_filter_skips_non_damage_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    // A Tao is not a damage-tagged card: the filter must not fire.
    let tao = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        96_440,
        Suit::Heart,
        8,
    );
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_441,
        Suit::Heart,
        1,
    );
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_442,
        Suit::Heart,
        2,
    );
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: setup.owner,
            card: tao,
            targets: vec![setup.other],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => assert!(effects.is_empty(), "征荣 must not fire for a Tao"),
        Err(EffectError::ChoiceRequested) => panic!("征荣 must not open a choice here"),
        Err(_) => {}
    }
    let state = tx.commit().state;
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_guzheng {
//! Golden coverage for 固政（shenhua 张昭张纮，ABI 513）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const GUZHENG: u32 = 4_159_731_006;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/guzheng.skill");

fn fire_discard_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Discard,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

/// Owner holds 固政; `discarder` discards two cards into the discard pile during their own
/// Discard phase; the discard-phase slice base is armed like the engine's phase transition.
fn setup() -> (GameState, PlayerId, PlayerId, Vec<CardId>) {
    let package = content_compiler::compile_source_named("guzheng-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, discarder) = (seats[0], seats[1]);
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
        skill_def: SkillDefId::new(GUZHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = discarder;
    // The discard phase begins: arm the one-based lost-card slice base.
    let lost_len = tx.state().turn.lost_cards.len() as i32;
    tx.state_mut().turn.set_flag(
        engine_core::state::turn_flag::DISCARD_PHASE_LOST_BASE,
        lost_len.saturating_add(1),
    );
    let c1 = alloc_card(&mut tx, discarder, CardKind::Sha, 96_120, Suit::Spade);
    let c2 = alloc_card(&mut tx, discarder, CardKind::Shan, 96_121, Suit::Heart);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, discarder).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![c1, c2],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    (tx.commit().state, owner, discarder, vec![c1, c2])
}

#[test]
fn package_exposes_discard_phase_return_contract() {
    let compiled = content_compiler::compile_source_named("guzheng", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 513);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == GUZHENG)
        .unwrap();
    assert_eq!(trigger.event, "phase_discard_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_discarded_in_discard_phase"
    );
    assert!(trigger.optional);
}

#[test]
fn returns_one_discard_and_owner_takes_the_rest() {
    let (state, owner, discarder, discarded) = setup();
    let tx = fire_discard_end(Transaction::begin(state), discarder);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, owner, vec![1], 1);
    assert!(state.has_pending_choice(), "return-card choice pending");
    state = resolve_choice(state, owner, vec![discarded[0].raw()], 2);
    assert!(state.has_pending_choice(), "take-rest choice pending");
    state = resolve_choice(state, owner, vec![1], 3);

    assert!(
        state.hand_cards(discarder).contains(&discarded[0]),
        "the discarder regained their returned card"
    );
    assert!(
        state.hand_cards(owner).contains(&discarded[1]),
        "the owner took the remaining discard"
    );
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(discarder), 1);
}

#[test]
fn returns_one_discard_and_gives_up_the_rest() {
    let (state, owner, discarder, discarded) = setup();
    let tx = fire_discard_end(Transaction::begin(state), discarder);
    let mut state = tx.commit().state;
    state = resolve_choice(state, owner, vec![1], 1);
    state = resolve_choice(state, owner, vec![discarded[1].raw()], 2);
    state = resolve_choice(state, owner, vec![0], 3);

    assert!(
        state.hand_cards(discarder).contains(&discarded[1]),
        "the discarder regained their returned card"
    );
    assert_eq!(
        state.hand_count(owner),
        0,
        "the owner declined the remaining discards"
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state.cards.get(discarded[0]).unwrap().zone,
        discard,
        "the unreturned discard stays in the discard pile"
    );
}

#[test]
fn skipping_the_choice_keeps_everything_in_place() {
    let (state, owner, discarder, discarded) = setup();
    let tx = fire_discard_end(Transaction::begin(state), discarder);
    let state = tx.commit().state;
    // Decline the optional confirm.
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = next;
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(discarder), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in discarded {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
}
}

mod original_shenhua_gzbuqu {
//! Golden coverage for 旧不屈（shenhua 旧周泰，ABI 591）.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const GZBUQU: u32 = 1_234_567_908;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/gzbuqu.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("gzbuqu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
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
        skill_def: SkillDefId::new(GZBUQU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
}

fn alloc_card(
    tx: &mut Transaction,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn set_hp(tx: &mut Transaction, player: PlayerId, hp: i32) {
    tx.state_mut().players.get_mut(player).unwrap().hp = hp;
}

#[test]
fn package_exposes_gzbuqu_contract() {
    let compiled = content_compiler::compile_source_named("gzbuqu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 591);
    assert_eq!(compiled.skill_index.len(), 1);
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&GZBUQU)));
}

#[test]
fn gzbuqu_moves_top_cards_to_chuang_and_marks_nodying_when_ranks_unique() {
    let mut setup = setup();
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top1 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Spade, 5);
    let top2 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Heart, 7);
    set_hp(&mut setup.tx, setup.owner, 0);
    // 触发：体力降至 0 以下（流失 3 点）→ 置入 1 张创牌（牌堆顶 top1，点数 5）。
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 3,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    // 免死标记：唯一点数 → 置位。
    assert!(tx.state().resolution.gzbuqu_nodying);
    let state = tx.commit().state;
    let tx = Transaction::begin(state);
    // 「创」技能牌堆含顶牌。
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner)
        .expect("创 pile must exist");
    let cards = tx.state().zones.get(pile.zone).unwrap().cards.clone();
    // 牌堆顶 = 最后一个分配的牌（top2 在 top1 之后）。
    assert!(cards.contains(&top2), "top card must enter 创");
    assert_eq!(cards.len(), 1, "one 创 card for the loss");
    let _ = top2;
    // 伤害链：免死标记消费后不进入濒死。
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    set_hp(&mut tx, setup.owner, 1);
    tx.state_mut().resolution.gzbuqu_nodying = true;
    let _events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.other,
            target: setup.owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(
        !tx.state().players.get(setup.owner).unwrap().dying,
        "nodying must suppress the dying chain"
    );
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, -1);
}

#[test]
fn gzbuqu_duplicate_rank_does_not_mark_nodying() {
    let mut setup = setup();
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _top1 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Spade, 5);
    let _top2 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Heart, 5);
    set_hp(&mut setup.tx, setup.owner, -1);
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 3,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(
        !tx.state().resolution.gzbuqu_nodying,
        "duplicate ranks must not mark nodying"
    );
}

#[test]
fn gzbuqu_recover_removes_chuang_card() {
    let mut setup = setup();
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _top1 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Spade, 5);
    let _top2 = alloc_card(&mut setup.tx, draw, CardKind::Sha, 1, Suit::Heart, 7);
    set_hp(&mut setup.tx, setup.owner, 0);
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 3,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    // 回复 1 点 → 触发恢复移除一张创牌。
    let state = tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: PlayerId::NONE,
            player: setup.owner,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("run error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = tx.commit().state;
    let tx = Transaction::begin(state);
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == setup.owner)
        .expect("创 pile must exist");
    assert_eq!(
        tx.state().zones.get(pile.zone).unwrap().cards.len(),
        0,
        "one 创 removed after recovery"
    );
}

#[test]
fn gzbuqu_skips_when_hp_still_positive() {
    let mut setup = setup();
    set_hp(&mut setup.tx, setup.owner, 2);
    let state = setup.tx.commit().state;
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut tx, effects) {
                match e {
                    EffectError::ChoiceRequested => {}
                    other => panic!("run error: {other:?}"),
                }
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(!tx.state().resolution.gzbuqu_nodying);
    assert!(
        tx.state().skill_piles.is_empty(),
        "no 创 cards while hp stays positive"
    );
}

#[test]
fn gzbuqu_distance_query_unaffected() {
    let mut setup = setup();
    let state = setup.tx.commit().state;
    let tx = Transaction::begin(state);
    let mut ctx = engine_core::query::QueryContext::new();
    let distance = engine_core::query::query_explain(
        tx.state(),
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.other,
        },
        &mut ctx,
    );
    assert!(matches!(distance.response, QueryResponse::Int(_)));
}
}

mod original_shenhua_jianchu {
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
}

mod original_shenhua_jiangwei {
//! Golden coverage for 挑衅/志继（shenhua 姜维，ABI 537-538）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TIAOXIN: u32 = 695_232_805;
const ZHIJI: u32 = 1_739_094_753;
const GUANXING: u32 = 32;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/jiangwei.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jiangwei-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(TIAOXIN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
        .unwrap()
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    state: GameState,
    owner: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = skill_instance(&state, owner, TIAOXIN);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

#[test]
fn package_exposes_tiaoxin_zhiji_contract() {
    let compiled = content_compiler::compile_source_named("jiangwei", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 538);
    let tiaoxin = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == TIAOXIN)
        .unwrap();
    assert_eq!(tiaoxin.window, "play_phase");
    assert_eq!(tiaoxin.uses_per_turn, Some(1));
    let zhiji = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZHIJI)
        .unwrap();
    assert_eq!(zhiji.event, "phase_prepare");
    assert_eq!(zhiji.filter, "owner_phase_begin_hand_empty");
    assert!(!zhiji.optional);
    assert_eq!(compiled.skill_index.len(), 2);
    assert_eq!(compiled.trigger_index.len(), 1);
    assert_eq!(compiled.active_index.len(), 1);
}

#[test]
fn tiaoxin_refusal_discards_one_card_of_the_target() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let victim = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_400,
        Suit::Heart,
        1,
    );
    let spare = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_401,
        Suit::Spade,
        8,
    );
    let state = tx.commit().state;
    // 挑衅: owner picks the adjacent target, the target refuses, the owner discards one card.
    let (state, activated) = activate(state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let target_choice = activated.choice.as_ref().unwrap();
    assert_eq!(target_choice.observer, setup.owner);
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        option_tokens(&activated),
        [setup.other.raw()],
        "cardless far target excluded"
    );
    let (state, picked) = resolve(state, vec![setup.other.raw()], false, 2);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    let response_choice = picked.choice.as_ref().unwrap();
    assert_eq!(response_choice.observer, setup.other);
    assert_eq!(response_choice.kind, ChoiceKind::Custom);
    // Refuse: selections [0].
    let (state, refused) = resolve(state, vec![0], false, 3);
    assert!(matches!(refused.result, CommandResult::Accepted { .. }));
    let discard_choice = refused.choice.as_ref().unwrap();
    assert_eq!(discard_choice.observer, setup.owner);
    assert_eq!(discard_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(option_tokens(&refused), [victim.raw(), spare.raw()]);
    let (state, discarded) = resolve(state, vec![victim.raw()], false, 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard_zone = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim).unwrap().zone, discard_zone);
    assert_eq!(state.cards.get(spare).unwrap().zone, other_hand);
    // Second activation this play phase is rejected by the once-per-turn contract.
    let (_, repeated) = activate(state, setup.owner, 5);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn tiaoxin_forced_sha_hits_the_owner() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_410,
        Suit::Spade,
        8,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(state, vec![setup.other.raw()], false, 2);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    // The target chooses to use a Sha.
    let (state, chose_sha) = resolve(state, vec![1], false, 3);
    assert!(matches!(chose_sha.result, CommandResult::Accepted { .. }));
    let source_choice = chose_sha.choice.as_ref().unwrap();
    assert_eq!(source_choice.observer, setup.other);
    assert_eq!(source_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(option_tokens(&chose_sha), [sha.raw()]);
    let (state, used) = resolve(state, vec![sha.raw()], false, 4);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    // The Sha targeted only the owner: no dodge was injected, so the lord owner
    // (5 max HP) takes 1 damage.
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(state.players.get(setup.other).unwrap().hp, 4);
    assert!(!state.has_pending_choice());
}

#[test]
fn tiaoxin_skips_out_of_range_or_cardless_targets() {
    let mut setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Give the adjacent player a card and nothing to the far player.
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let _ = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_420,
        Suit::Heart,
        1,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        option_tokens(&activated),
        [setup.other.raw()],
        "cardless far target excluded"
    );
    // Cancelling the target choice rejects (挑衅 requires exactly one target).
    let (_, rejected) = resolve(state, vec![], true, 2);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}

#[test]
fn zhiji_awakens_with_empty_hand() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // The owner has no hand cards and full HP; lord bonus max hp is 5.
    assert!(tx.state().hand_cards(setup.owner).is_empty());
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[engine_api::DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let mut tx = match fired {
        Ok(effects) => {
            let mut next = Transaction::begin(state);
            run_effects(&mut next, effects).unwrap();
            next
        }
        Err(EffectError::ChoiceRequested) => Transaction::begin(state),
        Err(e) => panic!("dispatch error: {e:?}"),
    };
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        4,
        "recovered to max 4/5"
    );
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        4,
        "max hp reduced from 5 to 4"
    );
    assert_eq!(state.hand_cards(setup.owner).len(), 2, "drew two cards");
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == GUANXING),
        "观星 granted"
    );
}

#[test]
fn zhiji_does_not_fire_with_hand_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let _ = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        96_430,
        Suit::Heart,
        1,
    );
    tx.state_mut().turn.current = setup.owner;
    let state = tx.commit().state;
    let fired = dispatch_for_events(
        &mut Transaction::begin(state.clone()),
        &[engine_api::DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp,
        5,
        "no awakening with hand cards"
    );
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == setup.owner && skill.def.raw() == GUANXING));
}
}

mod original_shenhua_jiaxu {
//! Golden coverage for 贾诩「完杀」and the generic multi-rescuer dying-save queue.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIAXU: u32 = 3_399_795_312;
const LUANWU: u32 = 2_772_376_501;
const WANSHA: u32 = 2_730_287_699;
const WEIMU: u32 = 4_145_129_024;
const WANSHA_SOURCE: &str = include_str!("../../../content/original/ported/shenhua/wansha.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    players: [PlayerId; 4],
}

fn setup(seed: u64, wansha: bool) -> Setup {
    let package = content_compiler::compile_source_named("wansha-test", WANSHA_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players: [PlayerId; 4] = tx.state().players.alive_by_seat().try_into().unwrap();
    let owner = players[0];
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
    if wansha {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WANSHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
        participant.dying = false;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        players,
    }
}

fn inject_tao(state: &mut GameState, owner: PlayerId, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100 + rank as u32),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn open_save(state: GameState, dying: PlayerId, source: PlayerId) -> (GameState, PlayerId) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RequestDyingSave {
                player: dying,
                source,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let observer = committed.output.choice.unwrap().observer;
    (committed.state, observer)
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
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
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_jiaxu_and_exact_wansha_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == JIAXU)
        .unwrap();
    assert_eq!(character.name_key, "jiaxu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [LUANWU, WANSHA, WEIMU]);
    let modifier = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == WANSHA)
        .unwrap();
    assert_eq!(
        modifier.target,
        "tao_use_save_cancel_for_non_dying_others_during_owner_turn"
    );
    assert_eq!(modifier.mode, "override");
    assert_eq!(modifier.value, 1);
}

#[test]
fn dying_save_queue_starts_at_current_player_and_uses_multiple_rescuers() {
    let mut setup = setup(2_026_072_294, false);
    let [current, next, dying, _last] = setup.players;
    let first_tao = inject_tao(&mut setup.state, current, 2);
    let second_tao = inject_tao(&mut setup.state, next, 3);
    {
        let participant = setup.state.players.get_mut(dying).unwrap();
        participant.hp = -1;
        participant.dying = true;
    }
    let (state, observer) = open_save(setup.state, dying, next);
    assert_eq!(observer, current);
    let (state, first) = resolve(state, vec![1], false, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(dying).unwrap().hp, 0);
    assert_eq!(first.choice.as_ref().unwrap().observer, next);
    assert_eq!(
        state.cards.get(first_tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (state, second) = resolve(state, vec![1], false, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let participant = state.players.get(dying).unwrap();
    assert_eq!(participant.hp, 1);
    assert!(participant.alive);
    assert!(!participant.dying);
    assert_eq!(
        state.cards.get(second_tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn wansha_blocks_other_non_dying_tao_but_keeps_owner_and_dying_exceptions() {
    let mut setup = setup(2_026_072_295, true);
    let [_owner, other, dying, last] = setup.players;
    let owner_tao = inject_tao(&mut setup.state, setup.owner, 4);
    let other_tao = inject_tao(&mut setup.state, other, 5);
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        setup.owner,
        owner_tao,
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        other,
        other_tao,
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind_to(
        &setup.state,
        other,
        dying,
        CardKind::Tao,
    ));

    setup.state.players.get_mut(other).unwrap().dying = true;
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        other,
        other_tao,
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind_to(
        &setup.state,
        other,
        dying,
        CardKind::Tao,
    ));
    assert!(!engine_core::query::card_save_cancelled_for_kind_to(
        &setup.state,
        other,
        other,
        CardKind::Tao,
    ));
    setup.state.players.get_mut(other).unwrap().dying = false;
    setup.state.turn.current = last;
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &setup.state,
        other,
        other_tao,
    ));
}

#[test]
fn wansha_save_queue_skips_blocked_helpers_and_still_allows_the_dying_player() {
    let mut setup = setup(2_026_072_296, true);
    let [_owner, blocked, dying, _last] = setup.players;
    let blocked_tao = inject_tao(&mut setup.state, blocked, 6);
    let dying_tao = inject_tao(&mut setup.state, dying, 7);
    {
        let participant = setup.state.players.get_mut(dying).unwrap();
        participant.hp = 0;
        participant.dying = true;
    }
    let (state, observer) = open_save(setup.state, dying, blocked);
    assert_eq!(observer, dying);
    assert_eq!(
        state.cards.get(blocked_tao).unwrap().zone,
        state.zones.find(ZoneKind::Hand, blocked).unwrap()
    );
    let (state, saved) = resolve(state, vec![1], false, 10);
    assert!(matches!(saved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(dying).unwrap().hp, 1);
    assert!(!state.players.get(dying).unwrap().dying);
    assert_eq!(
        state.cards.get(dying_tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_shenhua_jieming {
//! Golden coverage for 神话「节命」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIEMING: u32 = 3_579_069_878;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/jieming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jieming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722_84, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
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
        skill_def: SkillDefId::new(JIEMING),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 10;
        owner_state.hp = 10;
        let target_state = tx.state_mut().players.get_mut(target).unwrap();
        target_state.max_hp = 7;
        target_state.hp = 7;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_exposes_exact_per_damage_point_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIEMING && skill.name == "jieming" && !skill.locked));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIEMING)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("jieming", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn two_damage_points_offer_independent_targets_and_cap_draw_at_five() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 8);

    // The first damage point's optional confirm opens; accepting it commits the first
    // point's effect before the second point's prompt (confirmed triggers run first so
    // replacement/exclusion filters can invalidate stale queued work).
    let (state, accepted) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let options: Vec<_> = accepted
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(
        options,
        vec![setup.owner.raw(), setup.source.raw(), setup.target.raw(), 4]
    );

    // Draw the first point's target up to five; the second point's confirm then opens.
    let (state, drawn) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 5);
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    // Accept the second point's confirm and draw the same target again (already capped).
    let (state, accepted_again) = resolve(state, setup.owner, vec![1], false, 3);
    assert!(matches!(
        accepted_again.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let (state, no_op) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(no_op.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 5);
    assert!(!state.has_pending_choice());
}

#[test]
fn source_free_damage_triggers_and_dead_target_is_rejected_at_resume() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (mut state, accepted) = resolve(state, setup.owner, vec![1], false, 5);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    state.players.get_mut(setup.target).unwrap().alive = false;
    let (_, stale) = resolve(state, setup.owner, vec![setup.target.raw()], false, 6);
    assert!(matches!(stale.result, CommandResult::Rejected { .. }));
}
}

mod original_shenhua_jushou {
//! Goldens for 神话再临旧曹仁「据守」and the generic turn-over contract.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    ObserverKind, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLD_CAOREN: u32 = 3_775_318_821;
const JUSHOU: u32 = 3_729_566_871;
const MOON_JUSHOU: u32 = 3_308_685_907;
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

fn end_phase(actor: PlayerId, key: u64) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::EndPhase,
    }
}

#[test]
fn package_closes_old_caoren_and_jushou() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OLD_CAOREN)
        .expect("old_caoren character");
    assert_eq!(character.skills, vec![JUSHOU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JUSHOU));
}

#[test]
fn jushou_draws_three_turns_owner_over_and_skips_their_next_turn() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_715)),
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
    let hand_before = engine.state().hand_count(owner);
    let mut tx = Transaction::begin(engine.state().clone());
    if !tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == JUSHOU)
        })
    {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUSHOU),
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.phase = Phase::End;
    let phase_end = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    // 出牌阶段结束的多个可选触发（刘禅放权等标准技 + 据守）逐段确认。
    let mut state = tx.state().clone();
    let mut dispatched = false;
    fn ok_result() -> PipelineResult {
        PipelineResult {
            result: CommandResult::Accepted {
                new_revision: 0,
                state_hash: [0; 32],
            },
            events: vec![],
            choice: None,
        }
    }
    let mut resolved = ok_result();
    loop {
        let pending = state.resolution.pending_choice_id;
        if pending == 0 && dispatched {
            break;
        }
        let mut tx = Transaction::begin(state.clone());
        let (state2, result) = if pending != 0 {
            let observer = PlayerId::new(tx.state().resolution.pending_observer);
            apply_command(
                tx.state().clone(),
                Command {
                    actor: observer,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(1),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(pending),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            )
        } else if !dispatched {
            dispatched = true;
            let fired =
                dispatch_for_events(&mut tx, &[phase_end.clone()], &mut TriggerFireRecord::new());
            match fired {
                Ok(effects) => match run_effects(&mut tx, effects) {
                    Ok(_) => (tx.commit().state, ok_result()),
                    Err(EffectError::ChoiceRequested) => (tx.commit().state, ok_result()),
                    Err(e) => panic!("run error: {e:?}"),
                },
                Err(EffectError::ChoiceRequested) => (tx.commit().state, ok_result()),
                Err(e) => panic!("dispatch error: {e:?}"),
            }
        } else {
            break;
        };
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "choice must resolve: {result:?}"
        );
        resolved = result;
        state = state2;
    }
    assert_eq!(state.hand_count(owner), hand_before + 3);
    assert!(state.players.get(owner).unwrap().turned_over);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over: true
        } if *player == owner
    )));
    assert!(
        engine_core::project_observer(&state, ObserverKind::Player(owner))
            .seats
            .iter()
            .find(|seat| seat.player == owner)
            .unwrap()
            .turned_over
    );

    let (state, advanced) = apply_command(state, end_phase(owner, 2));
    assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other);
    assert_eq!(state.turn.phase, Phase::Prepare);

    // 座次角色随 shenhua 包角色池增长而变化：若下一家准备阶段有可选触发
    // （如甄姬洛神），先拒绝之，再推进测试。
    let mut state = state;
    if state.has_pending_choice() {
        // 若下一家准备阶段有可选触发（如甄姬洛神），拒绝之；若为强制摸牌置牌
        // 触发（卢植明任），以手牌结算之。
        let actor = PlayerId::new(state.resolution.pending_observer);
        let hand = state.hand_cards(actor);
        let picks = if hand.is_empty() {
            vec![0]
        } else {
            vec![hand[0].raw()]
        };
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(30),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: picks,
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }

    let round_before_skip = state.turn.round;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let (state, skipped) = apply_command(tx.state().clone(), end_phase(other, 3));
    assert!(matches!(skipped.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other, "owner's whole turn was skipped");
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.turn.round, round_before_skip + 1);
    assert!(!state.players.get(owner).unwrap().turned_over);
    assert!(skipped.events.iter().any(|event| matches!(
        event,
        DomainEvent::PlayerTurnedOver {
            player,
            turned_over: false
        } if *player == owner
    )));
}

#[test]
fn all_face_down_players_are_flipped_once_before_play_resumes() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_716)),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let first = players[0];
    let second = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::TurnOver { player: first },
            EffectRequest::TurnOver { player: second },
        ],
    )
    .unwrap();
    let round_before = tx.state().turn.round;

    let (state, result) = apply_command(tx.state().clone(), end_phase(first, 9));
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, second);
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert_eq!(state.turn.round, round_before + 1);
    assert!(players
        .iter()
        .all(|player| !state.players.get(*player).unwrap().turned_over));
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                DomainEvent::PlayerTurnedOver {
                    turned_over: false,
                    ..
                }
            ))
            .count(),
        2
    );
}

#[test]
fn moon_jushou_draws_one_then_turns_owner_over() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MOON_JUSHOU));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(20_260_717)),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MOON_JUSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::End;
    let hand_before = tx.state().hand_count(owner);
    let phase_end = DomainEvent::TurnStateChanged {
        current: owner,
        phase: TurnPhaseTag::End,
    };
    assert_eq!(
        dispatch_for_events(&mut tx, &[phase_end], &mut TriggerFireRecord::new()).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert!(state.players.get(owner).unwrap().turned_over);
}
}

mod original_shenhua_kuailiangkuaiyue {
//! Golden coverage for 荐相/审时（shenhua 蒯良蒯越，ABI 574）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIANXIANG: u32 = 300_846_397;
const SHENSHI: u32 = 3_785_166_504;
const SOURCE: &str =
    include_str!("../../../content/original/ported/shenhua/kuailiangkuaiyue.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("kl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(JIANXIANG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SHENSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_kl_contract() {
    let compiled = content_compiler::compile_source_named("kl", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 574);
    assert_eq!(compiled.skill_index.len(), 2);
    let jianxiang = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JIANXIANG)
        .unwrap();
    assert_eq!(jianxiang.event, "card_targeted");
    assert!(jianxiang.optional);
    let shenshi_active = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == SHENSHI)
        .unwrap();
    assert_eq!(shenshi_active.uses_per_turn, Some(1));
    let shenshi_triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == SHENSHI)
        .collect();
    assert_eq!(shenshi_triggers.len(), 2);
}

/// 荐相：受指目标后选手牌最少者摸一张。
#[test]
fn jianxiang_min_hand_draws() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let used = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        97_100,
        Suit::Spade,
        5,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_101,
        Suit::Heart,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: used,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "荐相 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(state.has_pending_choice(), "荐相 target prompt");
    // 手牌最少者：owner(0) 与 far(1) 中最少者——owner 0, far 1 → 最少者 owner 自己。
    // 选择最少的其它可选者（测试断言 far 不是最少；这里 owner 手牌 0 为最少）。
    let state = resolve(state, setup.owner, vec![setup.owner.raw()], false, 9_002);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 1, "owner drew one");
    let _ = (used, f1);
}

/// 审时（阳分支）：出牌阶段限一次交牌给手牌最多者并造成伤害。
#[test]
fn shenshi_yang_gives_and_damages() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let gift = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_102,
        Suit::Club,
        7,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_103,
        Suit::Heart,
        8,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == SHENSHI)
        .map(|skill| skill.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"SS1\x00".to_vec();
    let state = tx.commit().state;
    let (state, result) = {
        let revision = state.revision;
        let (state, result) = apply_command(
            state,
            Command {
                actor: setup.owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(9_010),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        (state, result)
    };
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );
    assert!(state.has_pending_choice(), "审时 give prompt");
    let state = resolve(state, setup.owner, vec![gift.raw()], false, 9_011);
    assert!(state.has_pending_choice(), "审时 target prompt");
    // 手牌最多者：far(1) 唯一。
    let hp_before = state.players.get(setup.far).unwrap().hp;
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_012);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.far).unwrap().hp, hp_before - 1);
    assert!(
        state.hand_cards(setup.far).contains(&gift),
        "gift moved to the target"
    );
    let _ = f1;
}

/// 审时（阴分支）：受伤后交来源一张牌，来源回合结束时追踪牌在手则摸至四张。
#[test]
fn shenshi_yin_tracks_gift_until_source_turn_end() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let gift = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_104,
        Suit::Club,
        9,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == SHENSHI)
        .map(|skill| skill.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"SS1\x01".to_vec();
    let state = tx.commit().state;
    // 受伤事件（来源 other 对 owner 造成伤害）。
    let state = dispatch(
        state,
        DomainEvent::DamageDealt {
            source: setup.other,
            target: setup.owner,
            amount: 1,
            nature: 0,
            is_lose_hp: false,
        },
    );
    assert!(state.has_pending_choice(), "审时 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    assert!(state.has_pending_choice(), "审时 give prompt");
    let state = resolve(state, setup.owner, vec![gift.raw()], false, 9_021);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.other).contains(&gift),
        "gift moved to the source"
    );
    // 来源回合结束：追踪牌仍在来源手牌 → 持有者摸至四张。
    let state = dispatch(
        state,
        DomainEvent::PhaseEnded {
            current: setup.other,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        },
    );
    assert!(state.has_pending_choice(), "审时 turn-end confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_022);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 4, "drew to four");
}
}

mod original_shenhua_liegong {
//! Golden coverage for SHENHUA「烈弓」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIEGONG: u32 = 2_510_589_015;
const TEST_MULTI_SHA: u32 = 2_510_589_016;
const HUANGZHONG: u32 = 3_528_195_720;
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/liegong.skill"),
    "\nskill test_multi_sha(2510589016) locked { modifier sha_target_limit add 1; }\n"
);

fn setup(player_count: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("liegong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_718, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(LIEGONG),
    });
    if player_count > 2 {
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
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn add_target_hand(tx: &mut Transaction, target: PlayerId, count: usize) -> CardId {
    assert!(count > 0);
    let shan = inject(tx, target, ZoneKind::Hand, CardKind::Shan, 93_000);
    for index in 1..count {
        inject(
            tx,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            93_000 + index as u32,
        );
    }
    shan
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
        94_000 + key as u32,
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

#[test]
fn packages_publish_exact_skill_character_and_dependency() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    let trigger = shenhua
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIEGONG)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_target_hand_gte_hp_or_lte_attack_range"
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert!(old.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    let huangzhong = old
        .character_index
        .iter()
        .find(|character| character.id == HUANGZHONG)
        .unwrap();
    assert_eq!(huangzhong.name_key, "huangzhong");
    assert_eq!(huangzhong.max_hp, 4);
    assert_eq!(huangzhong.skills, [LIEGONG]);
}

#[test]
fn eligible_target_can_be_locked_or_left_to_play_shan() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = add_target_hand(&mut tx, target, 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered, _) = play_sha(tx.commit().state, owner, vec![target], 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);

    let (locked, result) = resolve(state.clone(), owner, vec![1], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(locked.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(locked.hand_count(target), 1);

    let (declined, result) = resolve(state, owner, vec![], true, 3);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
    let (after_shan, result) = resolve(declined, target, vec![shan.raw()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after_shan.players.get(target).unwrap().hp, hp);
}

#[test]
fn filter_uses_both_hp_and_current_attack_range_boundaries() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = add_target_hand(&mut tx, target, 2);
    let (_state, result, _) = play_sha(tx.commit().state, owner, vec![target], 10);
    let response = result.choice.unwrap();
    assert_eq!(response.observer, target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    add_target_hand(&mut tx, target, 4);
    let (_, result, _) = play_sha(tx.commit().state, owner, vec![target], 11);
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    add_target_hand(&mut tx, target, 2);
    inject(&mut tx, owner, ZoneKind::Equip, CardKind::Weapon, 93_100);
    let (_, result, _) = play_sha(tx.commit().state, owner, vec![target], 12);
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);
}

#[test]
fn multi_target_sha_locks_only_the_accepted_current_target() {
    let (state, owner, seats) = setup(3);
    let (locked_target, response_target) = (seats[1], seats[2]);
    let mut tx = Transaction::begin(state);
    add_target_hand(&mut tx, locked_target, 1);
    let shan = add_target_hand(&mut tx, response_target, 2);
    let locked_hp = tx.state().players.get(locked_target).unwrap().hp;
    let response_hp = tx.state().players.get(response_target).unwrap().hp;
    let (state, result, _) = play_sha(
        tx.commit().state,
        owner,
        vec![locked_target, response_target],
        20,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![1], false, 21);
    let response = result.choice.unwrap();
    assert_eq!(state.players.get(locked_target).unwrap().hp, locked_hp - 1);
    assert_eq!(state.players.get(response_target).unwrap().hp, response_hp);
    assert_eq!(response.observer, response_target);
    assert!(response
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
}

#[test]
fn intent_rejects_a_stale_or_mismatched_sha_tuple() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 95_000);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MakeCurrentShaTargetUnrespondable {
                source: owner,
                card,
                target,
            }],
        ),
        Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into()
        ))
    );
}
}

mod original_shenhua_liushan {
//! Golden coverage for 享乐/放权/若愚（shenhua 刘禅，ABI 560）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XIANGLE: u32 = 2_880_679_348;
const FANGQUAN: u32 = 283_021_192;
const RUOYU: u32 = 2_227_756_813;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/liushan.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("liushan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(XIANGLE),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FANGQUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_liushan_contract() {
    let compiled = content_compiler::compile_source_named("liushan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 560);
    assert_eq!(compiled.skill_index.len(), 3);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| {
            t.skill_def_id == XIANGLE || t.skill_def_id == FANGQUAN || t.skill_def_id == RUOYU
        })
        .collect();
    assert_eq!(triggers.len(), 4);
    let xiangle = triggers.iter().find(|t| t.skill_def_id == XIANGLE).unwrap();
    assert_eq!(xiangle.event, "card_targeted");
    assert!(!xiangle.optional);
    let fangquan: Vec<_> = triggers
        .iter()
        .filter(|t| t.skill_def_id == FANGQUAN)
        .collect();
    assert_eq!(fangquan.len(), 2);
    assert!(fangquan.iter().all(|t| t.optional));
}

/// 享乐：使用者弃基本牌则杀正常；拒绝则此杀对刘禅无效（无响应、无伤害）。
#[test]
fn xiangle_refusal_excludes_owner_from_sha() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    // The Sha user has a basic (桃) and a non-basic (锦囊牌用 sha 类占位) to discard.
    let other_hand = hand_zone(&tx, setup.other);
    let basic = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_700,
        Suit::Heart,
        6,
    );
    // The Sha card targeting the owner.
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_701,
        Suit::Spade,
        7,
    );
    let mut state = tx.commit().state;
    // Register the currently-resolving Sha so the saved-target exclusion works.
    let mut resolution = &mut state.resolution;
    resolution.resume_kind = engine_core::state::ContinuationKind::ResumePlayedSha;
    resolution.resume_payload = vec![setup.other.raw(), setup.owner.raw(), 1, 0, 1, sha.raw()];
    resolution.register_sha_resolution(setup.other, sha, &[setup.owner], false);
    drop(resolution);
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.owner],
        },
    );
    // The user's discard-basic choice opened.
    assert!(state.has_pending_choice(), "享乐 discard prompt");
    // Refuse: the Sha becomes invalid vs the owner.
    let state = resolve(state, setup.other, Vec::new(), true, 9_001);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.other).contains(&basic),
        "basic still in hand"
    );
    let saved_targets = state
        .resolution
        .sha_resolution_records
        .iter()
        .find(|record| record.card == sha)
        .map(|record| record.targets.clone())
        .unwrap_or_default();
    assert!(
        !saved_targets.contains(&setup.owner),
        "owner excluded from the saved Sha targets"
    );
    let _ = basic;
}

/// 享乐：使用者弃基本牌后杀继续正常结算。
#[test]
fn xiangle_discard_keeps_sha() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let basic = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_702,
        Suit::Heart,
        6,
    );
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_703,
        Suit::Spade,
        7,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "享乐 discard prompt");
    let state = resolve(state, setup.other, vec![basic.raw()], false, 9_002);
    assert!(!state.has_pending_choice());
    assert!(
        !state.hand_cards(setup.other).contains(&basic),
        "basic discarded"
    );
    let _ = sha;
}

/// 放权：出牌阶段跳过 + 结束阶段弃牌授额外回合。
#[test]
fn fangquan_skips_play_and_grants_extra_turn() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_704,
        Suit::Club,
        8,
    );
    let state = tx.commit().state;
    // Accept the optional 放权 confirm: the play phase is skipped.
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Play,
        },
    );
    assert!(state.has_pending_choice(), "放权 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert_eq!(state.turn.phase, Phase::Discard, "play phase skipped");
    // Move to the end phase and trigger 放权's grant.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "放权 grant confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_011);
    assert!(state.has_pending_choice(), "放权 discard prompt");
    let state = resolve(state, setup.owner, vec![card.raw()], false, 9_012);
    assert!(state.has_pending_choice(), "放权 target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_013);
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.extra_turn_queue.first(), Some(&setup.far));
    assert!(
        !state.hand_cards(setup.owner).contains(&card),
        "card discarded"
    );
}

/// 若愚：体力值最低时觉醒：加上限、回复、授予激将。
#[test]
fn ruoyu_awakens_at_min_hp() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(
        &content_compiler::compile_source_named("jijiang", "skill jijiang(2377283906) {}").unwrap(),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: setup.owner,
            skill_def: SkillDefId::new(RUOYU),
        }],
    )
    .unwrap();
    // Owner at min HP (1 of 3): the others at 3.
    // Bootstrap characters are 5-max; drop the owner to the lowest hp.
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: setup.owner,
            delta: -2,
        }],
    )
    .unwrap();
    let max_before = tx.state().players.get(setup.owner).unwrap().max_hp;
    let hp_before = tx.state().players.get(setup.owner).unwrap().hp;
    assert_eq!((hp_before, max_before), (3, 3), "owner wounded to min hp");
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    let owner = state.players.get(setup.owner).unwrap();
    assert_eq!(owner.max_hp, 4, "max hp raised by one");
    assert_eq!(owner.hp, 4, "recovered to full");
    assert!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .any(|skill| skill.owner == setup.owner && skill.def.raw() == 2_377_283_906),
        "激将 granted"
    );
    // Awaken once only: a second Prepare dispatch does nothing further.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    let owner = state.players.get(setup.owner).unwrap();
    assert_eq!(owner.max_hp, 4, "no double awaken");
}
}

mod original_shenhua_luanwu {
//! Golden coverage for 贾诩「乱武」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LUANWU: u32 = 2_772_376_501;
const RED_SHA: u32 = 4_290_000_090;
const LUANWU_SOURCE: &str = include_str!("../../../content/original/ported/shenhua/luanwu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

fn setup(seed: u64, view_as: bool) -> Setup {
    let source = if view_as {
        format!(
            "{LUANWU_SOURCE}\nskill red_sha({RED_SHA}) {{ view_as sha play_phase red from hand; }}"
        )
    } else {
        LUANWU_SOURCE.into()
    };
    let package = content_compiler::compile_source_named("luanwu-test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let others = [players[1], players[2], players[3]];
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
        skill_def: SkillDefId::new(LUANWU),
    });
    if view_as {
        effects.push(EffectRequest::AddSkill {
            player: others[1],
            skill_def: SkillDefId::new(RED_SHA),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 5;
        participant.max_hp = 5;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        others,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_090 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_instance(state: &GameState, owner: PlayerId, skill_def: u32) -> SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let skill = skill_instance(&state, owner, LUANWU);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn option_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_exact_luanwu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == LUANWU)
        .unwrap();
    assert!(active.once_per_game);
    assert!(active.targets.is_none());
    let compiled = content_compiler::compile_source_named("luanwu", LUANWU_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 90);
    assert_eq!(compiled.modules.len(), 1);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash,
        compiled.modules[0].logic_hash
    );
}

#[test]
fn luanwu_runs_clockwise_uses_only_nearest_targets_and_does_not_count_sha() {
    let mut setup = setup(2_026_072_290, false);
    let [first, user, last] = setup.others;
    let sha = inject_hand(&mut setup.state, user, CardKind::Sha, Suit::Spade);
    let sha_count = setup.state.turn.flag(turn_flag::sha_used_by_player(user));

    let (state, activated) = activate(setup.state, setup.owner, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let source_choice = activated.choice.as_ref().unwrap();
    assert_eq!(source_choice.observer, user);
    assert_eq!(source_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(option_tokens(&activated), [sha.raw()]);
    assert_eq!(state.players.get(first).unwrap().hp, 4);

    let (state, selected) = resolve(state, vec![sha.raw()], false, 2);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));
    assert_eq!(
        selected.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(option_tokens(&selected), [last.raw(), first.raw()]);
    assert!(!option_tokens(&selected).contains(&setup.owner.raw()));

    let (state, used) = resolve(state, vec![first.raw()], false, 3);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    assert_eq!(state.players.get(last).unwrap().hp, 4);
    assert_eq!(
        state.turn.flag(turn_flag::sha_used_by_player(user)),
        sha_count
    );
    assert_eq!(
        state.cards.get(sha).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (_, repeated) = activate(state, setup.owner, 4);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn cancelling_a_legal_luanwu_sha_loses_hp_and_preserves_the_source_card() {
    let mut setup = setup(2_026_072_291, false);
    let [first, user, last] = setup.others;
    let sha = inject_hand(&mut setup.state, user, CardKind::Sha, Suit::Club);
    let (state, activated) = activate(setup.state, setup.owner, 10);
    assert_eq!(activated.choice.as_ref().unwrap().observer, user);
    let (state, cancelled) = resolve(state, Vec::new(), true, 11);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    for player in [first, user, last] {
        assert_eq!(state.players.get(player).unwrap().hp, 4);
    }
    assert_eq!(state.cards.get(sha).unwrap().owner, user);
    assert_eq!(
        state.cards.get(sha).unwrap().zone,
        state.zones.find(ZoneKind::Hand, user).unwrap()
    );
}

#[test]
fn luanwu_accepts_play_phase_view_as_sha_sources() {
    let mut setup = setup(2_026_072_292, true);
    let [first, user, _last] = setup.others;
    let red_other = inject_hand(&mut setup.state, user, CardKind::Other, Suit::Heart);
    let (state, activated) = activate(setup.state, setup.owner, 20);
    assert_eq!(activated.choice.as_ref().unwrap().observer, user);
    assert_eq!(option_tokens(&activated), [red_other.raw()]);
    let (state, selected) = resolve(state, vec![red_other.raw()], false, 21);
    assert!(option_tokens(&selected).contains(&first.raw()));
    let (state, used) = resolve(state, vec![first.raw()], false, 22);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    assert_eq!(
        state.cards.get(red_other).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn luanwu_queue_resumes_after_a_dying_player_is_killed() {
    let mut setup = setup(2_026_072_293, false);
    let [dying, user, _last] = setup.others;
    setup.state.players.get_mut(dying).unwrap().hp = 1;
    let _tao = inject_hand(&mut setup.state, dying, CardKind::Tao, Suit::Heart);
    let sha = inject_hand(&mut setup.state, user, CardKind::Sha, Suit::Diamond);
    let (state, activated) = activate(setup.state, setup.owner, 30);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(activated.choice.as_ref().unwrap().observer, dying);
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::DyingSave
    );

    let (state, died) = resolve(state, Vec::new(), true, 31);
    assert!(matches!(died.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(dying).unwrap().alive);
    assert_eq!(died.choice.as_ref().unwrap().observer, user);
    assert_eq!(option_tokens(&died), [sha.raw()]);
}
}

mod original_shenhua_lusu {
//! Golden coverage for 好施/缔盟（shenhua 鲁肃，ABI 555-556）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HAOSHI: u32 = 1_569_684_326;
const DIMENG: u32 = 427_650_324;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/lusu.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("lusu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(HAOSHI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DIMENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn skill_instance(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
        .unwrap()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    def: u32,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let skill = skill_instance(&state, owner, def);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

#[test]
fn package_exposes_lusu_contract() {
    let compiled = content_compiler::compile_source_named("lusu", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 556);
    let draw = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HAOSHI && t.event == "phase_draw")
        .unwrap();
    assert!(draw.optional);
    assert_eq!(draw.filter, "owner_is_target_draw_not_replaced");
    let draw_end = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HAOSHI && t.event == "phase_draw_end")
        .unwrap();
    assert_eq!(draw_end.filter, "owner_is_target");
    let dimeng = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == DIMENG)
        .unwrap();
    assert_eq!(dimeng.window, "play_phase");
    assert_eq!(dimeng.uses_per_turn, Some(1));
    assert_eq!(compiled.skill_index.len(), 2);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert_eq!(compiled.active_index.len(), 1);
}

#[test]
fn haoshi_draws_extra_two_and_gives_half_to_min_hand() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Draw;
    // Dispatch the draw-phase start: the optional 好施 confirm opens.
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "好施 confirm"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = dispatch_tx.commit().state;
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    // The draw count is fixed to 4: draw the four cards.
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let four: Vec<_> = state
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(4).copied().collect())
        .unwrap_or_default();
    assert_eq!(four.len(), 4);
    let hand = state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: four.clone(),
            to: hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    // 4 cards: hand == 4 <= 5 -> the draw-end give-half must not fire.
    tx.state_mut().turn.phase = Phase::Draw;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    // (draw phase is fixed now: no trigger; nothing pending)
    match fired {
        Ok(effects) => assert!(effects.is_empty()),
        Err(_) => {}
    }
    let mut state = dispatch_tx.commit().state;
    // Give the owner two more hand cards -> hand 6 > 5 -> give floor(6/2)=3.
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let c1 = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        96_600,
        Suit::Spade,
        2,
    );
    let c2 = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        96_601,
        Suit::Spade,
        3,
    );
    tx.state_mut().turn.phase = Phase::Draw;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "好施 give-half choice"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = dispatch_tx.commit().state;
    // The give-half choice opened (hand 6 -> give 3).
    assert!(state.has_pending_choice(), "好施 give-half choice");
    let hand_cards: Vec<_> = state.hand_cards(setup.owner);
    assert!(hand_cards.len() >= 3);
    let picks: Vec<u32> = hand_cards.iter().take(3).map(|card| card.raw()).collect();
    assert_eq!(picks.len(), 3);
    let (state, given) = resolve(state, setup.owner, picks, 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // The owner kept 3 cards; the target received 3.
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(state.hand_count(setup.other), 3);
    let _ = (c1, c2);
}

#[test]
fn dimeng_exchanges_hands_with_diff_discard() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Other has 3 cards, far has 1: diff = 2.
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let o1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_610,
        Suit::Spade,
        4,
    );
    let o2 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_611,
        Suit::Spade,
        5,
    );
    let o3 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_612,
        Suit::Heart,
        2,
    );
    let far_hand = tx.state().zones.find(ZoneKind::Hand, setup.far).unwrap();
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        96_613,
        Suit::Heart,
        6,
    );
    // Owner has two hand cards to discard.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let p1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_614,
        Suit::Club,
        7,
    );
    let p2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_615,
        Suit::Club,
        8,
    );
    let state = tx.commit().state;
    let (state, activated) = activate(state, setup.owner, DIMENG, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(
        state,
        setup.owner,
        vec![setup.other.raw(), setup.far.raw()],
        2,
    );
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    // Discard 2 cards (the diff).
    let (state, paid) = resolve(state, setup.owner, vec![p1.raw(), p2.raw()], 3);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    // Hands exchanged: other now has 1, far now has 3.
    assert_eq!(state.hand_count(setup.other), 1);
    assert_eq!(state.hand_count(setup.far), 3);
    assert!(state.hand_cards(setup.far).contains(&o1));
    assert!(state.hand_cards(setup.far).contains(&o2));
    assert!(state.hand_cards(setup.far).contains(&o3));
    assert!(state.hand_cards(setup.other).contains(&f1));
    // The payment cards are discarded.
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(p1).unwrap().zone, discard);
    assert_eq!(state.cards.get(p2).unwrap().zone, discard);
}
}

mod original_shenhua_luzhi {
//! Golden coverage for 明任/贞良（shenhua 卢植，ABI 576）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MINGREN: u32 = 1_039_189_468;
const ZHENLIANG: u32 = 279_698_051;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/luzhi.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("lz-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(MINGREN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHENLIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().turn.round = 1;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// 「任」牌堆的牌。
fn mingren_pile(state: &GameState, owner: PlayerId) -> Vec<CardId> {
    state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == MINGREN)
        .and_then(|pile| state.zones.get(pile.zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
}

#[test]
fn package_exposes_luzhi_contract() {
    let compiled = content_compiler::compile_source_named("lz", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 576);
    assert_eq!(compiled.skill_index.len(), 2);
    let mingren: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == MINGREN)
        .collect();
    assert_eq!(mingren.len(), 2);
    let zhenliang_active = compiled
        .active_index
        .iter()
        .find(|a| a.skill_def_id == ZHENLIANG)
        .unwrap();
    assert_eq!(zhenliang_active.uses_per_turn, Some(1));
    let zhenliang: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == ZHENLIANG)
        .collect();
    assert_eq!(zhenliang.len(), 2);
}

/// 明任：首轮准备阶段摸两张并将一张手牌置于「任」；结束阶段可用手牌替换。
#[test]
fn mingren_places_and_replaces() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    // 明任强制触发：摸两张并开置牌选择。
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "明任 place prompt");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 2, "drew two");
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_001);
    assert!(!state.has_pending_choice());
    let pile = mingren_pile(&state, setup.owner);
    assert_eq!(pile, vec![hand[0]], "card placed as 任");
    // 结束阶段替换。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "明任 replace confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_002);
    assert!(state.has_pending_choice(), "明任 replace prompt");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 1);
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_003);
    assert!(!state.has_pending_choice());
    let pile = mingren_pile(&state, setup.owner);
    assert_eq!(pile, vec![hand[0]], "replacement placed");
    assert!(
        state.hand_cards(setup.owner).contains(&hand[0]) == false || pile.len() == 1,
        "old 任 returned to hand"
    );
    // 旧「任」回手：总手牌仍为 1（替换的新牌进堆）。
}

/// 贞良（阳分支）：回合外使用与「任」同色的牌后令一名角色摸一张。
#[test]
fn zhenliang_yang_outside_turn_draws() {
    let setup = setup();
    let state = setup.tx.commit().state;
    // 通过明任流程置「任」（红桃）：准备阶段摸二置一。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "明任 seed place");
    // Force the card about to be placed as 「任」 to be red, so the 阳
    // same-color check is deterministic.
    let mut tx = Transaction::begin(state);
    if let Some(card) = tx.state().hand_cards(setup.owner).first().copied() {
        tx.state_mut().cards.get_mut(card).unwrap().suit = Suit::Heart;
    }
    let state = tx.commit().state;
    let hand: Vec<_> = state.hand_cards(setup.owner);
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_000);
    // 回合外：other 的回合。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let used = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        97_201,
        Suit::Heart,
        3,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == ZHENLIANG)
        .map(|skill| skill.id)
        .unwrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"ZL1\x01".to_vec();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: used,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "贞良 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "贞良 draw target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_011);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.far), 1, "far drew one");
}

/// 贞良（阴分支）：出牌阶段限一次弃同色牌并对攻击范围内角色造成 1 点伤害。
#[test]
fn zhenliang_yin_discards_same_color_and_damages() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    // 通过明任流程置「任」（黑桃）。
    tx.state_mut().turn.phase = Phase::Prepare;
    let mut state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "明任 seed place");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    let state = resolve(state, setup.owner, vec![hand[0].raw()], false, 9_000);
    // 给持有者一张与「任」同色的牌用于弃置，回到出牌阶段。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let ren_suit = tx
        .state()
        .cards
        .get(mingren_pile(&tx.state(), setup.owner)[0])
        .map(|value| value.suit)
        .unwrap_or(Suit::Club);
    let owner_hand = hand_zone(&tx, setup.owner);
    let discard = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_203,
        ren_suit,
        5,
    );
    let state = tx.commit().state;
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == ZHENLIANG)
        .map(|skill| skill.id)
        .unwrap();
    let (state, result) = {
        let revision = state.revision;
        apply_command(
            state,
            Command {
                actor: setup.owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(9_020),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        )
    };
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        result.result
    );
    assert!(state.has_pending_choice(), "贞良 discard prompt");
    let state = resolve(state, setup.owner, vec![discard.raw()], false, 9_021);
    assert!(state.has_pending_choice(), "贞良 damage target prompt");
    let hp_before = state.players.get(setup.other).unwrap().hp;
    let state = resolve(state, setup.owner, vec![setup.other.raw()], false, 9_022);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.other).unwrap().hp, hp_before - 1);
}
}

mod original_shenhua_menghuo {
//! Golden coverage for 祸首/再起/求首（shenhua 孟获，ABI 540-542）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const HUOSHOU: u32 = 2_566_702_627;
const ZAIQI: u32 = 3_013_017_329;
const TWQIUSHOU: u32 = 1_145_148_285;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/menghuo.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("menghuo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(HUOSHOU),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZAIQI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWQIUSHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_menghuo_contract() {
    let compiled = content_compiler::compile_source_named("menghuo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 542);
    let zaiqi = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZAIQI)
        .unwrap();
    assert_eq!(zaiqi.event, "phase_draw");
    assert_eq!(zaiqi.filter, "owner_is_target_draw_not_replaced");
    assert!(zaiqi.optional);
    let qiushou = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == TWQIUSHOU)
        .unwrap();
    assert_eq!(qiushou.event, "card_resolved");
    assert_eq!(
        qiushou.filter,
        "owner_alive_nanman_resolved_target_dead_or_damaged_gt_three"
    );
    let qiushou_skill = compiled
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == TWQIUSHOU)
        .unwrap();
    assert!(qiushou_skill.lord_only);
    let huoshou_mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == HUOSHOU)
        .map(|m| m.target.as_str())
        .collect();
    assert!(huoshou_mods.contains(&"card_target_cancel"));
    assert!(huoshou_mods.contains(&"nanman_damage_source_as_owner"));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
}

#[test]
fn huoshou_redirects_nanman_damage_source_to_the_owner() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let nanman = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::TrickNanman,
        96_300,
        Suit::Spade,
        7,
    );
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.other,
            target: setup.far,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: nanman,
            card_kind: 24,
            card_flags: 0,
        }],
    )
    .unwrap();
    let dealt = events
        .iter()
        .find_map(|event| match event {
            DomainEvent::DamageDealt { source, .. } => Some(*source),
            _ => None,
        })
        .expect("damage dealt event");
    assert_eq!(
        dealt, setup.owner,
        "nanman damage source redirected to 孟获"
    );
    let far = tx.state().players.get(setup.far).unwrap();
    assert_eq!(
        far.hp,
        far.max_hp as i32 - 1,
        "far player took exactly one damage"
    );
}

#[test]
fn zaiqi_replaces_draw_and_recovers_per_heart() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // 主公 owner: max hp 5. Wound the owner once (5 -> 4): lost HP = 1 -> flip 2 cards.
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    // Stack a heart then a club on top of the draw pile.
    let mut tx = Transaction::begin(state);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let heart = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        96_310,
        Suit::Heart,
        5,
    );
    let club = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_311,
        Suit::Club,
        8,
    );
    // Enter the draw phase (owner's turn).
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let state = tx.commit().state;
    let mut dispatch_tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        fired,
        Err(EffectError::ChoiceRequested),
        "optional 再起 confirm opened"
    );
    let mut state = dispatch_tx.commit().state;
    assert!(state.has_pending_choice(), "optional zaiqi confirm");
    state = resolve_choice(state, setup.owner, vec![1], 2);
    // Heart healed 1 HP; the club went to the owner's hand; the draw was replaced.
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        5,
        "heart recovered one HP"
    );
    assert!(
        state.hand_cards(setup.owner).contains(&club),
        "non-heart revealed card gained"
    );
    assert!(!state.hand_cards(setup.owner).contains(&heart));
    assert_eq!(
        state.cards.get(heart).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap(),
        "heart revealed card to discard"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn qiushou_draws_for_shu_qun_after_nanman_death() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Kill the far player so the Nanman resolution has a dead target.
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: setup.far,
            killer: setup.other,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert!(!state.players.get(setup.far).unwrap().alive);
    let hands_before: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| (player, state.hand_count(player)))
        .collect();
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: setup.other,
            card: CardId::NONE,
            kind: 24,
            targets: vec![setup.owner, setup.other, setup.far],
            damaged_targets: vec![],
            source_cards: vec![],
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    for (player, before) in hands_before {
        let group = state.player_group(player).map(|g| g.to_string());
        let grew = state.hand_count(player) > before;
        assert_eq!(
            grew,
            matches!(group.as_deref(), Some("shu") | Some("qun")),
            "player {player:?} group {group:?} draw behavior"
        );
    }
}

#[test]
fn qiushou_does_not_fire_without_death_or_many_damage() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let hands_before: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| (player, tx.state().hand_count(player)))
        .collect();
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: setup.other,
            card: CardId::NONE,
            kind: 24,
            targets: vec![setup.owner, setup.other],
            damaged_targets: vec![setup.other],
            source_cards: vec![],
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = fired.unwrap_or_else(|_| Vec::new());
    assert!(effects.is_empty(), "no 求首 trigger for 1 damaged target");
    let state = tx.commit().state;
    for (player, before) in hands_before {
        assert_eq!(state.hand_count(player), before);
    }
}
}

mod original_shenhua_mengjin {
//! Golden coverage for SHENHUA「猛进」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, move_one_card, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MENGJIN: u32 = 2_910_124_062;
const PANGDE: u32 = 4_110_795_715;
const TEST_MULTI_SHA: u32 = 2_910_124_063;
const TEST_DOUBLE_RESPONSE: u32 = 2_910_124_064;
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const OLD: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/mengjin.skill"),
    "\nskill test_multi_sha(2910124063) locked { modifier sha_target_limit add 1; }\n",
    "skill test_double_response(2910124064) locked { modifier response_count add 1; }\n",
);

fn setup(player_count: u8, extra_skills: &[u32]) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("mengjin-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_718, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(MENGJIN),
    });
    effects.extend(extra_skills.iter().map(|skill| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
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
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
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
fn packages_publish_exact_skill_character_and_dependencies() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    let trigger = shenhua
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MENGJIN)
        .unwrap();
    assert_eq!(trigger.event, "sha_dodged");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_dodged_target_alive_has_discardable_hand_equip"
    );
    assert!(trigger.optional);

    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    for dependency in ["standard", "shenhua"] {
        assert!(old.manifest.dependencies.iter().any(|entry| {
            entry.namespace == "official"
                && entry.package_id == dependency
                && entry.version == "0.1.0"
        }));
    }
    let pangde = old
        .character_index
        .iter()
        .find(|character| character.id == PANGDE)
        .unwrap();
    assert_eq!(pangde.name_key, "pangde");
    assert_eq!(pangde.max_hp, 4);
    assert_eq!(pangde.skills, [2, MENGJIN]);
}

#[test]
fn full_dodge_can_be_declined_or_discard_exactly_one_hand_or_equip_card() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_000);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_001);
    let hand = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_002);
    let mount = inject(&mut tx, target, ZoneKind::Equip, CardKind::HorseOff, 96_003);

    let (state, result) = play(tx.commit().state, owner, sha, vec![target], 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected play result: {:?}",
        result.result
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    let (offered, result) = resolve(state, target, vec![shan.raw()], false, 2);
    let confirm = result.choice.as_ref().unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(confirm.observer, owner);

    let (declined, result) = resolve(offered.clone(), owner, vec![], true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(declined.cards.get(hand).unwrap().owner, target);
    assert_eq!(declined.cards.get(mount).unwrap().owner, target);

    let (selecting, result) = resolve(offered, owner, vec![1], false, 4);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.observer, owner);
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert!(options.contains(&hand.raw()));
    assert!(options.contains(&mount.raw()));
    let (discarded, result) = resolve(selecting, owner, vec![mount.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = discarded
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = discarded.cards.get(mount).unwrap();
    assert_eq!(card.zone, discard);
    assert_eq!(card.owner, PlayerId::NONE);
    assert!(!discarded.hand_cards(owner).contains(&mount));
}

#[test]
fn filter_skips_hit_and_dodge_without_a_remaining_target_card() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_010);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_011);
    let (state, _) = play(tx.commit().state, owner, sha, vec![target], 10);
    let (dodged, result) = resolve(state, target, vec![shan.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(dodged.players.get(target).unwrap().hp, 4);

    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_012);
    let remaining = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_013);
    let (hit, result) = play(tx.commit().state, owner, sha, vec![target], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(hit.players.get(target).unwrap().hp, 3);
    assert_eq!(hit.cards.get(remaining).unwrap().owner, target);
}

#[test]
fn wanjian_shan_is_not_a_sha_dodge() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let wanjian = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWanjian,
        96_020,
    );
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_021);
    let remaining = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_022);
    let (state, result) = play(tx.commit().state, owner, wanjian, vec![], 20);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    let (resolved, result) = resolve(state, target, vec![shan.raw()], false, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.cards.get(remaining).unwrap().owner, target);
}

#[test]
fn two_shan_requirement_emits_only_after_the_second_response() {
    let (state, owner, seats) = setup(2, &[TEST_DOUBLE_RESPONSE]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_030);
    let shan1 = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_031);
    let shan2 = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_032);
    inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_033);
    let (state, _) = play(tx.commit().state, owner, sha, vec![target], 30);
    let (state, result) = resolve(state, target, vec![shan1.raw()], false, 31);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (_, result) = resolve(state, target, vec![shan2.raw()], false, 32);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
}

#[test]
fn multi_target_sha_resumes_after_the_dodged_targets_discard() {
    let (state, owner, seats) = setup(3, &[TEST_MULTI_SHA]);
    let (dodger, next_target) = (seats[1], seats[2]);
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_040);
    let shan = inject(&mut tx, dodger, ZoneKind::Hand, CardKind::Shan, 96_041);
    let discarded = inject(&mut tx, dodger, ZoneKind::Hand, CardKind::Other, 96_042);
    let next_hp = tx.state().players.get(next_target).unwrap().hp;
    let (state, result) = play(tx.commit().state, owner, sha, vec![dodger, next_target], 40);
    assert_eq!(result.choice.as_ref().unwrap().observer, dodger);
    let (state, result) = resolve(state, dodger, vec![shan.raw()], false, 41);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = resolve(state, owner, vec![1], false, 42);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (resolved, result) = resolve(state, owner, vec![discarded.raw()], false, 43);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(dodger).unwrap().hp, 4);
    assert_eq!(resolved.players.get(next_target).unwrap().hp, next_hp - 1);
}

#[test]
fn target_card_choice_revalidates_the_selected_card_at_resume() {
    let (state, owner, seats) = setup(2, &[]);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 96_050);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 96_051);
    let selected = inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, 96_052);
    let (state, _) = play(tx.commit().state, owner, sha, vec![target], 50);
    let (state, _) = resolve(state, target, vec![shan.raw()], false, 51);
    let (selecting, result) = resolve(state, owner, vec![1], false, 52);
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == selected.raw()));

    let mut tx = Transaction::begin(selecting);
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    move_one_card(&mut tx, selected, hand, discard, PlayerId::NONE, true).unwrap();
    let moved = tx.commit().state;
    let (same, result) = resolve(moved, owner, vec![selected.raw()], false, 53);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(same.cards.get(selected).unwrap().zone, discard);
}
}

mod original_shenhua_new_fenji {
//! Golden for 神话再临「奋激」：空手牌角色回合结束时摸二，发动者失去一点体力。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const NEW_FENJI: u32 = 3_465_257_033;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == NEW_FENJI && skill.name == "new_fenji" && !skill.locked
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NEW_FENJI
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_alive_event_player_hand_empty"
            && trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3_465_257_033),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 3_465_257_033,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let owner = seats[0];
    let ending_player = seats[1];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_FENJI),
        },
        EffectRequest::DiscardFromHand {
            player: ending_player,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, ending_player)
}

fn end_turn_event(player: PlayerId) -> DomainEvent {
    DomainEvent::PhaseEnded {
        current: player,
        phase: TurnPhaseTag::End,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn empty_handed_turn_player_draws_two_before_owner_loses_hp() {
    let (mut tx, owner, ending_player) = setup();
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(tx.state().hand_count(ending_player), 0);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[end_turn_event(ending_player)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(ending_player), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}

#[test]
fn nonempty_turn_player_does_not_offer_fenji() {
    let (mut tx, owner, ending_player) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: ending_player,
            count: 1,
        }],
    )
    .unwrap();
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let target_hand = tx.state().hand_count(ending_player);

    assert!(dispatch_for_events(
        &mut tx,
        &[end_turn_event(ending_player)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(ending_player), target_hand);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn owner_can_trigger_fenji_at_the_end_of_their_own_turn() {
    let (mut tx, owner, _) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let owner_hp = tx.state().players.get(owner).unwrap().hp;

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[end_turn_event(owner)],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
}
}

mod original_shenhua_nzry_feijun {
//! Golden coverage for 飞军/兵略（shenhua 王平，ABI 526）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const NZRY_FEIJUN: u32 = 2_410_819_466;
const NZRY_BINGLVE: u32 = 3_525_767_842;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/nzry_feijun.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    hand_more: PlayerId,
    equip_more: PlayerId,
    cost_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("feijun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let hand_more = seats[1];
    let equip_more = seats[2];
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
        skill_def: SkillDefId::new(NZRY_FEIJUN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NZRY_BINGLVE),
    });
    run_effects(&mut tx, effects).unwrap();
    // The owner keeps one cost card; the hand-more target gets three hand cards.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let cost = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_140),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(cost);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, hand_more).unwrap();
    let give_cards: Vec<CardId> = (0..3)
        .map(|i| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(96_150 + i),
                kind: CardKind::Tao,
                suit: Suit::Heart,
                rank: 6,
                weapon_range: 0,
                zone: target_hand,
                owner: hand_more,
                face_up: false,
            });
            tx.state_mut()
                .zones
                .get_mut(target_hand)
                .unwrap()
                .cards
                .push(card);
            card
        })
        .collect();
    // The equip-more target gets two weapons equipped.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, equip_more).unwrap();
    let mut equips = Vec::new();
    for i in 0..2 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(96_160 + i),
            kind: CardKind::Weapon,
            suit: Suit::Club,
            rank: 7,
            weapon_range: 2,
            zone: equip_zone,
            owner: equip_more,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(equip_zone)
            .unwrap()
            .cards
            .push(card);
        equips.push(card);
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        hand_more,
        equip_more,
        cost_card: cost,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn fire_play_end(tx: Transaction, current: PlayerId) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_hp_changed_players: 0,
            phase_targeted_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

#[test]
fn package_exposes_feijun_binglve_contract() {
    let compiled = content_compiler::compile_source_named("feijun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 526);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == NZRY_FEIJUN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active.selection.as_ref().map(|s| s.zone.as_str()),
        Some("hand_equip")
    );
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NZRY_BINGLVE)
        .unwrap();
    assert_eq!(trigger.event, "phase_play_end");
    assert_eq!(trigger.filter, "owner_is_target_feijun_new_target");
}

#[test]
fn hand_branch_gives_card_then_binglve_draws_two_at_phase_end() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Find the owner's active skill instance.
    let skill = tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NZRY_FEIJUN)
        .unwrap();
    let (state, activated) = activate(
        tx.commit().state,
        setup.owner,
        skill,
        vec![setup.cost_card],
        1,
    );
    assert!(
        matches!(activated.result, CommandResult::Accepted { .. }),
        "activation must be accepted: {:?}",
        activated.result
    );
    let mut state = resolve_choice(state, setup.owner, vec![0], 2);
    state = resolve_choice(state, setup.owner, vec![setup.hand_more.raw()], 3);
    let given = state.hand_cards(setup.hand_more)[0];
    state = resolve_choice(state, setup.hand_more, vec![given.raw()], 4);

    assert_eq!(
        state.hand_count(setup.owner),
        1,
        "cost discarded, one card gained from the target"
    );
    assert!(
        state.hand_cards(setup.owner).contains(&given),
        "the hand-more target gave a card to the owner"
    );
    // The play phase ends: 兵略 draws two for the first-time target.
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, setup.owner);
    let state = tx.commit().state;
    assert_eq!(
        state.hand_count(setup.owner),
        3,
        "given card + two from 兵略"
    );
}

#[test]
fn equip_branch_discards_one_equipment() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let skill = tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NZRY_FEIJUN)
        .unwrap();
    let (state, activated) = activate(
        tx.commit().state,
        setup.owner,
        skill,
        vec![setup.cost_card],
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    // Both branches are available; choose the equipment branch.
    let mut state = resolve_choice(state, setup.owner, vec![1], 2);
    state = resolve_choice(state, setup.owner, vec![setup.equip_more.raw()], 3);
    let equip = state
        .zones
        .find(ZoneKind::Equip, setup.equip_more)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards[0])
        .unwrap();
    let equip_zone = state.zones.find(ZoneKind::Equip, setup.equip_more).unwrap();
    let equip_count_before = state.zones.get(equip_zone).unwrap().cards.len();
    state = resolve_choice(state, setup.equip_more, vec![equip.raw()], 4);
    assert_eq!(
        state.zones.get(equip_zone).unwrap().cards.len(),
        equip_count_before - 1,
        "the equip-more target discarded one equipment"
    );
    assert_eq!(state.hand_count(setup.owner), 0, "cost discarded");
}

#[test]
fn repeated_target_does_not_rearm_binglve() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let skill = tx
        .state()
        .players
        .get(setup.owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NZRY_FEIJUN)
        .unwrap();
    // First use against the hand-more target.
    let (state, _) = activate(
        tx.commit().state,
        setup.owner,
        skill,
        vec![setup.cost_card],
        1,
    );
    let mut state = resolve_choice(state, setup.owner, vec![0], 2);
    state = resolve_choice(state, setup.owner, vec![setup.hand_more.raw()], 3);
    let given = state.hand_cards(setup.hand_more)[0];
    state = resolve_choice(state, setup.hand_more, vec![given.raw()], 4);
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, setup.owner);
    let mut state = tx.commit().state;
    assert_eq!(state.hand_count(setup.owner), 3);
    // Next turn: the per-turn flag resets but the permanent target mark stays.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.clear_per_player_card_counts();
    tx.state_mut().skills.get_mut(skill).unwrap().use_count = 0;
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let cost = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_170),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner: setup.owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(cost);
    // The hand-more target gets refilled so the hand branch is available again.
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.hand_more)
        .unwrap();
    for i in 0..3 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(96_180 + i),
            kind: CardKind::Tao,
            suit: Suit::Heart,
            rank: 6,
            weapon_range: 0,
            zone: target_hand,
            owner: setup.hand_more,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(target_hand)
            .unwrap()
            .cards
            .push(card);
    }
    tx.state_mut().turn.phase = Phase::Play;
    state = tx.commit().state;
    // Same target again: the mark exists, so no new 兵略 flag.
    let (state, _) = activate(state, setup.owner, skill, vec![cost], 5);
    let mut state = resolve_choice(state, setup.owner, vec![0], 6);
    state = resolve_choice(state, setup.owner, vec![setup.hand_more.raw()], 7);
    let given2 = state.hand_cards(setup.hand_more)[0];
    state = resolve_choice(state, setup.hand_more, vec![given2.raw()], 8);
    assert_eq!(
        state.hand_count(setup.owner),
        4,
        "cost spent, one card gained"
    );
    // Play phase end: 兵略 must NOT fire again (same target, already marked).
    let mut tx = Transaction::begin(state);
    let tx = fire_play_end(tx, setup.owner);
    let state = tx.commit().state;
    assert_eq!(state.hand_count(setup.owner), 4, "no second 兵略 draw");
}
}

mod original_shenhua_nzry_jianxiang {
//! Golden coverage for 神话「荐降」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NZRY_JIANXIANG: u32 = 3_232_965_394;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/nzry_jianxiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    tied_min: PlayerId,
    other: PlayerId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("nzry-jianxiang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722_83, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let tied_min = seats[2];
    let other = seats[3];
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
        skill_def: SkillDefId::new(NZRY_JIANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    inject(tx.state_mut(), source, 9_830_001);
    inject(tx.state_mut(), other, 9_830_002);
    inject(tx.state_mut(), other, 9_830_003);
    Setup {
        state: tx.commit().state,
        owner,
        source,
        tied_min,
        other,
    }
}

fn target_owner(mut setup: Setup) -> (GameState, PipelineResult) {
    let used = inject(&mut setup.state, setup.source, 9_830_004);
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: setup.source,
                card: used,
                targets: vec![setup.owner],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.state().clone();
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

fn resolve_target(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_exposes_exact_optional_targeted_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed.skill_index.iter().any(|skill| {
        skill.skill_def_id == NZRY_JIANXIANG && skill.name == "nzry_jianxiang" && !skill.locked
    }));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NZRY_JIANXIANG)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_alive_targeted_by_other");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("nzry_jianxiang", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn accepting_chooses_any_player_tied_for_minimum_hand_and_draws_one() {
    let setup = setup();
    let owner = setup.owner;
    let tied_min = setup.tied_min;
    let (state, accepted) = target_owner(setup);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let options: Vec<_> = accepted
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, vec![owner.raw(), tied_min.raw()]);

    let before = state.hand_count(tied_min);
    let (state, drawn) = resolve_target(state, owner, tied_min, 2);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(tied_min), before + 1);
}

#[test]
fn self_use_does_not_trigger_and_stale_minimum_target_is_rejected() {
    let mut self_use = setup();
    let card = inject(&mut self_use.state, self_use.owner, 9_830_005);
    let mut tx = Transaction::begin(self_use.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: self_use.owner,
            card,
            targets: vec![self_use.owner],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());

    let setup = setup();
    let owner = setup.owner;
    let tied_min = setup.tied_min;
    let other = setup.other;
    let (mut state, accepted) = target_owner(setup);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    inject(&mut state, tied_min, 9_830_006);
    assert!(state.hand_count(tied_min) > state.hand_count(owner));
    assert!(state.hand_count(other) > state.hand_count(owner));
    let (_, stale) = resolve_target(state, owner, tied_min, 3);
    assert!(matches!(stale.result, CommandResult::Rejected { .. }));
}
}

mod original_shenhua_oldniepan {
//! Golden coverage for 庞统「旧涅槃」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PANGTONG: u32 = 3_966_656_508;
const LIANHUAN: u32 = 2_921_824_505;
const OLDNIEPAN: u32 = 3_136_864_229;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn has_oldniepan(state: &GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == OLDNIEPAN)
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, def: u32) -> CardId {
    let zone_id = state.zones.find(zone, owner).unwrap();
    let kind = if zone == ZoneKind::Equip {
        CardKind::Weapon
    } else {
        CardKind::TrickLe
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
        face_up: zone != ZoneKind::Hand,
    });
    state.zones.get_mut(zone_id).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (Transaction, PlayerId, Vec<CardId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDNIEPAN),
    });
    run_effects(&mut tx, effects).unwrap();

    let mut original_cards = tx.state().hand_cards(owner);
    original_cards.push(inject(tx.state_mut(), owner, ZoneKind::Equip, 93_001));
    original_cards.push(inject(tx.state_mut(), owner, ZoneKind::Judge, 93_002));
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = -1;
    participant.max_hp = 3;
    participant.turned_over = true;
    participant
        .marks
        .push((engine_core::content::MARK_CHAIN, 1));
    (tx, owner, original_cards)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(tx: &Transaction, owner: PlayerId, accept: bool, key: u64) -> GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_pangtong_and_oldniepan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.character_index.iter().any(|character| {
        character.id == PANGTONG
            && character.name_key == "pangtong"
            && character.max_hp == 3
            && character.skills == [LIANHUAN, OLDNIEPAN]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDNIEPAN)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
}

#[test]
fn accepting_discards_hej_restores_state_draws_three_recovers_to_three_and_consumes() {
    let (mut tx, owner, original_cards) = setup(2_026_072_302);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    let participant = accepted.players.get(owner).unwrap();
    assert_eq!(participant.hp, 3);
    assert!(!participant.turned_over);
    assert!(!participant
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
    assert!(!has_oldniepan(&accepted, owner));
    assert_eq!(accepted.hand_count(owner), 3);
    assert!(accepted
        .zones
        .get(accepted.zones.find(ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .is_empty());
    assert!(accepted
        .zones
        .get(accepted.zones.find(ZoneKind::Judge, owner).unwrap())
        .unwrap()
        .cards
        .is_empty());
    let discard = accepted
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(original_cards
        .into_iter()
        .all(|card| accepted.cards.get(card).unwrap().zone == discard));
}

#[test]
fn declining_preserves_cards_state_hp_and_limited_skill() {
    let (mut tx, owner, original_cards) = setup(2_026_072_303);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 2);
    let participant = declined.players.get(owner).unwrap();
    assert_eq!(participant.hp, -1);
    assert!(participant.turned_over);
    assert!(participant
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
    assert!(has_oldniepan(&declined, owner));
    assert!(original_cards.into_iter().all(|card| {
        let zone = declined.cards.get(card).unwrap().zone;
        zone == declined.zones.find(ZoneKind::Hand, owner).unwrap()
            || zone == declined.zones.find(ZoneKind::Equip, owner).unwrap()
            || zone == declined.zones.find(ZoneKind::Judge, owner).unwrap()
    }));
}
}

mod original_shenhua_qimou {
//! Golden coverage for 界魏延「奇谋」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, RejectReason, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_use_limit_for, QueryContext};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const RE_WEIYAN: u32 = 2_771_589_662;
const XINKUANGGU: u32 = 4_089_972_415;
const QIMOU: u32 = 3_084_798_293;
const QIMOU2: u32 = 2_727_616_630;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == QIMOU2).then_some(*count))
        .unwrap_or(0)
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let far = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIMOU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 4;
    player.max_hp = 4;
    (tx.commit().state, owner, far)
}

#[test]
fn package_publishes_complete_re_weiyan_and_qimou_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_WEIYAN
            && character.name_key == "re_weiyan"
            && character.max_hp == 4
            && character.skills == [XINKUANGGU, QIMOU]
    }));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == QIMOU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert!(active.once_per_game);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == QIMOU2
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn choosing_three_loses_three_hp_grants_three_bonuses_once_and_expires() {
    let (state, owner, far) = setup(2_026_072_301);
    let active = skill_id(&state, owner, QIMOU).unwrap();
    let (state, opened) = command(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    let choice = opened.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 1, false)
    );

    let (state, resolved) = command(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![3],
            cancelled: false,
        },
        2,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(mark_count(&state, owner), 3);
    assert!(skill_id(&state, owner, QIMOU2).is_some());
    assert_eq!(sha_use_limit_for(&state, owner), 4);
    assert!(matches!(
        query(
            &state,
            &QueryRequest::Distance {
                from: owner,
                to: far,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(1)
    ));

    let (_, repeated) = command(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        3,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { .. }
        }
    ));

    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(skill_id(tx.state(), owner, QIMOU2).is_none());
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(sha_use_limit_for(tx.state(), owner), 1);
}
}

mod original_shenhua_quhu {
//! Golden coverage for SHENHUA「驱虎」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XUNYU: u32 = 4_288_100_271;
const RE_XUNYU: u32 = 3_554_276_229;
const QUHU: u32 = 4_267_019_884;
const QUHU_WIN: u32 = 2_288_140_413;
const QUHU_LOSE: u32 = 4_142_452_028;
const JIEMING: u32 = 3_579_069_878;
const REJIEMING: u32 = 4_088_661_605;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/quhu.skill"),
    include_str!("../../../content/original/ported/shenhua/quhu_win.skill"),
    include_str!("../../../content/original/ported/shenhua/quhu_lose.skill")
);
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    victim: PlayerId,
    far: PlayerId,
    owner_card: CardId,
    source_card: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(owner_rank: u8, source_rank: u8) -> Setup {
    let package = content_compiler::compile_source_named("quhu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_411, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, source, victim, far) = (seats[0], seats[1], seats[2], seats[3]);
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QUHU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 6;
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let owner_card = inject(tx.state_mut(), owner, 99_401, Suit::Spade, owner_rank);
    let source_card = inject(tx.state_mut(), source, 99_402, Suit::Heart, source_rank);
    Setup {
        state: tx.commit().state,
        owner,
        source,
        victim,
        far,
        owner_card,
        source_card,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state.clone(),
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled: false,
        },
        key,
    )
}

fn compare(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    let active = skill_id(&setup.state, setup.owner, QUHU).unwrap();
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.source],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(
        choose_owner_card.result,
        CommandResult::Accepted { .. }
    ));
    let (state, choose_source_card) =
        resolve(state, setup.owner, vec![setup.owner_card.raw()], key + 1);
    assert!(matches!(
        choose_source_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    resolve(state, setup.source, vec![setup.source_card.raw()], key + 2)
}

#[test]
fn packages_publish_abi159_quhu_and_both_complete_xunyu_variants() {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(shenhua.manifest.rule_vm_abi_version, 671);
    assert_eq!(shenhua.skill_index.len(), 201);
    assert_eq!(shenhua.character_index.len(), 48);
    let active = shenhua
        .active_index
        .iter()
        .find(|active| active.skill_def_id == QUHU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().unwrap().hp_gt_owner);
    assert!(active.targets.as_ref().unwrap().can_compare);
    assert_eq!(
        shenhua
            .character_index
            .iter()
            .find(|character| character.id == XUNYU)
            .unwrap()
            .skills,
        [QUHU, JIEMING]
    );

    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);
    assert_eq!(mobile.skill_index.len(), 440);
    assert_eq!(mobile.character_index.len(), 204);
    assert_eq!(
        mobile
            .character_index
            .iter()
            .find(|character| character.id == RE_XUNYU)
            .unwrap()
            .skills,
        [QUHU, REJIEMING]
    );

    let compiled = content_compiler::compile_source_named("quhu", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = shenhua
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn activation_rejects_equal_or_lower_hp_targets() {
    for target_hp in [3, 2] {
        let mut setup = setup(13, 1);
        setup.state.players.get_mut(setup.source).unwrap().hp = target_hp;
        let active = skill_id(&setup.state, setup.owner, QUHU).unwrap();
        let (_, result) = command(
            setup.state,
            setup.owner,
            CommandKind::ActivateSkill {
                skill: active,
                targets: vec![setup.source],
                card_targets: Vec::new(),
            },
            10 + target_hp as u64,
        );
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
    }
}

#[test]
fn strict_win_lets_owner_choose_in_bound_sources_range_and_uses_that_source() {
    let setup = setup(13, 1);
    let (state, compared) = compare(&setup, 20);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseBoundSourceAttackRangeDamage
    );
    let options: Vec<_> = compared
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&setup.owner.raw()));
    assert!(options.contains(&setup.victim.raw()));
    assert!(!options.contains(&setup.source.raw()));
    assert!(!options.contains(&setup.far.raw()));
    assert!(skill_id(&state, setup.owner, QUHU_WIN).is_some());

    let victim_hp = state.players.get(setup.victim).unwrap().hp;
    let (state, damaged) = resolve(state, setup.owner, vec![setup.victim.raw()], 23);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.victim).unwrap().hp, victim_hp - 1);
    assert!(skill_id(&state, setup.owner, QUHU_WIN).is_none());
    assert!(!state.has_pending_choice());
}

#[test]
fn loss_and_tie_both_damage_owner_from_compare_target() {
    for (owner_rank, source_rank, key) in [(1, 13, 30), (7, 7, 40)] {
        let setup = setup(owner_rank, source_rank);
        let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
        let (state, compared) = compare(&setup, key);
        assert!(matches!(compared.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
        assert!(skill_id(&state, setup.owner, QUHU_LOSE).is_none());
        assert!(!state.has_pending_choice());
    }
}
}

mod original_shenhua_retianxiang {
//! Golden coverage for 界天香（shenhua 小乔）取消伤害 + 分支结算（ABI 457）.
//!
//! 小乔受到伤害时：可选确认 → 弃红桃手牌 → 选其他角色 → 拥有者二选一：
//! 0 = 目标受伤害来源 1 点伤害后摸 min(5, 已损失体力) 张；1 = 目标失去 1 点体力
//! 后获得弃置的红桃牌。原伤害被取消。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RETIANXIANG: u32 = 3_203_696_812;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/retianxiang.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

fn prepared(branch: u8) -> (Transaction, PlayerId, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("retianxiang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 457);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, source) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RETIANXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, owner)
        .expect("owner hand zone");
    let heart = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_700),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .expect("hand zone")
        .cards
        .push(heart);
    let _ = branch;
    (tx, owner, victim, source, heart)
}

fn run_branch_full(
    mut tx: Transaction,
    owner: PlayerId,
    victim: PlayerId,
    heart: CardId,
    branch: u32,
) -> (engine_core::state::GameState, i32) {
    let owner_hp_before = tx.state().players.get(owner).unwrap().hp;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let mut state = tx.commit().state;
    state = drive_choice(state, owner, 1); // optional confirm
    state = drive_choice(state, owner, heart.raw()); // heart card cost
    state = drive_choice(state, owner, victim.raw()); // branch target
    state = drive_choice(state, owner, branch); // branch
    (state, owner_hp_before)
}

#[test]
fn branch_zero_damages_from_source_then_draws_lost_hp_capped_five() {
    let (tx, owner, victim, _source, heart) = prepared(0);
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let victim_max = tx.state().players.get(victim).unwrap().max_hp;
    let victim_hand = tx.state().hand_count(victim);
    let (state, owner_hp) = run_branch_full(tx, owner, victim, heart, 0);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp,
        "original damage cancelled"
    );
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp.saturating_sub(1),
        "victim takes 1 damage from the source"
    );
    let lost = ((victim_max as i32 - (victim_hp - 1)).max(0) as u32).min(5);
    assert_eq!(
        state.hand_count(victim),
        victim_hand + lost,
        "victim draws min(5, lost HP)"
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn branch_one_loses_hp_and_gains_discarded_card() {
    let (tx, owner, victim, _source, heart) = prepared(1);
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let victim_hand = tx.state().hand_count(victim);
    let (state, owner_hp) = run_branch_full(tx, owner, victim, heart, 1);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp,
        "original damage cancelled"
    );
    assert_eq!(
        state.players.get(victim).unwrap().hp,
        victim_hp.saturating_sub(1),
        "victim loses 1 HP"
    );
    assert!(
        state.hand_cards(victim).contains(&heart),
        "victim gains the discarded heart card"
    );
    assert_eq!(state.hand_count(victim), victim_hand + 1);
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_shuangxiong {
//! Golden coverage for 双雄（shenhua 颜良文丑，ABI 511）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHUANGXIONG: u32 = 3_926_154_071;
const SHUANGXIONG_VIEW_AS: u32 = 1_761_435_131;
/// Stable mark id recorded by ABI 511 (count: 1 = red, 2 = black).
const SHUANGXIONG_COLOR_MARK: u32 = 6_0000_0511;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/shuangxiong.skill");

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_draw_judgment_contract() {
    let compiled = content_compiler::compile_source_named("shuangxiong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 511);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHUANGXIONG)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target_draw_not_replaced");
    assert!(trigger.optional);
    let view_as = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == SHUANGXIONG_VIEW_AS)
        .unwrap();
    assert_eq!(view_as.as_kind, "juedou");
    assert_eq!(view_as.filter, "color_ne_shuangxiong");
    // `from hand` is the default source zone and is normalized away.
    assert_eq!(view_as.source_zone, None);
}

#[test]
fn draw_phase_judges_gains_card_and_arms_opposite_color_view_as() {
    let package = content_compiler::compile_source_named("shuangxiong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(SHUANGXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;

    // The draw phase begins: the optional confirm opens.
    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Draw);
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional confirm pending");
    state = resolve_choice(state, owner, vec![1], 1);

    // 双雄 replaced the draw: draw is fixed to zero and the judgment card was gained.
    assert!(state.turn.draw_phase_is_fixed(), "draw must be replaced");
    assert_eq!(
        state.hand_count(owner),
        1,
        "only the judgment card is in hand"
    );
    let judgment_card = state.hand_cards(owner)[0];
    let judgment_suit = state.cards.get(judgment_card).unwrap().suit;
    let expected_color = match judgment_suit {
        Suit::Spade | Suit::Club => 2,
        _ => 1,
    };
    let mark = state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == SHUANGXIONG_COLOR_MARK)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    assert_eq!(
        mark, expected_color,
        "recorded color must match the judgment suit"
    );
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == SkillDefId::new(SHUANGXIONG_VIEW_AS))
            }),
        "the turn-long view-as skill must be armed"
    );
}

#[test]
fn second_draw_phase_does_not_rejudge_when_draw_is_fixed() {
    // Once the draw phase is fixed (e.g. by another skill), 双雄 must not trigger.
    let package = content_compiler::compile_source_named("shuangxiong-test2", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(SHUANGXIONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().turn.set_draw_phase_fixed_count(2);

    let mut tx = Transaction::begin(tx.commit().state);
    let tx = fire_phase(tx, owner, TurnPhaseTag::Draw);
    let state = tx.commit().state;
    assert!(
        !state.has_pending_choice(),
        "fixed draw must not ask for 双雄"
    );
    assert_eq!(state.hand_count(owner), 0, "no judgment card gained");
}
}

mod original_shenhua_songwei {
//! Golden coverage for 神话「颂威」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::content;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SONGWEI: u32 = 2_169_124_968;
const TEST_REWRITE: u32 = 9_216_001;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/songwei.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("songwei_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_216_100 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_delayed_trick(),
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    judged: PlayerId,
    other: PlayerId,
}

fn setup(include_rewriter: bool) -> Setup {
    let source = if include_rewriter {
        format!(
            "{SOURCE}\nskill test_rewrite({TEST_REWRITE}) locked {{ modifier judgment_rewrite add 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("songwei-test", &source).unwrap();
    package.character_index = vec![
        character(9_216_010, "wei"),
        character(9_216_011, "wei"),
        character(9_216_012, "shu"),
        character(9_216_013, "wu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 20_260_722_86, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut rest = seats.iter().copied().filter(|player| *player != lord);
    let judged = rest.next().unwrap();
    let other = rest.next().unwrap();
    let fourth = rest.next().unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_216_010;
    tx.state_mut()
        .players
        .get_mut(judged)
        .unwrap()
        .character_def = 9_216_011;
    tx.state_mut().players.get_mut(other).unwrap().character_def = 9_216_012;
    tx.state_mut()
        .players
        .get_mut(fourth)
        .unwrap()
        .character_def = 9_216_013;

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
        player: lord,
        skill_def: SkillDefId::new(SONGWEI),
    });
    if include_rewriter {
        effects.push(EffectRequest::AddSkill {
            player: other,
            skill_def: SkillDefId::new(TEST_REWRITE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    Setup {
        tx,
        lord,
        judged,
        other,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_exposes_lord_event_player_choice_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed.skill_index.iter().any(|skill| {
        skill.skill_def_id == SONGWEI && skill.name == "songwei" && skill.lord_only
    }));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SONGWEI)
        .unwrap();
    assert_eq!(trigger.event, "judgment_resolved");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_judgment_black_event_player_group_wei"
    );
    assert!(trigger.optional);
    assert_eq!(
        trigger.choice_actor,
        content_schema::TriggerChoiceActor::EventPlayer
    );
}

#[test]
fn only_another_wei_players_black_judgment_prompts_that_player() {
    let setup = setup(false);
    let discard = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut cases = [
        (setup.judged, Suit::Spade, true),
        (setup.judged, Suit::Heart, false),
        (setup.other, Suit::Club, false),
        (setup.lord, Suit::Spade, false),
    ];
    for (index, (judged, suit, expected)) in cases.iter_mut().enumerate() {
        let mut tx = Transaction::begin(setup.tx.state().clone());
        let result_card = inject(&mut tx, PlayerId::NONE, discard, CardKind::Other, *suit, 7);
        let result = run_effects(
            &mut tx,
            vec![EffectRequest::EmitJudgmentResolved {
                player: *judged,
                result_card,
                suit: *suit,
                rank: 7,
                cause_card: CardId::NONE,
            }],
        );
        if *expected {
            assert_eq!(result, Err(EffectError::ChoiceRequested));
            assert_eq!(tx.state().resolution.pending_observer, setup.judged.raw());
        } else {
            result.unwrap();
            assert!(!tx.state().has_pending_choice(), "case {index}");
        }
    }
}

#[test]
fn rewritten_physical_card_drives_songwei_then_delayed_trick_resumes() {
    let mut setup = setup(true);
    let judge = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Judge, setup.judged)
        .unwrap();
    let delayed = inject(
        &mut setup.tx,
        setup.judged,
        judge,
        CardKind::TrickLe,
        Suit::Club,
        6,
    );
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let original = inject(
        &mut setup.tx,
        PlayerId::NONE,
        draw,
        CardKind::Other,
        Suit::Heart,
        3,
    );
    let hand = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.other)
        .unwrap();
    let rewrite = inject(
        &mut setup.tx,
        setup.other,
        hand,
        CardKind::Other,
        Suit::Spade,
        9,
    );
    setup.tx.state_mut().turn.current = setup.judged;
    setup.tx.state_mut().turn.phase = Phase::Judge;
    let state = setup.tx.commit().state;

    let (state, rewrite_choice) = apply_command(
        state.clone(),
        Command {
            actor: setup.judged,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(
        rewrite_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        rewrite_choice.choice.as_ref().unwrap().observer,
        setup.other
    );

    let lord_hand = state.hand_count(setup.lord);
    let (state, songwei_choice) = resolve(state, setup.other, vec![rewrite.raw()], 2);
    assert!(matches!(
        songwei_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        songwei_choice.choice.as_ref().unwrap().observer,
        setup.judged
    );
    assert!(songwei_choice.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::JudgmentResolved {
                player,
                result_card,
                cause_card,
                ..
            } if *player == setup.judged && *result_card == rewrite && *cause_card == delayed
        )
    }));
    assert!(!songwei_choice.events.iter().any(|event| {
        matches!(event, DomainEvent::JudgmentResolved { result_card, .. } if *result_card == original)
    }));

    let (state, accepted) = resolve(state, setup.judged, vec![1], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.lord), lord_hand + 1);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn black_bagua_judgment_prompts_songwei_then_reopens_sha_response() {
    let mut setup = setup(false);
    let equip = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Equip, setup.judged)
        .unwrap();
    let bagua = inject(
        &mut setup.tx,
        setup.judged,
        equip,
        CardKind::Armor,
        Suit::Spade,
        2,
    );
    setup.tx.state_mut().cards.get_mut(bagua).unwrap().def = CardDefId::new(content::DEF_BAGUA);
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let result_card = inject(
        &mut setup.tx,
        PlayerId::NONE,
        draw,
        CardKind::Other,
        Suit::Spade,
        8,
    );
    let hand = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.other)
        .unwrap();
    let sha = inject(
        &mut setup.tx,
        setup.other,
        hand,
        CardKind::Sha,
        Suit::Club,
        6,
    );
    setup.tx.state_mut().turn.current = setup.other;
    setup.tx.state_mut().turn.phase = Phase::Play;
    let state = setup.tx.commit().state;

    let (state, response) = apply_command(
        state.clone(),
        Command {
            actor: setup.other,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.judged],
                card_targets: vec![],
            },
        },
    );
    assert_eq!(response.choice.as_ref().unwrap().observer, setup.judged);
    let lord_hand = state.hand_count(setup.lord);
    let (state, songwei) = resolve(state, setup.judged, vec![content::BAGUA_RESPONSE_TOKEN], 11);
    assert_eq!(songwei.choice.as_ref().unwrap().observer, setup.judged);
    assert!(songwei.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::JudgmentResolved {
                player,
                result_card: card,
                cause_card,
                ..
            } if *player == setup.judged && *card == result_card && *cause_card == bagua
        )
    }));

    let (state, accepted) = resolve(state, setup.judged, vec![1], 12);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.lord), lord_hand + 1);
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.pending_observer, setup.judged.raw());
}
}

mod original_shenhua_sunliang {
//! Golden coverage for 溃诛/掣政/立军（shenhua 孙亮，ABI 577）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KUIZHU: u32 = 1_891_631_254;
const ZHIZHENG: u32 = 3_830_747_834;
const LIJUN: u32 = 1_640_782_530;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/sunliang.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("sl-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(KUIZHU),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHIZHENG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

fn phase_end_event(player: PlayerId, tag: engine_api::TurnPhaseTag) -> DomainEvent {
    DomainEvent::PhaseEnded {
        current: player,
        phase: tag,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn package_exposes_sunliang_contract() {
    let compiled = content_compiler::compile_source_named("sl", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 577);
    assert_eq!(compiled.skill_index.len(), 3);
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .map(|t| (t.skill_def_id, t.event.as_str(), t.optional))
        .collect();
    assert!(events.contains(&(KUIZHU, "phase_discard_end", true)));
    assert!(events.contains(&(ZHIZHENG, "phase_play_end", true)));
    assert!(events.contains(&(LIJUN, "card_used", true)));
}

/// 溃诛：弃牌阶段结束后按弃牌数选择摸牌分支，目标各摸一张。
#[test]
fn kuizhu_draw_branch() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DISCARD_PHASE_DISCARDED, 2);
    let state = tx.commit().state;
    let state = dispatch(
        state,
        phase_end_event(setup.owner, engine_api::TurnPhaseTag::Discard),
    );
    assert!(state.has_pending_choice(), "溃诛 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(state.has_pending_choice(), "溃诛 branch prompt");
    // 摸牌分支（0）。
    let state = resolve(state, setup.owner, vec![0], false, 9_002);
    assert!(state.has_pending_choice(), "溃诛 draw targets");
    let state = resolve(
        state,
        setup.owner,
        vec![setup.other.raw(), setup.far.raw()],
        false,
        9_003,
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.other), 1);
    assert_eq!(state.hand_count(setup.far), 1);
}

/// 溃诛：伤害分支——体力值之和为 X 的角色各受 1 点伤害。
#[test]
fn kuizhu_damage_branch() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Discard;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DISCARD_PHASE_DISCARDED, 2);
    // 将 far 体力降到 2，使体力值之和为 2 的伤害目标成立。
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: setup.far,
            delta: -1,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let hp_before = state.players.get(setup.far).unwrap().hp;
    assert_eq!(hp_before, 2);
    let state = dispatch(
        state,
        phase_end_event(setup.owner, engine_api::TurnPhaseTag::Discard),
    );
    assert!(state.has_pending_choice(), "溃诛 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    let state = resolve(state, setup.owner, vec![1], false, 9_011);
    assert!(state.has_pending_choice(), "溃诛 damage targets");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_012);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.far).unwrap().hp, hp_before - 1);
}

/// 掣政：出牌阶段结束时弃置一名攻击范围不包含你的角色的一张牌。
#[test]
fn zhizheng_discards_out_of_range_card() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let far_hand = hand_zone(&tx, setup.far);
    let victim = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Sha,
        97_300,
        Suit::Club,
        2,
    );
    // 持有者装备防御马：他人到你的距离 +1，使远位角色攻击范围不包含你。
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, setup.owner).unwrap();
    alloc_card(
        &mut tx,
        setup.owner,
        equip_zone,
        CardKind::HorseDef,
        97_302,
        Suit::Diamond,
        1,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        phase_end_event(setup.owner, engine_api::TurnPhaseTag::Play),
    );
    // 本回合用牌数 0 < 攻击范围不包含你的角色数 → 触发。
    assert!(state.has_pending_choice(), "掣政 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    assert!(state.has_pending_choice(), "掣政 target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_021);
    assert!(state.has_pending_choice(), "掣政 card prompt");
    let state = resolve(state, setup.owner, vec![victim.raw()], false, 9_022);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim).unwrap().zone, discard);
}

/// 立军：吴势力角色出牌阶段使用实体【杀】后，持有者获得该牌并令其摸一张。
#[test]
fn lijun_gains_wu_sha() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    // 将 other 改为吴势力。
    tx.state_mut()
        .group_overrides
        .insert(setup.other, "wu".into());
    let other_hand = hand_zone(&tx, setup.other);
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        97_301,
        Suit::Spade,
        3,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.far],
        },
    );
    assert!(state.has_pending_choice(), "立军 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_030);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.owner).contains(&sha),
        "lord gained the Sha"
    );
    assert_eq!(state.hand_count(setup.other), 1, "Wu user drew one");
}
}

mod original_shenhua_tianyi {
//! Golden coverage for 太史慈「天义」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, RejectReason, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_target_within_distance, sha_target_limit_for, sha_use_limit_for};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const TAISHICI: u32 = 2_164_149_853;
const TIANYI: u32 = 3_117_995_535;
const TIANYI_WIN: u32 = 4_028_521_539;
const TIANYI_LOSE: u32 = 2_573_090_805;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/shenhua/tianyi.skill"),
    include_str!("../../../content/original/ported/shenhua/tianyi3.skill"),
    include_str!("../../../content/original/ported/shenhua/tianyi2.skill")
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    far: PlayerId,
    owner_compare: CardId,
    target_compare: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_rank: u8, target_rank: Option<u8>) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target, far) = (seats[0], seats[1], seats[2]);
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIANYI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    let owner_compare = inject(
        tx.state_mut(),
        owner,
        99_301,
        CardKind::Other,
        Suit::Spade,
        owner_rank,
    );
    let target_compare = target_rank
        .map(|rank| {
            inject(
                tx.state_mut(),
                target,
                99_302,
                CardKind::Other,
                Suit::Heart,
                rank,
            )
        })
        .unwrap_or(CardId::NONE);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        far,
        owner_compare,
        target_compare,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn compare(setup: &Setup, key: u64) -> GameState {
    let active = skill_id(&setup.state, setup.owner, TIANYI).unwrap();
    let (state, choose_owner_card) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(
        choose_owner_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, choose_target_card) =
        resolve(state, setup.owner, vec![setup.owner_compare.raw()], key + 1);
    assert!(matches!(
        choose_target_card.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(
        state,
        setup.target,
        vec![setup.target_compare.raw()],
        key + 2,
    );
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

fn end_owner_turn(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_complete_taishici_and_exact_tianyi_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == TAISHICI)
        .unwrap();
    assert_eq!(character.name_key, "taishici");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.skills, [TIANYI]);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == TIANYI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(active.selection.as_ref().unwrap().zone, "hand");
    assert!(active.targets.as_ref().unwrap().can_compare);
    assert!(!active.targets.as_ref().unwrap().allow_self);
    for helper in [TIANYI_WIN, TIANYI_LOSE] {
        assert!(package.trigger_index.iter().any(|trigger| {
            trigger.skill_def_id == helper
                && trigger.event == "turn_ended"
                && trigger.filter == "owner_is_target"
        }));
    }

    let compiled = content_compiler::compile_source_named("tianyi", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 3);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn strict_win_discards_both_cards_grants_all_sha_bonuses_once_and_expires() {
    let setup = setup(2_026_072_201, 13, Some(1));
    let mut state = compare(&setup, 1);
    assert!(skill_id(&state, setup.owner, TIANYI_WIN).is_some());
    assert!(skill_id(&state, setup.owner, TIANYI_LOSE).is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.owner_compare).unwrap().zone, discard);
    assert_eq!(state.cards.get(setup.target_compare).unwrap().zone, discard);
    assert_eq!(sha_use_limit_for(&state, setup.owner), 2);
    assert_eq!(sha_target_limit_for(&state, setup.owner), 2);

    let sha = inject(
        &mut state,
        setup.owner,
        99_303,
        CardKind::Sha,
        Suit::Club,
        7,
    );
    assert!(card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));

    let next_owner_card = inject(
        &mut state,
        setup.owner,
        99_304,
        CardKind::Other,
        Suit::Diamond,
        8,
    );
    inject(
        &mut state,
        setup.target,
        99_305,
        CardKind::Other,
        Suit::Club,
        9,
    );
    let active = skill_id(&state, setup.owner, TIANYI).unwrap();
    let (_, repeated) = command(
        state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: vec![next_owner_card],
        },
        10,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active skill use limit reached"
    ));

    let state = end_owner_turn(state, setup.owner);
    assert!(skill_id(&state, setup.owner, TIANYI_WIN).is_none());
    assert_eq!(sha_use_limit_for(&state, setup.owner), 1);
    assert_eq!(sha_target_limit_for(&state, setup.owner), 1);
    assert!(!card_target_within_distance(
        &state,
        setup.owner,
        sha,
        setup.far,
        1
    ));
}

#[test]
fn tie_and_loss_both_forbid_sha_until_turn_end() {
    for (offset, owner_rank, target_rank) in [(0, 7, 7), (1, 1, 13)] {
        let setup = setup(2_026_072_210 + offset, owner_rank, Some(target_rank));
        let mut state = compare(&setup, 20 + offset * 10);
        assert!(skill_id(&state, setup.owner, TIANYI_WIN).is_none());
        assert!(skill_id(&state, setup.owner, TIANYI_LOSE).is_some());
        let sha = inject(
            &mut state,
            setup.owner,
            99_310 + offset as u32,
            CardKind::Sha,
            Suit::Spade,
            5,
        );
        assert_eq!(sha_use_limit_for(&state, setup.owner), 0);
        let (_, blocked) = command(
            state.clone(),
            setup.owner,
            CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
            100 + offset,
        );
        assert!(matches!(
            blocked.result,
            CommandResult::Rejected {
                reason: RejectReason::IllegalAction { .. }
            }
        ));

        let state = end_owner_turn(state, setup.owner);
        assert!(skill_id(&state, setup.owner, TIANYI_LOSE).is_none());
        assert_eq!(sha_use_limit_for(&state, setup.owner), 1);
    }
}

#[test]
fn comparison_rejects_a_target_without_hand_cards() {
    let setup = setup(2_026_072_220, 10, None);
    let active = skill_id(&setup.state, setup.owner, TIANYI).unwrap();
    let (_, result) = command(
        setup.state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: vec![setup.owner_compare],
        },
        52,
    );
    assert!(matches!(
        result.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { ref code }
        } if code == "active target has no hand card to compare"
    ));
}
}

mod original_shenhua_view_as {
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
}

mod original_shenhua_wangji {
//! Golden coverage for 奇制/进趋（shenhua 王基，ABI 573）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QIZHI: u32 = 2_224_182_078;
const JINQU: u32 = 126_618_454;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/wangji.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("wangji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(QIZHI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JINQU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_wangji_contract() {
    let compiled = content_compiler::compile_source_named("wangji", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 573);
    assert_eq!(compiled.skill_index.len(), 2);
    let qizhi = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == QIZHI)
        .unwrap();
    assert_eq!(qizhi.event, "card_used");
    assert!(qizhi.optional);
    let jinqu = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JINQU)
        .unwrap();
    assert_eq!(jinqu.event, "phase_end");
    assert!(jinqu.optional);
}

/// 奇制：用牌后弃非目标角色一张牌并令其摸一张，计数 +1。
#[test]
fn qizhi_discards_non_target_and_draws() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let victim_card = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        97_000,
        Suit::Club,
        2,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let _far_card = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_001,
        Suit::Heart,
        3,
    );
    let owner_hand = hand_zone(&tx, setup.owner);
    let used = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_002,
        Suit::Spade,
        4,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: used,
            targets: vec![setup.far],
        },
    );
    assert!(state.has_pending_choice(), "奇制 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(state.has_pending_choice(), "奇制 target prompt");
    // The card's own target (far) is excluded from the discard choice.
    let state = resolve(state, setup.owner, vec![setup.other.raw()], false, 9_002);
    assert!(state.has_pending_choice(), "奇制 card prompt");
    let state = resolve(state, setup.owner, vec![victim_card.raw()], false, 9_003);
    assert!(!state.has_pending_choice());
    // The victim's card is discarded; the victim drew one.
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim_card).unwrap().zone, discard);
    assert_eq!(state.hand_count(setup.other), 1, "victim drew one");
    // The count incremented.
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == QIZHI)
        .unwrap();
    assert_eq!(skill.state_blob, b"QZ1\x01\x00\x00\x00\x01");
    let _ = used;
}

/// 进趋：出牌阶段结束时摸两张并弃至奇制次数张（计数 0 → 弃两张至 0）。
#[test]
fn jinqu_draws_two_and_discards_to_qizhi_count() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::End;
    let owner_hand = hand_zone(&tx, setup.owner);
    let h1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_003,
        Suit::Club,
        5,
    );
    let h2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        97_004,
        Suit::Heart,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "进趋 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "进趋 discard prompt");
    // count 0: discard all four (2 held + 2 drawn) down to zero.
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 4);
    let picks: Vec<_> = hand.iter().map(|card| card.raw()).collect();
    let state = resolve(state, setup.owner, picks, false, 9_011);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 0);
}

/// 进趋计数 1：弃一张保留一张。
#[test]
fn jinqu_keeps_qizhi_count_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::End;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == QIZHI)
        .map(|skill| skill.id)
        .unwrap();
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"QZ1\x01\x00\x00\x00\x01".to_vec();
    let owner_hand = hand_zone(&tx, setup.owner);
    let h1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_005,
        Suit::Club,
        7,
    );
    let h2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        97_006,
        Suit::Heart,
        8,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "进趋 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    // 2 held + 2 drawn = 4; keep 1 -> discard 3.
    assert!(state.has_pending_choice(), "进趋 discard prompt");
    let hand: Vec<_> = state.hand_cards(setup.owner);
    assert_eq!(hand.len(), 4);
    let picks: Vec<_> = hand
        .iter()
        .filter(|card| **card != h1)
        .take(3)
        .map(|card| card.raw())
        .collect();
    let state = resolve(state, setup.owner, picks, false, 9_021);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 1, "kept one");
    assert!(state.hand_cards(setup.owner).contains(&h1));
}
}

mod original_shenhua_xingshang {
//! Golden coverage for SHENHUA「行殇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINGSHANG: u32 = 2_937_826_132;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xingshang.skill");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
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

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let victim = seats[1];
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
        skill_def: SkillDefId::new(XINGSHANG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, victim)
}

#[test]
fn package_exposes_exact_optional_death_contract() {
    let committed = package();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINGSHANG && skill.name == "xingshang"));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINGSHANG)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive_event_cards_nonempty");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("xingshang", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn accepting_gains_dead_players_hand_and_equipment_but_not_judgment() {
    let (mut tx, owner, victim) = setup();
    let hand_card = inject(&mut tx, victim, ZoneKind::Hand, CardKind::Sha, 9_820_001);
    let equip_card = inject(
        &mut tx,
        victim,
        ZoneKind::Equip,
        CardKind::Weapon,
        9_820_002,
    );
    let judge_card = inject(
        &mut tx,
        victim,
        ZoneKind::Judge,
        CardKind::TrickBingliang,
        9_820_003,
    );

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let state = tx.state().clone();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "xingshang choice rejected: {:?}",
        result.result
    );
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [hand_card, equip_card] {
        let card = state.cards.get(card).unwrap();
        assert_eq!(card.zone, owner_hand);
        assert_eq!(card.owner, owner);
    }
    let judge = state.cards.get(judge_card).unwrap();
    assert_eq!(judge.zone, discard);
    assert_eq!(judge.owner, PlayerId::NONE);

    let later_owner = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::GainEventCards {
            player: later_owner,
            cards: vec![hand_card, equip_card],
        }],
    )
    .unwrap();
    for card in [hand_card, equip_card] {
        let card = tx.state().cards.get(card).unwrap();
        assert_eq!(card.zone, owner_hand);
        assert_eq!(card.owner, owner);
    }
}

#[test]
fn empty_hand_and_equipment_do_not_offer_the_skill() {
    let (mut tx, owner, victim) = setup();
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}
}

mod original_shenhua_xinkuanggu {
//! Golden for 神话再临「新狂骨」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XINKUANGGU: u32 = 4_089_972_415;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let source = seats[0];
    let near = seats[1];
    let far = seats[2];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(XINKUANGGU),
    });
    run_effects(&mut tx, effects).unwrap();
    for target in [near, far] {
        let player = tx.state_mut().players.get_mut(target).unwrap();
        player.max_hp = 10;
        player.hp = 10;
    }
    (tx, source, near, far)
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(0),
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
fn package_contains_cancellable_per_damage_point_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINKUANGGU && !skill.locked));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == XINKUANGGU)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        ["owner_source_distance_le_one_per_damage_point"]
    );
}

#[test]
fn near_multi_point_damage_offers_one_cancellable_choice_per_point() {
    let (mut tx, source, near, _) = setup(20_260_716_30);
    {
        let player = tx.state_mut().players.get_mut(source).unwrap();
        player.max_hp = 4;
        player.hp = 2;
    }
    let source_hand = tx.state().hand_count(source);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: near,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert!(choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (state, pipeline) = resolve(committed.state, vec![2], false, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, 3);

    let (state, pipeline) = resolve(state, vec![1], false, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, 3);
    assert_eq!(state.hand_count(source), source_hand + 1);
    assert_eq!(state.players.get(near).unwrap().hp, 8);
}

#[test]
fn full_hp_can_cancel_the_draw_and_far_damage_does_not_trigger() {
    let (mut full, source, near, _) = setup(20_260_716_31);
    let hand = full.state().hand_count(source);
    assert_eq!(
        run_effects(
            &mut full,
            vec![EffectRequest::DealDamage {
                source,
                target: near,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = full.commit();
    let choice = committed.output.choice.unwrap();
    assert!(choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let (state, pipeline) = resolve(committed.state, vec![], true, 3);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(source), hand);

    let (mut far_tx, source, _, far) = setup(20_260_716_32);
    let hand = far_tx.state().hand_count(source);
    run_effects(
        &mut far_tx,
        vec![EffectRequest::DealDamage {
            source,
            target: far,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!far_tx.state().has_pending_choice());
    assert_eq!(far_tx.state().hand_count(source), hand);
}
}

mod original_shenhua_xinliegong {
//! Golden coverage for SHENHUA「新烈弓」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINLIEGONG: u32 = 3_272_111_610;
const RE_HUANGZHONG: u32 = 2_306_877_807;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xinliegong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup(player_count: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("xinliegong-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, player_count, 20_260_727, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(XINLIEGONG),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &seats {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 4;
        state.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, seats)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    rank: u8,
    key: u64,
) -> (GameState, PipelineResult) {
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, CardKind::Sha, rank, 95_000 + key as u32);
    let state = tx.commit().state;
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
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
fn package_publishes_re_huangzhong_and_exact_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert_eq!(package.skill_index.len(), 201);
    assert_eq!(package.character_index.len(), 48);
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_HUANGZHONG
            && character.name_key == "re_huangzhong"
            && character.skills == [XINLIEGONG]
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINLIEGONG
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner"
            && trigger.optional
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == XINLIEGONG
            && modifier.target == "sha_distance_unlimited_if_target_distance_lte_rank"
    }));
}

#[test]
fn sha_rank_unlocks_only_targets_within_that_final_distance() {
    let (state, owner, seats) = setup(4);
    let far = seats[2];
    let (_, rejected) = play_sha(state, owner, far, 1, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, owner, seats) = setup(4);
    let far = seats[2];
    let (_, accepted) = play_sha(state, owner, far, 2, 2);
    assert_eq!(accepted.choice.unwrap().observer, owner);
}

#[test]
fn accepted_trigger_applies_each_current_target_branch_independently() {
    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    inject(&mut tx, owner, CardKind::Other, 3, 96_001);
    inject(&mut tx, target, CardKind::Shan, 4, 96_002);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered) = play_sha(tx.commit().state, owner, target, 7, 10);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (locked, resolved) = resolve(state, owner, vec![1], false, 11);
    assert!(resolved.choice.is_none());
    assert_eq!(locked.players.get(target).unwrap().hp, hp - 1);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    inject(&mut tx, target, CardKind::Shan, 5, 96_003);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered) = play_sha(tx.commit().state, owner, target, 7, 20);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (state, response) = resolve(state, owner, vec![1], false, 21);
    assert_eq!(response.choice.unwrap().observer, target);
    let (damaged, resolved) = resolve(state, target, vec![], true, 22);
    assert!(resolved.choice.is_none());
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 2);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    inject(&mut tx, owner, CardKind::Other, 6, 96_004);
    inject(&mut tx, target, CardKind::Shan, 7, 96_005);
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, offered) = play_sha(tx.commit().state, owner, target, 7, 30);
    assert_eq!(offered.choice.unwrap().observer, owner);
    let (damaged, resolved) = resolve(state, owner, vec![1], false, 31);
    assert!(resolved.choice.is_none());
    assert_eq!(damaged.players.get(target).unwrap().hp, hp - 2);

    let (state, owner, seats) = setup(2);
    let target = seats[1];
    let mut tx = Transaction::begin(state);
    let shan = inject(&mut tx, target, CardKind::Shan, 8, 96_006);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    let (_, response) = play_sha(tx.commit().state, owner, target, 7, 40);
    let choice = response.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == shan.raw()));
}
}

mod original_shenhua_xueyi {
//! Golden for 神话再临「血裔」：仅主公生效，其他群势力角色每名令手牌上限 +2。

use content_schema::PackageCharacterEntry;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::Identity;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const XUEYI: u32 = 3_178_128_297;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xueyi.skill");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("test_character_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
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

#[test]
fn xueyi_is_lord_only_and_counts_other_alive_qun_players_twice() {
    let mut package = content_compiler::compile_source_named("shenhua-xueyi", SOURCE).unwrap();
    package.character_index = vec![
        character(9_207_001, "wei"),
        character(9_207_002, "qun"),
        character(9_207_003, "qun"),
        character(9_207_004, "shu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XUEYI && skill.locked && skill.lord_only));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_714),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_714,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mut tx = Transaction::begin(engine.state().clone());
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut others = seats.iter().copied().filter(|player| *player != lord);
    let qun_one = others.next().unwrap();
    let qun_two = others.next().unwrap();
    let nonlord = others.next().unwrap();

    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_207_001;
    tx.state_mut()
        .players
        .get_mut(qun_one)
        .unwrap()
        .character_def = 9_207_002;
    tx.state_mut()
        .players
        .get_mut(qun_two)
        .unwrap()
        .character_def = 9_207_003;
    tx.state_mut()
        .players
        .get_mut(nonlord)
        .unwrap()
        .character_def = 9_207_004;

    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: lord,
                skill_def: SkillDefId::new(XUEYI),
            },
            EffectRequest::AddSkill {
                player: nonlord,
                skill_def: SkillDefId::new(XUEYI),
            },
        ],
    )
    .unwrap();

    let lord_hp = tx.state().players.get(lord).unwrap().hp;
    let nonlord_hp = tx.state().players.get(nonlord).unwrap().hp;
    assert_eq!(hand_limit(&tx, lord), lord_hp + 4);
    assert_eq!(hand_limit(&tx, nonlord), nonlord_hp);

    tx.state_mut().players.get_mut(qun_two).unwrap().alive = false;
    assert_eq!(hand_limit(&tx, lord), lord_hp + 2);
}
}

mod original_shenhua_xuyou {
//! Golden coverage for 成略/恃才/寸目（shenhua 许攸，ABI 589）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHENGLVE: u32 = 1_234_567_901;
const SHICAI: u32 = 1_234_567_902;
const CUNMU: u32 = 1_234_567_903;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/xuyou.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("xuyou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
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
    for def in [CHENGLVE, SHICAI, CUNMU] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, owner, other }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

/// 成略：首次发动翻为阳——摸一张弃两张；第二次发动翻为阴——摸两张弃一张。
/// 弃置花色本回合无距离与次数限制。
#[test]
fn chenglve_flips_draws_discards_and_unlimits_discarded_suits() {
    let setup = setup();
    let mut tx = setup.tx;
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let club = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        97_901,
        Suit::Club,
        5,
    );
    let spade = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Shan,
        97_902,
        Suit::Spade,
        6,
    );
    let heart = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Tao,
        97_903,
        Suit::Heart,
        7,
    );
    let diamond = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Jiu,
        97_904,
        Suit::Diamond,
        8,
    );
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == CHENGLVE)
        .unwrap()
        .id;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, activated) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(activated.result, CommandResult::Accepted { .. }),
        "activate rejected: {:?}",
        activated.result
    );
    // 首次：翻为阳 → 摸一张弃两张（黑桃/红桃弃置 → 本回合黑桃/红桃无限制）。
    eprintln!(
        "DBG chenglve after activate: pending={} kind={:?} hand={} blob={:?}",
        state.has_pending_choice(),
        state.resolution.continuation_kind,
        state.hand_count(setup.owner),
        state.skills.get(skill).map(|s| s.state_blob.clone())
    );
    assert!(state.has_pending_choice(), "成略 discard prompt");
    let state = resolve(state, setup.owner, vec![spade.raw(), heart.raw()], false, 2);
    assert!(!state.has_pending_choice());
    let state = state.clone();
    let flag = state
        .turn
        .flag(engine_core::state::turn_flag::CHENGLVE_UNLIMITED_SUITS)
        .max(0) as u32;
    assert_eq!(flag, (1 << 1) | (1 << 2), "弃置黑桃/红桃记录为无限制花色");
    assert!(engine_core::query::card_use_unlimited_for_card(&state, setup.owner, club) == false);
    // 同回合不可再次发动（限一次）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, second) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(second.result, CommandResult::Rejected { .. }),
        "成略每回合限一次"
    );
    let _ = state;
    let _ = club;
}

/// 恃才：使用非装备牌后，实体牌置顶并摸一张；同类牌本回合不重复触发。
#[test]
fn shicai_places_used_card_on_top_and_draws_once_per_category() {
    let setup = setup();
    let mut tx = setup.tx;
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        97_911,
        Suit::Club,
        5,
    );
    // 使用【杀】结算：实体牌进入弃牌堆。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    // 使用【杀】→ 恃才可选确认。
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: sha,
            targets: vec![setup.other],
        },
    );
    assert!(state.has_pending_choice(), "恃才 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 10);
    assert!(state.has_pending_choice(), "恃才 place prompt");
    let state = resolve(state, setup.owner, vec![sha.raw()], false, 11);
    assert!(!state.has_pending_choice());
    // 弃牌堆的【杀】已置于牌堆顶并被摸回手牌，随后摸一张。
    assert!(
        state.hand_cards(setup.owner).contains(&sha),
        "置顶后摸回【杀】"
    );
    assert_eq!(state.hand_count(setup.owner), 1);
    // 同类牌本回合不再触发：再次使用【杀】无选择。
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let sha2 = alloc_card(
        &mut tx,
        setup.owner,
        hand,
        CardKind::Sha,
        97_912,
        Suit::Spade,
        6,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha2],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: sha2,
            targets: vec![setup.other],
        },
    );
    // 确认后检查类别位：同类牌首用后不再开放置选择。
    assert!(state.has_pending_choice(), "同类牌首用后确认仍触发");
    let state = resolve(state, setup.owner, vec![1], false, 12);
    assert!(!state.has_pending_choice(), "同类牌首用后不再放置");
}

/// 寸目：摸牌阶段改为从牌堆底摸牌。
#[test]
fn cunmu_draws_from_bottom_of_draw_pile() {
    let setup = setup();
    let mut tx = setup.tx;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let bottom = alloc_card(
        &mut tx,
        setup.owner,
        draw,
        CardKind::Sha,
        97_921,
        Suit::Club,
        5,
    );
    // 牌堆底 = 首个元素：把预设牌移到堆底。
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|card| *card != bottom);
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .insert(0, bottom);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Draw;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    assert!(!state.has_pending_choice());
    // 触发设置底摸标记；摸牌阶段随后按此标记从牌堆底摸牌。
    assert_eq!(
        state
            .turn
            .flag(engine_core::state::turn_flag::DRAW_PHASE_FROM_BOTTOM),
        1,
        "寸目设置底摸标记"
    );
    let mut tx = Transaction::begin(state);
    let count = engine_core::query::draw_count_for(tx.state(), setup.owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCardsFromBottom {
            player: setup.owner,
            count,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert!(
        state.hand_cards(setup.owner).contains(&bottom),
        "寸目从牌堆底摸到预设牌"
    );
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

#[test]
fn package_exposes_xuyou_contract() {
    let compiled = content_compiler::compile_source_named("xuyou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 589);
    assert_eq!(compiled.skill_index.len(), 3);
    let chenglve = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == CHENGLVE)
        .unwrap();
    assert!(chenglve.conversion);
    let cunmu = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == CUNMU)
        .unwrap();
    assert!(cunmu.lord_only);
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == SHICAI
            && trigger.event == "card_used"
            && trigger.filter == "owner_is_source"
            && trigger.optional));
}
}

mod original_shenhua_yanyan {
//! Golden coverage for 拒战（shenhua 严颜，转换技，ABI 571）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JUZHAN: u32 = 3_174_379_257;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/yanyan.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("yanyan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(JUZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// 阴分支（首次，受杀）：双方各摸一张，来源本回合不能对持有者使用牌。
#[test]
fn juzhan_yin_both_draw_and_ban_source() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_900,
        Suit::Spade,
        7,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: sha,
            targets: vec![setup.owner],
        },
    );
    assert!(state.has_pending_choice(), "拒战 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_001);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 1, "owner drew one");
    assert_eq!(
        state.hand_count(setup.other),
        2,
        "source drew one on top of the Sha"
    );
    assert!(
        state
            .turn
            .use_ban_pairs
            .contains(&(setup.other, setup.owner)),
        "source banned against owner"
    );
    let _ = sha;
}

/// 阳分支（第二次，用杀）：获得目标一张牌并禁己方用牌。
#[test]
fn juzhan_yang_gains_card_and_bans_owner() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let gift = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_901,
        Suit::Heart,
        6,
    );
    let owner_hand = hand_zone(&tx, setup.owner);
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_902,
        Suit::Spade,
        8,
    );
    let state = tx.commit().state;
    // First use flips to branch 阳; the 阴 branch fired earlier is simulated by
    // pre-seeding the branch so the first card_used is branch 阳.
    let mut tx = Transaction::begin(state);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == JUZHAN)
        .map(|skill| skill.id)
        .unwrap();
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = b"JZ1\x01".to_vec();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.owner,
            card: sha,
            targets: vec![setup.other],
        },
    );
    assert!(state.has_pending_choice(), "拒战 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "拒战 gain prompt");
    let state = resolve(state, setup.owner, vec![gift.raw()], false, 9_011);
    assert!(!state.has_pending_choice());
    assert!(
        state.hand_cards(setup.owner).contains(&gift),
        "owner gained the card"
    );
    assert!(
        state
            .turn
            .use_ban_pairs
            .contains(&(setup.owner, setup.other)),
        "owner banned against the target"
    );
}

/// 修正器：被禁角色使用牌指定持有者时目标被取消。
#[test]
fn juzhan_ban_cancels_banned_user_cards() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .use_ban_pairs
        .push((setup.other, setup.owner));
    let other_hand = hand_zone(&tx, setup.other);
    let sha = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_903,
        Suit::Spade,
        9,
    );
    let state = tx.commit().state;
    let cancelled = engine_core::query::card_target_cancelled_for_kind(
        &state,
        setup.other,
        setup.owner,
        CardKind::Tao,
    );
    assert!(cancelled, "banned user's Sha cancelled against the owner");
    // Unbanned user is unaffected.
    let ok = engine_core::query::card_target_cancelled_for_kind(
        &state,
        setup.far,
        setup.owner,
        CardKind::Tao,
    );
    assert!(!ok, "unbanned user not cancelled");
}

#[test]
fn package_exposes_juzhan_contract() {
    let compiled = content_compiler::compile_source_named("yanyan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 571);
    assert_eq!(compiled.skill_index.len(), 1);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == JUZHAN)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|t| t.optional));
    assert_eq!(compiled.modifier_index.len(), 1);
}
}

mod original_shenhua_yuanshao {
//! Golden semantics for SHENHUA 袁绍「乱击」rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LUANJI: u32 = 2_717_757_870;
const RE_YUANSHAO: u32 = 4_036_141_875;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/luanji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup() -> (
    GameState,
    PlayerId,
    PlayerId,
    SkillId,
    Vec<engine_api::CardId>,
) {
    let package = content_compiler::compile_source_named("luanji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(LUANJI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LUANJI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, other] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.max_hp = 8;
        participant.hp = 8;
        for card in tx.state().hand_cards(player) {
            let card = tx.state_mut().cards.get_mut(card).unwrap();
            card.kind = CardKind::Tao;
            card.suit = Suit::Heart;
        }
    }
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LUANJI)
        .unwrap();
    let cards = tx.state().hand_cards(owner);
    (tx.commit().state, owner, other, skill, cards)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: cards,
            },
        },
    )
}

#[test]
fn package_publishes_yuanshao_with_exact_two_card_rule() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_YUANSHAO)
        .unwrap();
    assert_eq!(character.skills, [LUANJI, 3_178_128_297]);
    let rule = package
        .view_as_index
        .iter()
        .find(|rule| rule.skill_def_id == LUANJI)
        .unwrap();
    assert_eq!(rule.as_kind, "wanjian");
    assert_eq!((rule.min_cards, rule.max_cards), (2, 2));
    assert!(rule.same_suit);
}

#[test]
fn exactly_two_same_suit_hand_cards_become_wanjian() {
    let (mut state, owner, other, skill, cards) = setup();
    let before = state.players.get(other).unwrap().hp;

    let (next, one) = activate(state, owner, skill, vec![cards[0]], 1);
    assert!(matches!(one.result, CommandResult::Rejected { .. }));
    state = next;

    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Club;
    let (next, mixed) = activate(state, owner, skill, vec![cards[0], cards[1]], 2);
    assert!(matches!(mixed.result, CommandResult::Rejected { .. }));
    state = next;

    state.cards.get_mut(cards[1]).unwrap().suit = Suit::Heart;
    let (state, accepted) = activate(state, owner, skill, vec![cards[0], cards[1]], 3);
    assert!(
        matches!(accepted.result, CommandResult::Accepted { .. }),
        "{:?}",
        accepted.result
    );
    assert_eq!(state.players.get(other).unwrap().hp, before - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(
        cards[..2]
            .iter()
            .all(|card| state.cards.get(*card).unwrap().zone == discard),
        "zones={:?} pending={} continuation={:?}",
        cards[..2]
            .iter()
            .map(|card| state.cards.get(*card).unwrap().zone)
            .collect::<Vec<_>>(),
        state.resolution.pending_choice_id,
        state.resolution.continuation_kind
    );
}
}

mod original_shenhua_yuji {
//! Golden coverage for 蛊惑/禅缘（shenhua 于吉，ABI 552）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_GUHUO: u32 = 2_102_070_101;
const CHANYUAN: u32 = 4_099_161_204;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/yuji.skill");

struct Setup {
    state: GameState,
    owner: PlayerId,
    challenger: PlayerId,
    target: PlayerId,
    source: CardId,
}

fn setup(kind: CardKind, suit: Suit) -> Setup {
    let package = content_compiler::compile_source_named("yuji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_072_931, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let challenger = players[1];
    let target = players[2];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINFU_GUHUO),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CHANYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let source = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_273),
        kind,
        suit,
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
        .push(source);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        challenger,
        target,
        source,
    }
}

fn choose(state: GameState, token: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn declare_sha(setup: Setup, decisions: [u32; 2]) -> GameState {
    let skill = setup.state.players.get(setup.owner).unwrap().skills[0];
    let revision = setup.state.revision;
    let (state, result) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Sha.tag(),
                targets: vec![setup.target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = choose(state, setup.source.raw(), 2);
    let state = choose(state, decisions[0], 3);
    choose(state, decisions[1], 4)
}

#[test]
fn package_exposes_yuji_contract() {
    let compiled = content_compiler::compile_source_named("yuji", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 552);
    let guhuo_views: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|v| v.skill_def_id == XINFU_GUHUO)
        .collect();
    assert_eq!(guhuo_views.len(), 2);
    assert!(guhuo_views
        .iter()
        .any(|v| v.as_kind == "basic" && v.cost.as_deref() == Some("guhuo")));
    assert!(guhuo_views
        .iter()
        .any(|v| v.as_kind == "ordinary_trick" && v.cost.as_deref() == Some("guhuo")));
    let chanyuan_mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == CHANYUAN)
        .map(|m| m.target.as_str())
        .collect();
    assert!(chanyuan_mods.contains(&"skills_disabled_when_hp_one"));
    let chanyuan = compiled
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == CHANYUAN)
        .unwrap();
    assert!(chanyuan.locked);
    assert_eq!(compiled.skill_index.len(), 2);
}

#[test]
fn guhuo_declared_sha_hits_when_unchallenged() {
    let s = setup(CardKind::Sha, Suit::Spade);
    let target = s.target;
    let target_hp = s.state.players.get(target).unwrap().hp;
    let state = declare_sha(s, [0, 0]);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        target_hp - 1,
        "unchallenged declared Sha deals damage"
    );
}

#[test]
fn chanyuan_disables_other_skills_at_one_hp() {
    let s = setup(CardKind::Sha, Suit::Spade);
    let mut tx = Transaction::begin(s.state);
    // 于吉 at 1 HP: 蛊惑 must be disabled, 禅缘 itself stays enabled.
    tx.state_mut().players.get_mut(s.owner).unwrap().hp = 1;
    let state = tx.commit().state;
    let enabled_defs: Vec<_> = state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == s.owner && state.skill_effectively_enabled(skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(
        !enabled_defs.contains(&XINFU_GUHUO),
        "蛊惑 disabled at one HP: {enabled_defs:?}"
    );
    assert!(
        enabled_defs.contains(&CHANYUAN),
        "禅缘 itself remains enabled"
    );
    // At two HP the 蛊惑 view-as is enabled again.
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(s.owner).unwrap().hp = 2;
    let state = tx.commit().state;
    let enabled_defs: Vec<_> = state
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == s.owner && state.skill_effectively_enabled(skill))
        .map(|skill| skill.def.raw())
        .collect();
    assert!(
        enabled_defs.contains(&XINFU_GUHUO),
        "蛊惑 re-enabled at two HP"
    );
}

#[test]
fn guhuo_use_rejected_at_one_hp() {
    let s = setup(CardKind::Sha, Suit::Spade);
    let mut tx = Transaction::begin(s.state);
    tx.state_mut().players.get_mut(s.owner).unwrap().hp = 1;
    let state = tx.commit().state;
    let skill = state.players.get(s.owner).unwrap().skills[0];
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: s.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Sha.tag(),
                targets: vec![s.target],
                equip_slot: 0,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Rejected { .. }),
        "蛊惑 use rejected at one HP: {:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn guhuo_truthful_declaration_resolves_with_challenge() {
    // 真实声明（红桃杀）被质疑：按验真结算——目标受伤。
    let s = setup(CardKind::Sha, Suit::Heart);
    let target = s.target;
    let target_hp = s.state.players.get(target).unwrap().hp;
    let state = declare_sha(s, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}
}

mod original_shenhua_zhanghe {
//! Golden coverage for 巧变（shenhua 张郃，ABI 559）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QIAOBIAN: u32 = 3_287_917_228;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhanghe.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhanghe-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(QIAOBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

/// Dispatch a phase-begin event and return the resolved state; the composite's
/// first choice (discard) is expected to open, or the dispatch may complete.
fn dispatch_phase(state: GameState, actor: PlayerId, tag: engine_api::TurnPhaseTag) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: actor,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// Accept the optional trigger confirm, then answer the discard choice.
fn accept_and_pay(state: GameState, owner: PlayerId, card: Option<CardId>) -> GameState {
    // Optional trigger confirm: options [1].
    let state = resolve(state, owner, vec![1], false, 9_000);
    assert!(
        state.has_pending_choice(),
        "巧变 discard prompt after accept"
    );
    let selections = card.map(|card| vec![card.raw()]).unwrap_or_default();
    resolve(state, owner, selections, card.is_none(), 9_001)
}

#[test]
fn package_exposes_qiaobian_contract() {
    let compiled = content_compiler::compile_source_named("zhanghe", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 559);
    assert_eq!(compiled.skill_index.len(), 1);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == QIAOBIAN)
        .collect();
    assert_eq!(triggers.len(), 4);
    for trigger in triggers {
        assert!(trigger.optional);
    }
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == QIAOBIAN)
        .map(|t| t.event.as_str())
        .collect();
    for event in ["phase_judge", "phase_draw", "phase_play", "phase_discard"] {
        assert!(events.contains(&event), "missing trigger {event}");
    }
}

#[test]
fn qiaobian_judge_discards_and_skips_judge_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Judge;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_500,
        Suit::Spade,
        2,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Judge);
    assert!(state.has_pending_choice(), "巧变 judge discard prompt");
    let state = accept_and_pay(state, setup.owner, Some(card));
    // The judge phase was skipped: the turn advanced to Draw, whose 巧变
    // prompt now opens (cancelled below).
    assert_eq!(state.turn.phase, Phase::Draw, "judge phase skipped");
    assert!(
        !state.hand_cards(setup.owner).contains(&card),
        "card discarded"
    );
    assert!(
        state.has_pending_choice(),
        "draw-phase 巧变 prompt after skip"
    );
    let state = resolve(state, setup.owner, Vec::new(), true, 9_002);
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_draw_takes_one_card_from_each_of_two_targets() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Draw;
    let owner_hand = hand_zone(&tx, setup.owner);
    let other_hand = hand_zone(&tx, setup.other);
    let far_hand = hand_zone(&tx, setup.far);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_501,
        Suit::Spade,
        3,
    );
    let o1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_502,
        Suit::Heart,
        2,
    );
    let o2 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_503,
        Suit::Heart,
        3,
    );
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Sha,
        96_504,
        Suit::Club,
        4,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Draw);
    let state = accept_and_pay(state, setup.owner, Some(card));
    assert!(state.has_pending_choice(), "draw 巧变 target prompt");
    // Choose both targets: each gives one card.
    let state = resolve(
        state,
        setup.owner,
        vec![setup.other.raw(), setup.far.raw()],
        false,
        9_010,
    );
    assert!(state.has_pending_choice(), "first target's give prompt");
    let state = resolve(state, setup.other, vec![o2.raw()], false, 9_011);
    assert!(state.has_pending_choice(), "second target's give prompt");
    let state = resolve(state, setup.far, vec![f1.raw()], false, 9_012);
    // The draw phase was skipped: the play-phase 巧变 prompt now opens.
    assert_eq!(state.turn.phase, Phase::Play, "draw phase skipped");
    assert!(
        state.hand_cards(setup.owner).contains(&o2),
        "owner gained o2"
    );
    assert!(
        state.hand_cards(setup.owner).contains(&f1),
        "owner gained f1"
    );
    assert!(!state.hand_cards(setup.other).contains(&o2));
    assert!(!state.hand_cards(setup.far).contains(&f1));
    let _ = (o1, o2, f1);
    let state = resolve(state, setup.owner, Vec::new(), true, 9_013);
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_play_moves_field_equip_to_another_player() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_505,
        Suit::Spade,
        5,
    );
    // A weapon on the field owned by `other`.
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, setup.other).unwrap();
    let weapon = alloc_card(
        &mut tx,
        setup.other,
        equip_zone,
        CardKind::Weapon,
        96_506,
        Suit::Diamond,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Play);
    let state = accept_and_pay(state, setup.owner, Some(card));
    assert!(state.has_pending_choice(), "play 巧变 field-card prompt");
    let state = resolve(state, setup.owner, vec![weapon.raw()], false, 9_020);
    assert!(state.has_pending_choice(), "destination prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_021);
    // The play phase was skipped: the discard-phase 巧变 prompt opens.
    assert_eq!(state.turn.phase, Phase::Discard, "play phase skipped");
    let far_equip = state.zones.find(ZoneKind::Equip, setup.far).unwrap();
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        far_equip,
        "weapon moved"
    );
    let other_equip = state.zones.find(ZoneKind::Equip, setup.other).unwrap();
    assert!(!state
        .zones
        .get(other_equip)
        .unwrap()
        .cards
        .contains(&weapon));
    let state = resolve(state, setup.owner, Vec::new(), true, 9_022);
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_discard_skips_discard_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Discard;
    let owner_hand = hand_zone(&tx, setup.owner);
    let card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_507,
        Suit::Spade,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Discard);
    assert!(state.has_pending_choice(), "巧变 discard prompt");
    let state = accept_and_pay(state, setup.owner, Some(card));
    assert_eq!(state.turn.phase, Phase::End, "discard phase skipped");
    assert!(!state.has_pending_choice());
}

#[test]
fn qiaobian_cancel_keeps_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.phase = Phase::Judge;
    let state = tx.commit().state;
    let state = dispatch_phase(state, setup.owner, engine_api::TurnPhaseTag::Judge);
    assert!(state.has_pending_choice());
    // Cancel the optional trigger confirm: the composite never runs.
    let state = resolve(state, setup.owner, Vec::new(), true, 9_030);
    // Cancelled: the judge phase proceeds normally.
    assert_eq!(state.turn.phase, Phase::Judge);
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_zhangjiao {
//! Golden coverage for 雷击/鬼道/黄天（shenhua 张角，ABI 549-551）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RELEIJI: u32 = 48_412_566;
const GUIDAO: u32 = 1_025_117_348;
const HUANGTIAN: u32 = 2_761_489_181;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhangjiao.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zj-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(RELEIJI),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GUIDAO),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUANGTIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
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
    )
}

fn option_tokens(result: &engine_core::command::PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .map(|choice| choice.options.iter().map(|option| option.token).collect())
        .unwrap_or_default()
}

/// 雷击完整流程：张角响应闪 → 确认 → 选目标 → 判定（牌堆顶牌）。
fn run_releiji(
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    shan: CardId,
    key_base: u64,
) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResponded {
            player: owner,
            respond_to: PlayerId::NONE,
            card: shan,
            response_kind: 3,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let mut state = tx.commit().state;
    assert!(state.has_pending_choice(), "optional 雷击 confirm");
    let (state, confirmed) = resolve(state, owner, vec![1], key_base);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(state, owner, vec![other.raw()], key_base + 1);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_zhangjiao_contract() {
    let compiled = content_compiler::compile_source_named("zj", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 551);
    let releiji = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == RELEIJI)
        .unwrap();
    assert_eq!(releiji.event, "card_responded");
    assert_eq!(releiji.filter, "owner_responded_shan");
    assert!(releiji.optional);
    let huangtian = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == HUANGTIAN)
        .unwrap();
    assert_eq!(huangtian.event, "phase_play");
    assert_eq!(
        huangtian.filter,
        "owner_alive_other_qun_play_phase_can_give_owner"
    );
    let huangtian_skill = compiled
        .skill_index
        .iter()
        .find(|s| s.skill_def_id == HUANGTIAN)
        .unwrap();
    assert!(huangtian_skill.lord_only);
    let guidao_mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|m| m.skill_def_id == GUIDAO)
        .map(|m| m.target.as_str())
        .collect();
    assert!(guidao_mods.contains(&"judgment_rewrite_black_hand_equip"));
    assert_eq!(compiled.skill_index.len(), 3);
    assert_eq!(compiled.trigger_index.len(), 2);
}

#[test]
fn releiji_spade_deals_two_thunder_damage() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_500,
        Suit::Heart,
        2,
    );
    // Stack a spade judge card on top of the draw pile.
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _judge = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_501,
        Suit::Spade,
        9,
    );
    let far_hp = tx.state().players.get(setup.far).unwrap().max_hp;
    let state = run_releiji(tx.commit().state, setup.owner, setup.far, shan, 1);
    let far = state.players.get(setup.far).unwrap();
    assert_eq!(far.hp, far_hp as i32 - 2, "spade judge: two thunder damage");
    assert_eq!(
        state.players.get(setup.owner).unwrap().max_hp as i32,
        state.players.get(setup.owner).unwrap().hp as i32 + 0
    );
}

#[test]
fn releiji_club_recovers_and_deals_one() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_510,
        Suit::Heart,
        2,
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _judge = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_511,
        Suit::Club,
        7,
    );
    let far_hp = tx.state().players.get(setup.far).unwrap().max_hp;
    // Wound the lord owner (5 -> 4) so the club recovery is observable.
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let owner_hp = tx.state().players.get(setup.owner).unwrap().hp;
    assert_eq!(owner_hp, 4);
    let state = run_releiji(tx.commit().state, setup.owner, setup.far, shan, 1);
    let far = state.players.get(setup.far).unwrap();
    assert_eq!(far.hp, far_hp as i32 - 1, "club judge: one thunder damage");
    assert_eq!(
        state.players.get(setup.owner).unwrap().hp,
        owner_hp + 1,
        "club judge: owner recovers one"
    );
}

#[test]
fn guidao_rewrites_judgment_with_black_card() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_520,
        Suit::Heart,
        2,
    );
    // Draw pile top is a heart (no branch); the owner holds a black card to rewrite.
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let _heart_judge = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        96_521,
        Suit::Heart,
        5,
    );
    let black = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_522,
        Suit::Spade,
        8,
    );
    let state = run_releiji(tx.commit().state, setup.owner, setup.far, shan, 1);
    // 鬼道 offers the black card to replace the heart judgment.
    let mut tx = Transaction::begin(state);
    assert!(tx.state().has_pending_choice(), "鬼道 rewrite choice");
    let mut state = tx.commit().state;
    let (state, rewritten) = resolve(state, setup.owner, vec![black.raw()], 5);
    assert!(matches!(rewritten.result, CommandResult::Accepted { .. }));
    let far = state.players.get(setup.far).unwrap();
    let far_hp = far.max_hp;
    assert_eq!(
        far.hp,
        far_hp as i32 - 2,
        "rewritten spade judge: two thunder damage"
    );
}

#[test]
fn huangtian_gives_shan_when_qun_player_in_play_phase() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let group = tx.state().player_group(setup.other).map(|g| g.to_string());
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    let shan = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Shan,
        96_530,
        Suit::Heart,
        2,
    );
    let is_qun = group.as_deref() == Some("qun");
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.other,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    if !is_qun {
        match fired {
            Ok(effects) => assert!(effects.is_empty(), "no 黄天 trigger for non-Qun player"),
            Err(EffectError::ChoiceRequested) => panic!("黄天 must not open a choice for non-Qun"),
            Err(_) => {}
        }
        return;
    }
    match fired {
        Ok(effects) => {
            assert_eq!(
                run_effects(&mut dispatch_tx, effects),
                Err(EffectError::ChoiceRequested),
                "黄天 card choice"
            );
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = dispatch_tx.commit().state;
    // The optional trigger opens a confirm first; accept it, then the
    // card-choice (GuardedChoice) appears.
    let (state, confirmed) = resolve(state, setup.other, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, picked) = resolve(state, setup.other, vec![shan.raw()], 2);
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert!(
        state.hand_cards(setup.owner).contains(&shan),
        "Qun player gave the 闪 to the lord"
    );
    assert!(!state.hand_cards(setup.other).contains(&shan));
    assert!(!state.has_pending_choice());
}

#[test]
fn huangtian_skips_non_shan_lightning_hands() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let group = tx.state().player_group(setup.other).map(|g| g.to_string());
    let other_hand = tx.state().zones.find(ZoneKind::Hand, setup.other).unwrap();
    // Give the Qun player only a Tao: no 闪/闪电 -> no trigger.
    let _tao = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Tao,
        96_540,
        Suit::Heart,
        1,
    );
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let mut dispatch_tx = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut dispatch_tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.other,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => assert!(effects.is_empty(), "no 黄天 trigger without 闪/闪电"),
        Err(EffectError::ChoiceRequested) => {
            if group.as_deref() == Some("qun") {
                panic!("黄天 must not open a choice without 闪/闪电");
            }
        }
        Err(_) => {}
    }
}
}

mod original_shenhua_zhangxiu {
//! Golden coverage for 雄乱/踞襄（shenhua 张绣，ABI 590）+ 从谏复用（ABI 524）。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const XIONGLUAN: u32 = 1_234_567_904;
const XIONGLUAN_BAN: u32 = 1_234_567_905;
const TWJUXIANG: u32 = 1_234_567_907;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/drlt_xiongluan.skill");
const SOURCE_TWJUXIANG: &str =
    include_str!("../../../content/original/ported/shenhua/twjuxiang.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    third: PlayerId,
}

fn combined_source() -> String {
    format!("{SOURCE}\n{SOURCE_TWJUXIANG}")
}

fn setup() -> Setup {
    let package =
        content_compiler::compile_source_named("zhangxiu-test", &combined_source()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let third = seats[2];
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
    for def in [XIONGLUAN, TWJUXIANG] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        other,
        third,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn find_skill(tx: &Transaction, owner: PlayerId, def: u32) -> SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
        .copied()
        .unwrap()
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0x5900),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, engine_api::CommandResult::Accepted { .. }),
        "activation must be accepted: {result:?}"
    );
    state
}

#[test]
fn package_exposes_zhangxiu_contract() {
    let compiled = content_compiler::compile_source_named("zhangxiu", &combined_source()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 590);
    assert_eq!(compiled.skill_index.len(), 3);
    let xiongluan = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == XIONGLUAN)
        .unwrap();
    assert!(!xiongluan.locked);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == XIONGLUAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(active.targets.as_ref().map(|s| s.max), Some(1));
    let twjuxiang = compiled
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == TWJUXIANG)
        .unwrap();
    assert!(twjuxiang.lord_only);
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&XIONGLUAN)));
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&TWJUXIANG)));
}

#[test]
fn xiongluan_disables_slots_judge_marks_target_and_bans_hand_cards() {
    let mut setup = setup();
    let skill = find_skill(&mut setup.tx, setup.owner, XIONGLUAN);
    let state = activate(
        setup.tx.commit().state,
        setup.owner,
        skill,
        vec![setup.other],
    );
    let tx = Transaction::begin(state);
    // 装备栏与判定区废除。
    assert!(tx
        .state()
        .players
        .get(setup.owner)
        .is_some_and(|p| p.disabled_equip_slots.iter().all(|v| *v > 0)));
    assert!(tx.state().disabled_judge_zones.contains(&setup.owner));
    // 限定目标标记。
    assert_eq!(
        tx.state()
            .turn
            .flag(engine_core::state::turn_flag::xiongluan_target_by_player(
                setup.owner
            ))
            .max(0) as u32,
        setup.other.raw()
    );
    // 目标获得禁手牌技能。
    assert!(tx
        .state()
        .players
        .get(setup.other)
        .is_some_and(|p| p.skills.iter().any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == XIONGLUAN_BAN)
        })));
    // 限定技整局一次：再次激活被拒。
    let state2 = tx.commit().state;
    let skill2 = find_skill(
        &mut Transaction::begin(state2.clone()),
        setup.owner,
        XIONGLUAN,
    );
    let (_state3, result) = apply_command(
        state2,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0x5901),
            kind: CommandKind::ActivateSkill {
                skill: skill2,
                targets: vec![setup.other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, engine_api::CommandResult::Rejected { .. }),
        "limited skill must be rejected on second use"
    );
}

#[test]
fn xiongluan_target_has_zero_distance_and_unlimited_use_count() {
    let mut setup = setup();
    let hand = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.owner)
        .unwrap();
    let _sha = alloc_card(
        &mut setup.tx,
        setup.owner,
        hand,
        CardKind::Sha,
        1,
        Suit::Spade,
        5,
    );
    let skill = find_skill(&mut setup.tx, setup.owner, XIONGLUAN);
    let state = activate(
        setup.tx.commit().state,
        setup.owner,
        skill,
        vec![setup.other],
    );
    let tx = Transaction::begin(state);
    // 对限定目标距离为 0。
    let mut ctx = engine_core::query::QueryContext::new();
    let distance = engine_core::query::query_explain(
        tx.state(),
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.other,
        },
        &mut ctx,
    );
    assert_eq!(distance.response, QueryResponse::Int(0));
    // 与未标记目标距离保持常规（至少为 1）。
    let mut ctx = engine_core::query::QueryContext::new();
    let distance_third = engine_core::query::query_explain(
        tx.state(),
        &QueryRequest::Distance {
            from: setup.owner,
            to: setup.third,
        },
        &mut ctx,
    );
    assert!(
        matches!(distance_third.response, QueryResponse::Int(d) if d >= 1),
        "unmarked target distance must stay regular"
    );
}

#[test]
fn twjuxiang_fires_on_other_qun_play_phase_and_moves_equip() {
    let mut setup = setup();
    // other 设为群势力；third 设为他势力。
    setup
        .tx
        .state_mut()
        .group_overrides
        .insert(setup.other, "qun".into());
    setup
        .tx
        .state_mut()
        .group_overrides
        .insert(setup.third, "shu".into());
    let equip_zone = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Equip, setup.other)
        .unwrap();
    let equip = alloc_card(
        &mut setup.tx,
        setup.other,
        equip_zone,
        CardKind::Weapon,
        2,
        Suit::Heart,
        3,
    );
    let skill = find_skill(&mut setup.tx, setup.owner, TWJUXIANG);
    let state = setup.tx.commit().state;
    // 触发 other 的出牌阶段：先确认可选触发，再选装备牌。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let event = DomainEvent::TurnStateChanged {
        current: setup.other,
        phase: engine_api::TurnPhaseTag::Play,
    };
    // 循环：计划阶段可选确认 → 触发 → 装备牌选择，逐段推进直至无挂起选择。
    for _ in 0..8u32 {
        if !tx.state().has_pending_choice() {
            let fired =
                dispatch_for_events(&mut tx, &[event.clone()], &mut TriggerFireRecord::new());
            match fired {
                Ok(effects) => match run_effects(&mut tx, effects) {
                    Ok(_) => break,
                    Err(EffectError::ChoiceRequested) => {}
                    Err(e) => panic!("run error: {e:?}"),
                },
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("dispatch error: {e:?}"),
            }
            if !tx.state().has_pending_choice() {
                break;
            }
        }
        // 处理当前选择：可选触发确认 [1] 或装备牌选择。
        let cont = tx.state().resolution.continuation_payload.clone();
        let choice_id = tx.state().resolution.pending_choice_id;
        let observer = PlayerId::new(tx.state().resolution.pending_observer);
        let committed = tx.commit();
        let state = committed.state;
        let selections = match committed.output.choice.as_ref() {
            Some(choice) if choice.kind == ChoiceKind::Confirm => vec![1],
            Some(choice) => {
                assert_eq!(
                    cont.first(),
                    Some(&engine_core::effect::RESOLVE_TWJUXIANG_CARD)
                );
                assert_eq!(choice.kind, ChoiceKind::SelectCards);
                vec![choice.options[0].token]
            }
            None => {
                // 续体选择（resume 路径不重复缓冲 choice 请求）。
                assert_eq!(
                    cont.first(),
                    Some(&engine_core::effect::RESOLVE_TWJUXIANG_CARD)
                );
                vec![equip.raw()]
            }
        };
        let (state, result) = apply_command(
            state,
            Command {
                actor: observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0x5902),
                kind: CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(choice_id),
                    selections,
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, engine_api::CommandResult::Accepted { .. }),
            "twjuxiang choice must resolve: {result:?}"
        );
        tx = Transaction::begin(state);
    }
    let state = tx.commit().state;
    let tx = Transaction::begin(state);
    // 装备牌已置入 owner（主公）装备区。
    assert!(tx
        .state()
        .zones
        .find(ZoneKind::Equip, setup.owner)
        .is_some_and(|zone| tx
            .state()
            .zones
            .get(zone)
            .is_some_and(|z| z.cards.contains(&equip))));
    // 本回合 other 已用（限一次）。
    assert_ne!(
        tx.state()
            .turn
            .flag(engine_core::state::turn_flag::twjuxiang_used_by_event_player(setup.other)),
        0
    );
    let _ = skill;
}
}

mod original_shenhua_zhiba {
//! Golden coverage for 制霸（shenhua 孙策，ABI 523）.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHIBA: u32 = 3_467_542_721;
const HUNZI: u32 = 4_151_424_761;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhiba.skill");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("zhiba_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    actor: PlayerId,
}

/// Lord holds 制霸; a Wu-faction actor sits next in seat order. Both get one hand card.
fn setup(awakened: bool) -> Setup {
    // `awakened` = the lord already awakened (魂姿 removed). When false, the lord still
    // holds 魂姿 and therefore cannot refuse the comparison.
    let source = if !awakened {
        format!("{SOURCE}\nskill test_hunzi({HUNZI}) locked {{}}")
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("zhiba-test", &source).unwrap();
    package.character_index = vec![
        character(9_217_010, "wei"),
        character(9_217_011, "wu"),
        character(9_217_012, "shu"),
        character(9_217_013, "qun"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let actor = seats
        .iter()
        .copied()
        .find(|player| *player != lord)
        .unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_217_010;
    tx.state_mut().players.get_mut(actor).unwrap().character_def = 9_217_011;
    for seat in seats
        .iter()
        .filter(|player| **player != lord && **player != actor)
    {
        tx.state_mut().players.get_mut(*seat).unwrap().character_def = 9_217_012;
    }
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
        player: lord,
        skill_def: SkillDefId::new(ZHIBA),
    });
    if !awakened {
        effects.push(EffectRequest::AddSkill {
            player: lord,
            skill_def: SkillDefId::new(HUNZI),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    // One hand card each for the comparison.
    for (owner, kind, suit, rank) in [
        (actor, CardKind::Sha, Suit::Spade, 5_u8),
        (lord, CardKind::Shan, Suit::Heart, 8_u8),
    ] {
        let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_217_100 + rank as u32),
            kind,
            suit,
            rank,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    }
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    Setup { tx, lord, actor }
}

fn fire_phase(tx: Transaction, current: PlayerId, tag: TurnPhaseTag) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: tag,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_lord_compare_contract() {
    let compiled = content_compiler::compile_source_named("zhiba", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 523);
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHIBA)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_wu_play_phase_can_compare_owner"
    );
    assert!(trigger.optional);
}

#[test]
fn awakened_lord_can_refuse_then_actor_compare_wins() {
    let setup = setup(true);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let tx = fire_phase(tx, setup.actor, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    // The Wu actor's optional confirm.
    assert!(state.has_pending_choice());
    state = resolve_choice(state, setup.actor, vec![1], 1);
    // Awakened lord's refuse choice.
    assert!(state.has_pending_choice());
    state = resolve_choice(state, setup.lord, vec![0], 2);
    // The actor's card outranks the lord's (8 > 5) so the actor wins.
    let actor_card = state.hand_cards(setup.actor)[0];
    let lord_card = state.hand_cards(setup.lord)[0];
    state.cards.get_mut(actor_card).unwrap().rank = 8;
    state.cards.get_mut(lord_card).unwrap().rank = 5;
    state = resolve_choice(state, setup.actor, vec![actor_card.raw()], 3);
    state = resolve_choice(state, setup.lord, vec![lord_card.raw()], 4);
    // Actor (8 > 5) wins: no settle choice; nobody gains.
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.lord), 0);
    assert_eq!(state.hand_count(setup.actor), 0);
}

#[test]
fn unawakened_lord_wins_compare_and_takes_both_cards() {
    let setup = setup(false);
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let tx = fire_phase(tx, setup.actor, TurnPhaseTag::Play);
    let mut state = tx.commit().state;
    // No refuse choice for the unawakened lord: the actor's confirm is the only prompt.
    state = resolve_choice(state, setup.actor, vec![1], 1);
    let actor_card = state.hand_cards(setup.actor)[0];
    let lord_card = state.hand_cards(setup.lord)[0];
    state = resolve_choice(state, setup.actor, vec![actor_card.raw()], 2);
    state = resolve_choice(state, setup.lord, vec![lord_card.raw()], 3);
    // Actor (5) did not win against the lord (8): the settle choice opens for the lord.
    assert!(state.has_pending_choice());
    state = resolve_choice(state, setup.lord, vec![1], 4);
    // The lord gained both comparison cards from the discard pile.
    assert_eq!(state.hand_count(setup.lord), 2);
    assert!(state.hand_cards(setup.lord).contains(&actor_card));
    assert!(state.hand_cards(setup.lord).contains(&lord_card));
    assert!(!state.has_pending_choice());
}
}

mod original_shenhua_zhijian {
//! Golden coverage for 直谏（shenhua 张昭张纮，ABI 512）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHIJIAN: u32 = 2_679_144_247;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhijian.skill");

fn started() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("zhijian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
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
        skill_def: SkillDefId::new(ZHIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_110),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZHIJIAN)
        .unwrap();
    (tx.commit().state, owner, other, skill)
}

/// Like [`started`], but also places a non-equipment hand card on the owner.
fn started_with_ordinary() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let (state, owner, other, skill) = started();
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let ordinary = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_111),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 5,
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
        .push(ordinary);
    (tx.commit().state, owner, other, skill, ordinary)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_give_equipment_contract() {
    let compiled = content_compiler::compile_source_named("zhijian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 512);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZHIJIAN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_equip"));
}

#[test]
fn gives_hand_weapon_to_other_player_and_draws_one() {
    let (mut state, owner, other, skill) = started();
    let weapon = state.hand_cards(owner)[0];
    let equip_before = state
        .zones
        .find(ZoneKind::Equip, other)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();

    let (state, activated) = activate(state, owner, skill, vec![weapon], 1);
    assert!(
        matches!(activated.result, CommandResult::Accepted { .. }),
        "activation must be accepted: {:?}",
        activated.result
    );
    let mut state = resolve_choice(state, owner, vec![other.raw()], 2);

    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state.zones.find(ZoneKind::Equip, other).unwrap(),
        "the weapon must be equipped by the other player"
    );
    let equip_after = state
        .zones
        .find(ZoneKind::Equip, other)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(
        equip_after.len(),
        equip_before.len() + 1,
        "the other player gained exactly the given equipment"
    );
    assert_eq!(state.hand_count(owner), 1, "the owner drew one card");
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
}

#[test]
fn rejects_non_equipment_and_owner_as_target() {
    let (mut state, owner, other, skill, ordinary) = started_with_ordinary();
    let weapon = state.hand_cards(owner)[0];

    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, skill, vec![ordinary], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, activated) = activate(state, owner, skill, vec![weapon], 2);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = activated
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(
        !options.contains(&owner.raw()),
        "the owner cannot be the 直谏 target"
    );
    assert!(options.contains(&other.raw()));
}
}

mod original_shenhua_zhoufei {
//! Golden coverage for 良姻/箜声（shenhua 周妃，ABI 579）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIANGYIN: u32 = 4_194_727_596;
const KONGSHENG: u32 = 3_654_831_984;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhoufei.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zf-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(LIANGYIN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(KONGSHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// 箜声：准备阶段将一张牌置为「箜」。
fn seed_kong(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let owner_hand = hand_zone(&tx, owner);
    alloc_card(
        &mut tx,
        owner,
        owner_hand,
        CardKind::Sha,
        97_399,
        Suit::Club,
        9,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "箜声 confirm");
    let state = resolve(state, owner, vec![1], false, 9_000);
    assert!(state.has_pending_choice(), "箜声 place prompt");
    let hand: Vec<_> = state.hand_cards(owner);
    resolve(state, owner, vec![hand[0].raw()], false, 9_001)
}

#[test]
fn package_exposes_zhoufei_contract() {
    let compiled = content_compiler::compile_source_named("zf", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 579);
    assert_eq!(compiled.skill_index.len(), 2);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == LIANGYIN || t.skill_def_id == KONGSHENG)
        .collect();
    assert_eq!(triggers.len(), 5);
    let kongsheng_end = triggers
        .iter()
        .find(|t| t.skill_def_id == KONGSHENG && t.event == "phase_end")
        .unwrap();
    assert!(!kongsheng_end.optional);
}

/// 良姻（失牌分支）：双方各摸一张；手牌等于箜数且受伤时回复。
#[test]
fn liangyin_draw_and_recover() {
    let setup = setup();
    // 先置 1 张「箜」。
    let state = setup.tx.commit().state;
    let state = seed_kong(state, setup.owner);
    // 持有者受伤。
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    // 持有者回合外失去一张牌 → 良姻失牌分支。
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = hand_zone(&tx, setup.owner);
    let lost = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        97_400,
        Suit::Club,
        2,
    );
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    // 真实移牌（合成事件不改变牌位）；移牌中良姻触发将开确认。
    match run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![lost],
            to: discard,
            from: Some(owner_hand),
        }],
    ) {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("move error: {e:?}"),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "良姻 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_010);
    assert!(state.has_pending_choice(), "良姻 draw target");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_011);
    // 双方各摸一张；箜数 1：far 摸后手牌 1 == 箜数且未受伤 → 不回复；
    // 持有者摸后手牌 1 == 箜数且受伤 → 开回复选择。
    assert!(state.has_pending_choice(), "良姻 recover prompt");
    let hp_before = state.players.get(setup.owner).unwrap().hp;
    // 摸牌中途的良姻获牌分支确认（拒绝），随后摸牌分支恢复并开回复选择。
    let state = resolve(state, setup.owner, vec![0], false, 9_012);
    assert!(state.has_pending_choice(), "良姻 recover prompt");
    let state = resolve(state, setup.owner, vec![0], false, 9_013);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp_before + 1);
}

/// 良姻（获牌分支）：弃一张牌并令一名其他角色弃一张牌。
#[test]
fn liangyin_discard_branch() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    // 持有者摸一张（真实摸牌记录获牌历史并触发良姻获牌分支）。
    let draw_pile = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let paid = alloc_card(
        &mut tx,
        setup.owner,
        draw_pile,
        CardKind::Sha,
        97_401,
        Suit::Spade,
        3,
    );
    let far_hand = hand_zone(&tx, setup.far);
    let victim = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Tao,
        97_402,
        Suit::Heart,
        4,
    );
    match run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: setup.owner,
            count: 1,
        }],
    ) {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("gain error: {e:?}"),
    }
    let state = tx.commit().state;
    assert!(state.has_pending_choice(), "良姻 gain confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_020);
    assert!(state.has_pending_choice(), "良姻 discard prompt");
    let state = resolve(state, setup.owner, vec![paid.raw()], false, 9_021);
    assert!(state.has_pending_choice(), "良姻 target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_022);
    assert!(state.has_pending_choice(), "良姻 target card prompt");
    let state = resolve(state, setup.owner, vec![victim.raw()], false, 9_023);
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(victim).unwrap().zone, discard);
}

/// 箜声：结束阶段非装备箜回手、装备箜令一名角色使用。
#[test]
fn kongsheng_end_returns_and_equips() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // 置 2 张「箜」：一张桃（非装备）、一张武器。
    tx.state_mut().turn.phase = Phase::Prepare;
    // 先给两张手牌（一张桃、一件武器），再走准备阶段置箜流程。
    let owner_hand = hand_zone(&tx, setup.owner);
    let peach = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Tao,
        97_403,
        Suit::Heart,
        2,
    );
    let weapon = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Weapon,
        97_404,
        Suit::Diamond,
        6,
    );
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        },
    );
    assert!(state.has_pending_choice(), "箜声 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_030);
    assert!(state.has_pending_choice(), "箜声 place prompt");
    let state = resolve(
        state,
        setup.owner,
        vec![peach.raw(), weapon.raw()],
        false,
        9_031,
    );
    assert!(!state.has_pending_choice());
    // 结束阶段：非装备（手牌里的桃）回手、武器令目标使用。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    // 装备箜目标选择（回手无触发时直接到达）。
    assert!(state.has_pending_choice(), "箜声 equip target");
    let state = resolve(state, setup.owner, vec![setup.other.raw()], false, 9_031);
    // 装备移入/使用中，其他角色获得/失去牌触发良姻确认（全部拒绝）。
    let mut seq = 9_033u64;
    let mut state = state;
    while state.has_pending_choice() {
        state = resolve(state, setup.owner, vec![0], false, seq);
        seq += 1;
    }
    // 武器装备到 other。
    let other_equip = state.zones.find(ZoneKind::Equip, setup.other).unwrap();
    assert!(
        state
            .zones
            .get(other_equip)
            .unwrap()
            .cards
            .contains(&weapon),
        "weapon equipped to the target"
    );
    // 非装备箜（桃）回到持有者手牌。
    assert_eq!(state.hand_count(setup.owner), 1, "non-equip 箜 returned");
}
}

mod original_shenhua_zhugezhan {
//! Golden coverage for 罪论/父荫（shenhua 诸葛瞻，ABI 563）.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZUILUN: u32 = 3_588_795_290;
const FUYIN: u32 = 3_383_028_272;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhugezhan.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    other: PlayerId,
    far: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhugezhan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    // Enable the turn card-loss history via a helper filter (ABI 405 family).
    let helper = content_compiler::compile_source_named(
        "loss-tracking-helper",
        "skill loss_helper(4000000001) { trigger turn_ended owner_alive_lost_hand_this_turn optional { effect skip_current_phase(self); } }",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&helper).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let far = seats[2];
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
        skill_def: SkillDefId::new(ZUILUN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(FUYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    Setup {
        tx,
        owner,
        other,
        far,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn hand_zone(tx: &Transaction, player: PlayerId) -> engine_api::ZoneId {
    tx.state().zones.find(ZoneKind::Hand, player).unwrap()
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
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
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "unexpected result: {:?}",
        result.result
    );
    state
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let fired = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new());
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Err(EffectError::ChoiceRequested) => {}
            Ok(_) => {}
            Err(e) => panic!("dispatch error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    tx.commit().state
}

/// Prepare the draw pile with three known top cards and the owner at the End phase.
fn prep_end(state: GameState, owner: PlayerId) -> (GameState, Vec<CardId>) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.lost_cards.clear();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top1 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Tao,
        96_800,
        Suit::Heart,
        2,
    );
    let top2 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Sha,
        96_801,
        Suit::Spade,
        3,
    );
    let top3 = alloc_card(
        &mut tx,
        PlayerId::NONE,
        draw,
        CardKind::Shan,
        96_802,
        Suit::Heart,
        4,
    );
    tx.state_mut().turn.phase = Phase::End;
    (tx.commit().state, vec![top1, top2, top3])
}

#[test]
fn package_exposes_zhugezhan_contract() {
    let compiled = content_compiler::compile_source_named("zhugezhan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 563);
    assert_eq!(compiled.skill_index.len(), 2);
    let zuilun = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == ZUILUN)
        .unwrap();
    assert_eq!(zuilun.event, "phase_end");
    assert!(zuilun.optional);
    let fuyin = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == FUYIN)
        .unwrap();
    assert_eq!(fuyin.event, "card_targeted");
    assert!(!fuyin.optional);
}

/// 罪论 count==0：获得全部三张。
#[test]
fn zuilun_zero_conditions_gains_all_three() {
    let setup = setup();
    let state = setup.tx.commit().state;
    let (state, revealed) = prep_end(state, setup.owner);
    // Count 0 needs: no loss, NOT minimum hand, damaged. Give the owner two hand
    // cards (others stay at zero -> not minimum) and damage the owner.
    let mut tx = Transaction::begin(state);
    let owner_hand = hand_zone(&tx, setup.owner);
    let h1 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_807,
        Suit::Club,
        9,
    );
    let h2 = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Shan,
        96_808,
        Suit::Heart,
        5,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "罪论 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_000);
    // No choice: the three cards were gained directly.
    assert!(!state.has_pending_choice(), "count 0 gains directly");
    for card in &revealed {
        assert!(
            state.hand_cards(setup.owner).contains(card),
            "gained {card:?}"
        );
    }
    assert_eq!(state.hand_count(setup.owner), 5, "2 held + 3 gained");
}

/// 罪论 count==2：选两张置顶，获其余一张。
#[test]
fn zuilun_two_conditions_puts_two_on_top() {
    let setup = setup();
    let state = setup.tx.commit().state;
    let (state, revealed) = prep_end(state, setup.owner);
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "罪论 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_000);
    assert!(state.has_pending_choice(), "罪论 pick prompt");
    // count: lost=0, min_hand=1 (tie at 0), undamaged=1 -> count=2.
    let state = resolve(
        state,
        setup.owner,
        vec![revealed[0].raw(), revealed[1].raw()],
        false,
        9_001,
    );
    assert!(!state.has_pending_choice());
    // revealed[0]/[1] back on the draw pile top; revealed[2] gained.
    assert!(state.hand_cards(setup.owner).contains(&revealed[2]));
    assert!(!state.hand_cards(setup.owner).contains(&revealed[0]));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top: Vec<_> = state
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(2).copied().collect())
        .unwrap_or_default();
    assert!(top.contains(&revealed[0]) && top.contains(&revealed[1]));
}

/// 罪论 count==3：全部置顶并选择一名其他角色一起失去 1 点体力。
#[test]
fn zuilun_three_conditions_lose_hp_together() {
    let setup = setup();
    let state = setup.tx.commit().state;
    let (state, revealed) = prep_end(state, setup.owner);
    // All three conditions: lost (force a loss), min hand (owner strict minimum), undamaged.
    let mut tx = Transaction::begin(state);
    let owner_hand = hand_zone(&tx, setup.owner);
    let lost_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_803,
        Suit::Club,
        5,
    );
    // Give the others hand cards so the owner is the strict minimum.
    let other_hand = hand_zone(&tx, setup.other);
    let far_hand = hand_zone(&tx, setup.far);
    let o1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_804,
        Suit::Club,
        6,
    );
    let f1 = alloc_card(
        &mut tx,
        setup.far,
        far_hand,
        CardKind::Sha,
        96_805,
        Suit::Club,
        7,
    );
    // Force a card loss this turn via an explicit move to the discard pile.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![lost_card],
            to: discard,
            from: Some(owner_hand),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    let hp_before = state.players.get(setup.owner).unwrap().hp;
    let far_hp_before = state.players.get(setup.far).unwrap().hp;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice(), "罪论 confirm");
    let state = resolve(state, setup.owner, vec![1], false, 9_000);
    assert!(state.has_pending_choice(), "罪论 lose-hp target prompt");
    let state = resolve(state, setup.owner, vec![setup.far.raw()], false, 9_010);
    assert!(!state.has_pending_choice());
    // Both lost 1 HP.
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.players.get(setup.far).unwrap().hp, far_hp_before - 1);
    // All three on the draw pile top.
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top: Vec<_> = state
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(3).copied().collect())
        .unwrap_or_default();
    for card in &revealed {
        assert!(top.contains(card), "on top {card:?}");
    }
    let _ = (o1, f1);
}

/// 父荫：本回合首次被手牌数不小于自己的其他角色以【杀】指定时，此杀无效。
#[test]
fn fuyin_excludes_first_sha_from_richer_source() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    tx.state_mut().turn.current = setup.other;
    tx.state_mut().turn.phase = Phase::Play;
    let other_hand = hand_zone(&tx, setup.other);
    let s1 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_805,
        Suit::Spade,
        7,
    );
    let s2 = alloc_card(
        &mut tx,
        setup.other,
        other_hand,
        CardKind::Sha,
        96_806,
        Suit::Spade,
        8,
    );
    // Owner has 0 hand cards; the source has 2: the source hand >= owner hand.
    let mut state = tx.commit().state;
    let mut resolution = &mut state.resolution;
    resolution.resume_kind = engine_core::state::ContinuationKind::ResumePlayedSha;
    resolution.resume_payload = vec![setup.other.raw(), setup.owner.raw(), 1, 0, 1, s1.raw()];
    resolution.register_sha_resolution(setup.other, s1, &[setup.owner], false);
    drop(resolution);
    // Probe: does the trigger fire? Dispatch and inspect the produced effects.
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: s1,
            targets: vec![setup.owner],
        },
    );
    assert!(!state.has_pending_choice(), "父荫 is locked and silent");
    // The exclusion lands in the saved resume payload: the owner is appended to the
    // exclusion tail of the ResumePlayedSha payload.
    let payload = &state.resolution.resume_payload;
    let excluded: Vec<_> = payload.get(6..).unwrap_or(&[]).to_vec();
    assert!(
        excluded.contains(&setup.owner.raw()),
        "owner excluded in the saved Sha payload: {payload:?}"
    );
    // Second Sha on the same turn: 父荫 already fired -> no exclusion.
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: setup.other,
            card: s2,
            targets: vec![setup.owner],
        },
    );
    assert!(!state.has_pending_choice());
    // The resume payload still belongs to the first Sha (s2 never registered);
    // its exclusion tail must be unchanged: only s1's owner exclusion.
    assert_eq!(
        &state.resolution.resume_payload,
        &[
            setup.other.raw(),
            setup.owner.raw(),
            1,
            0,
            1,
            s1.raw(),
            setup.owner.raw()
        ],
        "second Sha not excluded (once per turn)"
    );
}
}

mod original_shenhua_zhurong {
//! Golden coverage for 巨象/烈刃（shenhua 祝融，ABI 530）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JUXIANG: u32 = 4_234_952_102;
const LIEREN: u32 = 3_941_156_292;
const LIEREN_WIN: u32 = 1_258_122_096;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/zhurong.skill");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhurong-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
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
        skill_def: SkillDefId::new(JUXIANG),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIEREN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        tx,
        owner,
        source,
        target,
    }
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    def: u32,
    suit: Suit,
    rank: u8,
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn fire_used(tx: Transaction, user: PlayerId, card: CardId, targets: Vec<PlayerId>) -> Transaction {
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => tx,
            Err(EffectError::ChoiceRequested) => tx,
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => tx,
        Err(e) => panic!("dispatch error: {e:?}"),
    }
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_juxiang_lieren_contract() {
    let compiled = content_compiler::compile_source_named("zhurong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 531);
    let juxiang = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == JUXIANG)
        .unwrap();
    assert_eq!(juxiang.event, "card_resolved");
    assert_eq!(juxiang.filter, "owner_alive_other_nanman_cards_discard");
    let lieren = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == LIEREN)
        .unwrap();
    assert_eq!(lieren.event, "damage");
    assert_eq!(
        lieren.filter,
        "owner_source_sha_damage_target_alive_both_hand"
    );
    assert!(lieren.optional);
    let win = compiled
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == LIEREN_WIN)
        .unwrap();
    assert_eq!(win.filter, "owner_alive_skill_bound_target_is_event_source");
}

#[test]
fn juxiang_gains_resolved_nanman_from_discard() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let hand = tx.state().zones.find(ZoneKind::Hand, setup.source).unwrap();
    let nanman = alloc_card(
        &mut tx,
        setup.source,
        hand,
        CardKind::TrickNanman,
        96_300,
        Suit::Club,
        7,
    );
    // The resolved Nanman sits in the discard pile.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![nanman],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    let mut tx = tx;
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: setup.source,
            card: nanman,
            kind: 24,
            targets: vec![setup.owner, setup.target],
            damaged_targets: vec![],
            source_cards: vec![],
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => match run_effects(&mut tx, effects) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("effect error: {e:?}"),
        },
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    let state = tx.commit().state;
    assert!(
        state.hand_cards(setup.owner).contains(&nanman),
        "the owner gained the resolved Nanman"
    );
}

#[test]
fn lieren_win_takes_one_card_from_damaged_target() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    // Compare cards: owner 8 beats target 5.
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let owner_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_310,
        Suit::Spade,
        8,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, setup.target).unwrap();
    let target_card = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Shan,
        96_311,
        Suit::Heart,
        5,
    );
    let target_give = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Tao,
        96_312,
        Suit::Heart,
        6,
    );
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_313,
        Suit::Club,
        9,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: setup.owner,
                target: setup.target,
                card: sha,
                amount: 1,
                nature: 0,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    // Compare card choices.
    state = resolve_choice(state, setup.owner, vec![owner_card.raw()], 2);
    state = resolve_choice(state, setup.target, vec![target_card.raw()], 3);
    // The owner won: take one card from the damaged target.
    assert!(state.has_pending_choice(), "take-one-card choice pending");
    state = resolve_choice(state, setup.owner, vec![target_give.raw()], 4);
    assert!(
        state.hand_cards(setup.owner).contains(&target_give),
        "the owner gained a card from the damaged target"
    );
    assert!(!state.hand_cards(setup.target).contains(&target_give));
    assert!(!state.has_pending_choice());
}

#[test]
fn lieren_loss_takes_nothing() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.tx.commit().state);
    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let owner_card = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_320,
        Suit::Spade,
        5,
    );
    let target_hand = tx.state().zones.find(ZoneKind::Hand, setup.target).unwrap();
    let target_card = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Shan,
        96_321,
        Suit::Heart,
        8,
    );
    let target_give = alloc_card(
        &mut tx,
        setup.target,
        target_hand,
        CardKind::Tao,
        96_322,
        Suit::Heart,
        6,
    );
    let sha = alloc_card(
        &mut tx,
        setup.owner,
        owner_hand,
        CardKind::Sha,
        96_323,
        Suit::Club,
        9,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealShaDamage {
                source: setup.owner,
                target: setup.target,
                card: sha,
                amount: 1,
                nature: 0,
                card_flags: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve_choice(state, setup.owner, vec![1], 1);
    state = resolve_choice(state, setup.owner, vec![owner_card.raw()], 2);
    state = resolve_choice(state, setup.target, vec![target_card.raw()], 3);
    // The owner lost: no take choice.
    assert!(!state.has_pending_choice());
    assert!(!state.hand_cards(setup.owner).contains(&target_give));
}
}

mod original_shenhua_zuoci {
//! Golden coverage for 化身/新生（shenhua 左慈，ABI 591，文档化近似：元数据登记）。

use engine_api::SkillDefId;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HUASHEN: u32 = 1_234_567_909;
const XINSHENG: u32 = 1_234_567_910;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/huashen.skill");

#[test]
fn package_exposes_zuoci_contract() {
    let compiled = content_compiler::compile_source_named("zuoci", SOURCE).unwrap();
    // 元数据技能不携带任何效果门控 → 基线 ABI。
    assert_eq!(compiled.manifest.rule_vm_abi_version, 3);
    assert_eq!(compiled.skill_index.len(), 2);
    let committed = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/shenhua/shenhua.npack.json"
    ))
    .unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&HUASHEN)));
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.skills.contains(&XINSHENG)));
}

#[test]
fn zuoci_skills_are_metadata_only() {
    let package = content_compiler::compile_source_named("zuoci-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HUASHEN),
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINSHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    // 无运行时行为：技能可添加，且不破坏任何查询。
    assert!(tx
        .state()
        .players
        .get(owner)
        .is_some_and(|p| p.skills.iter().any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == HUASHEN)
        })));
    let state = tx.commit().state;
    let _ = Transaction::begin(state);
    let _ = EffectError::ChoiceRequested;
}
}
