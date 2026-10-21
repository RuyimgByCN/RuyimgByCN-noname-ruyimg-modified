//! Golden coverage for 程普「醇醪」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardZoneTag, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::invariants::{check_invariants, InvariantError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::{canonical_state_hash, project_observer};
use rule_api::EffectRequest;

const CHENGPU: u32 = 2_944_307_433;
const LIHUO: u32 = 3_289_572_540;
const CHUNLAO: u32 = 3_165_720_039;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/chunlao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    dying: PlayerId,
    sha: [CardId; 2],
    other: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let dying = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &seats {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    // 先设结束阶段再挂技能，避免技能自身的 phase_end 可选触发在 setup 中开确认。
    effects.extend([
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHUNLAO),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = [
        inject(tx.state_mut(), owner, 99_001, CardKind::Sha, Suit::Spade),
        inject(
            tx.state_mut(),
            owner,
            99_002,
            CardKind::ShaFire,
            Suit::Heart,
        ),
    ];
    let other = inject(
        tx.state_mut(),
        owner,
        99_003,
        CardKind::TrickWuzhong,
        Suit::Club,
    );
    Setup {
        state: tx.commit().state,
        owner,
        dying,
        sha,
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
    let revision = state.revision;
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

fn store(setup: &Setup, key: u64) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    // 结束阶段可选触发：先确认，再选牌存储。

    let (state, confirmed) = resolve(tx.state().clone(), setup.owner, vec![1], key);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseStoreHandShaInSkillPile
    );
    let offered: Vec<_> = confirmed
        .choice
        .as_ref()
        .map(|choice| {
            choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(offered, setup.sha.map(CardId::raw));
    let (state, stored) = resolve(
        state,
        setup.owner,
        setup.sha.into_iter().map(CardId::raw).collect(),
        key + 1,
    );
    assert!(matches!(stored.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

fn enter_dying(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().players.get(target).unwrap().hp, 0);
    assert!(tx.state().players.get(target).unwrap().dying);
    tx.state().clone()
}

#[test]
fn package_publishes_complete_chengpu_and_exact_abi53_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHENGPU)
        .unwrap();
    assert_eq!(character.skills, [LIHUO, CHUNLAO]);
    let compiled = content_compiler::compile_source_named("chunlao", SOURCE).unwrap();
    assert_eq!(compiled.modules.len(), 2);
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
fn end_phase_stores_only_hand_sha_in_one_public_canonical_skill_pile() {
    let setup = setup(2_026_072_002);
    let state = store(&setup, 1);

    assert_eq!(state.skill_pile_cards(setup.owner, CHUNLAO), setup.sha);
    assert_eq!(state.hand_cards(setup.owner), [setup.other]);
    assert_eq!(state.skill_piles.len(), 1);
    let pile = state.skill_piles[0];
    let zone = state.zones.get(pile.zone).unwrap();
    assert_eq!((pile.owner, pile.skill_def), (setup.owner, CHUNLAO));
    assert_eq!((zone.kind, zone.owner), (ZoneKind::Expansion, setup.owner));
    assert!(setup
        .sha
        .iter()
        .all(|card| state.cards.get(*card).unwrap().face_up));
    check_invariants(&state).unwrap();
    assert_eq!(
        canonical_state_hash(&state),
        canonical_state_hash(&state.clone())
    );

    let mut wrong_zone = state.clone();
    wrong_zone.skill_piles[0].zone = wrong_zone.zones.find(ZoneKind::Hand, setup.dying).unwrap();
    assert!(matches!(
        check_invariants(&wrong_zone),
        Err(InvariantError::InvalidSkillPile { .. })
    ));
    let mut duplicate = state.clone();
    duplicate.skill_piles.push(duplicate.skill_piles[0]);
    assert_eq!(
        check_invariants(&duplicate),
        Err(InvariantError::SkillPileOrder)
    );

    let view = project_observer(&state, ObserverKind::Player(setup.dying));
    for card in setup.sha {
        let public = view
            .known_cards
            .iter()
            .find(|known| known.id == card)
            .unwrap();
        assert_eq!(public.zone, CardZoneTag::Expansion);
        assert!(public.def.is_some());
    }

    let mut tx = Transaction::begin(state);
    // 同一回合内再次 dispatch phase_end 会再次触发可选存牌（游戏中 phase_end 每回合仅一次）；
    // 确认并存入剩余手牌后应无悬挂 choice。
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let optional = tx.commit().state;
    assert_eq!(
        optional.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    // 确认后效果发现 skill pile 已非空（首次已存）而跳过，无后续选牌。
    let (state, confirmed) = resolve(optional, setup.owner, vec![1], 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn each_stored_sha_can_save_a_dying_player_through_the_virtual_jiu_pipeline() {
    let setup = setup(2_026_072_003);
    let mut state = store(&setup, 10);

    for (index, spent) in setup.sha.into_iter().enumerate() {
        state = enter_dying(state, setup.owner, setup.dying);
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::OptionalTrigger
        );

        let (next, accepted) = resolve(state, setup.owner, vec![1], 20 + index as u64 * 2);
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        assert_eq!(
            next.resolution.continuation_kind,
            ContinuationKind::ChooseSpendSkillPileForDyingJiu
        );
        let offered: Vec<_> = accepted
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect();
        assert!(offered.contains(&spent.raw()));

        let (next, saved) = resolve(next, setup.owner, vec![spent.raw()], 21 + index as u64 * 2);
        assert!(matches!(saved.result, CommandResult::Accepted { .. }));
        assert!(!next.has_pending_choice());
        let rescued = next.players.get(setup.dying).unwrap();
        assert_eq!(rescued.hp, 1);
        assert!(!rescued.dying);
        let discard = next
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(next.cards.get(spent).unwrap().zone, discard);
        assert!(saved.events.iter().any(|event| matches!(
            event,
            DomainEvent::CardUsed {
                player,
                targets,
                ..
            } if *player == setup.dying && targets == &[setup.dying]
        )));
        assert!(saved.events.iter().any(|event| matches!(
            event,
            DomainEvent::HpRecovered { player, amount, .. }
                if *player == setup.dying && *amount == 1
        )));
        check_invariants(&next).unwrap();
        state = next;
    }

    assert!(state.skill_pile_cards(setup.owner, CHUNLAO).is_empty());
}
