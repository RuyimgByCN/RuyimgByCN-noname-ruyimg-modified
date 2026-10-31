//! 宿主机器的询问派生（自 `snap/host_asks.rs` 迁入，拆 crate 第 2 步）：
//! `snap::asks` 经 `Catalog::hooks.host_ask` 回调。

use super::*;
use crate::catalog::Catalog;
use crate::snap::*;
use crate::state::GameStateCore;

use super::asks_mtg::active_ask_mtg;
use super::asks_wudian::active_ask_wudian;

pub(super) fn xuanying_target_spec(core: &GameStateCore, owner: PlayerId) -> ChoiceSpec {
    ChoiceSpec {
        kind: ChoiceKind::SelectPlayers,
        prompt: "xuanying_target",
        options: core.alive_players().filter(|p| *p != owner).map(ChoiceOption::Player).collect(),
        min: 1,
        max: 1,
        allow_cancel: true,
    }
}

/// 宿主机器的询问派生：按行为 + 阶段还原 host.rs 各 step 中
/// 创建该 `WaitChoice` 时的 `ChoiceSpec`（挂起时 stage 已前移到下一阶段）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn active_ask_host(
    behavior: u16,
    stage: u8,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    use crate::content::host::{
        D3_JINGGOU_TARGET, D3_MOYAN_GAIN, D3_MOYAN_TARGET, GJ_BINGPO_SHOW_OWN, GJ_BINGPO_SHOW_TARGET, GJ_DINVXUANSHUANG_RIDER, GJ_FEIBIAO_DISCARD, GJ_JINLIANZHU_CONFIRM, GJ_LONGXU_PICK, GJ_LUYUGENG_COST, GJ_SHUJINSAN_DISCARD, GJ_XIAJIAO_DISCARD, GW_BAOXUE_DISCARD, HOST_TARGET_DISCARD_EQUIP, WD_CAOCHUAN_BRANCH, WD_FUDI_PLAYER_PICK, WD_FUDI_TARGET_PICK, WD_SANJIAN_DISCARD, WD_SANJIAN_TARGET, WD_TOULIANG_OWN, WD_TOULIANG_TARGET, WUDIAN_XJ_DIEWU_CARDS, WUDIAN_XJ_DIEWU_TARGET, WUDIAN_XJ_GUIYUAN_CARDS, WUDIAN_XJ_LINGYU_TARGET, WUXING_SELECT_CARD, WUXING_SELECT_NATURE, YC_GEAN_T1_PICK, YC_GEAN_T2_PICK,
    };
    let behavior = HostBehavior::from_tag(behavior)?; // behavior: u16
    let stage = HostStage::from_tag(stage)?;
    match behavior {
        HostBehavior::SwdXuanying => {
            if stage != HostStage::TargetChoice { return None; }
            let spec = xuanying_target_spec(core, owner);
            if spec.options.is_empty() { return None; }
            Some(ask(owner, spec.prompt, spec.kind, spec.options, spec.min, spec.max, spec.allow_cancel))
        }
        HostBehavior::Liaoyuan => {
            if stage != HostStage::SelectCard {
                return None;
            }
            // Host.card is the suit scalar; Host.cards is already-discarded history.
            let suit = card.filter(|suit| *suit <= 3)?;
            let player = core.player(owner);
            let options = player.hand.iter().chain(player.equips.iter()).copied()
                .filter(|id| catalog.cards[*id as usize].suit as u32 == suit)
                .map(ChoiceOption::Card).collect();
            Some(ask(owner, "liaoyuan_discard", ChoiceKind::SelectCards, options, 1, 1, true))
        }
        // 五行盘：改属性手牌 → 新属性
        HostBehavior::ChangeCardNature => match stage {
            HostStage::SelectNature => Some(ask(
                owner,
                WUXING_SELECT_CARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                true,
            )),
            HostStage::Act => Some(ask(
                owner,
                WUXING_SELECT_NATURE,
                ChoiceKind::PickTag,
                (0..=4).map(ChoiceOption::Tag).collect(),
                1,
                1,
                true,
            )),
            _ => None,
        },
        // 手里剑：目标弃装备或受伤
        HostBehavior::TargetDiscardEquipOrDamage => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(t)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                t,
                HOST_TARGET_DISCARD_EQUIP,
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        // 潜偷（swd 第三批 v92）：代价牌选择询问（owner）
        HostBehavior::SwdShentouEquipSkill => {
            if stage != HostStage::Act {
                return None;
            }
            let nh = core.player(owner).hand.len();
            if nh == 0 {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::SWD_SHENTOU_COST,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                false,
            ))
        }
        // 御夫录（swd 第十二批 v96）：武器选择（挂起时 stage 已前移到
        // SelectCard）→ 目标选择（stage=TargetChoice）
        HostBehavior::SwdYufuluEquipSkill => match stage {
            HostStage::SelectCard => {
                let mut options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        catalog.card_slot_of(core, *c) == Some(crate::catalog::EquipSlot::Weapon)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    core.player(owner)
                        .equips
                        .iter()
                        .copied()
                        .filter(|c| {
                            catalog.card_slot_of(core, *c)
                                == Some(crate::catalog::EquipSlot::Weapon)
                        })
                        .map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::SWD_YUFULU_WEAPON,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::SWD_YUFULU_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        // 蝶舞（U05 xianjian 龙葵，规则 v153）：杀牌（挂起时 stage 已前
        // 移到 TargetChoice）→目标（挂起时 stage 已前移到 Act）
        HostBehavior::XianjianDiewu => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c).is_sha_like())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_DIEWU_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::Act => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_DIEWU_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 灵愈（U05 xianjian 龙葵，规则 v153）：目标询问（挂起时 stage
        // 已前移到 TargetChoice；仅其他已受伤角色）
        HostBehavior::XianjianLingyu => match stage {
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| *p != owner && (core.player(*p).hp as u32) < core.player(*p).max_hp)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_LINGYU_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 归元（U05 xianjian 林月如，规则 v147）：杀弃牌询问（挂起时
        // stage 已前移到 Act）
        HostBehavior::XianjianGuiyuan => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .filter(|c| catalog.card_kind(*c).is_sha_like())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_XJ_GUIYUAN_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 羊皮卷（swd 第十三批 v97）：类别询问（挂起时 stage 已前移到
        // SelectCard，静态 Tag 0–2）→ 发现三选一（stage=TargetChoice，
        // 选项读快照 cards——亮出的牌仍在牌堆）
        HostBehavior::SwdYangpijuanCard => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                crate::content::host::SWD_YANGPIJUAN_CATEGORY,
                ChoiceKind::PickTag,
                (0..3).map(ChoiceOption::Tag).collect(),
                1,
                1,
                false,
            )),
            HostStage::TargetChoice => {
                if cards.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::SWD_YANGPIJUAN_PICK,
                    ChoiceKind::SelectCards,
                    cards.iter().copied().map(ChoiceOption::Card).collect(),
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 暴雪药水：目标自选两张手牌（逐技对照批五 v79）
        HostBehavior::GwentBaoxueCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let t = target?;
            let n = core.player(t).hand.len().min(2);
            if n == 0 {
                return None;
            }
            Some(ask(
                t,
                GW_BAOXUE_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(t).hand),
                n,
                n,
                false,
            ))
        }
        // 冰魄通：双段自选展示（逐技对照批五 v79）
        HostBehavior::JiguanBingpoCard => match stage {
            HostStage::ComparePlayerPick => Some(ask(
                owner,
                GJ_BINGPO_SHOW_OWN,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                false,
            )),
            HostStage::CompareTargetPick => {
                let t = target?;
                Some(ask(
                    t,
                    GJ_BINGPO_SHOW_TARGET,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(t).hand),
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 龙须钩：使用者自选目标装备（逐技对照批五 v79）
        HostBehavior::JiguanLongxuCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let t = target?;
            Some(ask(
                owner,
                GJ_LONGXU_PICK,
                ChoiceKind::SelectCards,
                core.player(t)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect(),
                1,
                1,
                false,
            ))
        }
        // 末焱：选获得者（含自己，可取消）→ 该角色从展示牌中至少取一张
        HostBehavior::D3Moyan => match stage {
            HostStage::TargetChoice => {
                let options: Vec<ChoiceOption> =
                    core.alive_players().map(ChoiceOption::Player).collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    D3_MOYAN_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            HostStage::SelectCard => {
                let target = target?;
                if cards.is_empty() {
                    return None;
                }
                Some(ask(
                    target,
                    D3_MOYAN_GAIN,
                    ChoiceKind::SelectCards,
                    cards.iter().copied().map(ChoiceOption::Card).collect(),
                    1,
                    cards.len(),
                    false,
                ))
            }
            _ => None,
        },
        // 最远射程武器（精彀）：指定一名其他角色受 1 点伤害
        HostBehavior::MaxRangeWeaponDamage => {
            if stage != HostStage::Act {
                return None;
            }
            Some(ask(
                owner,
                D3_JINGGOU_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            ))
        }
        // 三尖武器技：弃一张手牌 → 选转移伤害目标
        HostBehavior::SanjianWeaponDamage => match stage {
            HostStage::SelectNature => Some(ask(
                owner,
                WD_SANJIAN_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                true,
            )),
            HostStage::Act => {
                let damaged = target?;
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != owner
                            && *p != damaged
                            && core.computed_distance(catalog, damaged, *p) <= 1
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    owner,
                    WD_SANJIAN_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 偷梁换柱：己方选牌 → 目标选牌（交换）
        HostBehavior::TouliangSwap => {
            let t = target?;
            match stage {
                HostStage::SelectNature => Some(ask(
                    owner,
                    WD_TOULIANG_OWN,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(owner).hand),
                    1,
                    1,
                    true,
                )),
                HostStage::Act => Some(ask(
                    owner,
                    WD_TOULIANG_TARGET,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(t).hand),
                    1,
                    1,
                    true,
                )),
                _ => None,
            }
        }
        // 釜底抽薪：目标选拼点牌 → 己方选拼点牌
        HostBehavior::FudichouxinCompare => {
            let t = target?;
            match stage {
                HostStage::ComparePlayerPick => Some(ask(
                    t,
                    WD_FUDI_TARGET_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(t).hand),
                    1,
                    1,
                    false,
                )),
                HostStage::Act => Some(ask(
                    owner,
                    WD_FUDI_PLAYER_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(owner).hand),
                    1,
                    1,
                    false,
                )),
                _ => None,
            }
        }
        // 隔岸观火：目标一选牌 → 目标二选牌
        HostBehavior::GeanguanhuoCompare => {
            if targets.len() != 2 {
                return None;
            }
            match stage {
                HostStage::ComparePlayerPick => Some(ask(
                    targets[0],
                    YC_GEAN_T1_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(targets[0]).hand),
                    1,
                    1,
                    false,
                )),
                HostStage::Act => Some(ask(
                    targets[1],
                    YC_GEAN_T2_PICK,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(targets[1]).hand),
                    1,
                    1,
                    false,
                )),
                _ => None,
            }
        }
        // 草船借箭：分支选择（有杀 → 目标双选；无杀 → 使用者单选）
        HostBehavior::CaochuanCard => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let has_sha = core
                .player(t)
                .hand
                .iter()
                .any(|c| catalog.card_kind(*c) == CardKind::Sha);
            if has_sha {
                Some(ask(
                    t,
                    WD_CAOCHUAN_BRANCH,
                    ChoiceKind::PickTag,
                    vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    1,
                    1,
                    false,
                ))
            } else {
                Some(ask(
                    owner,
                    WD_CAOCHUAN_BRANCH,
                    ChoiceKind::PickTag,
                    vec![ChoiceOption::Tag(1)],
                    1,
                    1,
                    false,
                ))
            }
        }
        // 舒筋散：目标弃任意张手牌摸等量
        HostBehavior::GujianShujinsanCard => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let hand = &core.player(t).hand;
            let n = hand.len();
            Some(ask(
                t,
                GJ_SHUJINSAN_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(hand),
                1,
                n,
                true,
            ))
        }
        // 飞镖：目标弃一张黑色手牌（可取消）
        HostBehavior::GujianFeibiaoCard => {
            if stage != HostStage::Act {
                return None;
            }
            let t = target?;
            let options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                t,
                GJ_FEIBIAO_DISCARD,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                true,
            ))
        }
        // 金莲烛：持有者确认打出
        HostBehavior::JinlianzhuCancel => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            Some(ask(
                owner,
                GJ_JINLIANZHU_CONFIRM,
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                false,
            ))
        }
        // 菱角：摸牌阶段额外摸 1 后弃 1
        HostBehavior::GujianFoodXiajiao => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                GJ_XIAJIAO_DISCARD,
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(owner).hand),
                1,
                1,
                false,
            ))
        }
        // 卤玉羹：弃一张基本牌获得牌堆顶
        HostBehavior::GujianFoodLuyugeng => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    matches!(
                        catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask(
                owner,
                GJ_LUYUGENG_COST,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        // 帝女玄霜：回复后可弃任意张手牌摸等量
        HostBehavior::JiguanDinvCard => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let hand = &core.player(owner).hand;
            let n = hand.len();
            Some(ask(
                owner,
                GJ_DINVXUANSHUANG_RIDER,
                ChoiceKind::SelectCards,
                hand_card_options(hand),
                0,
                n,
                true,
            ))
        }
        // 元素毁灭：逐目标弃 1–2（可取消）
        HostBehavior::HearthYuansuCard => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let t = target?;
            let mut options: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .chain(core.player(t).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            options.truncate(2);
            if options.is_empty() {
                return None;
            }
            let n = options.len();
            Some(ask(
                t,
                crate::content::host::HH_YUANSU_DISCARD,
                ChoiceKind::SelectCards,
                options,
                1,
                n,
                true,
            ))
        }
        // 石会粉：目标出闪否则禁用
        HostBehavior::GujianShihuifenCard => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let t = target?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == CardKind::Shan)
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                t,
                "respond_shan",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        // 神火飞鸦：逐目标需闪
        HostBehavior::JiguanShenhuoCard => {
            if stage != HostStage::TargetChoice {
                return None;
            }
            let t = target?;
            let mut options = vec![ChoiceOption::Pass];
            options.extend(
                core.player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| catalog.card_kind(*c) == CardKind::Shan)
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                t,
                "aoe_response",
                ChoiceKind::RespondCard,
                options,
                1,
                1,
                false,
            ))
        }
        HostBehavior::WudianPingzhang
        | HostBehavior::WudianFeitiao
        | HostBehavior::WudianQianggu
        | HostBehavior::WudianQiangguGuard
        | HostBehavior::WudianYinbo
        | HostBehavior::WudianFeidan
        | HostBehavior::WudianLiangou
        | HostBehavior::WudianBaoxue
        | HostBehavior::WudianYuedong
        | HostBehavior::WudianKuoyin
        | HostBehavior::WudianHuhuan
        | HostBehavior::WudianShuaiyan
        | HostBehavior::WudianKangyin
        | HostBehavior::WudianMujing => {
            active_ask_wudian(behavior, stage, owner, target, targets, cards, core, catalog)
        }
        HostBehavior::MtgLongshanfengLand
        | HostBehavior::MtgHaidaoLand
        | HostBehavior::MtgLindixiliuLand
        | HostBehavior::MtgYoulinDiscard
        | HostBehavior::MtgBingheyaosaiLand
        | HostBehavior::MtgLinzhongjiantaLand
        | HostBehavior::MtgLinzhongjiantaStrike
        | HostBehavior::MtgShamolvzhouLand
        | HostBehavior::MtgYixialanLand
        | HostBehavior::MtgYixialanRecast
        | HostBehavior::MtgFeixuLand
        | HostBehavior::MtgShuimomuxueLand => {
            active_ask_mtg(behavior, stage, owner, target, targets, cards, core, catalog)
        }
        _ => None,
    }
}
