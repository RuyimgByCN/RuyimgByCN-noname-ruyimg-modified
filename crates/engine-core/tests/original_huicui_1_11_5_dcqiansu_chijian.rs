//! Golden for HUICUI 1.11.5 新杀费祎「谦素」与 新杀夏侯恩「持剑」近似语义。
//!
//! 谦素：其他角色结束阶段，若其手牌多于你，可摸三张并交给其一张。
//! 持剑：出杀次数 +1，忽略防具。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::sha_use_limit_for;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCQIANSU: u32 = 4_169_409_504;
const CHIJIAN: u32 = 3_230_974_968;
const DC_FEIYI: u32 = 3_373_769_823;
const DC_XIAHOUEN: u32 = 3_482_689_208;
const DCQIANSU_SRC: &str =
    include_str!("../../../content/original/ported/huicui/dcqiansu.skill");
const CHIJIAN_SRC: &str = include_str!("../../../content/original/ported/huicui/chijian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/huicui/huicui.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
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
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn setup_qiansu() -> (Transaction, PlayerId, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("dcqiansu-test", DCQIANSU_SRC).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_801, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
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
        skill_def: SkillDefId::new(DCQIANSU),
    });
    run_effects(&mut tx, effects).unwrap();
    // other has more hand cards than owner
    inject_hand(tx.state_mut(), owner, CardKind::Sha, 201_001);
    inject_hand(tx.state_mut(), other, CardKind::Shan, 201_002);
    inject_hand(tx.state_mut(), other, CardKind::Other, 201_003);
    inject_hand(tx.state_mut(), other, CardKind::Other, 201_004);
    let gift = *tx
        .state()
        .hand_cards(owner)
        .first()
        .expect("owner keeps one hand card for give");
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, other, gift)
}

#[test]
fn package_publishes_1_11_5_feiyi_and_xiahouen_closures() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 487);
    assert_eq!(package.skill_index.len(), 254);
    assert_eq!(package.character_index.len(), 119);
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == DCQIANSU && s.name == "dcqiansu"));
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == CHIJIAN && s.name == "chijian"));
    let feiyi = package
        .character_index
        .iter()
        .find(|c| c.id == DC_FEIYI)
        .unwrap();
    assert_eq!(feiyi.name_key, "dc_feiyi");
    assert_eq!(feiyi.skills, [DCQIANSU, 3_735_974_633, 2_598_339_084]);
    let xiahouen = package
        .character_index
        .iter()
        .find(|c| c.id == DC_XIAHOUEN)
        .unwrap();
    assert_eq!(xiahouen.name_key, "dc_xiahouen");
    assert_eq!(xiahouen.skills, [CHIJIAN, 3_773_213_822]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn qiansu_draws_three_then_gives_one_when_other_has_more_hand() {
    let (mut tx, owner, other, gift) = setup_qiansu();
    assert!(tx.state().hand_count(other) > tx.state().hand_count(owner));
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(other)],
            &mut TriggerFireRecord::new()
        ),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    assert_eq!(
        offered.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );

    // confirm optional trigger
    let (state, accepted) = resolve(offered.state, owner, 1, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveOneHandOrEquip
    );
    // after draw 3, owner has owner_hand+3 before giving
    assert_eq!(state.hand_count(owner), owner_hand + 3);

    let (state, given) = resolve(state, owner, gift.raw(), 2);
    assert!(matches!(given.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(other), other_hand + 1);
    assert_eq!(
        state.cards.get(gift).unwrap().zone,
        state.zones.find(ZoneKind::Hand, other).unwrap()
    );
}

#[test]
fn qiansu_skips_when_owner_hand_not_less() {
    let compiled = content_compiler::compile_source_named("dcqiansu-skip", DCQIANSU_SRC).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_802, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
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
        skill_def: SkillDefId::new(DCQIANSU),
    });
    run_effects(&mut tx, effects).unwrap();
    // equal hands → if condition fails after optional? filter is only not_target;
    // body if may no-op after confirm. Give other fewer hands.
    inject_hand(tx.state_mut(), owner, CardKind::Sha, 202_001);
    inject_hand(tx.state_mut(), owner, CardKind::Sha, 202_002);
    inject_hand(tx.state_mut(), other, CardKind::Shan, 202_003);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::End;
    let owner_hand = tx.state().hand_count(owner);
    // may still offer optional; if confirm, if-body no-ops
    let dispatched = dispatch_for_events(
        &mut tx,
        &[phase_end(other)],
        &mut TriggerFireRecord::new(),
    );
    match dispatched {
        Ok(_effects) => assert_eq!(tx.state().hand_count(owner), owner_hand),
        Err(EffectError::ChoiceRequested) => {
            let offered = tx.commit();
            let (state, accepted) = resolve(offered.state, owner, 1, 11);
            assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
            // condition false → no draw
            assert_eq!(state.hand_count(owner), owner_hand);
        }
        Err(e) => panic!("unexpected: {e:?}"),
    }
}

#[test]
fn chijian_adds_sha_use_limit_and_ignore_armor() {
    let compiled = content_compiler::compile_source_named("chijian-test", CHIJIAN_SRC).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_803, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(CHIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // base identity sha use limit is typically 1; +1 → 2
    assert!(
        sha_use_limit_for(&state, owner) >= 2,
        "持剑应提升出杀次数上限, got {}",
        sha_use_limit_for(&state, owner)
    );
    // ignore_armor is a package modifier entry for 持剑
    let package = package();
    assert!(
        package
            .modifier_index
            .iter()
            .any(|m| m.skill_def_id == CHIJIAN),
        "持剑应发布 ignore_armor / sha_use_limit 修正器"
    );
}
