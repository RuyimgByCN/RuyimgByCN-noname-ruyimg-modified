//! Golden coverage for 燕幽赵云与【冲阵】的旧 JavaScript 语义重写。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LONGDAN: u32 = 17;
const CHONGZHEN: u32 = 4_144_500_448;
const YICONG: u32 = 4_025_383_419;
const YY_ZHAOYUN: u32 = 728_577_123;
const SOURCE: &str = concat!(
    include_str!("../../../../content/standard/longdan.skill"),
    "\n",
    include_str!("../../../../content/original/ported/sp/chongzhen.skill")
);

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone != state.zones.find(ZoneKind::Hand, owner).unwrap(),
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, owner_turn: bool) -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("chongzhen-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGDAN),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHONGZHEN),
        },
        EffectRequest::SetTurnState {
            current: if owner_turn { owner } else { other },
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let longdan = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LONGDAN)
        .unwrap();
    (tx.commit().state, owner, other, longdan)
}

fn resolve(
    state: GameState,
    pipeline: PipelineResult,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.unwrap();
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn packages_publish_chongzhen_and_yy_zhaoyun_without_javascript() {
    let sp = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../../content/original/ported/sp/sp.npack.json"
    ))
    .unwrap();
    let offline = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../../content/original/ported/offline/offline.npack.json"
    ))
    .unwrap();
    assert!(sp.manifest.rule_vm_abi_version >= 647);
    assert!(sp
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHONGZHEN && skill.name == "chongzhen"));
    let character = offline
        .character_index
        .iter()
        .find(|character| character.id == YY_ZHAOYUN)
        .unwrap();
    assert_eq!(character.skills, [LONGDAN, CHONGZHEN, YICONG]);
    assert!(!serde_json::to_string(&(sp, offline))
        .unwrap()
        .contains(".js"));
}

#[test]
fn longdan_shan_as_sha_takes_only_first_targets_hand_card() {
    let (mut state, owner, target, longdan) = setup(2_026_072_801, true);
    let shan = inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 231_001);
    let hand = inject(&mut state, target, ZoneKind::Hand, CardKind::Tao, 231_002);
    let equip = inject(
        &mut state,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        231_003,
    );
    let revision = state.revision;
    let (state, optional) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: longdan,
                targets: vec![target],
                card_targets: vec![shan],
            },
        },
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(optional.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, pick) = resolve(state, optional, owner, vec![1], 2);
    assert_eq!(pick.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);
    assert_eq!(pick.choice.as_ref().unwrap().options[0].token, hand.raw());
    assert!(!pick
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == equip.raw()));
    let (state, done) = resolve(state, pick, owner, vec![hand.raw()], 3);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(hand).unwrap().owner, owner);
}

#[test]
fn physical_sha_does_not_trigger_but_longdan_response_does() {
    let (mut state, owner, attacker, _) = setup(2_026_072_802, false);
    let sha = inject(&mut state, attacker, ZoneKind::Hand, CardKind::Sha, 231_011);
    let loot = inject(&mut state, attacker, ZoneKind::Hand, CardKind::Tao, 231_012);
    let response = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 231_013);
    let revision = state.revision;
    let (state, response_choice) = apply_command(
        state,
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "实体杀本身不能触发冲阵"
    );
    let (state, optional) = resolve(state, response_choice, owner, vec![response.raw()], 11);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, pick) = resolve(state, optional, owner, vec![1], 12);
    assert_eq!(pick.choice.as_ref().unwrap().kind, ChoiceKind::SelectCards);
    assert!(pick
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == loot.raw()));
    let (state, done) = resolve(state, pick, owner, vec![loot.raw()], 13);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(loot).unwrap().owner, owner);
}
