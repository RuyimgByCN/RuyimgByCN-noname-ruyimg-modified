//! 原生内核验收测试：目标覆盖
//! 「嵌套选择、伤害转移、濒死救援、技能中途失效、动态技能、回滚、重放」。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::{
    catalog::CardKind, event::DamageNature, state::GameMode, ChoiceOption, CommandOutcome, Engine,
    GameAction, GameCommand, Phase, RejectReason, Role, Winner,
};

// ---------------------------------------------------------------------------
// 测试脚手架
// ---------------------------------------------------------------------------

/// 进程级去重键发生器：命令信任边界要求 dedup_id 严格递增。
static DEDUP: AtomicU64 = AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 以当前修订号 + 新去重键提交（测试便捷入口；边界测试直接构造 GameCommand）。
fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    e.submit(GameCommand {
        actor,
        expected_revision,
        dedup_id: fresh_dedup(),
        action,
    })
}

/// 当前回合玩家推进阶段。
fn next_phase(e: &mut Engine) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::NextPhase)
}

/// 回合推进：Answer 当前选择（取前 min 个牌选项——弃牌等场景）。
fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let turn = e.turn();
        if turn.current == player && turn.phase == Phase::Play {
            return;
        }
        assert!(e.outcome().is_none(), "match over during advance");
        match next_phase(e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => panic!("默认不限片，不应出现时间片暂停"),
            CommandOutcome::Pending { choice } => {
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(choice.spec.min.max(1))
                    .collect();
                assert!(!sel.is_empty(), "pending choice has no card options");
                let outcome = answer_sel(e, sel);
                assert!(
                    matches!(outcome, CommandOutcome::Accepted { .. }),
                    "{outcome:?}"
                );
            }
            CommandOutcome::Rejected(r) => panic!("advance rejected: {r:?}"),
        }
    }
    panic!("advance_to_play did not converge");
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

fn answer_empty(e: &mut Engine) -> CommandOutcome {
    answer_sel(e, vec![])
}

fn prompt_of(e: &Engine) -> &'static str {
    e.open_choice().expect("choice open").spec.prompt
}

fn open_choice_snapshot(e: &Engine) -> (u64, &'static str) {
    let c = e.open_choice().expect("choice open");
    (c.id, c.spec.prompt)
}

fn hp(e: &Engine, p: u8) -> i32 {
    e.debug_hp(p)
}

fn count_log<F: Fn(&engine_native::LogEvent) -> bool>(e: &Engine, f: F) -> usize {
    e.log().iter().filter(|l| f(l)).count()
}

use engine_native::LogEvent;

fn log_has<F: Fn(&LogEvent) -> bool>(e: &Engine, f: F) -> bool {
    e.log().iter().any(f)
}

/// 当前玩家出一张牌；需要牌在手。
fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::PlayCard { card, targets })
}

// ---------------------------------------------------------------------------
// Boss 郭嘉（群雄乱入，规则 v259）：鬼谋（混乱标记）+远策（伤害后判定）
// +奇佐（锦囊复用）。条目 [天妒,鬼谋,远策,奇佐] hp4（上游 group shen→群
// 目录近似）；鬼谋 mad=PlayerState.boss_mad（其回合收束解除）；远策=Damage
// commit 尾臂（chooseControl AI 态度面→恒选项一登记）；奇佐=CardUsed
// AfterTriggers 一步机（useCardAfter 前置近似登记；载体槽守卫递归）。
// ---------------------------------------------------------------------------

fn guojia_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 37)
        as u16 // BOSS_GENERALS[37] 郭嘉
}

// ---------------------------------------------------------------------------
// Boss 张春华（群雄乱入，规则 v260）：绝情（伤害转失去体力）+无心（延时
// 禁他人/受伤转失 1）+伤逝（四 commit 臂补至 4+子技 shangshix2 全体失 1）。
// 条目 [绝情,无心,伤逝] hp4（上游 group shen→群目录近似）。
// ---------------------------------------------------------------------------

fn zhangchunhua_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 38)
        as u16 // BOSS_GENERALS[38] 张春华
}

// ---------------------------------------------------------------------------
// Boss 刘备（群雄乱入，规则 v261）：枭雄（他人基本/锦囊入手副本+子技未用
// 牌 End 伤害）+章武（受伤后双方弃首张或来源受反伤）。条目 [枭雄,章武]
// hp8（上游 group shen→群目录近似）。
// ---------------------------------------------------------------------------

fn liubei_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 39)
        as u16 // BOSS_GENERALS[39] 刘备
}

// ---------------------------------------------------------------------------
// Boss 甄姬（群雄乱入，规则 v262）：凌波（闪→摸 2）+娇孝（红牌目标→摸 1）
// +踏水（黑牌→翻面首座）+芳华（翻面者失 1）。条目 [踏水,凌波,娇孝,芳华]
// hp4（上游 group shen→群目录近似）。
// ---------------------------------------------------------------------------

fn zhenji_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 40)
        as u16 // BOSS_GENERALS[40] 甄姬
}

// ---------------------------------------------------------------------------
// Boss 周瑜（群雄乱入，规则 v263）：火神（火伤免疫+回 1）+红莲·周瑜
//（End 摸 2+全场火伤；与毕方「红莲」同名按迅影批改名映射先例以
// 「红莲·周瑜」入目录）+仙音（非己回合失牌判定红→首个其他存活失 1）。
// 条目 [火神,红莲·周瑜,仙音] hp6（上游 group shen→群目录近似）。
// ---------------------------------------------------------------------------

fn zhouyu_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 41)
        as u16 // BOSS_GENERALS[41] 周瑜
}

// ---------------------------------------------------------------------------
// Boss 张角（群雄乱入，规则 v263）：天道（改判机扩展——hes 任意牌改判+
// 获得原判定牌）+雷击·张角（闪后首个其他存活判定黑→1 雷伤+摸 1；与青龙
// BOSS_RELEIJI 同名按迅影批改名映射先例）+极点（造成伤害后距受伤者 ≤1
// 首个其他存活判定黑→1 雷伤；链挡一层不自我递归=登记差异）。
// 条目 [雷击·张角,天道,极点] hp8（上游 group shen→群目录近似）。
// ---------------------------------------------------------------------------

fn zhangjiao_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 42)
        as u16 // BOSS_GENERALS[42] 张角
}

// ---------------------------------------------------------------------------
// 十殿阎罗五官王（规则 v265）：枳枉（他人获得牌→其弃首张手牌；CardsMoved
// 面，摸牌走 CardsDrawn 天然隔离）+公正（判定阶段开始弃己判定区首张，鬼
// 计同款）+血池（End 座位序首个其他存活失 2）。条目 [枳枉,公正,血池] hp4
//（上游 group qun→群目录直取）。
// ---------------------------------------------------------------------------

fn wuguanwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 43)
        as u16 // BOSS_GENERALS[43] 五官王
}

// ---------------------------------------------------------------------------
// 十殿阎罗宋帝王（规则 v266）：黑绳（死亡时全体其他横置——Death 前置窗
// 口遗言面）+胜负（End 首个有坐骑的其他存活弃其一）。恩怨=外部包组技未
// 落登记。条目 [黑绳,胜负] hp4（上游 group qun→群目录直取）。
// ---------------------------------------------------------------------------

fn songdiwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 44)
        as u16 // BOSS_GENERALS[44] 宋帝王
}

// ---------------------------------------------------------------------------
// 十殿阎罗卞城王（规则 v267）：雷舟（准备阶段随机一名其他 1 雷伤）+
// 雷斧（结束阶段随机一名其他横置态切换）+雷诛（死亡时座位序逐发 1 雷
// 伤——Death 遗言机，黑绳同位）。随机目标→座位序首个确定性近似登记。
// ---------------------------------------------------------------------------

fn bianchengwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 45)
        as u16 // BOSS_GENERALS[45] 卞城王
}

// ---------------------------------------------------------------------------
// 十殿阎罗泰山王（规则 v268）：服毒（他人用桃→随机另一其他失 1）+
// 苦酒（其他角色准备阶段失 1+视为用酒）+热恼（死亡遗言随机一名 3 火伤）。
// 随机→座位序首个确定性近似登记。显示名勘误：boss_fudu=服毒/boss_kujiu=
// 苦酒（勘验存量误记复读/苦久）。
// ---------------------------------------------------------------------------

fn taishanwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 46)
        as u16 // BOSS_GENERALS[46] 泰山王
}

// ---------------------------------------------------------------------------
// 十殿阎罗转轮王（规则 v269）：魔道（共用技，准备阶段摸 2）+轮回（准备
// 阶段 hp≤2 与体力最高且>2 的一名其他交换体力值）+往生（出牌阶段开始视
// 为随机南蛮/万箭→恒定南蛮近似）+反噬（每回合首次受伤置 terra 旗，同回
// 合再受伤→随机其他 1 伤；回合结束全清）。随机→确定性近似登记。
// ---------------------------------------------------------------------------

fn zhuanlunwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 47)
        as u16 // BOSS_GENERALS[47] 转轮王
}

// ---------------------------------------------------------------------------
// 十殿阎罗孟婆（规则 v270）：拾忧（其他角色弃牌阶段弃牌——获得花色互
// 异者）+往事（其他角色回合开始——本回合禁基本牌，回合结束全清）+忘魂
// （死亡遗言——清全场封禁+前两个其他存活各禁用首个启用技）。随机→确定
// 性近似登记。
// ---------------------------------------------------------------------------

fn mengpo_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 48)
        as u16 // BOSS_GENERALS[48] 孟婆
}

// ---------------------------------------------------------------------------
// 十殿阎罗阎罗王（规则 v271）：铁面（防具槽空视为仁王盾）+铡刀（杀穿
// 透防具，上游 inherit qinggang_skill）+诛心（死亡遗言——最低体力其他
// 存活受 2 伤）。确定性近似登记。
// ---------------------------------------------------------------------------

fn yanluowang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 49)
        as u16 // BOSS_GENERALS[49] 阎罗王
}

/// 从牌堆取一张黑杀置入 p 手牌（实体发现，测试布阵用）。
fn give_black_sha(e: &mut Engine, p: u8) -> u32 {
    let id = {
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|&c| {
                let d = &e.catalog().cards[c as usize];
                d.kind == CardKind::Sha
                    && matches!(
                        d.suit,
                        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                    )
            })
            .expect("牌堆中无黑杀")
    };
    let core = e.state_core_mut();
    core.deck.retain(|&x| x != id);
    core.player_mut(p).hand.push(id);
    id
}

/// 按目录名取牌并直接装备到 p 的装备区（上帝视角布阵；空槽前提）。
fn equip_card(e: &mut Engine, p: u8, name: &str) -> u32 {
    let id = e.debug_place(p, name);
    let core = e.state_core_mut();
    core.player_mut(p).hand.retain(|&x| x != id);
    core.player_mut(p).equips.push(id);
    id
}

// ---------------------------------------------------------------------------
// 十殿阎罗平等王（规则 v272）：锁足（准备阶段全部其他存活横置）+阿鼻
//（受有来源伤害后来源受 1 火伤，恒定火近似）+平等（死亡遗言——最高体
// 力两波火伤 2+1，座位序首个+恒定火近似）。
// ---------------------------------------------------------------------------

fn pingdengwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 50)
        as u16 // BOSS_GENERALS[50] 平等王
}

// ---------------------------------------------------------------------------
// 十殿阎罗都市王（规则 v273）：热闷（防具槽空——plain 杀目标不可指定+
// 南蛮/万箭窗口前跳过）+炙焚（准备阶段取座位序首个其他首张手牌+其受 1
// 火伤）+火刑（死亡遗言——全部其他存活逐座火伤）。确定性近似登记。
// ---------------------------------------------------------------------------

fn dushiwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 51)
        as u16 // BOSS_GENERALS[51] 都市王
}

// ---------------------------------------------------------------------------
// 十殿阎罗地藏王（规则 v274）：不佛（伤害>1 减 1+准备阶段距离 1 火伤）
// +无量（结束摸 2+准备体力<3 回复至 3；建局摸 3 面登记未落）+谛听（距
// 离双向 -1/+1）。大愿=判定改写机登记不落。
// ---------------------------------------------------------------------------

fn dizangwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 52)
        as u16 // BOSS_GENERALS[52] 地藏王
}

// ---------------------------------------------------------------------------
// 十殿阎罗楚江王（规则 v275）：帷幕（黑色锦囊无效——显式目标拒+黑色
// AOE 跳过）+反馈（标准 FAN_KUI 复用=界反馈语义）+冰封（死亡翻面来
// 源）。
// ---------------------------------------------------------------------------

fn chujiangwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 53)
        as u16 // BOSS_GENERALS[53] 楚江王
}

// ---------------------------------------------------------------------------
// 十殿阎罗秦广王（规则 v276）：判官（延时锦囊免目标）+拘魂（结束阶段
// 翻面首个其他）+望乡（死亡全场装备全弃）。还人=关卡流程面登记不落。
// ---------------------------------------------------------------------------

fn qinguangwang_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 54)
        as u16 // BOSS_GENERALS[54] 秦广王
}

// ---------------------------------------------------------------------------
// 群雄乱入 Boss 庞统·涅槃凤雏（规则 v277）：天狱（结束阶段自横置先解
// 除+全场未横置横置）+栖梧（梅花牌使用后受伤回复 1）。涅槃/浴火=外部
// 包技依赖登记不落；诸葛亮（hp Infinity+四技 offline 龙血玄黄包）勘验
// 登记不落。
// ---------------------------------------------------------------------------

fn pangtong_def_of(e: &engine_native::Engine) -> u16 {
    (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len() + 55)
        as u16 // BOSS_GENERALS[55] 庞统
}

// ---------------------------------------------------------------------------
// 重放确定性：同种子同命令序列 → 同哈希；不同种子 → 不同
// ---------------------------------------------------------------------------

/// 命令策略：确定性脚本（不依赖种子差异——应答取选项首项/固定目标）。
fn run_script(seed: u64) -> ([u8; 32], usize, i32) {
    let mut e = Engine::new(seed, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    e.debug_set_hp(0, 2);
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);

    // 确定性应答循环：直到本段 Accepted
    for _ in 0..32 {
        let Some(choice) = e.open_choice().cloned() else {
            break;
        };
        let sel = match choice.spec.prompt {
            "trigger_confirm" => vec![ChoiceOption::Confirm],
            "yishang_target" => vec![ChoiceOption::Player(2)],
            "sancai_target" => vec![ChoiceOption::Player(3)],
            _ => choice
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .map(|o| vec![o])
                .unwrap_or_default(),
        };
        let outcome = submit_cmd(
            &mut e,
            choice.observer,
            GameAction::Answer {
                choice_id: choice.id,
                selection: sel,
            },
        );
        assert!(
            matches!(
                outcome,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{outcome:?}"
        );
    }
    (e.state_hash(), e.debug_hand(0).len(), hp(&e, 0))
}

// ---------------------------------------------------------------------------
// 铁索连环与火/雷属性伤害（阶段 2 扩面第二项）
// ---------------------------------------------------------------------------

use engine_native::LogEvent as LE;

fn answer_cards(e: &mut Engine, choice: &engine_native::OpenChoice) {
    let sel: Vec<ChoiceOption> = choice
        .spec
        .options
        .iter()
        .filter(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .take(choice.spec.min.max(1))
        .collect();
    let _ = submit_cmd(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection: sel,
        },
    );
}

// ---------------------------------------------------------------------------
// 身份局（正式模式胜负与奖惩，ADR 0705）
// ---------------------------------------------------------------------------

/// 5 人身份局布阵（主忠反反内）并断言身份分布与主公 +1 体力上限。
fn identity_5p(seed: u64) -> Engine {
    let e = Engine::new_identity(seed, 5);
    assert_eq!(e.debug_hp(0), 5, "主公体力上限 +1（4+1）");
    let roles = e.roles().expect("identity roles");
    assert_eq!(roles[0], Role::Lord, "座位 0 固定为主公");
    assert_eq!(roles.iter().filter(|r| **r == Role::Loyalist).count(), 1);
    assert_eq!(roles.iter().filter(|r| **r == Role::Rebel).count(), 2);
    assert_eq!(roles.iter().filter(|r| **r == Role::Traitor).count(), 1);
    e
}

fn seats_with_role(e: &Engine, role: Role) -> Vec<u8> {
    e.roles()
        .expect("identity roles")
        .iter()
        .enumerate()
        .filter(|(_, r)| **r == role)
        .map(|(i, _)| i as u8)
        .collect()
}

/// 排空当前挂起询问（空选应答=拒绝可取消触发/Pass 类），直到无询问。
fn drain_choices(e: &mut Engine) {
    for _ in 0..16 {
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![],
                    },
                );
            }
            None => break,
        }
    }
}

/// NP 一次并妥善应答挂起询问（选牌类取 min 个牌选项；确认类取首项）。
fn next_phase_settled(e: &mut Engine) {
    if let CommandOutcome::Pending { .. } = next_phase(e) {
        let choice = e.open_choice().expect("choice open").clone();
        let mut sel: Vec<ChoiceOption> = choice
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(choice.spec.min.max(1))
            .collect();
        if sel.is_empty() && !choice.spec.options.is_empty() {
            sel.push(choice.spec.options[0]);
        }
        let _ = answer_sel(e, sel);
    }
}

/// 逐窗放行无懈（Pass 显式应答——min=1 空应答被拒）直至无挂起。
fn pass_nullify_windows(e: &mut Engine) {
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
}

/// 从牌库/弃牌堆取一张指定花色（黑/红）的杀入手（state 精确搬移——
/// debug_place 的手牌回收分支只查不删，会产生重影实体）。
fn place_sha_of_color(e: &mut engine_native::Engine, p: u8, black: bool) -> u32 {
    let suits: Vec<engine_native::catalog::Suit> =
        e.catalog().cards.iter().map(|d| d.suit).collect();
    let kinds: Vec<engine_native::catalog::CardKind> =
        e.catalog().cards.iter().map(|d| d.kind).collect();
    use engine_native::catalog::CardKind;
    let core = e.state_core_mut();
    for from_deck in [true, false] {
        let src: &mut Vec<u32> = if from_deck {
            &mut core.deck
        } else {
            &mut core.discard
        };
        for i in 0..src.len() {
            let id = src[i];
            if kinds[id as usize] != CardKind::Sha {
                continue;
            }
            let is_black = matches!(
                suits[id as usize],
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            );
            if is_black == black {
                let c = src.remove(i);
                core.player_mut(p).hand.push(c);
                return c;
            }
        }
    }
    panic!("no sha of desired color in deck/discard");
}

// ---------------------------------------------------------------------------
// N01–N03（并行评审复现项的工程回归）：牌区守恒 / 判定中死亡清队列 / 闪电转移查重
// ---------------------------------------------------------------------------

/// 牌区守恒：全部实体牌在所有牌区中恰出现一次（deck/discard/table/手牌/装备/判定区）。
fn assert_zone_conservation(e: &Engine) {
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
    ids.sort_unstable();
    let total = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), total, "存在同一实体牌出现在多个牌区");
    assert_eq!(
        ids.len(),
        e.catalog().cards.len(),
        "每张目录牌必须在牌区中恰出现一次"
    );
}

fn discard_count(e: &Engine, card: u32) -> usize {
    e.state_core()
        .discard
        .iter()
        .filter(|c| **c == card)
        .count()
}

// ---------------------------------------------------------------------------
// N07：宿主时间片——暂停续行与连续运行一致，硬预算语义不变
// ---------------------------------------------------------------------------

use engine_native::OpenChoice;

/// 散财链应答脚本：确认 → 目标 2 → 首张牌。
fn sancai_answer(choice: &OpenChoice) -> Vec<ChoiceOption> {
    match choice.spec.prompt {
        "trigger_confirm" => vec![ChoiceOption::Confirm],
        "sancai_target" => vec![ChoiceOption::Player(2)],
        _ => vec![choice
            .spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .expect("card option")],
    }
}

/// 以给定时间片跑完整散财链：出杀 → 确认 → 摸牌 → 目标 → 给牌。
/// 返回（引擎, 暂停次数）。
fn drive_sancai_sliced(slice: Option<u32>) -> (Engine, usize) {
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    // 时间片在布阵/推进完成后设置（advance 助手不支持暂停）
    e.set_time_slice(slice);

    let mut suspends = 0usize;
    let mut answered = 0usize;
    let mut out = play(&mut e, sha, vec![0]);
    loop {
        match out {
            CommandOutcome::Suspended => {
                suspends += 1;
                assert!(suspends < 500, "时间片续行未收敛");
                assert!(
                    e.open_choice().is_none(),
                    "Suspended 时不得有开放选择（Pending 优先）"
                );
                out = e.resume();
            }
            CommandOutcome::Pending { choice } => {
                answered += 1;
                assert!(answered <= 3, "散财链只应有 3 次应答");
                let sel = sancai_answer(&choice);
                let actor = choice.observer;
                let cid = choice.id;
                out = e.submit(GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                });
            }
            CommandOutcome::Accepted { .. } => break,
            CommandOutcome::Rejected(r) => panic!("sliced run rejected: {r:?}"),
        }
    }
    assert_eq!(answered, 3, "散财链应完整走完 3 个选择点");
    (e, suspends)
}

/// 构建到「暂停段预算耗尽卡住」状态：返回（引擎, 段启动键, 段前哈希, 段前修订号）。
fn build_stuck_suspended_segment(seed: u64, starter_dedup: u64) -> (Engine, [u8; 32], u64) {
    let mut e = Engine::new(seed, 4);
    e.debug_set_budget(2);
    let pre_hash = e.state_hash();
    let pre_rev = e.revision();
    e.set_time_slice(Some(1));
    let cur = e.turn().current;
    let out = e.submit(GameCommand {
        actor: cur,
        expected_revision: e.revision(),
        dedup_id: starter_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    let out = e.resume();
    assert!(matches!(
        out,
        CommandOutcome::Rejected(RejectReason::BudgetExhausted)
    ));
    assert!(e.is_suspended());
    (e, pre_hash, pre_rev)
}

// ---------------------------------------------------------------------------
// N08 首组官方技能：张角「鬼道」（改判）+ 郭嘉「天妒」（判定牌获得）
// 规则来源：标准包规则文本；改判流程对齐旧线 judgment_rewriters/judgment_black_rewrite_cards
// ---------------------------------------------------------------------------

use engine_native::catalog::Suit;

/// 从目录挑一张黑色（黑桃/梅花）杀并布进 `player` 手牌，返回其 id。
fn place_black_sha(e: &mut Engine, player: u8) -> u32 {
    for _ in 0..6 {
        let c = e.debug_place(player, "sha");
        if matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club) {
            return c;
        }
    }
    panic!("目录中应存在黑色杀");
}

/// 布阵到「1 号判定阶段等待鬼道改判询问」：乐在 1 判定区、判定牌为红桃
///（红桃 id 由测试经 Judged 日志取回）。
fn build_guidao_pending() -> (Engine, u32) {
    let mut e = Engine::new(207, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    (e, le)
}

fn judged_card_of(e: &Engine) -> u32 {
    e.log()
        .iter()
        .find_map(|l| match l {
            engine_native::LogEvent::Judged { card, .. } => Some(*card),
            _ => None,
        })
        .expect("判定应已翻牌")
}

// ---------------------------------------------------------------------------
// N05 扩展：洗牌期间牌区守恒（牌库耗尽 → 弃牌堆洗回，逐次断言恰一份）
// ---------------------------------------------------------------------------

/// 按提示给出合法确定性应答并排干链式选择，直至 Accepted/Rejected。
fn answer_and_drive(e: &mut Engine, mut out: CommandOutcome) {
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 5000, "drive 未收敛");
        out = match out {
            CommandOutcome::Suspended => e.resume(),
            CommandOutcome::Pending { choice } => {
                let sel = if choice.spec.prompt == "discard_to_limit" {
                    choice
                        .spec
                        .options
                        .iter()
                        .filter(|o| matches!(o, ChoiceOption::Card(_)))
                        .copied()
                        .take(choice.spec.min)
                        .collect()
                } else {
                    vec![choice.spec.options[0]]
                };
                let a = choice.observer;
                let cid = choice.id;
                e.submit(GameCommand {
                    actor: a,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                })
            }
            CommandOutcome::Accepted { .. } | CommandOutcome::Rejected(_) => return,
        };
    }
}

/// 逐命令断言牌区守恒：每张实体牌恰出现一次（无重复、无丢失）。
fn assert_zone_conservation_named(e: &Engine, at: &str) {
    let core = e.state_core();
    let mut ids: Vec<u32> = Vec::new();
    for pl in &core.players {
        ids.extend(pl.hand.iter().copied());
        ids.extend(pl.equips.iter().copied());
        ids.extend(pl.judge.iter().copied());
    }
    ids.extend(core.deck.iter().copied());
    ids.extend(core.discard.iter().copied());
    ids.extend(core.table.iter().copied());
    ids.sort_unstable();
    let _total = ids.len();
    let dups: Vec<u32> = ids
        .windows(2)
        .filter(|w| w[0] == w[1])
        .map(|w| w[0])
        .collect();
    ids.dedup();
    assert!(dups.is_empty(), "{at}: 重复实体牌 {dups:?}");
    assert_eq!(ids.len(), e.catalog().cards.len(), "{at}: 牌区不守恒");
}

// ---------------------------------------------------------------------------
// N08-d：判定事件次数/全序（ADR 0716：Judged → Rewritten×N → Finalized →
// 效果 → Gained；Finalized 恰好一次）。以事件序列断言替代仅 log_has。
// ---------------------------------------------------------------------------

fn log_pos<F: Fn(&LogEvent) -> bool>(e: &Engine, f: F) -> usize {
    e.log().iter().position(f).expect("事件应存在于日志")
}

fn log_count<F: Fn(&LogEvent) -> bool>(e: &Engine, f: F) -> usize {
    e.log().iter().filter(|l| f(l)).count()
}

// ---------------------------------------------------------------------------
// N06：距离/传导参照实现差分 + 奖惩先后时序断言（ADR 0723）
// ---------------------------------------------------------------------------

/// 参照模型：存活座位环上全对全比对距离（±马、下限 1）、攻击范围（武器射程）、
/// next_alive 与 seat_order_from（传导：顺时针跳过死座）。
fn assert_distance_range_reference(e: &Engine, at: &str) {
    let core = e.state_core();
    let seats = core.players.len() as u8;
    let alive: Vec<u8> = (0..seats)
        .filter(|i| core.players[*i as usize].alive)
        .collect();
    assert!(!alive.is_empty(), "{at}: 至少一名存活");
    let pos = |p: u8| alive.iter().position(|x| *x == p).expect("p alive") as i16;
    let n = alive.len() as i16;
    let kind_count = |p: u8, want: CardKind| -> u32 {
        core.players[p as usize]
            .equips
            .iter()
            .filter(|c| e.catalog().cards[**c as usize].kind == want)
            .count() as u32
    };
    for &a in &alive {
        for &b in &alive {
            // 座位距离（只计存活，顺/逆取小）± 守方+1马 / 攻方-1马，下限 1
            let cw = (pos(b) - pos(a)).rem_euclid(n) as u32;
            let ccw = (pos(a) - pos(b)).rem_euclid(n) as u32;
            let expect = cw
                .min(ccw)
                .saturating_add(kind_count(b, CardKind::HorsePlus))
                .saturating_sub(kind_count(a, CardKind::HorseMinus))
                .max(1);
            let got = core.computed_distance(e.catalog(), a, b);
            assert_eq!(got, expect, "{at}: {a}->{b} 距离");
        }
        // 攻击范围：武器射程最大，无武器 1
        let want = core.players[a as usize]
            .equips
            .iter()
            .filter(|c| e.catalog().cards[**c as usize].kind == CardKind::Weapon)
            .map(|c| e.catalog().cards[*c as usize].range)
            .max()
            .unwrap_or(1);
        assert_eq!(
            core.attack_range(e.catalog(), a),
            want,
            "{at}: {a} 攻击范围"
        );
        // 传导参照：顺时针下一位存活；座位队列自 a 起顺时针
        let idx = pos(a) as usize;
        let next = alive[(idx + 1) % alive.len()];
        assert_eq!(core.next_alive(a), next, "{at}: {a} 的下一位存活");
        let want_order: Vec<u8> = alive
            .iter()
            .cycle()
            .skip(idx)
            .take(alive.len())
            .copied()
            .collect();
        assert_eq!(
            core.seat_order_from(a),
            want_order,
            "{at}: 自 {a} 的座位队列"
        );
    }
}

/// 至多前 4 名存活玩家各真实装备一匹马（目录仅 4 张同名马；
/// PlayCard → Equipped）。混合装备分布同样覆盖参照差分。
fn equip_horse_for_all(e: &mut Engine, seats: u8, name: &str) {
    // v64 数量对齐后马牌仅 ±2~4 张：直接快照注入装备区（按牌堆余量equip，
    // 差分参照逐状态重算，不要求全座装满）。
    let kind = match name {
        "horse_plus" => engine_native::CardKind::HorsePlus,
        _ => engine_native::CardKind::HorseMinus,
    };
    let mut snap = e.save();
    for p in 0..seats {
        if !e.debug_alive(p) {
            continue;
        }
        let free = {
            let cat = e.catalog();
            let in_zone = |c: u32| -> bool {
                snap.core.deck.contains(&c)
                    || snap.core.discard.contains(&c)
                    || snap.core.table.contains(&c)
                    || snap.core.players.iter().any(|pl| {
                        pl.hand.contains(&c) || pl.equips.contains(&c) || pl.judge.contains(&c)
                    })
            };
            (0..cat.cards.len() as u32)
                .find(|c| cat.cards[*c as usize].kind == kind && !in_zone(*c))
        };
        let Some(card) = free else { break };
        snap.core.deck.retain(|c| *c != card);
        snap.core.players[p as usize].equips.push(card);
    }
    *e = Engine::restore(&snap).expect("马牌注入恢复");
}

// ---------------------------------------------------------------------------
// N06：确定性脚本重放/恢复逐命令事件对照（ADR 0727）
// ---------------------------------------------------------------------------

/// 脚本策略：开放选择按策略应答；出牌阶段出杀（仅当本回合未出杀且
/// 距离可达，避免拒绝死循环）；否则推进阶段。
fn next_script_command(e: &mut Engine) -> GameCommand {
    if let Some(c) = e.open_choice() {
        let sel = match c.spec.prompt {
            "nullify_window" => vec![ChoiceOption::Pass],
            "dying_save" | "gui_dao_rewrite" => c
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .map(|o| vec![*o])
                .unwrap_or_else(|| vec![c.spec.options[0]]),
            "tian_du_gain" => vec![ChoiceOption::Confirm],
            "discard_to_limit" => c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect(),
            _ => vec![c.spec.options[0]],
        };
        return GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        };
    }
    let t = e.turn();
    if t.phase == Phase::Play && !t.sha_used_play_phase {
        let actor = t.current;
        let sha = e
            .debug_hand(actor)
            .iter()
            .copied()
            .find(|c| e.catalog().card_kind(*c) == CardKind::Sha);
        if let Some(sha) = sha {
            let core = e.state_core();
            let target = core.next_alive(actor);
            let dist = core.computed_distance(e.catalog(), actor, target);
            if dist <= core.attack_range(e.catalog(), actor) {
                return GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::PlayCard {
                        card: sha,
                        targets: vec![target],
                    },
                };
            }
        }
    }
    let actor = e.turn().current;
    GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    }
}

/// 非 Prototype 模式的逐命令重放/恢复对照（T11-EVENT-RNG-RESTORE-PARITY）。
/// 同一确定性脚本在基准线/重放线逐命令比对状态哈希与全量日志；恢复线在
/// **每一个命令点** save→restore 后跑完剩余脚本，终态哈希须与基准一致。
/// `canonical_hash` 含 `rng.state()` 与牌堆，故哈希一致即随机流消费序一致。
fn non_prototype_script_parity(build: impl Fn() -> Engine, label: &str) {
    let mut base = build();
    let mut script: Vec<GameCommand> = Vec::new();
    let mut hashes: Vec<[u8; 32]> = vec![base.state_hash()];
    let mut logs: Vec<Vec<LogEvent>> = vec![Vec::new()];
    while script.len() < 80 && base.outcome().is_none() {
        let cmd = next_script_command(&mut base);
        let _ = base.submit(cmd.clone());
        script.push(cmd);
        hashes.push(base.state_hash());
        logs.push(base.log().to_vec());
    }
    assert!(script.len() >= 20, "{label} 脚本应有足够命令量");
    let terminal = *hashes.last().unwrap();

    // 重放线：同脚本从零重跑，逐命令比对哈希与全量日志。
    let mut rep = build();
    for (i, cmd) in script.iter().enumerate() {
        let _ = rep.submit(cmd.clone());
        assert_eq!(
            rep.state_hash(),
            hashes[i + 1],
            "{label} 重放第 {i} 条命令后哈希不一致"
        );
        assert_eq!(
            rep.log(),
            &logs[i + 1][..],
            "{label} 重放第 {i} 条命令后日志不一致"
        );
    }

    // 恢复线：每一个命令点恢复后跑完剩余脚本，终态须一致。
    for k in 1..script.len() {
        let mut half = build();
        for cmd in &script[..k] {
            let _ = half.submit(cmd.clone());
        }
        assert_eq!(half.state_hash(), hashes[k], "{label} 第 {k} 点基线不一致");
        let snap = half.save();
        let mut r = Engine::restore(&snap).expect("restore");
        assert_eq!(r.state_hash(), hashes[k], "{label} 第 {k} 点恢复态不一致");
        for cmd in &script[k..] {
            let _ = r.submit(cmd.clone());
        }
        assert_eq!(
            r.state_hash(),
            terminal,
            "{label} 自第 {k} 点恢复后终态与基准不一致"
        );
    }
}

// ---------------------------------------------------------------------------
// N08 第五十九批：夏侯惇（刚烈）/ 张辽（突袭）/ 许褚（裸衣）（ADR 0758）
// ---------------------------------------------------------------------------

/// 走到 `player` 的下一回合判定阶段（测试内小型推进循环：
/// 触发确认类询问按需应答，摸/弃阶段不产生卡牌询问的布阵下收敛）。
fn advance_to_judge(e: &mut Engine, player: u8) {
    for _ in 0..128 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Judge {
            return;
        }
        assert!(e.outcome().is_none(), "match over during advance");
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
                assert!(!sel.is_empty(), "unexpected choice: {}", choice.spec.prompt);
                answer_sel(e, sel);
            }
            CommandOutcome::Rejected(r) => panic!("advance rejected: {r:?}"),
            CommandOutcome::Suspended => panic!("suspended during advance"),
        }
    }
    panic!("advance_to_judge did not converge");
}

// ---------------------------------------------------------------------------
// N08 第六十一批：刘备（仁德/激将）+ 马超（马术/铁骑）（ADR 0760）
// ---------------------------------------------------------------------------

fn skill_def_id(e: &Engine, name: &str) -> engine_native::ids::SkillDefId {
    (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == name)
        .unwrap_or_else(|| panic!("目录应有 {name}"))
}

fn e_dummy() -> Engine {
    Engine::new(1, 2)
}

// ---------------------------------------------------------------------------
// P1/N08 六技能批（ADR 0770）：急救/护驾/流离/结姻/反间/离间
// ---------------------------------------------------------------------------

fn skill_id(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == name)
        .expect("skill in catalog")
}

// ---------------------------------------------------------------------------
// P2/N05、N06 复合状态验收扩展（ADR 0771）
// ---------------------------------------------------------------------------

/// 逐提交不变式：每次提交后快照可恢复（含牌归属多重集/头校验/关联
/// 校验全通道）且恢复态哈希与当前一致。
fn assert_per_submission_invariants(e: &Engine) {
    let snap = e.save();
    if let Err(err) = Engine::restore(&snap) {
        panic!(
            "逐提交快照必须可恢复: {err:?}; stacks={} parks={} open={:?}",
            snap.resolution.stack.len(),
            snap.resolution.parks.len(),
            snap.resolution.open_choice.as_ref().map(|c| c.spec.prompt)
        );
    }
    let restored = Engine::restore(&snap).expect("逐提交快照必须可恢复");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "恢复态哈希必须与当前一致"
    );
}

/// 固定策略驱动身份局到真实终局（纯命令，无 debug 干预）：
/// 返回（终局引擎，逐提交状态哈希迹，逐提交事件流摘要迹）。
fn drive_identity_fixed(seed: u64, max_commands: usize) -> (Engine, Vec<u64>, Vec<u64>) {
    let mut e = Engine::try_new_identity(seed, 5).expect("identity 5p");
    let mut hashes: Vec<u64> = Vec::new();
    let mut event_digests: Vec<u64> = Vec::new();
    for _ in 0..max_commands {
        assert_per_submission_invariants(&e);
        let seat = match e.open_choice() {
            Some(c) => c.observer,
            None => e.turn().current,
        };
        // 策略：询问取 min 选项；本回合有杀攻下一位存活；否则推进
        let session = e.ai_session(seat);
        let cmd = if let Some(engine_native::view::ChoiceView::Own {
            id, options, min, ..
        }) = session.choice()
        {
            let sel: Vec<ChoiceOption> = options.into_iter().take(min.max(1)).collect();
            GameCommand {
                actor: seat,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::Answer {
                    choice_id: id,
                    selection: sel,
                },
            }
        } else if e.turn().current == seat && e.turn().phase == Phase::Play {
            let view = session.view();
            let sha = match view.players[seat as usize].hand.clone() {
                engine_native::view::HandView::Own(hand) => hand.into_iter().find(|c| {
                    e.catalog().cards[*c as usize].kind == engine_native::catalog::CardKind::Sha
                }),
                _ => None,
            };
            match sha {
                Some(sha) => {
                    let n = view.players.len();
                    let target = (1..n as u8)
                        .map(|d| (seat + d) % n as u8)
                        .find(|s| view.players[*s as usize].alive);
                    match target {
                        Some(target) => GameCommand {
                            actor: seat,
                            expected_revision: e.revision(),
                            dedup_id: fresh_dedup(),
                            action: GameAction::PlayCard {
                                card: sha,
                                targets: vec![target],
                            },
                        },
                        None => GameCommand {
                            actor: seat,
                            expected_revision: e.revision(),
                            dedup_id: fresh_dedup(),
                            action: GameAction::NextPhase,
                        },
                    }
                }
                None => GameCommand {
                    actor: seat,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                },
            }
        } else {
            GameCommand {
                actor: seat,
                expected_revision: e.revision(),
                dedup_id: fresh_dedup(),
                action: GameAction::NextPhase,
            }
        };
        let out = e.submit(cmd);
        match out {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            CommandOutcome::Rejected(_) => {
                let s2 = e.turn().current;
                let _ = e.submit(GameCommand {
                    actor: s2,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::NextPhase,
                });
            }
            CommandOutcome::Suspended => {}
        }
        hashes.push(u64::from_le_bytes(
            e.state_hash()[0..8].try_into().expect("32B hash"),
        ));
        let log = e.log();
        // 完整事件流摘要：长度 + 逐条 Debug 哈希折叠
        let mut d: u64 = log.len() as u64;
        for entry in log {
            let bytes = format!("{entry:?}");
            for b in bytes.bytes() {
                d = d.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (b as u64);
            }
        }
        event_digests.push(d);
        if e.outcome().is_some() {
            break;
        }
    }
    (e, hashes, event_digests)
}

// ---------------------------------------------------------------------------
// P2/N10（ADR 0772）：策略类型隔离 + 有界任务调度 + AI 基线
// ---------------------------------------------------------------------------

/// 只依赖 PolicyInput 的最小策略（类型上无权威引擎入口）。
struct PassivePolicy;
impl engine_native::view::SeatPolicy for PassivePolicy {
    fn act(&mut self, _input: &engine_native::view::PolicyInput) -> Option<GameAction> {
        None // 无动作 → 驱动器回退推进阶段
    }
}

/// 测试辅助：从快照读目录牌定义（耀武红杀注入用）。
fn snap_deck_card(
    _snap: &engine_native::StateSnapshot,
    card: u32,
) -> engine_native::catalog::CardDef {
    // StateSnapshot 不携带目录；测试引擎的目录经全局标准目录读取。
    let e = Engine::new(1, 2);
    e.catalog().cards[card as usize].clone()
}

// ---------------------------------------------------------------------------
// D4/B1 青木隐藏链（tag 411–414）
// ---------------------------------------------------------------------------

fn boss_general_named(e: &Engine, name: &str) -> u16 {
    let base = (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len())
        as u16;
    (base..e.catalog().general_count() as u16)
        .find(|&d| e.catalog().general_meta(d).map(|g| g.0) == Some(name))
        .unwrap_or_else(|| panic!("boss general {name}"))
}

// ---------------------------------------------------------------------------
// D4 后援 / 还人 / 青赤白翼（规则 v295；tag 427–437）
// 从 new_boss_stage 起步，不预写 stage 结果。
// ---------------------------------------------------------------------------

fn general_named_first(e: &Engine, name: &str) -> u16 {
    (0..e.catalog().general_count() as u16)
        .find(|&d| e.catalog().general_meta(d).map(|g| g.0) == Some(name))
        .unwrap_or_else(|| panic!("general {name}"))
}

fn general_slot_name(e: &Engine, seat: u8) -> Option<&str> {
    e.state_core().players[seat as usize].general_slots[0]
        .and_then(|id| e.catalog().general_meta(id).map(|g| g.0))
}

fn living_named(e: &Engine, name: &str) -> usize {
    (0..e.state_core().players.len() as u8)
        .filter(|&s| e.debug_alive(s) && general_slot_name(e, s) == Some(name))
        .count()
}

fn has_named_skill(e: &Engine, seat: u8, name: &str) -> bool {
    e.catalog()
        .skill_by_name(name)
        .is_some_and(|def| e.state_core().skills.contains_key(&(seat, def)))
}

/// 排空挂起。还人选将必须点选项，不能空应答。
fn settle_boss_choices(e: &mut Engine) {
    for _ in 0..32 {
        let Some(choice) = e.open_choice().cloned() else {
            return;
        };
        let sel = if choice.spec.prompt == "huanren_pick" {
            assert!(!choice.spec.options.is_empty(), "还人候选为空");
            vec![choice.spec.options[0]]
        } else {
            let cards: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .copied()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .take(choice.spec.min.max(1))
                .collect();
            if !cards.is_empty() {
                cards
            } else if choice
                .spec
                .options
                .iter()
                .any(|o| matches!(o, ChoiceOption::Pass))
            {
                vec![ChoiceOption::Pass]
            } else if choice.spec.min == 0 {
                vec![]
            } else if !choice.spec.options.is_empty() {
                vec![choice.spec.options[0]]
            } else {
                vec![]
            }
        };
        let outcome = answer_sel(e, sel);
        assert!(
            !matches!(outcome, CommandOutcome::Rejected(_)),
            "settle rejected prompt={} {outcome:?}",
            choice.spec.prompt
        );
    }
    panic!("choices did not settle");
}

fn finish_cmd(e: &mut Engine, outcome: CommandOutcome) {
    match outcome {
        CommandOutcome::Accepted { .. } => {}
        CommandOutcome::Pending { .. } => settle_boss_choices(e),
        other => panic!("command {other:?}"),
    }
    assert!(e.open_choice().is_none(), "choice still open");
}

fn kill_boss(e: &mut Engine) {
    let n = e.state_core().players.len() as u8;
    for s in 0..n {
        e.debug_clear_hand(s);
    }
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 12, DamageNature::Normal);
    finish_cmd(e, out);
}

/// 结束末座回合，使轮次加一并在 Boss 座位打开判定阶段。
fn end_turn_into_round(e: &mut Engine, round: u32) {
    let last = (e.state_core().players.len() - 1) as u8;
    e.debug_set_turn(last, Phase::End, round - 1);
    let out = next_phase(e);
    finish_cmd(e, out);
    assert_eq!(e.turn().round, round, "round wrap");
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Judge);
}

fn bianshen_outcome(seed: u64, from: &str) -> String {
    let mut e = Engine::new_boss_stage(seed, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, from));
    assert!(
        has_named_skill(
            &e,
            0,
            if from == "牛头" || from == "马面" {
                "后援3"
            } else {
                "后援4"
            }
        ),
        "{from} 应装配后援"
    );
    kill_boss(&mut e);
    assert!(e.debug_alive(0), "转身后 Boss 座位应存活");
    let name = general_slot_name(&e, 0).unwrap_or("").to_string();
    assert_ne!(name, from);
    assert_zone_conservation(&e);
    name
}

fn boot_huanren(seed: u64) -> Engine {
    let mut e = Engine::new_boss_stage(seed, 4, 0, 0, 4, 4, 0);
    e.debug_set_general_def(0, boss_general_named(&e, "秦广王"));
    e.debug_set_general_def(1, general_named_first(&e, "陆逊"));
    assert!(has_named_skill(&e, 0, "还人"));
    let out = next_phase(&mut e);
    finish_cmd(&mut e, out);
    e
}

#[path = "native_kernel/activate.rs"]
mod activate;
#[path = "native_kernel/area_attacks.rs"]
mod area_attacks;
#[path = "native_kernel/boss_baihu.rs"]
mod boss_baihu;
#[path = "native_kernel/boss_bianchengwang.rs"]
mod boss_bianchengwang;
#[path = "native_kernel/boss_bifang.rs"]
mod boss_bifang;
#[path = "native_kernel/boss_chujiangwang.rs"]
mod boss_chujiangwang;
#[path = "native_kernel/boss_dizangwang.rs"]
mod boss_dizangwang;
#[path = "native_kernel/boss_dongzhuo.rs"]
mod boss_dongzhuo;
#[path = "native_kernel/boss_dushiwang.rs"]
mod boss_dushiwang;
#[path = "native_kernel/boss_futai2.rs"]
mod boss_futai2;
#[path = "native_kernel/boss_guojia.rs"]
mod boss_guojia;
#[path = "native_kernel/boss_heiwuchang.rs"]
mod boss_heiwuchang;
#[path = "native_kernel/boss_hundun.rs"]
mod boss_hundun;
#[path = "native_kernel/boss_huoshenzhurong.rs"]
mod boss_huoshenzhurong;
#[path = "native_kernel/boss_jinshenrushou.rs"]
mod boss_jinshenrushou;
#[path = "native_kernel/boss_liubei.rs"]
mod boss_liubei;
#[path = "native_kernel/boss_luocha.rs"]
mod boss_luocha;
#[path = "native_kernel/boss_mamian.rs"]
mod boss_mamian;
#[path = "native_kernel/boss_mengpo.rs"]
mod boss_mengpo;
#[path = "native_kernel/boss_mingxingzhu.rs"]
mod boss_mingxingzhu;
#[path = "native_kernel/boss_mode.rs"]
mod boss_mode;
#[path = "native_kernel/boss_mushengoumang.rs"]
mod boss_mushengoumang;
#[path = "native_kernel/boss_nianshou.rs"]
mod boss_nianshou;
#[path = "native_kernel/boss_pangtong.rs"]
mod boss_pangtong;
#[path = "native_kernel/boss_pingdengwang.rs"]
mod boss_pingdengwang;
#[path = "native_kernel/boss_qinglong.rs"]
mod boss_qinglong;
#[path = "native_kernel/boss_qinguangwang.rs"]
mod boss_qinguangwang;
#[path = "native_kernel/boss_qiongqi.rs"]
mod boss_qiongqi;
#[path = "native_kernel/boss_shuishengonggong.rs"]
mod boss_shuishengonggong;
#[path = "native_kernel/boss_shuishenxuanming.rs"]
mod boss_shuishenxuanming;
#[path = "native_kernel/boss_shujing.rs"]
mod boss_shujing;
#[path = "native_kernel/boss_songdiwang.rs"]
mod boss_songdiwang;
#[path = "native_kernel/boss_stage.rs"]
mod boss_stage;
#[path = "native_kernel/boss_taihao.rs"]
mod boss_taihao;
#[path = "native_kernel/boss_taishanwang.rs"]
mod boss_taishanwang;
#[path = "native_kernel/boss_taotie.rs"]
mod boss_taotie;
#[path = "native_kernel/boss_taowu.rs"]
mod boss_taowu;
#[path = "native_kernel/boss_wuguanwang.rs"]
mod boss_wuguanwang;
#[path = "native_kernel/boss_xiangliu.rs"]
mod boss_xiangliu;
#[path = "native_kernel/boss_xuanwu.rs"]
mod boss_xuanwu;
#[path = "native_kernel/boss_yandi.rs"]
mod boss_yandi;
#[path = "native_kernel/boss_yanling.rs"]
mod boss_yanling;
#[path = "native_kernel/boss_yanluowang.rs"]
mod boss_yanluowang;
#[path = "native_kernel/boss_yecha.rs"]
mod boss_yecha;
#[path = "native_kernel/boss_yingzhao.rs"]
mod boss_yingzhao;
#[path = "native_kernel/boss_yinjian.rs"]
mod boss_yinjian;
#[path = "native_kernel/boss_zhangchunhua.rs"]
mod boss_zhangchunhua;
#[path = "native_kernel/boss_zhangjiao.rs"]
mod boss_zhangjiao;
#[path = "native_kernel/boss_zhenji.rs"]
mod boss_zhenji;
#[path = "native_kernel/boss_zhouyu.rs"]
mod boss_zhouyu;
#[path = "native_kernel/boss_zhuanlunwang.rs"]
mod boss_zhuanlunwang;
#[path = "native_kernel/boss_zhuanxu.rs"]
mod boss_zhuanxu;
#[path = "native_kernel/boss_zhuque.rs"]
mod boss_zhuque;
#[path = "native_kernel/boss_zhuyan.rs"]
mod boss_zhuyan;
#[path = "native_kernel/boss_zhuyin.rs"]
mod boss_zhuyin;
#[path = "native_kernel/caocao.rs"]
mod caocao;
#[path = "native_kernel/catalog.rs"]
mod catalog;
#[path = "native_kernel/chain.rs"]
mod chain;
#[path = "native_kernel/cixiong.rs"]
mod cixiong;
#[path = "native_kernel/combat.rs"]
mod combat;
#[path = "native_kernel/d4.rs"]
mod d4;
#[path = "native_kernel/daqiao.rs"]
mod daqiao;
#[path = "native_kernel/dead.rs"]
mod dead;
#[path = "native_kernel/death.rs"]
mod death;
#[path = "native_kernel/diaochan.rs"]
mod diaochan;
#[path = "native_kernel/discard.rs"]
mod discard;
#[path = "native_kernel/distance.rs"]
mod distance;
#[path = "native_kernel/doudizhu.rs"]
mod doudizhu;
#[path = "native_kernel/duel.rs"]
mod duel;
#[path = "native_kernel/dying.rs"]
mod dying;
#[path = "native_kernel/e01.rs"]
mod e01;
#[path = "native_kernel/e02.rs"]
mod e02;
#[path = "native_kernel/e03.rs"]
mod e03;
#[path = "native_kernel/e04.rs"]
mod e04;
#[path = "native_kernel/e05.rs"]
mod e05;
#[path = "native_kernel/e06.rs"]
mod e06;
#[path = "native_kernel/equipment.rs"]
mod equipment;
#[path = "native_kernel/fangtian.rs"]
mod fangtian;
#[path = "native_kernel/ganfuren.rs"]
mod ganfuren;
#[path = "native_kernel/ganning.rs"]
mod ganning;
#[path = "native_kernel/general.rs"]
mod general;
#[path = "native_kernel/gongsunzan.rs"]
mod gongsunzan;
#[path = "native_kernel/guanshi.rs"]
mod guanshi;
#[path = "native_kernel/guanyu.rs"]
mod guanyu;
#[path = "native_kernel/guidao.rs"]
mod guidao;
#[path = "native_kernel/guojia.rs"]
mod guojia;
#[path = "native_kernel/guozhan.rs"]
mod guozhan;
#[path = "native_kernel/hanbing.rs"]
mod hanbing;
#[path = "native_kernel/host.rs"]
mod host;
#[path = "native_kernel/huanggai.rs"]
mod huanggai;
#[path = "native_kernel/huangyueying.rs"]
mod huangyueying;
#[path = "native_kernel/huatuo.rs"]
mod huatuo;
#[path = "native_kernel/hujia.rs"]
mod hujia;
#[path = "native_kernel/identity.rs"]
mod identity;
#[path = "native_kernel/jijiang.rs"]
mod jijiang;
#[path = "native_kernel/jiu.rs"]
mod jiu;
#[path = "native_kernel/judgement.rs"]
mod judgement;
#[path = "native_kernel/kof.rs"]
mod kof;
#[path = "native_kernel/last.rs"]
mod last;
#[path = "native_kernel/leisha.rs"]
mod leisha;
#[path = "native_kernel/lidian.rs"]
mod lidian;
#[path = "native_kernel/liubei.rs"]
mod liubei;
#[path = "native_kernel/luoshen.rs"]
mod luoshen;
#[path = "native_kernel/luxun.rs"]
mod luxun;
#[path = "native_kernel/lvbu.rs"]
mod lvbu;
#[path = "native_kernel/lvmeng.rs"]
mod lvmeng;
#[path = "native_kernel/machao.rs"]
mod machao;
#[path = "native_kernel/n04f.rs"]
mod n04f;
#[path = "native_kernel/n08.rs"]
mod n08;
#[path = "native_kernel/n10.rs"]
mod n10;
#[path = "native_kernel/n10a.rs"]
mod n10a;
#[path = "native_kernel/n11.rs"]
mod n11;
#[path = "native_kernel/n11a.rs"]
mod n11a;
#[path = "native_kernel/n11b.rs"]
mod n11b;
#[path = "native_kernel/negative.rs"]
mod negative;
#[path = "native_kernel/no.rs"]
mod no;
#[path = "native_kernel/nullify.rs"]
mod nullify;
#[path = "native_kernel/p2.rs"]
mod p2;
#[path = "native_kernel/presentation.rs"]
mod presentation;
#[path = "native_kernel/probe.rs"]
mod probe;
#[path = "native_kernel/qilin.rs"]
mod qilin;
#[path = "native_kernel/qinggang.rs"]
mod qinggang;
#[path = "native_kernel/qinglong.rs"]
mod qinglong;
#[path = "native_kernel/rejections.rs"]
mod rejections;
#[path = "native_kernel/rende.rs"]
mod rende;
#[path = "native_kernel/render.rs"]
mod render;
#[path = "native_kernel/renwang.rs"]
mod renwang;
#[path = "native_kernel/replay.rs"]
mod replay;
#[path = "native_kernel/reshuffle.rs"]
mod reshuffle;
#[path = "native_kernel/rollback.rs"]
mod rollback;
#[path = "native_kernel/sancai.rs"]
mod sancai;
#[path = "native_kernel/seat.rs"]
mod seat;
#[path = "native_kernel/shandian.rs"]
mod shandian;
#[path = "native_kernel/shouyi.rs"]
mod shouyi;
#[path = "native_kernel/simayi.rs"]
mod simayi;
#[path = "native_kernel/skill.rs"]
mod skill;
#[path = "native_kernel/snapshot.rs"]
mod snapshot;
#[path = "native_kernel/snatch.rs"]
mod snatch;
#[path = "native_kernel/standard.rs"]
mod standard;
#[path = "native_kernel/suidun.rs"]
mod suidun;
#[path = "native_kernel/sun.rs"]
mod sun;
#[path = "native_kernel/sunquan.rs"]
mod sunquan;
#[path = "native_kernel/suqing.rs"]
mod suqing;
#[path = "native_kernel/suspension.rs"]
mod suspension;
#[path = "native_kernel/tiandu.rs"]
mod tiandu;
#[path = "native_kernel/versus.rs"]
mod versus;
#[path = "native_kernel/weapon.rs"]
mod weapon;
#[path = "native_kernel/wugu.rs"]
mod wugu;
#[path = "native_kernel/wuzhong.rs"]
mod wuzhong;
#[path = "native_kernel/xiahoudun.rs"]
mod xiahoudun;
#[path = "native_kernel/xuchu.rs"]
mod xuchu;
#[path = "native_kernel/yiji.rs"]
mod yiji;
#[path = "native_kernel/yishang.rs"]
mod yishang;
#[path = "native_kernel/yuanshu.rs"]
mod yuanshu;
#[path = "native_kernel/zhangba.rs"]
mod zhangba;
#[path = "native_kernel/zhangfei.rs"]
mod zhangfei;
#[path = "native_kernel/zhangliao.rs"]
mod zhangliao;
#[path = "native_kernel/zhaoyun.rs"]
mod zhaoyun;
#[path = "native_kernel/zhiheng.rs"]
mod zhiheng;
#[path = "native_kernel/zhouyu.rs"]
mod zhouyu;
#[path = "native_kernel/zhu.rs"]
mod zhu;
#[path = "native_kernel/zhuge.rs"]
mod zhuge;
#[path = "native_kernel/zhugeliang.rs"]
mod zhugeliang;
