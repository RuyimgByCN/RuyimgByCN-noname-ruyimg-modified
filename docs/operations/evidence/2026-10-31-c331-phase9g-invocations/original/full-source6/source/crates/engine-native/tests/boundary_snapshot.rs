//! 命令信任边界 + 挂起机器快照/恢复验收（进度文档 P0 两项）。
//!
//! 边界：错误玩家、过期修订、重复命令、越界 ID 全部拒绝且不 panic；
//! 快照：任意静止点保存 → 恢复 → 每个选择点继续到相同结果；规则版本绑定。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::choice::ChoiceSpec;
use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot,
    Phase, RejectReason, SnapshotError,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    e.submit(GameCommand {
        actor,
        expected_revision,
        dedup_id: fresh_dedup(),
        action,
    })
}

fn next_phase(e: &mut Engine) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::NextPhase)
}

fn answer_sel(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let choice = e.open_choice().expect("choice open").clone();
    submit_cmd(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    )
}

fn answer(e: &mut Engine, opt: ChoiceOption) -> CommandOutcome {
    answer_sel(e, vec![opt])
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::PlayCard { card, targets })
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play {
            return;
        }
        match next_phase(e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(choice.spec.min.max(1))
                    .collect();
                assert!(matches!(
                    answer_sel(e, sel),
                    CommandOutcome::Accepted { .. }
                ));
            }
            other => panic!("advance rejected: {other:?}"),
        }
    }
    panic!("advance_to_play did not converge");
}

/// 布阵到「0 号在出牌阶段持一张杀、1 号无手牌」。
fn setup_play_with_sha() -> (Engine, u32) {
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    (e, sha)
}

/// 布阵到「散财确认选择挂起」（嵌套选择链共 3 个选择点）。
fn setup_sancai_pending() -> Engine {
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    assert_eq!(
        e.open_choice().expect("pending").spec.prompt,
        "trigger_confirm"
    );
    e
}

/// 按提示生成确定性应答。
fn answer_for(spec: &ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "trigger_confirm" => vec![ChoiceOption::Confirm],
        "sancai_target" => vec![ChoiceOption::Player(2)],
        "sancai_card" => vec![*spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
            .expect("card option")],
        other => panic!("unexpected prompt {other}"),
    }
}

/// 推进一个阶段；若挂起则按最小牌应答（弃牌等）直至本段完结。
fn drive_phase(e: &mut Engine) {
    match next_phase(e) {
        CommandOutcome::Accepted { .. } => {}
        CommandOutcome::Pending { choice } => {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(choice.spec.min.max(1))
                .collect();
            let out = answer_sel(e, sel);
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
        }
        other => panic!("drive_phase rejected: {other:?}"),
    }
}

/// 递归寻找快照中的散财机器并把目标改为越界玩家（闸门/窗口包裹的机器在深层）。
fn patch_sancai_target(m: &mut MachineSnapshot) -> bool {
    match m {
        MachineSnapshot::SanCai { target, .. } => {
            *target = Some(200);
            true
        }
        MachineSnapshot::OptionalGate { inner, .. } => inner
            .as_mut()
            .map(|m| patch_sancai_target(m))
            .unwrap_or(false),
        MachineSnapshot::NullifyWindow { effect, .. } => effect
            .as_mut()
            .map(|m| patch_sancai_target(m))
            .unwrap_or(false),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// N04 结构校验（快照内栈帧/挂起/身份/胜负关系）+ N05 逐机器暂停点往返矩阵
// ---------------------------------------------------------------------------

use engine_native::event::{EventData, QueuedTrigger};
use engine_native::resolution::Slot;
use engine_native::state::{GameMode, Outcome, Role, Winner};

// ---------------------------------------------------------------------------
// N05：逐机器暂停点往返矩阵
// ---------------------------------------------------------------------------

/// 通用矩阵：build() 停在首个挂起点；基准线直跑收集应答与终态；
/// 快照线从同一快照恢复，每个挂起点「保存→恢复→应答」，终态必须一致。
fn assert_pause_roundtrip(
    build: fn() -> Engine,
    answer_for: fn(&engine_native::ChoiceSpec) -> Vec<ChoiceOption>,
) {
    // 守恒断言（N01–N03 回归守护）：每张实体牌在各牌区或机器持牌中恰出现一次。
    // 判定阶段挂起点上，待判锦囊由机器队列持有；使用时窗口的待置入锦囊
    // 在桌面上（PlaceDelayed 不持牌）。
    fn machine_cards(m: &MachineSnapshot, out: &mut Vec<u32>) {
        match m {
            MachineSnapshot::JudgePhase { queue, current, .. } => {
                out.extend(queue.iter().copied());
                // 进行中判定：待判锦囊（current.trick）与判定牌（judge_card）
                // 均由机器持有；天妒获得（入手牌）、放弃或随效果弃置
                // （judge_held=false）后不再计机器持牌。
                if let Some(c) = current.as_ref() {
                    // finalized（效果已套用）后锦囊已离机（弃置/转移/判定区），
                    // current.trick 仅为簿记，不再计入机器持牌。
                    if !c.finalized {
                        out.extend(c.trick);
                    }
                    if c.judge_held {
                        out.push(c.judge_card);
                    }
                }
            }
            MachineSnapshot::OptionalGate { inner, .. } => {
                if let Some(m) = inner.as_ref() {
                    machine_cards(m, out);
                }
            }
            MachineSnapshot::NullifyWindow { effect, .. } => {
                if let Some(m) = effect.as_ref() {
                    machine_cards(m, out);
                }
            }
            MachineSnapshot::Aoe { inner, .. } => {
                if let Some(m) = inner.as_ref() {
                    machine_cards(m, out);
                }
            }
            _ => {}
        }
    }
    fn assert_conservation(e: &Engine) {
        let mut ids: Vec<u32> = Vec::new();
        let core = e.state_core();
        for p in &core.players {
            ids.extend(p.hand.iter().copied());
            ids.extend(p.equips.iter().copied());
            ids.extend(p.judge.iter().copied());
        }
        ids.extend(core.deck.iter().copied());
        ids.extend(core.discard.iter().copied());
        ids.extend(core.table.iter().copied());
        for park in e.save().resolution.parks {
            machine_cards(&park.machine, &mut ids);
        }
        ids.sort_unstable();
        let total = ids.len();
        let dups: Vec<u32> = ids
            .windows(2)
            .filter(|w| w[0] == w[1])
            .map(|w| w[0])
            .collect();
        ids.dedup();
        assert_eq!(ids.len(), total, "实体牌出现在多个牌区/机器: 重复={dups:?}");
        assert_eq!(
            ids.len(),
            e.catalog().cards.len(),
            "目录牌与牌区不守恒: 缺失={:?}",
            {
                let all: Vec<u32> = (0..e.catalog().cards.len() as u32).collect();
                let missing: Vec<u32> = all.iter().filter(|c| !ids.contains(c)).copied().collect();
                missing
            }
        );
    }

    // 基准线：记录每个选择点前的日志水位，收集应答序列
    let mut base = build();
    assert!(base.open_choice().is_some(), "build 必须停在首个挂起点");
    let first_hash = base.state_hash();
    let mut answers: Vec<Vec<ChoiceOption>> = Vec::new();
    let mut log_marks: Vec<usize> = Vec::new();
    for _ in 0..12 {
        assert_conservation(&base);
        let Some(choice) = base.open_choice().cloned() else {
            break;
        };
        let sel = answer_for(&choice.spec);
        assert!(
            !sel.is_empty() || choice.spec.allow_cancel,
            "answer_for 必须给出合法应答（空应答仅限可放弃选择）"
        );
        log_marks.push(base.log().len());
        answers.push(sel.clone());
        let out = answer_sel(&mut base, sel);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    assert!(!answers.is_empty());
    // 终止断言：两条线最终都不再有开放选择
    assert!(base.open_choice().is_none(), "基准线必须走到无挂起终态");
    let base_hash = base.state_hash();
    let base_log: Vec<_> = base.log().to_vec();

    // 快照线：从同一首点快照恢复，逐点保存/恢复后应答；
    // 比对每点规范摘要、终态摘要，以及恢复点之后的**事件增量**（恢复件日志为空）。
    let snap0 = build().save();
    let mut cur = Engine::restore(&snap0).expect("restore");
    assert_eq!(cur.state_hash(), first_hash, "恢复件与基准首点同哈希");
    for (i, sel) in answers.iter().enumerate() {
        let Some(_c) = cur.open_choice() else {
            panic!("第 {i} 个选择点应挂起");
        };
        let mid = cur.save();
        let mut resumed = Engine::restore(&mid).expect("restore mid");
        assert_eq!(
            resumed.state_hash(),
            cur.state_hash(),
            "选择点 {i} 往返一致"
        );
        assert!(resumed.log().is_empty(), "恢复件宿主日志应为空");
        let out = answer_sel(&mut resumed, sel.clone());
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
        // 增量段 = 本次应答到下一次挂起（或终态）之间的日志
        let seg_end = log_marks.get(i + 1).copied().unwrap_or(base_log.len());
        assert_eq!(
            resumed.log(),
            &base_log[log_marks[i]..seg_end],
            "选择点 {i} 之后的事件增量必须与基准线一致"
        );
        assert_conservation(&resumed);
        cur = resumed;
    }
    assert_eq!(cur.state_hash(), base_hash, "快照线终态必须与基准一致");
    assert!(cur.open_choice().is_none(), "快照线必须走到无挂起终态");
}

/// 按提示给出确定性应答：确认类答 Confirm，响应/窗口类答 Pass（首选项），
/// 目标类答玩家 2，弃牌类取前 min 张。
fn answer_by_prompt(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "trigger_confirm" => vec![ChoiceOption::Confirm],
        "sancai_target" | "yishang_target" => vec![ChoiceOption::Player(2)],
        "discard_to_limit" => spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(spec.min)
            .collect(),
        _ => vec![spec.options[0]],
    }
}

fn build_sha_respond() -> Engine {
    let (mut e, sha) = setup_play_with_sha();
    let _shan = e.debug_place(1, "shan");
    play(&mut e, sha, vec![1]);
    e
}

fn build_dying_save() -> Engine {
    let mut e = Engine::new(41, 4);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _tao = e.debug_place(0, "tao");
    let sha = e.debug_place(1, "sha");
    play(&mut e, sha, vec![0]);
    e
}

fn build_duel_respond() -> Engine {
    let mut e = Engine::new(141, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let _sha_t = e.debug_place(2, "sha");
    play(&mut e, duel, vec![2]);
    e
}

fn build_aoe_mid_queue() -> Engine {
    let mut e = Engine::new(131, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let _sha1 = e.debug_place(1, "sha");
    let _sha2 = e.debug_place(2, "sha");
    play(&mut e, savage, vec![]);
    e
}

fn build_judge_window() -> Engine {
    let mut e = Engine::new(175, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 使用时无懈窗口：先放无懈再出乐；队列自来源 0 起（0 无无懈跳过）→ 2 被询问
    let _nul = e.debug_place(2, "nullify");
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    e
}

fn build_discard_to_limit() -> Engine {
    let mut e = Engine::new(91, 4);
    e.debug_clear_hand(0);
    for _ in 0..4 {
        e.debug_place(0, "shan");
    }
    e.debug_set_hp(0, 2);
    advance_to_play(&mut e, 0);
    for _ in 0..8 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

fn build_yishang_target() -> Engine {
    let mut e = Engine::new(31, 4);
    e.debug_grant(0, "移伤");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    // 首个挂起点 = 可选触发确认
    e
}

/// 铁索逐目标无懈窗口（规则 v290）：Chain 机器新增 `idx/inner/dispatcher`
/// 编码（存档 v77），窗口挂起点必须可往返；窗口被抵消只影响该目标。
fn build_chain_window() -> Engine {
    let mut e = Engine::new(177, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let chain = e.debug_place(0, "chain");
    let _nul = e.debug_place(1, "nullify");
    play(&mut e, chain, vec![1, 2]);
    e
}

/// 应答策略：无懈窗口出无懈（若有），其余沿用 answer_by_prompt。
fn answer_plays_nullify(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    if spec.prompt == "nullify_window" {
        if let Some(card) = spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
        {
            return vec![*card];
        }
    }
    answer_by_prompt(spec)
}

fn build_nullify_chain() -> Engine {
    let mut e = Engine::new(153, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let _nul1 = e.debug_place(1, "nullify");
    let _nul2 = e.debug_place(2, "nullify");
    let _sha1 = e.debug_place(1, "sha");
    play(&mut e, savage, vec![]);
    e
}

/// 应答策略：主动判定负载——改判询问出第一张黑牌（无牌才 Pass），
/// 天妒确认获得；其余沿用 answer_by_prompt。
fn answer_active_judgement(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "gui_dao_rewrite" => {
            match spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
            {
                Some(card) => vec![*card],
                None => vec![spec.options[0]],
            }
        }
        "tian_du_gain" => vec![ChoiceOption::Confirm],
        _ => answer_by_prompt(spec),
    }
}

/// 布阵到「1 号判定阶段改判/天妒询问」：乐在 1 判定区、判定牌红桃；
/// grants 列出授予 1 号（或 2 号）的技能名。
fn build_judgement_window_with(grants: &[(&str, u8)], black_to: Option<u8>) -> Engine {
    let mut e = Engine::new(263, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for (skill, who) in grants {
        e.debug_grant(*who, skill);
    }
    if let Some(who) = black_to {
        let _black = place_black_sha(&mut e, who);
    }
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    let _ = play(&mut e, le, vec![1]);
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

fn build_guidao_active_rewrite() -> Engine {
    // 2 号鬼道持黑牌：首个挂起点 = 对 2 的改判询问（主动出黑牌）
    build_judgement_window_with(&[("鬼道", 2)], Some(2))
}

fn build_tiandu_confirm() -> Engine {
    // 1 号天妒（限本人）：无改判者，首个挂起点 = 天妒询问（Confirm 获得）
    build_judgement_window_with(&[("天妒", 1)], None)
}

fn build_rewrite_then_tiandu_combo() -> Engine {
    // 1 号兼持鬼道+天妒：改判（自出黑牌）→ 天妒 Confirm 两点链
    build_judgement_window_with(&[("鬼道", 1), ("天妒", 1)], Some(1))
}

/// 应答策略：濒死救援出桃（若有），其余沿用 answer_by_prompt。
fn answer_plays_tao(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    if spec.prompt == "dying_save" {
        if let Some(card) = spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
        {
            return vec![*card];
        }
    }
    answer_by_prompt(spec)
}

/// 布阵：1 血 + 闪电判定（雷击 3 伤 → -2 负体力）+ 三桃；
/// 停在首个濒死救援询问（每轮一人一桃，持桃者被排回队尾连续自救）。
fn build_negative_dying_rescue() -> Engine {
    let mut e = Engine::new(241, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let _sd = e.debug_place_judge(1, "shandian");
    for _ in 0..3 {
        e.debug_place(1, "tao");
    }
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    for _ in 0..32 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

/// 应答策略：改判/天妒一律放弃（Pass）——覆盖 Decline 分支
///（改判跳过、判定牌随效果弃置、天妒放弃后 judge_held→弃置）。
fn answer_declines_judgement(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "gui_dao_rewrite" => vec![ChoiceOption::Pass],
        // Confirm 类选择的放弃 = 空应答（allow_cancel）
        "tian_du_gain" => vec![],
        _ => answer_by_prompt(spec),
    }
}

// ---------------------------------------------------------------------------
// N05 补充：多目标无懈 × 真实洗牌（DeckReshuffled）× 恢复组合
// ---------------------------------------------------------------------------

/// 多重集守恒（含机器持牌）：每张实体牌在牌区或机器中恰出现一次。
fn assert_multiset_conservation_named(e: &Engine, at: &str) {
    fn machine_cards(m: &MachineSnapshot, out: &mut Vec<u32>) {
        match m {
            MachineSnapshot::JudgePhase { queue, current, .. } => {
                out.extend(queue.iter().copied());
                if let Some(c) = current.as_ref() {
                    if !c.finalized {
                        out.extend(c.trick);
                    }
                    if c.judge_held {
                        out.push(c.judge_card);
                    }
                }
            }
            MachineSnapshot::OptionalGate { inner, .. }
            | MachineSnapshot::NullifyWindow { effect: inner, .. }
            | MachineSnapshot::Aoe { inner, .. } => {
                if let Some(m) = inner.as_ref() {
                    machine_cards(m, out);
                }
            }
            _ => {}
        }
    }
    let mut ids: Vec<u32> = Vec::new();
    let core = e.state_core();
    for p in &core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(core.deck.iter().copied());
    ids.extend(core.discard.iter().copied());
    ids.extend(core.table.iter().copied());
    for park in e.save().resolution.parks {
        machine_cards(&park.machine, &mut ids);
    }
    ids.sort_unstable();
    let total = ids.len();
    let dups: Vec<u32> = ids
        .windows(2)
        .filter(|w| w[0] == w[1])
        .map(|w| w[0])
        .collect();
    ids.dedup();
    assert_eq!(ids.len(), total, "{at}: 重复={dups:?}");
    assert_eq!(
        ids.len(),
        e.catalog().cards.len(),
        "{at}: 目录牌与牌区不守恒（缺失或多余）"
    );
}

/// 布阵：牌库清空 + 南蛮（1 号持无懈）+ 乐留 0 手牌；停在首个无懈窗口。
fn build_savage_nullify_reshuffle() -> Engine {
    let mut e = Engine::new(379, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 牌库清空：下一次翻判定牌触发真实洗回（DeckReshuffled）
    e.debug_drain_deck_to_discard();
    let savage = e.debug_place(0, "savage");
    let _n1 = e.debug_place(1, "nullify");
    let le = e.debug_place(0, "le");
    let _ = play(&mut e, savage, vec![]);
    let _ = le;
    e
}

/// 无懈窗口排干：1 号有牌即出（南蛮多目标窗口与反无懈窗），
/// 其余一律 Pass；每次应答后多重集守恒（含机器持牌）。
fn drive_nullify_windows(e: &mut Engine) {
    while let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "nullify_window", "此段只应有无懈窗口");
        let sel = if c.observer == 1 {
            c.spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .map(|o| vec![*o])
                .unwrap_or_else(|| vec![ChoiceOption::Pass])
        } else {
            vec![ChoiceOption::Pass]
        };
        let cid = c.id;
        let obs = c.observer;
        let out = submit_cmd(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        assert_multiset_conservation_named(e, "无懈应答后");
    }
}

/// 南蛮结算后对 1 用乐，推进至 1 的判定阶段：牌库空 → 真实洗回。
/// 返回终态哈希（判定完成后）。
fn continue_to_judge_reshuffle(e: &mut Engine) -> [u8; 32] {
    let le = e
        .debug_hand(0)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == CardKind::Le)
        .expect("乐应在 0 手牌");
    let out = play(e, le, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    drive_nullify_windows(e);
    for _ in 0..32 {
        if e.log().iter().any(|l| matches!(l, LE::DeckReshuffled)) {
            break;
        }
        if let Some(c) = e.open_choice() {
            let cid = c.id;
            let obs = c.observer;
            let out = submit_cmd(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            continue;
        }
        match next_phase(e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(
        e.log().iter().any(|l| matches!(l, LE::DeckReshuffled)),
        "翻判定牌应触发真实 DeckReshuffled"
    );
    assert_multiset_conservation_named(e, "洗牌后");
    e.state_hash()
}

fn build_judge_lightning_dying() -> Engine {
    let mut e = Engine::new(209, 4);
    e.debug_set_hp(1, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _sd = e.debug_place_judge(1, "shandian");
    let _tao = e.debug_place(2, "tao");
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

/// 应答策略：濒死救援用桃救回，其余沿用 answer_by_prompt。
fn answer_saves_dying(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    if spec.prompt == "dying_save" {
        if let Some(card) = spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
        {
            return vec![*card];
        }
    }
    answer_by_prompt(spec)
}

// ---------------------------------------------------------------------------
// N08：鬼道改判窗口挂起点往返
// ---------------------------------------------------------------------------

use engine_native::catalog::Suit;

/// 从目录挑一张黑色（黑桃/梅花）杀布进 `player` 手牌。
fn place_black_sha(e: &mut Engine, player: u8) -> u32 {
    for _ in 0..6 {
        let c = e.debug_place(player, "sha");
        if matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club) {
            return c;
        }
    }
    panic!("目录中应存在黑色杀");
}

fn build_guidao_rewrite() -> Engine {
    let mut e = Engine::new(207, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼道");
    let _black = place_black_sha(&mut e, 2);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    // 推进到 1 的判定阶段：鬼道改判询问（自拥有者 1 起，1 无鬼道 → 2）
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

// ---------------------------------------------------------------------------
// N08/N04 补充：鬼道装备区改判 + 嵌套检查点层级拒绝
// ---------------------------------------------------------------------------

use engine_native::LogEvent as LE;

/// 逐段推进直至鬼道改判询问打开；途中自动应答弃牌至上限等非改判选择。
/// 每步重压红桃布顶：跨回合摸牌会消耗布顶，判定牌需为红桃以固定基线。
fn advance_to_rewrite_ask(e: &mut Engine) {
    for _ in 0..48 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "gui_dao_rewrite" {
                return;
            }
            let sel: Vec<ChoiceOption> = if c.spec.prompt == "nullify_window" {
                vec![ChoiceOption::Pass]
            } else {
                c.spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(c.spec.min.max(1))
                    .collect()
            };
            let out = submit_cmd(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            continue;
        }
        e.debug_set_deck_top_by(|def| def.suit.is_heart());
        match next_phase(e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    panic!("48 步内未到达改判询问");
}

/// N08-c 布阵：2 号经**真实出牌动作**装备黑武器（PlayCard → Equipped →
/// 装备区），随后清空其手牌；3 号对 1 使用乐，判定落在下一轮 1 的判定阶段。
/// 返回武器牌 id。
fn setup_real_equip_rewrite() -> (Engine, u32) {
    let mut e = Engine::new(217, 4);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼道");
    // 回合 2：2 号真实出牌装备黑武器
    advance_to_play(&mut e, 2);
    let weapon = e.debug_place(2, "weapon2");
    assert!(matches!(
        e.catalog().cards[weapon as usize].suit,
        Suit::Spade | Suit::Club
    ));
    let out = play(&mut e, weapon, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_equips(2), vec![weapon], "武器应进入装备区");
    // 回合 3：3 号对 1 使用乐（下一轮 1 的判定阶段触发改判）
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(2); // 此后 2 号仅剩装备区黑牌
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(3, "le");
    let _ = play(&mut e, le, vec![1]);
    // 无懈窗口：摸牌可能带来无懈，一律 Pass 使乐落入判定区
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let cid = c.id;
                let obs = c.observer;
                let out = submit_cmd(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
                assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            }
            _ => break,
        }
    }
    (e, weapon)
}

fn judged_card_of_1(e: &Engine) -> u32 {
    e.log()
        .iter()
        .find_map(|l| match l {
            LE::Judged {
                player: 1, card, ..
            } => Some(*card),
            _ => None,
        })
        .expect("判定应已翻牌")
}

/// 牌区守恒（本文件局部版）：每张实体牌恰出现在一个牌区。
fn assert_equip_window_conservation(e: &Engine) {
    let core = e.state_core();
    let mut ids: Vec<u32> = Vec::new();
    for p in &core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(core.deck.iter().copied());
    ids.extend(core.discard.iter().copied());
    ids.extend(core.table.iter().copied());
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "实体牌重复出现在多个牌区");
    assert_eq!(
        ids.len(),
        e.catalog().cards.len(),
        "每张目录牌须恰在一个牌区"
    );
}

fn rewrite_card_options(e: &Engine) -> Vec<u32> {
    e.open_choice()
        .unwrap()
        .spec
        .options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(id) => Some(*id),
            _ => None,
        })
        .collect()
}

/// 构造真实开放段：时间片 1 → 提交后 Suspended（段开放、携带检查点）。
fn build_open_segment() -> (Engine, engine_native::StateSnapshot) {
    let mut e = Engine::new(7, 4);
    e.debug_set_budget(100);
    e.set_time_slice(Some(1));
    let cur = e.turn().current;
    let out = submit_cmd(&mut e, cur, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    assert!(e.is_suspended());
    let snap = e.save();
    (e, snap)
}

// ---------------------------------------------------------------------------
// N11 字节存档 v2：选择种类显式编码（此前由提示词静态推导并已漂移——
// nullify_window 实为 Confirm、jiji_target 实为 SelectPlayers、
// tian_du_gain 未注册；三类挂起点在 v1 下字节恢复均被拒绝）。
// ---------------------------------------------------------------------------

use engine_native::archive::{decode_archive, encode_archive, ArchiveError};
use engine_native::ChoiceKind;

/// 挂起点字节往返：encode→decode→restore 后种类/提示词/哈希必须逐一还原。
fn assert_choice_byte_roundtrip(
    build: fn() -> Engine,
    expected_prompt: &str,
    expected_kind: ChoiceKind,
) {
    let e = build();
    let choice = e.open_choice().expect("build 必须停在挂起点");
    assert_eq!(choice.spec.prompt, expected_prompt, "挂起提示词");
    assert_eq!(choice.spec.kind, expected_kind, "机器实际种类");
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode");
    let restored = Engine::restore(&decoded).expect("字节恢复必须接受合法挂起快照");
    let rc = restored.open_choice().expect("恢复件仍挂起");
    assert_eq!(rc.spec.prompt, expected_prompt, "恢复后提示词");
    assert_eq!(rc.spec.kind, expected_kind, "恢复后种类必须逐字节还原");
    assert_eq!(restored.state_hash(), e.state_hash(), "恢复件同哈希");
}

#[path = "boundary_snapshot/abort.rs"]
mod abort;
#[path = "boundary_snapshot/answer.rs"]
mod answer;
#[path = "boundary_snapshot/archive.rs"]
mod archive;
#[path = "boundary_snapshot/commands.rs"]
mod commands;
#[path = "boundary_snapshot/duplicate.rs"]
mod duplicate;
#[path = "boundary_snapshot/guidao.rs"]
mod guidao;
#[path = "boundary_snapshot/open.rs"]
mod open;
#[path = "boundary_snapshot/out.rs"]
mod out;
#[path = "boundary_snapshot/pause.rs"]
mod pause;
#[path = "boundary_snapshot/pending.rs"]
mod pending;
#[path = "boundary_snapshot/restore.rs"]
mod restore;
#[path = "boundary_snapshot/savage.rs"]
mod savage;
#[path = "boundary_snapshot/save.rs"]
mod save;
#[path = "boundary_snapshot/stale.rs"]
mod stale;
#[path = "boundary_snapshot/xiaoji.rs"]
mod xiaoji;
