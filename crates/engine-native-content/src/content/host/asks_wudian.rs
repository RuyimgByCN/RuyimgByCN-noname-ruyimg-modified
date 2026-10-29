//! 宿主机器询问派生：武典（wudian）各子包行为。

use super::*;
use crate::catalog::Catalog;
use crate::snap::*;
use crate::state::GameStateCore;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_wudian(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        // 屏障（U05 ow 查莉娅，规则 v138）：花色弃牌询问（挂起时 stage
        // 已前移到 SelectCard；按受伤者归属定花色）
        HostBehavior::WudianPingzhang => match stage {
            HostStage::SelectCard => {
                let hurt = target.unwrap_or(owner);
                let want = if hurt == owner {
                    crate::catalog::Suit::Heart
                } else {
                    crate::catalog::Suit::Spade
                };
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .filter(|c| catalog.cards[*c as usize].suit == want)
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_PINGZHANG_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 跳跃（U05 ow 温斯顿，规则 v137）：弃牌（挂起时 stage 已前移
        // 到 TargetChoice）→ 目标（挂起时 stage 已前移到 Act）
        HostBehavior::WudianFeitiao => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_FEITIAO_CARDS,
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
                    WUDIAN_FEITIAO_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 强固（U05 ow 奥丽莎，规则 v136）：弃 2 张 he（挂起时 stage 已
        // 前移到 Act）
        HostBehavior::WudianQianggu => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.len() < 2 {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_QIANGGU_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    2,
                    2,
                    false,
                ))
            }
            _ => None,
        },
        // 强固守卫（U05 ow 奥丽莎，规则 v136）：来源基本牌询问（挂起时
        // stage 已前移到 SelectCard；observer=target=杀使用者）
        HostBehavior::WudianQiangguGuard => match stage {
            HostStage::SelectCard => {
                let Some(attacker) = target else {
                    return None;
                };
                let mut options = vec![ChoiceOption::Pass];
                options.extend(
                    core.player(attacker)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| {
                            matches!(
                                catalog.card_kind(*c),
                                crate::catalog::CardKind::Sha
                                    | crate::catalog::CardKind::Shan
                                    | crate::catalog::CardKind::Tao
                                    | crate::catalog::CardKind::Jiu
                            )
                        })
                        .map(ChoiceOption::Card),
                );
                Some(ask(
                    attacker,
                    WUDIAN_QIANGGU_GUARD_ASK,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 音波（U05 ow 半藏，规则 v135）：黑桃弃牌询问（挂起时 stage
        // 已前移到 Act）
        HostBehavior::WudianYinbo => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .filter(|c| {
                        matches!(catalog.cards[*c as usize].suit, crate::catalog::Suit::Spade)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_YINBO_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 飞弹（U05 ow 法老之鹰，规则 v130）：弃牌询问（挂起时 stage
        // 已前移到 SelectCard；he 区域=手牌+装备）
        HostBehavior::WudianFeidan => match stage {
            HostStage::SelectCard => {
                let p = core.player(owner);
                let options: Vec<ChoiceOption> = p
                    .hand
                    .iter()
                    .copied()
                    .chain(p.equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_FEIDAN_DISCARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    true,
                ))
            }
            _ => None,
        },
        // 链钩（U05 ow 路霸，规则 v128）：弃牌（挂起时 stage 已前移到
        // TargetChoice）→ 目标（挂起时 stage 已前移到 Act）
        HostBehavior::WudianLiangou => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let mut options: Vec<ChoiceOption> =
                    p.hand.iter().copied().map(ChoiceOption::Card).collect();
                options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_LIANGOU_CARDS,
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
                    WUDIAN_LIANGOU_TARGET,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            _ => None,
        },
        // 暴雪（U05 ow 美，规则 v127）：目标选择（挂起时 stage 已前移
        // 到 TargetChoice；上限=黑色 he 牌数，仅其他存活角色）
        HostBehavior::WudianBaoxue => match stage {
            HostStage::TargetChoice => {
                let p = core.player(owner);
                let black = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .filter(|c| {
                        matches!(
                            catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .count();
                if black == 0 {
                    return None;
                }
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|q| *q != owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_BAOXUE_TARGETS,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    black,
                    true,
                ))
            }
            _ => None,
        },
        // 乐动（U05 ow 卢西奥，规则 v126）：目标选择（挂起时 stage 已
        // 前移到 TargetChoice；选项按回复模式收窄未满体力者）
        HostBehavior::WudianYuedong => match stage {
            HostStage::TargetChoice => {
                let num = 1 + core.turn.yuedong_num as usize;
                let recover = core.turn.yuedong_recover;
                let options: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| !recover || (core.player(*p).hp as u32) < core.player(*p).max_hp)
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_YUEDONG_TARGETS,
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    num,
                    true,
                ))
            }
            _ => None,
        },
        // 扩音（U05 ow 卢西奥，规则 v126）：弃牌选择（挂起时 stage 已前
        // 移到 Act；1/2 张由存量决定，he 区域=手牌+装备）
        HostBehavior::WudianKuoyin => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let mut options: Vec<ChoiceOption> =
                    p.hand.iter().copied().map(ChoiceOption::Card).collect();
                options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                if options.is_empty() {
                    return None;
                }
                let (min, max) = if core.turn.yuedong_eff > 0 {
                    (1, 1)
                } else if core.turn.yuedong_num > 0 {
                    (2, 2)
                } else {
                    (1, 2)
                };
                Some(ask(
                    owner,
                    WUDIAN_KUOYIN_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    min,
                    max,
                    true,
                ))
            }
            _ => None,
        },
        // 呼唤（U05 ow 卢西奥，规则 v126）：弃 2 张（挂起时 stage 已前移
        // 到 Act）
        HostBehavior::WudianHuhuan => match stage {
            HostStage::Act => {
                let p = core.player(owner);
                let mut options: Vec<ChoiceOption> =
                    p.hand.iter().copied().map(ChoiceOption::Card).collect();
                options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                if options.len() < 2 {
                    return None;
                }
                Some(ask(
                    owner,
                    WUDIAN_HUHUAN_CARDS,
                    ChoiceKind::SelectCards,
                    options,
                    2,
                    2,
                    true,
                ))
            }
            _ => None,
        },
        // 率言（U07 批二 v112）：回复者二选一（stage=SelectCard，观察者
        // =target）→ 被弃者选牌（stage=TargetChoice）
        HostBehavior::WudianShuaiyan => {
            let Some(r) = target else {
                return None;
            };
            match stage {
                HostStage::SelectCard => Some(ask(
                    r,
                    crate::content::host::WUDIAN_SHUAIYAN_CHOICE,
                    ChoiceKind::PickTag,
                    vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    1,
                    1,
                    false,
                )),
                HostStage::TargetChoice => Some(ask(
                    r,
                    crate::content::host::WUDIAN_SHUAIYAN_DISCARD,
                    ChoiceKind::SelectCards,
                    hand_card_options(&core.player(r).hand),
                    1,
                    1,
                    false,
                )),
                _ => None,
            }
        }
        // 抗yin（U07 批四 v114）：目标选择（stage=SelectCard，owner）→
        // 被弃者选牌（stage=TargetChoice，observer=target）→ 基本分支多
        // 角色摸牌（stage=CardsAsk，owner）
        HostBehavior::WudianKangyin => match stage {
            HostStage::SelectCard => Some(ask(
                owner,
                crate::content::host::WUDIAN_KANGYIN_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            HostStage::TargetChoice => {
                let Some(t) = target else {
                    return None;
                };
                let mut options = hand_card_options(&core.player(t).hand);
                options.extend(
                    core.player(t)
                        .equips
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    t,
                    crate::content::host::WUDIAN_KANGYIN_DISCARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::CardsAsk => {
                let x = core
                    .player(owner)
                    .max_hp
                    .saturating_sub(core.player(owner).hp as u32)
                    .max(1) as usize;
                Some(ask(
                    owner,
                    crate::content::host::WUDIAN_KANGYIN_DRAW,
                    ChoiceKind::SelectPlayers,
                    core.alive_players().map(ChoiceOption::Player).collect(),
                    1,
                    x,
                    false,
                ))
            }
            _ => None,
        },
        // 目镜（U07 ow v118）：黑色牌选择（stage=SelectCard）→ 目标选择
        // （stage=TargetChoice，塔击同构）
        HostBehavior::WudianMujing => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
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
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::WUDIAN_MUJING_CARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::WUDIAN_MUJING_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        _ => unreachable!("behavior routed to the wrong ask group"),
    }
}
