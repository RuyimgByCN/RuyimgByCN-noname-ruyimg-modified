//! Golden for ONLYOL 曹休「倾袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OL_CAOXIU: u32 = 3_425_947_453;
const OLQIANJU: u32 = 2_755_577_993;
const OLQINGXI: u32 = 3_468_570_040;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/onlyOL/onlyOL.npack.json");

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
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
        skill_def: SkillDefId::new(OLQINGXI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = OL_CAOXIU;
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
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
                cancelled: false,
            },
        },
    )
}

fn deal_sha(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
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
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn accept_olqingxi(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, result) = deal_sha(state, owner, target, rule_api::damage_nature::NORMAL);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], key);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    (state, accepted)
}

#[test]
fn package_publishes_complete_ol_caoxiu_at_abi293() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // 平寇 effect tag 656 冲突修复后，包最低 ABI 为 758。
    assert_eq!(package.manifest.rule_vm_abi_version, 758);
    // N01 新增 ol_jie_danshou 后 193 个顶层技能。
    assert_eq!(package.skill_index.len(), 193);
    // N01 新增 ol_jie_zhuran 后 86 个角色。
    assert_eq!(package.character_index.len(), 86);
    assert_eq!(package.trigger_index.len(), 32); // 批162 olrenxin +1；N01 胆守 +1
    assert_eq!(package.modules.len(), 36); // 批162 olrenxin +1；N01 胆守 trigger+active +2
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_CAOXIU)
        .unwrap();
    assert_eq!(character.name_key, "ol_caoxiu");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLQIANJU, OLQINGXI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLQINGXI)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source_sha_direct_damage_before");
    assert!(trigger.optional);
}

#[test]
fn no_weapon_target_chooses_discard_or_increased_damage() {
    let mut discard = setup(2_026_072_501);
    let card = inject(
        &mut discard.state,
        discard.target,
        ZoneKind::Hand,
        99_501,
        CardKind::Other,
        0,
    );
    let (state, accepted) = accept_olqingxi(discard.state, discard.owner, discard.target, 1);
    let choice = accepted.choice.as_ref().unwrap();
    assert_eq!(choice.observer, discard.target);
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, branch) = resolve(state, discard.target, vec![1], 2);
    assert_eq!(
        branch.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, result) = resolve(state, discard.target, vec![card.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(discard.target).unwrap().hp, 9);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let mut increase = setup(2_026_072_502);
    inject(
        &mut increase.state,
        increase.target,
        ZoneKind::Hand,
        99_502,
        CardKind::Other,
        0,
    );
    let (state, _) = accept_olqingxi(increase.state, increase.owner, increase.target, 4);
    let (state, result) = resolve(state, increase.target, vec![2], 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(increase.target).unwrap().hp, 8);
    assert_eq!(state.hand_count(increase.target), 1);
}

#[test]
fn weapon_forces_discard_damage_and_weapon_loss_even_when_cards_are_short() {
    let mut exact = setup(2_026_072_503);
    let weapon = inject(
        &mut exact.state,
        exact.owner,
        ZoneKind::Equip,
        99_511,
        CardKind::Weapon,
        2,
    );
    let first = inject(
        &mut exact.state,
        exact.target,
        ZoneKind::Hand,
        99_512,
        CardKind::Other,
        0,
    );
    let second = inject(
        &mut exact.state,
        exact.target,
        ZoneKind::Equip,
        99_513,
        CardKind::Armor,
        0,
    );
    let (state, accepted) = accept_olqingxi(exact.state, exact.owner, exact.target, 10);
    let choice = accepted.choice.as_ref().unwrap();
    assert_eq!(choice.observer, exact.target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    let (state, result) = resolve(state, exact.target, vec![first.raw(), second.raw()], 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(exact.target).unwrap().hp, 8);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [first, second, weapon] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }

    let mut short = setup(2_026_072_504);
    let weapon = inject(
        &mut short.state,
        short.owner,
        ZoneKind::Equip,
        99_521,
        CardKind::Weapon,
        3,
    );
    let only = inject(
        &mut short.state,
        short.target,
        ZoneKind::Hand,
        99_522,
        CardKind::Other,
        0,
    );
    let (state, accepted) = accept_olqingxi(short.state, short.owner, short.target, 12);
    assert_eq!(
        (
            accepted.choice.as_ref().unwrap().min_count,
            accepted.choice.as_ref().unwrap().max_count,
        ),
        (1, 1)
    );
    let (state, result) = resolve(state, short.target, vec![only.raw()], 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(short.target).unwrap().hp, 8);
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn direct_filter_does_not_retrigger_for_chain_propagation() {
    let mut setup = setup(2_026_072_505);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_531,
        CardKind::Weapon,
        1,
    );
    let card = inject(
        &mut setup.state,
        setup.target,
        ZoneKind::Hand,
        99_532,
        CardKind::Other,
        0,
    );
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
    let (state, result) = deal_sha(
        tx.commit().state,
        setup.owner,
        setup.target,
        rule_api::damage_nature::FIRE,
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(state, setup.owner, vec![1], 100);
    let (state, result) = resolve(state, setup.target, vec![card.raw()], 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(setup.target).unwrap().hp, 8);
    assert_eq!(state.players.get(setup.other).unwrap().hp, 8);
}
