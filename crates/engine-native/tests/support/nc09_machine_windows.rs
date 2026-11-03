//! Real pending windows from the existing Li Dian, Yi Ji, Boss and SWD fixtures.
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

#[derive(Debug, Clone, Copy)]
// 共享支撑：单个用例只构造部分 Window 变体/字段；CI rustc（1.99）对每份
// 模块实例分别做 dead_code 分析，显式放行（仅测试支撑文件）。
#[allow(dead_code)]
pub enum Window {
    XunXun,
    JiJie,
    GaiMing,
    HaoTianTa,
    Liaoyuan,
}

#[allow(dead_code)]
pub struct Fixture {
    pub engine: Engine,
    pub packs: Vec<ExtensionPack>,
    pub physical_ids: Vec<u32>,
    pub held: Vec<u32>,
    pub selection: Vec<ChoiceOption>,
}

pub fn submit(engine: &mut Engine, action: GameAction) {
    let result = engine.submit(GameCommand {
        actor: engine
            .open_choice()
            .map_or(engine.turn().current, |c| c.observer),
        expected_revision: engine.revision(),
        dedup_id: engine.revision() + 1,
        action,
    });
    assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
}

pub fn answer(engine: &mut Engine, selection: Vec<ChoiceOption>) {
    let choice_id = engine.open_choice().unwrap().id;
    submit(
        engine,
        GameAction::Answer {
            choice_id,
            selection,
        },
    );
}

pub fn default_selection(engine: &Engine) -> Vec<ChoiceOption> {
    let open = engine.open_choice().unwrap();
    if open.spec.options.contains(&ChoiceOption::Pass) {
        vec![ChoiceOption::Pass]
    } else if open.spec.allow_cancel {
        vec![]
    } else {
        open.spec
            .options
            .iter()
            .take(open.spec.min)
            .copied()
            .collect()
    }
}

fn advance_to_play(engine: &mut Engine) {
    for _ in 0..64 {
        if engine.turn().current == 0
            && engine.turn().phase == Phase::Play
            && engine.open_choice().is_none()
        {
            return;
        }
        if engine.open_choice().is_some() {
            answer(engine, default_selection(engine));
        } else {
            submit(engine, GameAction::NextPhase);
        }
    }
    panic!("seat zero play phase");
}

fn pack(directory: &str) -> ExtensionPack {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(directory)
        .join("assets/cards.json");
    content_native_loader::bridge_pack(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .pack
}

pub fn fixture(window: Window) -> Fixture {
    let packs = match window {
        Window::GaiMing => vec![pack("boss-zhugeliang-upstream-liuli")],
        Window::HaoTianTa | Window::Liaoyuan => vec![pack("shahai-swd-upstream-noname")],
        _ => vec![],
    };
    let seed = match window {
        Window::XunXun => 80,
        Window::JiJie => 100,
        Window::GaiMing => 93003,
        Window::HaoTianTa => 62023,
        Window::Liaoyuan => 122001,
    };
    let seats = if packs.is_empty() { 3 } else { 4 };
    let refs: Vec<_> = packs.iter().collect();
    let mut engine = Engine::new_with_extensions(seed, seats, &refs).unwrap();
    // Capture actual initial ownership, independently of the collector under test.
    let mut physical_ids = engine.state_core().deck.clone();
    for player in &engine.state_core().players {
        physical_ids.extend(&player.hand);
    }
    physical_ids.sort_unstable();
    assert!(physical_ids.windows(2).all(|ids| ids[0] != ids[1]));
    let (prompt, held, selection) = match window {
        Window::XunXun => {
            advance_to_play(&mut engine);
            for seat in 0..seats as u8 {
                engine.debug_clear_hand(seat);
            }
            engine.debug_assemble_general(0, "李典");
            engine.debug_set_turn(2, Phase::End, engine.turn().round);
            submit(&mut engine, GameAction::NextPhase);
            submit(&mut engine, GameAction::NextPhase);
            assert_eq!(engine.open_choice().unwrap().spec.prompt, "trigger_confirm");
            answer(&mut engine, vec![ChoiceOption::Confirm]);
            let held = card_options(&engine);
            assert_eq!(held.len(), 4);
            let selection = held[..2].iter().copied().map(ChoiceOption::Card).collect();
            ("xunxun_arrange", held, selection)
        }
        Window::JiJie => {
            advance_to_play(&mut engine);
            for seat in 0..seats as u8 {
                engine.debug_clear_hand(seat);
            }
            engine.debug_assemble_general(0, "伊籍");
            let card = engine.state_core().deck[0];
            let skill = engine.catalog().skill_by_name("机捷").unwrap();
            submit(&mut engine, GameAction::ActivateSkill { skill });
            (
                "jijie_give_target",
                vec![card],
                vec![ChoiceOption::Player(1)],
            )
        }
        Window::GaiMing => {
            let def = (0..engine.catalog().general_count() as u16)
                .find(|def| {
                    engine
                        .catalog()
                        .general_meta(*def)
                        .is_some_and(|g| g.0 == "boss_zhugeliang")
                })
                .unwrap();
            engine.debug_assemble_general_def(0, def);
            engine.debug_place_judge(0, "le");
            engine.debug_set_deck_top_by(|card| card.suit.is_spade());
            engine.debug_set_turn(3, Phase::End, engine.turn().round);
            submit(&mut engine, GameAction::NextPhase);
            assert_eq!(engine.open_choice().unwrap().spec.prompt, "trigger_confirm");
            answer(&mut engine, vec![]);
            let held = card_options(&engine);
            assert_eq!(held.len(), 7);
            (
                "gaiming_pick",
                held.clone(),
                vec![ChoiceOption::Card(held[6])],
            )
        }
        Window::Liaoyuan => {
            engine.debug_grant(0, "liaoyuan");
            engine.debug_set_turn(0, Phase::Play, 1);
            let sha = engine.debug_move_named_card_to_hand(0, "sha");
            let suit = engine.catalog().cards[sha as usize].suit;
            let equip_name = engine
                .catalog()
                .cards
                .iter()
                .find(|card| {
                    card.suit == suit
                        && card.kind.equip_slot().is_some()
                        && card.equip_skills.is_empty()
                })
                .unwrap()
                .name;
            let equip = engine.debug_place(0, equip_name);
            submit(
                &mut engine,
                GameAction::PlayCard {
                    card: equip,
                    targets: vec![],
                },
            );
            for _ in 0..3 {
                let index = engine
                    .state_core()
                    .deck
                    .iter()
                    .position(|id| engine.catalog().cards[*id as usize].suit == suit)
                    .unwrap();
                let card = engine.state_core_mut().deck.remove(index);
                engine.state_core_mut().players[0].hand.push(card);
            }
            submit(
                &mut engine,
                GameAction::PlayCard {
                    card: sha,
                    targets: vec![1],
                },
            );
            for _ in 0..16 {
                let choice = engine.open_choice().unwrap();
                if choice.spec.prompt == "liaoyuan_discard" {
                    break;
                }
                let selection = if choice.spec.options.contains(&ChoiceOption::Confirm) {
                    vec![ChoiceOption::Confirm]
                } else {
                    choice
                        .spec
                        .options
                        .iter()
                        .take(choice.spec.min)
                        .copied()
                        .collect()
                };
                answer(&mut engine, selection);
            }
            let open = engine.open_choice().unwrap();
            assert_eq!(open.spec.prompt, "liaoyuan_discard");
            assert!(open.spec.options.contains(&ChoiceOption::Card(equip)));
            let selection = vec![open.spec.options[0]];
            ("liaoyuan_discard", vec![], selection)
        }
        Window::HaoTianTa => {
            advance_to_play(&mut engine);
            for seat in 0..seats as u8 {
                engine.debug_clear_hand(seat);
            }
            let name = engine
                .catalog()
                .cards
                .iter()
                .find(|card| card.kind == CardKind::Haotianta)
                .unwrap()
                .name;
            let card = engine.debug_place(2, name);
            engine.debug_set_turn(2, Phase::Play, engine.turn().round);
            submit(
                &mut engine,
                GameAction::PlayCard {
                    card,
                    targets: vec![],
                },
            );
            engine.debug_set_turn(0, Phase::Play, engine.turn().round);
            let le = engine.debug_place(0, "le");
            submit(
                &mut engine,
                GameAction::PlayCard {
                    card: le,
                    targets: vec![1],
                },
            );
            assert!(engine.open_choice().is_none());
            let heart = engine.debug_set_deck_top_by(|card| card.suit.is_heart());
            let spade = engine.debug_set_deck_top_by(|card| card.suit.is_spade());
            engine.debug_set_turn(0, Phase::End, engine.turn().round);
            submit(&mut engine, GameAction::NextPhase);
            assert_eq!(card_options(&engine), vec![spade, heart]);
            (
                "haotianta_pick",
                vec![spade, heart],
                vec![ChoiceOption::Card(heart)],
            )
        }
    };
    assert_eq!(engine.open_choice().unwrap().spec.prompt, prompt);
    Fixture {
        engine,
        packs,
        physical_ids,
        held,
        selection,
    }
}

fn card_options(engine: &Engine) -> Vec<u32> {
    engine
        .open_choice()
        .unwrap()
        .spec
        .options
        .iter()
        .filter_map(|option| match option {
            ChoiceOption::Card(id) => Some(*id),
            _ => None,
        })
        .collect()
}

impl Fixture {
    pub fn restore(
        &self,
        snapshot: &engine_native::StateSnapshot,
    ) -> Result<Engine, engine_native::SnapshotError> {
        let packs: Vec<_> = self.packs.iter().collect();
        Engine::restore_with_extensions(snapshot, &packs)
    }
}
