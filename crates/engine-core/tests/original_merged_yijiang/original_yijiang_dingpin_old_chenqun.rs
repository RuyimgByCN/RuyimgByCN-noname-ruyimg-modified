//! Golden coverage for 旧陈群「定品」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DINGPIN: u32 = 3_356_547_985;
const OLDFAEN: u32 = 3_151_099_816;
const OLD_CHENQUN: u32 = 2_336_327_482;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/dingpin.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("dingpin-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DINGPIN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn judgment(state: &mut GameState, suit: Suit, def: u32) {
    inject(
        state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        suit,
        def,
    );
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DINGPIN)
        .unwrap()
        .id
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    cost: CardId,
    key: u64,
) -> (GameState, CommandResult) {
    let runtime_skill = skill(&state, owner);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: runtime_skill,
                targets: vec![target],
                card_targets: vec![cost],
            },
        },
    );
    (state, result.result)
}

#[test]
fn dingpin_preserves_owner_history_judgment_target_and_old_character_semantics() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(
        yijiang
            .active_index
            .iter()
            .find(|active| active.skill_def_id == DINGPIN)
            .unwrap()
            .selection
            .as_ref()
            .unwrap()
            .unused_category_by_owner_history_this_turn
    );
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_CHENQUN)
        .unwrap();
    assert_eq!(character.skills, [DINGPIN, OLDFAEN]);

    // A red result turns the owner over but does not lock the target; the discarded
    // basic category alone prevents another basic cost.
    let (mut red, owner, _target) = setup(20_260_729_01);
    let basic = inject(
        &mut red,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        98_001,
    );
    judgment(&mut red, Suit::Heart, 98_002);
    let (mut red, first) = activate(red, owner, owner, basic, 1);
    assert!(
        matches!(first, CommandResult::Accepted { .. }),
        "unexpected first Dingpin result: {first:?}"
    );
    assert!(red.players.get(owner).unwrap().turned_over);
    let repeated_basic = inject(
        &mut red,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        98_003,
    );
    let (mut red, repeated) = activate(red, owner, owner, repeated_basic, 2);
    assert!(matches!(repeated, CommandResult::Rejected { .. }));
    let trick = inject(
        &mut red,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Club,
        98_004,
    );
    judgment(&mut red, Suit::Diamond, 98_005);
    let (_red, second) = activate(red, owner, owner, trick, 3);
    assert!(matches!(second, CommandResult::Accepted { .. }));

    // A black result draws current lost HP and only then locks that target.
    let (mut black, owner, target) = setup(20_260_729_02);
    {
        let target_state = black.players.get_mut(target).unwrap();
        target_state.max_hp = 4;
        target_state.hp = 2;
    }
    let before = black.hand_count(target);
    let equip = inject(
        &mut black,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
        98_011,
    );
    judgment(&mut black, Suit::Spade, 98_012);
    let (mut black, result) = activate(black, owner, target, equip, 4);
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(black.hand_count(target), before + 2);
    let basic = inject(
        &mut black,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        98_013,
    );
    let (_black, repeated_target) = activate(black, owner, target, basic, 5);
    assert!(matches!(repeated_target, CommandResult::Rejected { .. }));

    // Card-use history is independent from Dingpin's own costs.
    let (mut used, owner, target) = setup(20_260_729_03);
    let used_trick = inject(
        &mut used,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        98_021,
    );
    let mut tx = Transaction::begin(used);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: used_trick,
            targets: vec![],
        }],
    )
    .unwrap();
    let mut used = tx.commit().state;
    let repeated_trick = inject(
        &mut used,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Club,
        98_022,
    );
    let (_used, repeated) = activate(used, owner, target, repeated_trick, 6);
    assert!(matches!(repeated, CommandResult::Rejected { .. }));
}
