//! Golden coverage for 幻黄盖「焚险」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFENXIAN: u32 = 3_149_854_807;
const TWJUYAN: u32 = 2_964_355_463;
const HUAN_HUANGGAI: u32 = 3_838_174_457;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twfenxian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("twfenxian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 399);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TWFENXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == TWFENXIAN)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result
    );
    state
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![token],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn target_uses_field_card_as_duel_or_owner_uses_virtual_huogong() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 399);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == HUAN_HUANGGAI)
            .unwrap()
            .skills,
        [TWFENXIAN, TWJUYAN]
    );

    let (state, [owner, actor, victim], skill) = setup(2_026_080_205);
    let mut tx = Transaction::begin(state);
    let field = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        99_901,
    );
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let state = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![actor],
            card_targets: Vec::new(),
        },
        1,
    );
    let state = choose(state, actor, 0, 2);
    let state = choose(state, actor, field.raw(), 3);
    assert_eq!(state.resolution.pending_observer, actor.raw());
    let state = choose(state, actor, victim.raw(), 4);
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp - 1);

    let (state, [owner, target, _], skill) = setup(2_026_080_206);
    let mut tx = Transaction::begin(state);
    let shown = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Spade,
        99_902,
    );
    let matching = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        99_903,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        5,
    );
    let state = choose(state, target, 1, 6);
    let state = choose(state, target, shown.raw(), 7);
    let state = choose(state, owner, matching.raw(), 8);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}
