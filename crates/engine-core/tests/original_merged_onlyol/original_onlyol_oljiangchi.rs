//! Golden for ONLYOL 曹彰「将驰」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OL_CAOZHANG: u32 = 2_831_667_463;
const OLJIANGCHI: u32 = 3_424_528_555;
const OLJIANGCHI_LESS: u32 = 2_381_340_383;
const OLJIANGCHI_MORE: u32 = 4_245_625_871;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIANGCHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

fn inject_equip(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Armor,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn end_phase(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, owner, CommandKind::EndPhase, key)
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_choice(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, offered) = end_phase(setup.state, setup.owner, key);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrRecastAddSkill
    );
    (state, offered)
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
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
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_publishes_complete_ol_caozhang_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // N01「胆守」新增 host effect tag 771，包 ABI 提升到 757。
    assert_eq!(package.manifest.rule_vm_abi_version, 757);
    // N01 新增 ol_jie_danshou 后 193 个顶层技能。
    assert_eq!(package.skill_index.len(), 193);
    // N01 新增 ol_jie_zhuran 后 86 个角色。
    assert_eq!(package.character_index.len(), 86);
    assert_eq!(package.trigger_index.len(), 32); // 批162 olrenxin +1；N01 胆守 +1
    assert_eq!(package.modules.len(), 36); // 批162 olrenxin +1；N01 胆守 trigger+active +2
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_CAOZHANG)
        .unwrap();
    assert_eq!(character.name_key, "ol_caozhang");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLJIANGCHI]);
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(|modifier| {
                modifier.skill_def_id == OLJIANGCHI_LESS || modifier.skill_def_id == OLJIANGCHI_MORE
            })
            .count(),
        4
    );
}

#[test]
fn choice_can_cancel_or_draw_and_the_draw_helper_expires_at_turn_end() {
    let cancelled = setup(2_026_072_601);
    let owner = cancelled.owner;
    let (state, offered) = open_choice(cancelled, 1);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (0, 1, true)
    );
    assert_eq!(
        choice
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1]
    );
    let (state, result) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!has_skill(&state, owner, OLJIANGCHI_LESS));

    let drawn = setup(2_026_072_602);
    let owner = drawn.owner;
    let hand = drawn.state.hand_count(owner);
    let (state, _) = open_choice(drawn, 3);
    let (state, result) = resolve(state, owner, vec![1], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert!(has_skill(&state, owner, OLJIANGCHI_LESS));
    assert_eq!(engine_core::query::sha_use_limit_for(&state, owner), 0);
    assert!(engine_core::query::sha_ignored_for_hand_limit(
        &state, owner
    ));
    let (state, _) = end_phase(state, owner, 5);
    let (state, _) = end_phase(state, owner, 6);
    let (state, _) = end_phase(state, owner, 7);
    assert!(!has_skill(&state, owner, OLJIANGCHI_LESS));
}

#[test]
fn recast_branch_recasts_one_card_and_grants_sha_range_and_limit() {
    let mut setup = setup(2_026_072_603);
    let owner = setup.owner;
    let cost = inject_equip(&mut setup.state, owner, 99_603);
    let (state, offered) = open_choice(setup, 10);
    assert_eq!(
        offered
            .choice
            .unwrap()
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, select) = resolve(state, owner, vec![2], false, 11);
    assert_eq!(select.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, result) = resolve(state, owner, vec![cost.raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
    assert_eq!(state.hand_count(owner), 1);
    assert!(has_skill(&state, owner, OLJIANGCHI_MORE));
    assert_eq!(engine_core::query::sha_use_limit_for(&state, owner), 2);
    assert!(engine_core::query::sha_distance_unlimited_for(
        &state, owner
    ));
}
