//! Golden coverage for 曹休「千驹」「倾袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CAOXIU: u32 = 3_428_883_420;
const QIANJU: u32 = 2_828_017_083;
const QINGXI: u32 = 3_641_969_972;
const QIANJU_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/qianju.skill");
const QINGXI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/qingxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
        skill_def: SkillDefId::new(QINGXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = CAOXIU;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    weapon_range: u8,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
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

fn deal_card_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    kind: CardKind,
    nature: u8,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature,
            card: CardId::NONE,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn open_qingxi(state: GameState, owner: PlayerId, target: PlayerId) -> GameState {
    let (state, result) = deal_card_damage(
        state,
        owner,
        target,
        CardKind::Sha,
        rule_api::damage_nature::NORMAL,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_complete_caoxiu_and_exact_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOXIU)
        .unwrap();
    assert_eq!(character.name_key, "caoxiu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [QIANJU, QINGXI]);
    for (name, source, abi) in [("qianju", QIANJU_SOURCE, 3), ("qingxi", QINGXI_SOURCE, 59)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, abi);
        for expected in compiled.modules {
            let actual = package
                .modules
                .iter()
                .find(|module| module.name == expected.name)
                .unwrap();
            assert_eq!(actual.logic_hash, expected.logic_hash);
        }
    }
}

#[test]
fn qingxi_requires_direct_sha_damage_and_physical_weapon() {
    let no_weapon = setup(2_026_073_901);
    let (state, result) = deal_card_damage(
        no_weapon.state,
        no_weapon.owner,
        no_weapon.target,
        CardKind::Sha,
        rule_api::damage_nature::NORMAL,
    );
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(no_weapon.target).unwrap().hp, 9);

    let mut setup = setup(2_026_073_902);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_901,
        CardKind::Weapon,
        2,
    );
    let (state, result) = deal_card_damage(
        setup.state,
        setup.owner,
        setup.target,
        CardKind::TrickJuedou,
        rule_api::damage_nature::NORMAL,
    );
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
}

#[test]
fn qingxi_insufficient_hand_and_cancel_increase_damage_without_discarding_weapon() {
    let mut insufficient = setup(2_026_073_903);
    let weapon = inject(
        &mut insufficient.state,
        insufficient.owner,
        ZoneKind::Equip,
        99_911,
        CardKind::Weapon,
        3,
    );
    for def in [99_912, 99_913] {
        inject(
            &mut insufficient.state,
            insufficient.target,
            ZoneKind::Hand,
            def,
            CardKind::Other,
            0,
        );
    }
    let state = open_qingxi(insufficient.state, insufficient.owner, insufficient.target);
    let (state, accepted) = resolve(state, insufficient.owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(insufficient.target).unwrap().hp, 8);
    assert_eq!(state.cards.get(weapon).unwrap().owner, insufficient.owner);

    let mut cancelled = setup(2_026_073_904);
    let weapon = inject(
        &mut cancelled.state,
        cancelled.owner,
        ZoneKind::Equip,
        99_921,
        CardKind::Weapon,
        2,
    );
    for def in [99_922, 99_923] {
        inject(
            &mut cancelled.state,
            cancelled.target,
            ZoneKind::Hand,
            def,
            CardKind::Other,
            0,
        );
    }
    let state = open_qingxi(cancelled.state, cancelled.owner, cancelled.target);
    let (state, choice) = resolve(state, cancelled.owner, vec![1], false, 2);
    assert_eq!(
        choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(choice.choice.as_ref().unwrap().observer, cancelled.target);
    assert_eq!(choice.choice.as_ref().unwrap().min_count, 2);
    assert_eq!(choice.choice.as_ref().unwrap().max_count, 2);
    let (state, result) = resolve(state, cancelled.target, Vec::new(), true, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(cancelled.target).unwrap().hp, 8);
    assert_eq!(state.hand_count(cancelled.target), 2);
    assert_eq!(state.cards.get(weapon).unwrap().owner, cancelled.owner);
}

#[test]
fn qingxi_paid_discard_is_ordered_and_rejects_forgery_atomically() {
    let mut setup = setup(2_026_073_905);
    let weapon = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_931,
        CardKind::Weapon,
        2,
    );
    let cards: Vec<_> = [99_932, 99_933, 99_934]
        .into_iter()
        .map(|def| {
            inject(
                &mut setup.state,
                setup.target,
                ZoneKind::Hand,
                def,
                CardKind::Other,
                0,
            )
        })
        .collect();
    let state = open_qingxi(setup.state, setup.owner, setup.target);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 10);
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(
        state,
        setup.target,
        vec![cards[0].raw(), cards[0].raw()],
        false,
        11,
    );
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, paid) = resolve(
        state,
        setup.target,
        vec![cards[0].raw(), cards[1].raw()],
        false,
        12,
    );
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
    assert_eq!(state.hand_count(setup.target), 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [cards[0], cards[1], weapon] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert_eq!(state.cards.get(cards[2]).unwrap().owner, setup.target);
}

#[test]
fn qingxi_increases_chain_base_without_retriggering_on_propagation() {
    let mut setup = setup(2_026_073_906);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_941,
        CardKind::Weapon,
        2,
    );
    for def in [99_942, 99_943] {
        inject(
            &mut setup.state,
            setup.target,
            ZoneKind::Hand,
            def,
            CardKind::Other,
            0,
        );
    }
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: setup.target,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: setup.other,
                mark: MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let state = tx.commit().state;
    let (state, result) = deal_card_damage(
        state,
        setup.owner,
        setup.target,
        CardKind::Sha,
        rule_api::damage_nature::FIRE,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, setup.owner, vec![1], false, 20);
    let (state, result) = resolve(state, setup.target, Vec::new(), true, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 8);
    assert_eq!(state.players.get(setup.other).unwrap().hp, 8);
}
