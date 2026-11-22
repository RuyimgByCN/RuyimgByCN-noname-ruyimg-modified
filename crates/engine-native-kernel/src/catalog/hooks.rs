//! 内容层目录回调钩子定义（自 `catalog.rs` 按职责拆分）：内核经
//! `Catalog::hooks` 调用内容层，不直接引用 `crate::content`。

use super::{CardKind, Catalog};

/// 内容卡牌的固有结算：`None` = 不是内容卡牌（内核继续分派）；`Some(m)` = 已处理。
pub type CardIntrinsicHook = fn(
    &mut crate::machine::Ctx<'_>,
    &crate::event::CardUseData,
    CardKind,
) -> Option<Option<Box<dyn super::SkillMachine>>>;

/// 宿主机器挂起询问的派生（行为/阶段、owner/target、targets、行为专用 card、cards）。
pub type HostAskHook = fn(
    u16,
    u8,
    crate::ids::PlayerId,
    Option<crate::ids::PlayerId>,
    &[crate::ids::PlayerId],
    Option<u32>,
    &[crate::ids::CardId],
    &crate::state::GameStateCore,
    &Catalog,
) -> Option<crate::snap::ExpectedAsk>;

/// 内容机器的快照重建：`None` = 不是内容机器（内核继续分派）。
pub type RestoreMachineHook =
    fn(
        &crate::snap::MachineSnapshot,
        &Catalog,
    ) -> Option<Result<Box<dyn super::SkillMachine>, crate::snap::SnapshotError>>;

/// 内容层回调：内核在帧固有流程、询问派生与机器恢复处经此调用内容层，
/// 不直接引用 `crate::content`（拆 crate 第 2 步）。内容目录构建时装入
/// 内容实现；`Catalog::build` 默认 [`ContentHooks::NONE`]（不认识任何内容机器）。
#[derive(Clone, Copy)]
pub struct ContentHooks {
    pub card_intrinsic: CardIntrinsicHook,
    pub vm_active: fn(
        &rule_vm::BytecodeModule,
        &str,
        crate::ids::PlayerId,
        &crate::state::GameStateCore,
    ) -> bool,
    pub vm_validate:
        fn(&rule_api::EffectRequest, &[u32], &crate::state::GameStateCore, usize) -> bool,
    pub vm_commit: fn(
        &mut crate::machine::Ctx<'_>,
        &str,
        &rule_api::EffectRequest,
        &mut Vec<u32>,
    ) -> Option<crate::machine::Flow>,
    pub vm_ask: fn(
        &rule_api::EffectRequest,
        &[u32],
        &crate::state::GameStateCore,
        &Catalog,
    ) -> Option<crate::snap::ExpectedAsk>,
    pub host_ask: HostAskHook,
    pub restore_machine: RestoreMachineHook,
    pub borrow_options: fn(
        &crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        bool,
    ) -> Vec<crate::choice::ChoiceOption>,
    pub borrowed_owner: fn(
        &crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        crate::ids::CardId,
    ) -> Option<crate::ids::PlayerId>,
    /// R05-F（§6 四角色生命周期）：按角色名（limit/combs/current）取目录
    /// 中全部 R05 helper 定义号（装载器按 `#r05:{role}` 模板派生）。内核
    /// 到期/死亡/上限回调经此识别语义槽；未装（标准目录）返回空。
    pub r05_role_defs: fn(&Catalog, &'static str) -> Vec<crate::ids::SkillDefId>,
    /// 出牌阶段开始时钩子（phaseUseBegin 对应宿主时点，例如 `th_tianshu`）。
    pub play_begin_step: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId),
    /// 出牌阶段结束时钩子（phaseUseEnd 对应宿主时点，压于下一阶段帧之后先于弃牌执行）。
    pub play_end_step: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId),
    /// 外层事件结算完毕回调（对应 upstream `.after` 钩子，例如 `th_taipingyaoshu`）。
    pub on_outer_settled: fn(&mut crate::machine::Ctx<'_>),
    /// 他人卡牌进入弃牌堆回调（例如 `th_chengye` `useCardAfter`/`loseAfter`/`phaseJudge`）。
    pub on_cards_to_discard: fn(
        &mut crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        &[crate::ids::CardId],
        bool,
    ),
    /// 技能被移除/过期回调（例如 `th_chengye`/`th_zhengjing2`/`th_tiansuan2` `onremove`）。
    pub on_skill_removed: fn(
        &mut crate::state::GameStateCore,
        &Catalog,
        crate::ids::PlayerId,
        &str,
    ) -> Vec<crate::ids::CardId>,
    /// 杀目标结算前触发窗钩子（优先序 `priority >= 6` 先于仁王盾与八卦，
    /// `priority < 6` 仅次仁王盾且先于八卦/闪询问；返回 true 表示已挂入
    /// `HostMachine`，杀机 Yield 等待）。
    pub sha_begin_step: fn(
        &mut crate::machine::Ctx<'_>,
        crate::ids::EventId,
        crate::ids::PlayerId,
        crate::ids::PlayerId,
        Option<crate::ids::CardId>,
        bool,
    ) -> bool,
    /// 杀单目标结算出口钩子（`is_damage=true` 为所预测杀自身 `shaDamage`，
    /// `false` 为经防具/闪避/预防后正常 `shaEnd`；不涵盖 `shaBegin` 取消）。
    pub sha_end_step: fn(
        &mut crate::machine::Ctx<'_>,
        crate::ids::EventId,
        crate::ids::PlayerId,
        crate::ids::PlayerId,
        Option<crate::ids::CardId>,
        bool,
    ) -> bool,
    /// 摸牌阶段 `DP::Begin`（非跳过且 `!num_fixed`）摸牌数修改钩子。
    pub draw_begin_step: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId, &mut i32),
    /// 用牌结算完成弹出时（`useCardAfter`）宿主触发钩子（如 `jieyong`）。
    pub on_card_use_after: fn(&mut crate::machine::Ctx<'_>, &crate::event::CardUseData),
    /// 建局开局发牌后（`gameDrawEnd`）宿主初始化钩子（如 `xiefang`）。
    pub on_game_start: fn(&mut crate::state::GameStateCore, &Catalog),
    /// 角色死亡结算（`alive=false` 后、`dieEnd`）宿主刷新钩子（如 `xiefang`）。
    pub on_player_died: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId),
    /// 出牌阶段结束（`phaseUseEnd`）宿主临时技清理钩子（如 `new_zhixi`/`new_meibu_range`/`bolan`）。
    pub on_phase_use_end: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId),
    /// 轮次开始（`roundStart`）宿主轮标、展示清理与 `_relifeskill` 倒数复活钩子。
    pub on_round_start: fn(&mut crate::machine::Ctx<'_>),
    /// 手牌移除钩子（如 `xingchong` 在使用/打出/装备/弃置离手时立即摸 2 而不依赖额外事件表）。
    pub on_hand_card_removed:
        fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId, crate::ids::CardId),
    /// 响应窗（杀/决斗/AOE/濒死/无懈）查询玩家是否可发动【蛊惑】响应。
    pub reguhuo_response_skill: fn(
        &crate::state::GameStateCore,
        &Catalog,
        crate::ids::PlayerId,
        u8,
    ) -> Option<crate::ids::SkillDefId>,
    /// 响应窗（杀/决斗/AOE/濒死/无懈）查询玩家是否可发动【安弱】响应。
    pub scsanruo_response_skill: fn(
        &crate::state::GameStateCore,
        &Catalog,
        crate::ids::PlayerId,
        u8,
    ) -> Option<crate::ids::SkillDefId>,
    /// 响应窗内挂入宿主转化响应机器（【蛊惑】/【安弱】，NO_FRAME Trigger 挂起槽）。
    pub reguhuo_spawn_response: fn(
        &mut crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        u8,
        crate::ids::PlayerId,
        crate::ids::SkillDefId,
        crate::ids::PlayerId,
    ),
    /// 响应窗恢复后消费【蛊惑】真/未质疑产出的卡牌实体；取消或被识破作废返回 `None`。
    pub reguhuo_consume_resp:
        fn(&mut crate::state::GameStateCore, crate::ids::PlayerId) -> Option<crate::ids::CardId>,
    /// 响应牌生效后是否仍存在待询问的 `reguhuo_final`（真质疑者弃手牌或失体力）。
    pub reguhuo_has_pending_final: fn(&crate::state::GameStateCore, crate::ids::PlayerId) -> bool,
    /// 在指定挂起深度挂入 `reguhuo_final` 阶段机器，逐一处理质疑者后效。
    pub reguhuo_spawn_final: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId, usize),
    /// 回合结束（`Phase::End { begin: false }`）宿主清理/归还回调（如 `scschiyan_get`）。
    pub on_phase_end: fn(&mut crate::machine::Ctx<'_>),
    /// 回复前置（`recoverBefore`）宿主数值修正钩子（如 `L19 _wujinmoshiRecover`）。
    pub on_recover_before: fn(&mut crate::machine::Ctx<'_>, crate::ids::PlayerId, &mut u32),
    /// 死亡结算是否保留死者技能表（如存在存活 `Role::Lord` 持有 `L19 _relifeskill`）。
    pub preserves_skills_on_death: fn(&crate::state::GameStateCore, &Catalog, crate::ids::PlayerId) -> bool,
    /// 响应/打出牌窗口中可用的技能转牌/出牌选项（`is_use`: true=`chooseToUse`, false=`chooseToRespond`）。
    pub respond_skill_options: fn(
        &crate::state::GameStateCore,
        &Catalog,
        crate::ids::PlayerId,
        CardKind,
        bool,
    ) -> Vec<crate::choice::ChoiceOption>,
    /// 响应/打出窗口中选择 `ChoiceOption::Skill(def)` 时触发（返回 `Some(true)` 已立即完成响应、`Some(false)` 已挂入子机等待完成、`None` 未命中）。
    pub try_activate_respond_skill: fn(
        &mut crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        crate::ids::SkillDefId,
        CardKind,
        crate::ids::EventId,
    ) -> Option<bool>,
}

impl ContentHooks {
    pub const NONE: ContentHooks = ContentHooks {
        card_intrinsic: |_, _, _| None,
        vm_commit: |_, _, _, _| None,
        vm_validate: |_, state, _, _| state.is_empty(),
        vm_active: |_, _, _, _| true,
        vm_ask: |_, _, _, _| None,
        host_ask: |_, _, _, _, _, _, _, _, _| None,
        restore_machine: |_, _| None,
        borrow_options: |_, _, _| Vec::new(),
        borrowed_owner: |_, _, _| None,
        r05_role_defs: |_, _| Vec::new(),
        play_begin_step: |_, _| {},
        play_end_step: |_, _| {},
        on_outer_settled: |_| {},
        on_cards_to_discard: |_, _, _, _| {},
        on_skill_removed: |_, _, _, _| Vec::new(),
        sha_begin_step: |_, _, _, _, _, _| false,
        sha_end_step: |_, _, _, _, _, _| false,
        draw_begin_step: |_, _, _| {},
        on_card_use_after: |_, _| {},
        on_game_start: |_, _| {},
        on_player_died: |_, _| {},
        on_phase_use_end: |_, _| {},
        on_round_start: |_| {},
        on_hand_card_removed: |_, _, _| {},
        reguhuo_response_skill: |_, _, _, _| None,
        scsanruo_response_skill: |_, _, _, _| None,
        reguhuo_spawn_response: |_, _, _, _, _, _| {},
        reguhuo_consume_resp: |_, _| None,
        reguhuo_has_pending_final: |_, _| false,
        reguhuo_spawn_final: |_, _, _| {},
        on_phase_end: |_| {},
        on_recover_before: |_, _, _| {},
        preserves_skills_on_death: |_, _, _| false,
        respond_skill_options: |_, _, _, _, _| Vec::new(),
        try_activate_respond_skill: |_, _, _, _, _| None,
    };
}
