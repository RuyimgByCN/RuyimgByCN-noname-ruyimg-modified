//! Golden coverage for REFRESH「龙胆」与 SP赵云的旧 JavaScript 语义重写。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::dying_save_card_for;
use engine_core::state::{Card, CardKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLLONGDAN: u32 = 2_506_574_121;
const CHONGZHEN: u32 = 4_144_500_448;
const SP_ZHAOYUN: u32 = 2_240_373_669;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/ollongdan.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("ollongdan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, OLLONGDAN as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
            skill_def: SkillDefId::new(OLLONGDAN),
        },
        EffectRequest::SetTurnState {
            current: owner,
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLLONGDAN)
        .unwrap();
    (tx.commit().state, owner, target, skill)
}

fn create_expansion(state: &mut GameState, owner: PlayerId) -> ZoneId {
    state.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: Vec::new(),
    })
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
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

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn packages_publish_four_typed_rules_and_close_sp_zhaoyun() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    let rules: Vec<_> = refresh
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == OLLONGDAN)
        .collect();
    assert_eq!(rules.len(), 4);
    assert_eq!(
        rules
            .iter()
            .map(|rule| {
                (
                    rule.as_kind.as_str(),
                    rule.window.as_str(),
                    rule.filter.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        [
            ("shan", "response", "card_sha"),
            ("sha", "any", "card_shan"),
            ("jiu", "any", "card_tao"),
            ("tao", "any", "card_jiu"),
        ]
    );
    assert!(rules.iter().all(|rule| {
        rule.source_zone.as_deref() == Some("hand_expansion")
            && rule.min_cards == 1
            && rule.max_cards == 1
    }));
    assert!(sp
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert_eq!(
        sp.character_index
            .iter()
            .find(|character| character.id == SP_ZHAOYUN)
            .unwrap()
            .skills,
        [OLLONGDAN, CHONGZHEN]
    );
    assert!(!serde_json::to_string(&(refresh, sp))
        .unwrap()
        .contains(".js"));
}

#[test]
fn hs_accepts_expansion_rejects_equipment_and_supports_both_save_conversions() {
    let (mut state, owner, target, skill) = setup();
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let expansion = create_expansion(&mut state, owner);
    let equip_shan = inject(&mut state, owner, equip, CardKind::Shan, 290_001);
    let expansion_shan = inject(&mut state, owner, expansion, CardKind::Shan, 290_002);
    let unchanged = state.clone();
    let (rejected_state, rejected) = activate(state, owner, skill, target, equip_shan, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(rejected_state, unchanged);

    let hp = unchanged.players.get(target).unwrap().hp;
    let (state, accepted) = activate(unchanged, owner, skill, target, expansion_shan, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(state.zones.get(equip).unwrap().cards.contains(&equip_shan));

    let (mut state, owner, _, _) = setup();
    state.players.get_mut(owner).unwrap().hp = 0;
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let expansion = create_expansion(&mut state, owner);
    inject(&mut state, owner, equip, CardKind::Jiu, 290_003);
    let source = inject(&mut state, owner, expansion, CardKind::Jiu, 290_004);
    assert_eq!(dying_save_card_for(&state, owner, owner), Some(source));
    state.cards.get_mut(source).unwrap().kind = CardKind::Tao;
    assert_eq!(dying_save_card_for(&state, owner, owner), Some(source));
}
