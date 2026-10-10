//! Golden coverage for 界关平「界龙吟」and compatibility with the original 龙吟 ABI.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RELONGYIN: u32 = 2_474_771_514;
const JIEZHONG: u32 = 2_811_249_808;
const RE_GUANPING: u32 = 2_498_702_026;
const LONGYIN: u32 = 3_578_226_944;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/relongyin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const OLD_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/longyin.skill");
const OLD_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    owner: PlayerId,
    skill: SkillId,
    sha: CardId,
    cost: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(
    package_bytes: &[u8],
    skill_def: u32,
    seed: u64,
    sha_suit: Suit,
    sha_rank: u8,
    cost_rank: u8,
) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = tx.state().turn.current;
    let target = players
        .iter()
        .copied()
        .find(|player| *player != source)
        .unwrap();
    let owner = players
        .iter()
        .copied()
        .find(|player| *player != source && *player != target)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap();
    let sha = inject(
        tx.state_mut(),
        source,
        295_001,
        CardKind::Sha,
        sha_suit,
        sha_rank,
    );
    let cost = inject(
        tx.state_mut(),
        owner,
        295_002,
        CardKind::Jiu,
        Suit::Club,
        cost_rank,
    );
    Setup {
        state: tx.commit().state,
        source,
        target,
        owner,
        skill,
        sha,
        cost,
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

fn offer_and_accept(setup: &Setup, key: u64) -> GameState {
    let revision = setup.state.revision;
    let (state, offered) = apply_command(
        setup.state.clone(),
        Command {
            actor: setup.source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());
    let (state, accepted) = resolve(state, setup.owner, vec![1], key + 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed
    );
    state
}

fn pay(mut state: GameState, setup: &Setup, already_refunded: bool, key: u64) -> GameState {
    if already_refunded {
        state.turn.set_flag(turn_flag::SHA_PLAYED, 0);
        let flag_index = match state.resolution.resume_kind {
            ContinuationKind::ResumePlayedSha => 4,
            ContinuationKind::ResumePlayedShaQueue => 3,
            other => panic!("unexpected saved Sha continuation: {other:?}"),
        };
        state.resolution.resume_payload[flag_index] |= 4;
        state
            .resolution
            .sha_resolution_records
            .iter_mut()
            .filter(|record| record.source == setup.source && record.card == setup.sha)
            .for_each(|record| record.use_refunded = true);
    }
    let (state, paid) = resolve(state, setup.owner, vec![setup.cost.raw()], key);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

fn has_skill(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

#[test]
fn package_publishes_current_abi_and_complete_re_guanping() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 298);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert!(package.skill_index.len() >= 85);
    assert!(package.character_index.len() >= 26);
    assert!(package.modules.len() >= 63);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GUANPING)
        .unwrap();
    assert_eq!(character.name_key, "re_guanping");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [RELONGYIN, JIEZHONG]);
    let compiled = content_compiler::compile_source_named("relongyin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 295);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.code, compiled.modules[0].code);
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn effective_rank_and_refund_branches_match_legacy_relongyin() {
    struct Case {
        suit: Suit,
        sha_rank: u8,
        cost_rank: u8,
        already_refunded: bool,
        expected_hand: u32,
        expected_restore: bool,
    }
    let cases = [
        Case {
            suit: Suit::Heart,
            sha_rank: 7,
            cost_rank: 7,
            already_refunded: false,
            expected_hand: 1,
            expected_restore: true,
        },
        Case {
            suit: Suit::Spade,
            sha_rank: 8,
            cost_rank: 8,
            already_refunded: false,
            expected_hand: 0,
            expected_restore: true,
        },
        Case {
            suit: Suit::Diamond,
            sha_rank: 9,
            cost_rank: 10,
            already_refunded: false,
            expected_hand: 1,
            expected_restore: false,
        },
        Case {
            suit: Suit::Heart,
            sha_rank: 11,
            cost_rank: 11,
            already_refunded: true,
            expected_hand: 1,
            expected_restore: true,
        },
    ];
    for (index, case) in cases.into_iter().enumerate() {
        let setup = setup(
            PACKAGE,
            RELONGYIN,
            2_026_073_100 + index as u64,
            case.suit,
            case.sha_rank,
            case.cost_rank,
        );
        let state = offer_and_accept(&setup, 100 + index as u64 * 10);
        assert_eq!(
            state.resolution.continuation_payload,
            [
                setup.owner.raw(),
                setup.source.raw(),
                setup.sha.raw(),
                setup.skill.raw(),
                JIEZHONG,
            ],
            "case {index}"
        );
        let state = pay(
            state,
            &setup,
            case.already_refunded,
            102 + index as u64 * 10,
        );
        assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0, "case {index}");
        assert_eq!(
            state.hand_count(setup.owner),
            case.expected_hand,
            "case {index}"
        );
        assert_eq!(
            has_skill(&state, setup.owner, JIEZHONG),
            case.expected_restore,
            "case {index}"
        );
    }
}

#[test]
fn original_longyin_module_and_four_word_continuation_remain_stable() {
    let compiled = content_compiler::compile_source_named("longyin", OLD_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 43);
    assert_eq!(compiled.modules[0].code, [83, 85, 86, 80, 61, 1]);
    assert_eq!(
        compiled.modules[0].logic_hash,
        [
            23, 183, 110, 115, 100, 48, 169, 6, 243, 112, 120, 102, 154, 3, 105, 63, 222, 225, 72,
            185, 146, 62, 190, 239, 208, 78, 135, 229, 116, 236, 163, 70,
        ]
    );
    let package = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    let committed = package
        .modules
        .iter()
        .find(|module| module.name == "longyin_card_used")
        .unwrap();
    assert_eq!(committed.code, compiled.modules[0].code);
    assert_eq!(committed.logic_hash, compiled.modules[0].logic_hash);

    let setup = setup(OLD_PACKAGE, LONGYIN, 2_026_073_200, Suit::Heart, 7, 7);
    let state = offer_and_accept(&setup, 200);
    assert_eq!(
        state.resolution.continuation_payload,
        [
            setup.owner.raw(),
            setup.source.raw(),
            setup.sha.raw(),
            setup.skill.raw(),
        ]
    );
    assert_eq!(
        canonical_state_hash(&state),
        [
            93, 189, 86, 88, 231, 121, 116, 47, 231, 144, 116, 198, 180, 63, 94, 81, 158, 142, 151, 39, 225, 117, 114, 113, 228, 120, 149, 124, 26, 228, 190, 42,
        ]
    );
}
