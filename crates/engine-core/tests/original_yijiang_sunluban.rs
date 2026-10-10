//! Golden coverage for 孙鲁班「谮毁」「骄矜」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SUNLUBAN: u32 = 3_957_343_056;
const GUYONG: u32 = 3_099_069_622;
const CHANHUI: u32 = 2_955_673_837;
const JIAOJIN: u32 = 2_809_119_976;
const CHANHUI_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/chanhui.skill");
const JIAOJIN_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/jiaojin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    female: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let alternate = players[2];
    let female = players[3];
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
    effects.extend([CHANHUI, JIAOJIN].map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = SUNLUBAN;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = GUYONG;
    tx.state_mut()
        .players
        .get_mut(alternate)
        .unwrap()
        .character_def = GUYONG;
    tx.state_mut()
        .players
        .get_mut(female)
        .unwrap()
        .character_def = SUNLUBAN;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        female,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
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

fn play_sha(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn open_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, Vec<DomainEvent>, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (
        committed.state,
        committed.output.events,
        committed.output.choice.unwrap(),
    )
}

fn skill_phase_count(state: &GameState, owner: PlayerId, skill_def: u32) -> u32 {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .play_phase_fire_count
}

#[test]
fn package_publishes_complete_sunluban_and_exact_abi57_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SUNLUBAN)
        .unwrap();
    assert_eq!(character.name_key, "sunluban");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [CHANHUI, JIAOJIN]);
    for (name, source) in [("chanhui", CHANHUI_SOURCE), ("jiaojin", JIAOJIN_SOURCE)] {
        let compiled = content_compiler::compile_source_named(name, source).unwrap();
        assert_eq!(compiled.manifest.rule_vm_abi_version, 57);
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
fn chanhui_cancel_branch_adds_target_and_rejects_forged_gift_atomically() {
    let mut setup = setup(2_026_073_701);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_701,
        CardKind::Sha,
        Suit::Spade,
    );
    let gift = inject(
        &mut setup.state,
        setup.female,
        ZoneKind::Hand,
        99_702,
        CardKind::Other,
        Suit::Heart,
    );
    let (state, optional) = play_sha(setup.state, setup.owner, sha, setup.target, 1);
    assert_eq!(optional.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, targets) = resolve(state, setup.owner, vec![1], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCardUserOrAddTarget
    );
    assert!(targets
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == setup.female.raw()));
    let (state, branch) = resolve(state, setup.owner, vec![setup.female.raw()], false, 3);
    assert_eq!(branch.choice.as_ref().unwrap().observer, setup.female);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseGiveCardOrAddTarget
    );
    assert_eq!(skill_phase_count(&state, setup.owner, CHANHUI), 1);

    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.female, vec![sha.raw()], false, 4);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, added) = resolve(state, setup.female, Vec::new(), true, 5);
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { player, card, target, targets }
            if *player == setup.owner && *card == sha && *target == setup.female
                && targets == &[setup.target, setup.female]
    )));
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(setup.female).unwrap().hp, 9);
    assert_eq!(state.cards.get(gift).unwrap().owner, setup.female);
}

#[test]
fn chanhui_gift_branch_transfers_card_user_before_damage() {
    let mut setup = setup(2_026_073_702);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_711,
        CardKind::Sha,
        Suit::Club,
    );
    let gift = inject(
        &mut setup.state,
        setup.female,
        ZoneKind::Hand,
        99_712,
        CardKind::Other,
        Suit::Diamond,
    );
    let (state, _) = play_sha(setup.state, setup.owner, sha, setup.target, 10);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 11);
    let (state, _) = resolve(state, setup.owner, vec![setup.female.raw()], false, 12);
    let owner_hand = state.hand_count(setup.owner);
    let (state, transferred) = resolve(state, setup.female, vec![gift.raw()], false, 13);
    assert!(
        matches!(transferred.result, CommandResult::Accepted { .. }),
        "unexpected transfer result: {:?}",
        transferred.result
    );
    assert!(transferred.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUserChanged { previous, player, card }
            if *previous == setup.owner && *player == setup.female && *card == sha
    )));
    assert!(transferred.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt { source, target, amount: 1, .. }
            if *source == setup.female && *target == setup.target
    )));
    assert_eq!(state.hand_count(setup.owner), owner_hand + 1);
    assert_eq!(state.cards.get(gift).unwrap().owner, setup.owner);
    assert_eq!(state.players.get(setup.target).unwrap().hp, 9);
}

#[test]
fn chanhui_accepts_black_but_not_red_single_target_ordinary_tricks() {
    let mut red_setup = setup(2_026_073_706);
    let red_duel = inject(
        &mut red_setup.state,
        red_setup.owner,
        ZoneKind::Hand,
        99_713,
        CardKind::TrickJuedou,
        Suit::Heart,
    );
    let (state, red) = submit(
        red_setup.state,
        red_setup.owner,
        CommandKind::PlayCard {
            card: red_duel,
            targets: vec![red_setup.target],
            card_targets: Vec::new(),
        },
        14,
    );
    assert!(matches!(red.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(red_setup.target).unwrap().hp, 9);

    let mut black_setup = setup(2_026_073_707);
    let black_duel = inject(
        &mut black_setup.state,
        black_setup.owner,
        ZoneKind::Hand,
        99_714,
        CardKind::TrickJuedou,
        Suit::Club,
    );
    let (state, black) = submit(
        black_setup.state,
        black_setup.owner,
        CommandKind::PlayCard {
            card: black_duel,
            targets: vec![black_setup.target],
            card_targets: Vec::new(),
        },
        15,
    );
    assert_eq!(black.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, _) = resolve(state, black_setup.owner, vec![1], false, 16);
    let (state, added) = resolve(
        state,
        black_setup.owner,
        vec![black_setup.female.raw()],
        false,
        17,
    );
    assert!(matches!(added.result, CommandResult::Accepted { .. }));
    assert!(added.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { card, target, targets, .. }
            if *card == black_duel && *target == black_setup.female
                && targets == &[black_setup.target, black_setup.female]
    )));
    assert_eq!(state.players.get(black_setup.target).unwrap().hp, 9);
    assert_eq!(state.players.get(black_setup.female).unwrap().hp, 9);
}

#[test]
fn jiaojin_discards_effective_equipment_and_reduces_or_prevents_male_damage() {
    let mut first_setup = setup(2_026_073_703);
    let equipment = inject(
        &mut first_setup.state,
        first_setup.owner,
        ZoneKind::Hand,
        99_721,
        CardKind::Weapon,
        Suit::Spade,
    );
    let basic = inject(
        &mut first_setup.state,
        first_setup.owner,
        ZoneKind::Hand,
        99_722,
        CardKind::Tao,
        Suit::Heart,
    );
    let (state, _, optional) =
        open_damage(first_setup.state, first_setup.target, first_setup.owner, 2);
    assert_eq!(optional.kind, ChoiceKind::Confirm);
    let (state, cost) = resolve(state, first_setup.owner, vec![1], false, 20);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage
    );
    assert_eq!(
        cost.choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [equipment.raw()]
    );
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, first_setup.owner, vec![basic.raw()], false, 21);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, reduced) = resolve(state, first_setup.owner, vec![equipment.raw()], false, 22);
    assert!(matches!(reduced.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first_setup.owner).unwrap().hp, 9);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);

    let mut setup = setup(2_026_073_704);
    let equipment = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Equip,
        99_723,
        CardKind::Armor,
        Suit::Club,
    );
    let (state, _, _) = open_damage(setup.state, setup.target, setup.owner, 1);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 23);
    let (state, prevented) = resolve(state, setup.owner, vec![equipment.raw()], false, 24);
    assert!(matches!(prevented.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 10);
    assert_eq!(state.resolution.current_damage_id, 0);
    assert!(state.resolution.prevented_damage_ids.is_empty());
}

#[test]
fn jiaojin_does_not_trigger_for_female_damage_source() {
    let mut setup = setup(2_026_073_705);
    inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_731,
        CardKind::Treasure,
        Suit::Diamond,
    );
    let mut tx = Transaction::begin(setup.state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: setup.female,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt { source, target, amount: 1, .. }
            if *source == setup.female && *target == setup.owner
    )));
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 9);
    assert!(!state.has_pending_choice());
}
