//! PA08-4 压测载荷：满手牌对局存档 dump（表现层采样扩面）。
//!
//! 以 `cargo test --offline -p engine-native pa08_fullhand -- --nocapture` 运行，
//! 从输出提取 `FULLHAND_HEX:<hex>` 存成文件，经
//! `apps/web-client/scripts/web-frame-external.mjs --hex <file>` 在浏览器恢复开局，
//! 测量大手牌视图的回包序列化与 Vue 刷新预算（PA10 增量化裁决依据）。
//!
//! 场景：4 座身份局、标准目录、0 号座位出牌阶段、手牌 30 张（基本/锦囊/装备混排）。
//! 仅标准目录：与 worker 无 packsJson 的 `restore_hex` 路径一致（大扩展包场景
//! 另走 `--packs`，两场景分开测——restore 会校验包集合与存档一致）。

use engine_native::archive::{decode_archive, encode_archive};
use engine_native::view::HandView;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn own_hand_len(e: &Engine, seat: u8) -> usize {
    let view = e.view_for(seat);
    match &view.players[seat as usize].hand {
        HandView::Own(ids) => ids.len(),
        HandView::Hidden => panic!("seat {seat} 应为本人视角"),
    }
}

#[test]
fn pa08_fullhand_dump() {
    let mut e = Engine::new_identity(42, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    // 30 张混排：基本牌 20 + 锦囊 6 + 装备 4（覆盖手牌区主流牌面种类；目录牌名
    // 为罗马字 id：sha/shan/tao/jiu/wuzhong/dismantle/snatch/zhuge/armor/horse_plus/qinglong）。
    let composition: &[(&str, usize)] = &[
        ("sha", 8),
        ("shan", 6),
        ("tao", 4),
        ("jiu", 2),
        ("wuzhong", 2),
        ("dismantle", 2),
        ("snatch", 2),
        ("zhuge", 1),
        ("armor", 1),
        ("horse_plus", 1),
        ("qinglong", 1),
    ];
    assert_eq!(composition.iter().map(|(_, n)| n).sum::<usize>(), 30);
    for (name, n) in composition {
        for _ in 0..*n {
            let card = e.debug_place(0, name);
            e.debug_give_hand_card(0, 0, card);
        }
    }
    assert_eq!(own_hand_len(&e, 0), 30, "塞牌后 0 号手牌应为 30 张");
    // 保持出牌阶段落盘：后续浏览器侧 autodrive 逐步出牌/弃牌，每步信封都带 30 张
    // 手牌视图（若在此处 NextPhase，会立刻挂起 min=25 的弃牌询问，状态仍可用，
    // 但出牌阶段面更有代表性）。

    // 存档十六进制 + 恢复往返：手牌数保持。
    let bytes = encode_archive(&e.save());
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let restored = Engine::restore(&decode_archive(&bytes).expect("decode")).expect("restore");
    assert_eq!(own_hand_len(&restored, 0), 30, "恢复后手牌数应保持");

    println!("FULLHAND_HEX_LEN:{}", hex.len());
    println!("FULLHAND_HEX:{hex}");
}
