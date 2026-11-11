//! NC08 第⑦层（上游行为对照）样本 6：`_shengke`（五行生克，L06 wuxing）。
//!
//! 上游源码（`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-172`）：
//!
//! ```js
//! _shengke:{
//!     trigger:{target:'useCardToBegin'},
//!     forced:true,
//!     popup:false,
//!     filter:function(event,player){
//!         if(_status.connectMode) return false;
//!         return event.card.wunature&&player.wunature;
//!     },
//!     content:function(){
//!         switch(trigger.card.wunature){
//!             case 'metal':
//!             switch(player.wunature){
//!                 case 'wood':
//!                     if(player.countCards('he')){
//!                     game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
//!                     player.chooseToDiscard('你被金属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('金克木')}return;
//!                 case 'water':
//!                     game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
//!                     player.draw();player.popup('金生水');
//!                     return;
//!             }
//!             return;
//!             case 'wood':
//!             switch(player.wunature){
//!                 case 'soil':
//!                     if(player.countCards('he')){
//!                     game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
//!                     player.chooseToDiscard('你被木属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('木克土')}return;
//!                 case 'fire':
//!                     game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
//!                     player.draw();player.popup('木生火');
//!                     return;
//!             }
//!             return;
//!             case 'water':
//!             switch(player.wunature){
//!                 case 'fire':
//!                     if(player.countCards('he')){
//!                     game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
//!                     player.chooseToDiscard('你被水属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('水克火')}return;
//!                 case 'wood':
//!                     game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
//!                     player.draw();player.popup('水生木');
//!                     return;
//!             }
//!             return;
//!             case 'fire':
//!             switch(player.wunature){
//!                 case 'metal':
//!                     if(player.countCards('he')){
//!                     game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
//!                     player.chooseToDiscard('你被火属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('火克金')}return;
//!                 case 'soil':
//!                     game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
//!                     player.draw();player.popup('火生土');
//!                     return;
//!             }
//!             return;
//!             case 'soil':
//!             switch(player.wunature){
//!                 case 'water':
//!                     if(player.countCards('he')){
//!                     game.log(player,'被'+get.translation(trigger.card.wunature)+'属性的卡牌克制');
//!                     player.chooseToDiscard('你被土属性卡牌克制，需弃置一张牌',true,'he').ai=get.disvalue;player.popup('土克水')}return;
//!                 case 'metal':
//!                     game.log(player,'得到'+get.translation(trigger.card.wunature)+'属性卡牌的加成');
//!                     player.draw();player.popup('土生金');
//!                     return;
//!             }
//!             return;
//!         }
//!     },
//!     ai:{ effect:{ target:function(card,player,target,current){ /* 策略面，略 */ } } }
//! }
//! ```
//!
//! 角色/卡牌属性初始化（同文件 `element.player.init` / `element.card.init`）：
//! 每名角色开局随机一个属性（`list.randomGet()`）；每张牌以配置概率
//! `lib.config.wuxing_num_playpackconfig`（未配置时 `||0` → 默认 **0**）随机获得属性。
//!
//! 由该 JS 推导的规则点（本文件逐条验收）：
//! 1. **全局技**（`lib.skill._shengke`，上游 `game/index.js:8859-8861`：`_` 前缀技能
//!    自动 `addGlobalSkill`）→ 全体角色都持有，无需装备/武将；
//! 2. **锁定**（`forced: true`）→ 无「是否发动」确认门；
//! 3. **双方皆有属性**才触发（`event.card.wunature && player.wunature`）——
//!    无属性牌或无属性角色不触发；
//! 4. **被克**（牌属性克角色属性）：金克木、木克土、水克火、火克金、土克水 →
//!    角色须弃一张牌（`countCards('he')` 为真时强制弃 1 张，`he` = 手牌+装备）；
//! 5. **被生**（牌属性生角色属性）：金生水、木生火、水生木、火生土、土生金 →
//!    角色摸一张；无生克关系 → 无事；
//! 6. **时机**：`target: 'useCardToBegin'`——成为牌的目标、牌**生效前**结算。
//!
//! 原生实现引用：`content/host/filtering/common.rs:539-543`（damage→CardUsed 目标面
//! 过滤）、`content/host/step/skills_status.rs:680-706`（生克表与效果）、
//! `content/host/constants.rs:4-8`（`KE = [1,4,3,0,2]` / `SHENG = [2,3,1,4,0]`，
//! 下标 0..4 = 金/木/水/火/土）、`content/host/metadata/common.rs:619-625`（锁定、
//! 时机 `EventKind::CardUsed`）、`engine/construction.rs:437-470`（开局给全体座位
//! 授予生克技能 + 随机属性）、`engine/play_card.rs`（`CardUsed` 帧先于效果机器入栈）。
//!
//! 已登记差异（详见证据 README，逐条以本文件断言固定）：
//! - 差异 S1：弃牌从「角色自选 `he`」简化为「确定性弃首张手牌」——目标
//!   仅有装备时不弃牌（上游会强制弃装备）；
//! - 差异 S2：属性牌默认比例——上游默认 `wuxing_num_playpackconfig` 未配置 → 0
//!   张（文档意图 1/3），原生按包数据 `nature_fraction` 1/3 直接落地；
//! - 差异 S3：上游 `_` 前缀全局技 vs 原生「每座位一条技能实例」（构造期授予），
//!   语义等价但接口不同；
//! - 差异 S4：上游 `target: 'useCardToBegin'`（目标触发）vs 原生
//!   `EventKind::CardUsed` 帧；`CardUsed` 帧先于效果机器入栈，故**结算顺序对齐**。
//!
//! 不在规则对照范围（表现/策略面）：`popup`、`ai.effect`、`game.log` 文本、
//! 联机守卫（`_status.connectMode`，原生单机引擎无对应模式）。

use engine_native::content::extension::ExtensionPack;
use engine_native::ids::CardId;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn wuxing_engine(seed: u64) -> (Engine, ExtensionPack) {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("wuxing-upstream-liuli/assets/cards.json"))
        .expect("读五行包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    (e, pack)
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

/// 应答全部开放选择（返回出现过的 prompt，供「锁定无确认门」断言）。
fn drain_collect(e: &mut Engine) -> Vec<String> {
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        prompts.push(c.spec.prompt.to_string());
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let out = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "应答被拒: {out:?}"
        );
    }
    prompts
}

fn drain(e: &mut Engine) {
    let _ = drain_collect(e);
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let out = submit(e, actor, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "推进被拒: {out:?}"
        );
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 布阵：清空双方手牌 → 1 号持杀 → 注入 0 号属性与杀牌属性。
fn setup(
    e: Engine,
    pack: &ExtensionPack,
    player_nature: Option<u8>,
    card_nature: Option<u8>,
) -> (Engine, CardId, CardId, CardId) {
    let mut e = e;
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let first = e.debug_place(0, "tao");
    let second = e.debug_place(0, "jiu");
    let sha = e.debug_place(1, "sha");
    let mut snap = e.save();
    snap.core.players[0].wunature = player_nature;
    match card_nature {
        Some(n) => {
            snap.core.card_natures.insert(sha, n);
        }
        None => {
            snap.core.card_natures.remove(&sha);
        }
    }
    let e = Engine::restore_with_extensions(&snap, &[&pack]).expect("属性注入恢复");
    (e, sha, first, second)
}

fn play_sha_at_zero(e: &mut Engine, sha: CardId) -> Vec<String> {
    let out = submit(e, 1, GameAction::PlayCard {
        card: sha,
        targets: vec![0],
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "1 号出杀: {out:?}"
    );
    drain_collect(e)
}

/// 规则点 1+6（对齐）：生克是 `_` 前缀**全局技**——四名角色开局即持有；
/// 属性在开局由引擎 RNG 初始化（角色全有、牌堆按 1/3 分布）。
#[test]
fn upstream_shengke_is_global_and_natures_initialized() {
    let (e, _pack) = wuxing_engine(20262801);
    for p in 0..4 {
        assert!(
            e.debug_has_skill(p, "wuxing_shengke"),
            "{p} 号应持有生克（上游 _ 前缀全局技）"
        );
    }
    let snap = e.save();
    assert!(
        snap.core.players.iter().all(|p| p.wunature.is_some()),
        "开局每名角色一个属性"
    );
    assert!(!snap.core.card_natures.is_empty(), "牌堆属性牌由开局钩子分配");
    // 差异 S2：原生默认按包数据 1/3 落地（上游默认配置 0 → 0 张）
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let cfg = std::fs::read_to_string(root.join("wuxing-upstream-liuli/assets/wuxing.json"))
        .expect("读五行配置");
    assert!(
        cfg.contains("\"denominator\": 3"),
        "原生默认比例来自包数据：{cfg}"
    );
}

/// 规则点 4（对齐）：**被克**——牌属性克角色属性（金克木）→ 角色强制弃 1 张牌；
/// 锁定技，无「是否发动」确认门。
#[test]
fn upstream_shengke_ke_forces_discard() {
    let (e, pack) = wuxing_engine(20262802);
    let mut e = e;
    advance_to_play(&mut e, 1);
    let (mut e, sha, first, second) = setup(e, &pack, Some(1), Some(0)); // 木 / 金
    let before = e.debug_discard_len();
    let prompts = play_sha_at_zero(&mut e, sha);
    assert!(
        !prompts.iter().any(|p| p == "trigger_confirm"),
        "锁定技不得挂确认门：{prompts:?}"
    );
    assert_eq!(e.debug_hp(0), 3, "杀命中 1 点");
    assert!(
        !e.debug_hand(0).contains(&first),
        "金克木：被克角色弃一张牌"
    );
    assert!(
        e.debug_hand(0).contains(&second),
        "差异 S1：原生固定弃**首张**手牌（上游 chooseToDiscard(true,'he') 自选）"
    );
    assert_eq!(
        e.debug_discard_len(),
        before + 2,
        "弃牌堆 = 杀 + 被克弃牌"
    );
}

/// 规则点 5（对齐）：**被生**——牌属性生角色属性（金生水）→ 角色摸一张。
#[test]
fn upstream_shengke_sheng_draws() {
    let (e, pack) = wuxing_engine(20262803);
    let mut e = e;
    advance_to_play(&mut e, 1);
    let (mut e, sha, first, second) = setup(e, &pack, Some(2), Some(0)); // 水 / 金
    let hand_before = e.debug_hand(0).len();
    play_sha_at_zero(&mut e, sha);
    assert_eq!(e.debug_hp(0), 3, "杀仍命中 1 点");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "金生水：被生摸一张（含原两张）"
    );
    assert!(
        e.debug_hand(0).contains(&first) && e.debug_hand(0).contains(&second),
        "被生不弃牌"
    );
}

/// 规则点 3+5（对齐）：无生克关系 / 无属性牌 / 无属性角色 → 均无事。
#[test]
fn upstream_shengke_requires_both_natures_and_relation() {
    // ① 无关系：土(4) 对 金(0) —— 金克木、金生水，与土无关
    let (e, pack) = wuxing_engine(20262804);
    let mut e = e;
    advance_to_play(&mut e, 1);
    let (mut e, sha, first, second) = setup(e, &pack, Some(4), Some(0));
    let before = e.debug_discard_len();
    play_sha_at_zero(&mut e, sha);
    assert!(
        e.debug_hand(0).contains(&first) && e.debug_hand(0).contains(&second),
        "无生克关系不弃牌"
    );
    assert_eq!(e.debug_hand(0).len(), 2, "无生克关系不摸牌");
    assert_eq!(e.debug_discard_len(), before + 1, "弃牌堆只有杀本身");

    // ② 无属性牌：角色有属性、牌无属性 → 不触发
    let (e, pack) = wuxing_engine(20262805);
    let mut e = e;
    advance_to_play(&mut e, 1);
    let (mut e, sha, first, second) = setup(e, &pack, Some(1), None);
    play_sha_at_zero(&mut e, sha);
    assert!(
        e.debug_hand(0).contains(&first) && e.debug_hand(0).contains(&second),
        "无属性牌不触发（上游 filter 要求 event.card.wunature）"
    );

    // ③ 无属性角色：牌有属性、角色无属性 → 不触发
    let (e, pack) = wuxing_engine(20262806);
    let mut e = e;
    advance_to_play(&mut e, 1);
    let (mut e, sha, first, second) = setup(e, &pack, None, Some(0));
    play_sha_at_zero(&mut e, sha);
    assert!(
        e.debug_hand(0).contains(&first) && e.debug_hand(0).contains(&second),
        "无属性角色不触发（上游 filter 要求 player.wunature）"
    );
}

/// 上游对齐（`wuxing/extension.js:75-125`）：弃牌范围为 `he`（手牌+装备）且由角色自选
/// （`chooseToDiscard(true, 'he')`），仅持装备的被克角色同样会被强制弃置装备。
#[test]
fn upstream_shengke_he_range_difference() {
    let (e, pack) = wuxing_engine(20262807);
    let mut e = e;
    advance_to_play(&mut e, 1);
    let (mut e, sha, _first, _second) = setup(e, &pack, Some(1), Some(0));
    // 0 号清空手牌，只留一张装备
    e.debug_clear_hand(0);
    let equip = {
        let cat = e.catalog();
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|c| cat.card_kind(*c).equip_slot().is_some())
            .expect("牌堆有装备")
    };
    {
        let core = e.state_core_mut();
        core.deck.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    let before = e.debug_discard_len();
    let prompts = play_sha_at_zero(&mut e, sha);
    assert!(
        prompts.iter().any(|p| p == "shengke_discard"),
        "仅持装备时也应打开 shengke_discard 选择窗: {prompts:?}"
    );
    assert!(
        !e.debug_equips(0).contains(&equip),
        "上游对齐：仅持装备的被克角色同样被强制弃置装备（'he'）"
    );
    assert_eq!(e.debug_discard_len(), before + 2, "弃牌堆 = 杀 + 被克弃装备");
}
