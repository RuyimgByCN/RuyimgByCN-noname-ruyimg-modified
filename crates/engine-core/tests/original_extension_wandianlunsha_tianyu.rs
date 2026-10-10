//! Golden for archived extension 玩点论杀 田豫【车阵/诱战】与【诱敌深入】（ABI 613）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query_explain, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CHEZHEN: u32 = 2_288_694_956;
const YOUZHAN: u32 = 2_340_355_119;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/tianyu.skill");

fn setup() -> (GameState, [PlayerId; 4]) {
    let package = content_compiler::compile_source_named("wandianlunsha-tianyu", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 613);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 4] = tx.state().players.alive_by_seat().try_into().unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(CHEZHEN),
        },
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(YOUZHAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[2];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_130 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
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
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    (state, result)
}

fn choose(
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

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    let mut context = QueryContext::new();
    match query_explain(state, &QueryRequest::Distance { from, to }, &mut context).response {
        QueryResponse::Int(value) => value,
        response => panic!("unexpected distance response: {response:?}"),
    }
}

fn begin_youzhan(mut state: GameState, players: [PlayerId; 4], key: u64) -> (GameState, CardId) {
    let owner = players[0];
    let protected = players[1];
    let source = players[2];
    let cost = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        key as u32,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        key as u32 + 1,
    );
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: incoming,
            targets: vec![protected],
            card_targets: Vec::new(),
        },
        key,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, key + 1);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, _) = choose(state, owner, vec![cost.raw()], false, key + 2);
    (state, cost)
}

#[test]
fn package_and_chezhen_keep_the_equipment_test_boolean() {
    let package = content_compiler::compile_source_named("wandianlunsha-tianyu", SOURCE).unwrap();
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == YOUZHAN
            && trigger.filter == "owner_alive_sha_target_distance_le_one_has_equip_card"));
    let (mut state, players) = setup();
    let owner = players[0];
    let far = players[2];
    assert_eq!(distance(&state, owner, far), 2);
    assert_eq!(distance(&state, far, owner), 3);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor, 2);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Treasure, 3);
    assert_eq!(distance(&state, owner, far), 1);
    assert_eq!(distance(&state, far, owner), 2);
}

#[test]
fn youzhan_cancels_the_incoming_sha_and_refusal_damages_its_source() {
    let (state, players) = setup();
    let owner = players[0];
    let protected = players[1];
    let source = players[2];
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, cost) = begin_youzhan(state, players, 10);
    assert!(!state.hand_cards(owner).contains(&cost));
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn youdishenru_is_wuxieable_and_then_the_original_sha_continues() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let wuxie_owner = players[3];
    let wuxie = inject(
        &mut state,
        wuxie_owner,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        20,
    );
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, _) = begin_youzhan(state, players, 21);
    assert_eq!(state.resolution.pending_observer, wuxie_owner.raw());
    let (state, _) = choose(state, wuxie_owner, vec![wuxie.raw()], false, 24);
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp - 1);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);
}

#[test]
fn a_dodged_forced_sha_repeats_until_the_source_refuses() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let forced = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 30);
    let forced_again = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 31);
    let shan = inject(&mut state, protected, ZoneKind::Hand, CardKind::Shan, 32);
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, _) = begin_youzhan(state, players, 32);
    assert_eq!(state.resolution.pending_observer, source.raw());
    let (state, result) = choose(state, source, vec![forced.raw()], false, 35);
    assert_eq!(result.choice.unwrap().observer, protected);
    let (state, result) = choose(state, protected, vec![shan.raw()], false, 36);
    assert_eq!(result.choice.unwrap().observer, source);
    assert!(state.hand_cards(source).contains(&forced_again));
    let (state, _) = choose(state, source, Vec::new(), true, 37);
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn a_forced_sha_that_deals_damage_ends_the_loop() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let forced = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 38);
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, _) = begin_youzhan(state, players, 39);
    let (state, result) = choose(state, source, vec![forced.raw()], false, 42);
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp - 1);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);
}

#[test]
fn a_physical_youdishenru_uses_the_same_response_pipeline() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let youdi = inject(
        &mut state,
        protected,
        ZoneKind::Hand,
        CardKind::TrickYoudishenru,
        40,
    );
    let incoming = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 41);
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: incoming,
            targets: vec![protected],
            card_targets: Vec::new(),
        },
        40,
    );
    let result = result.choice.unwrap();
    assert_eq!(result.observer, protected);
    assert!(result
        .options
        .iter()
        .any(|option| option.token == youdi.raw()));
    let (state, _) = choose(state, protected, vec![youdi.raw()], false, 41);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
    assert!(!state.hand_cards(protected).contains(&youdi));
}
