use super::*;

/// 宿主机器的询问派生：按行为 + 阶段还原 host.rs 各 step 中
/// 创建该 `WaitChoice` 时的 `ChoiceSpec`（挂起时 stage 已前移到下一阶段）。
#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_host(
    behavior: u16,
    stage: u8,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    use crate::content::host::{
        D3_JINGGOU_TARGET, D3_MOYAN_GAIN, D3_MOYAN_TARGET, GJ_BINGPO_SHOW_OWN,
        GJ_BINGPO_SHOW_TARGET, GJ_DINVXUANSHUANG_RIDER, GJ_FEIBIAO_DISCARD, GJ_JINLIANZHU_CONFIRM,
        GJ_LONGXU_PICK, GJ_LUYUGENG_COST, GJ_SHUJINSAN_DISCARD, GJ_XIAJIAO_DISCARD,
        GW_BAOXUE_DISCARD, HOST_TARGET_DISCARD_EQUIP, WD_CAOCHUAN_BRANCH, WD_FUDI_PLAYER_PICK,
        WD_FUDI_TARGET_PICK, WD_SANJIAN_DISCARD, WD_SANJIAN_TARGET, WD_TOULIANG_OWN,
        WD_TOULIANG_TARGET, WUDIAN_BAOXUE_TARGETS, WUDIAN_FEIDAN_DISCARD, WUDIAN_FEITIAO_CARDS,
        WUDIAN_FEITIAO_TARGET, WUDIAN_HUHUAN_CARDS, WUDIAN_KUOYIN_CARDS, WUDIAN_LIANGOU_CARDS,
        WUDIAN_LIANGOU_TARGET, WUDIAN_PINGZHANG_CARDS, WUDIAN_QIANGGU_CARDS,
        WUDIAN_QIANGGU_GUARD_ASK, WUDIAN_XJ_DIEWU_CARDS, WUDIAN_XJ_DIEWU_TARGET,
        WUDIAN_XJ_GUIYUAN_CARDS, WUDIAN_XJ_LINGYU_TARGET, WUDIAN_YINBO_CARDS,
        WUDIAN_YUEDONG_TARGETS, WUXING_SELECT_CARD, WUXING_SELECT_NATURE, YC_GEAN_T1_PICK,
        YC_GEAN_T2_PICK,
    };
    let behavior = HostBehavior::from_tag(behavior)?; // behavior: u16
    let stage = HostStage::from_tag(stage)?;
    match behavior {
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
        // 龙颅山峰（mtg 首切片 v100）：获得杀/闪二选一（挂起时 stage 已
        // 前移到 SelectCard，静态 Tag 0–1）
        HostBehavior::MtgLongshanfengLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_LONGSHANFENG_GAIN,
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            ))
        }
        // 海岛（mtg 第二批 v101）：受伤分支弃牌/取消（挂起时 stage 已
        // 前移到 SelectCard；选项=手牌∪装备区，可取消）
        HostBehavior::MtgHaidaoLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options = hand_card_options(&core.player(owner).hand);
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                owner,
                crate::content::host::MTG_HAIDAO_DISCARD,
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
        // 水没墓穴（mtg 第三批 v102）：弃一摸二（挂起时 stage 已前移到
        // SelectCard；选项=手牌∪装备区，可取消）
        // 邻地溪流（mtg 第十批 v109）：弃任意张♥（挂起时 stage 已前移
        // 到 SelectCard；选项=手牌∪装备♥，可取消）
        HostBehavior::MtgLindixiliuLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart)
                .map(ChoiceOption::Card)
                .collect();
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .filter(|c| catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart)
                    .map(ChoiceOption::Card),
            );
            if options.is_empty() {
                return None;
            }
            let n = options.len();
            Some(ask(
                owner,
                crate::content::host::MTG_LINDIXILIU_DISCARD,
                ChoiceKind::SelectCards,
                options,
                0,
                n,
                true,
            ))
        }
        // 幽林地附技（mtg 第九批 v108）：锦囊选择（stage=SelectCard，
        // 手牌锦囊）+ 发现三选一（stage=TargetChoice，选项读快照 cards）
        HostBehavior::MtgYoulinDiscard => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        crate::content::host::yangpijuan_class(catalog.card_kind(*c)) == Some(1)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::MTG_YOULIN_DISCARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => {
                if cards.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::MTG_YOULIN_PICK,
                    ChoiceKind::SelectCards,
                    cards.iter().copied().map(ChoiceOption::Card).collect(),
                    1,
                    1,
                    false,
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
        // 冰河要塞（mtg 第八批 v107）：摸二弃二（挂起时 stage 已前移到
        // SelectCard；选项=手牌∪装备区，强制两张）
        HostBehavior::MtgBingheyaosaiLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options = hand_card_options(&core.player(owner).hand);
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            if options.len() < 2 {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_BINGHEYAOSAI_DISCARD,
                ChoiceKind::SelectCards,
                options,
                2,
                2,
                false,
            ))
        }
        // 林中尖塔（mtg 第七批 v106）：武器发现三选一（stage=SelectCard，
        // 选项读快照 cards）+ 塔击基本牌选择（stage=SelectCard）+ 目标
        // 选择（stage=TargetChoice）
        HostBehavior::MtgLinzhongjiantaLand => {
            if stage != HostStage::SelectCard || cards.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_LINZHONGJIANTA_DISCOVER,
                ChoiceKind::SelectCards,
                cards.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        HostBehavior::MtgLinzhongjiantaStrike => match stage {
            HostStage::SelectCard => {
                let options: Vec<ChoiceOption> = core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        crate::content::host::yangpijuan_class(catalog.card_kind(*c)) == Some(0)
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    crate::content::host::MTG_LINZHONGJIANTA_CARD,
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                ))
            }
            HostStage::TargetChoice => Some(ask(
                owner,
                crate::content::host::MTG_LINZHONGJIANTA_TARGET,
                ChoiceKind::SelectPlayers,
                other_player_options(core, owner),
                1,
                1,
                false,
            )),
            _ => None,
        },
        // 沙漠绿洲（mtg 第六批 v105）：基本牌发现三选一（挂起时 stage
        // 已前移到 SelectCard；选项读快照 cards——亮出的牌仍在牌堆）
        HostBehavior::MtgShamolvzhouLand => {
            if stage != HostStage::SelectCard || cards.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_SHAMOLVZHOU_DISCOVER,
                ChoiceKind::SelectCards,
                cards.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        // 依夏兰（mtg 第五批 v104）：类型选择（挂起时 stage 已前移到
        // SelectCard，静态 Tag 0–1）
        HostBehavior::MtgYixialanLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_YIXIALAN_GAIN,
                ChoiceKind::PickTag,
                vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                1,
                1,
                false,
            ))
        }
        // 依夏兰重铸（mtg 第五批 v104）：基本牌选择（stage=SelectCard；
        // 选项=手牌基本牌）
        HostBehavior::MtgYixialanRecast => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|c| {
                    crate::content::host::yangpijuan_class(catalog.card_kind(*c)) == Some(0)
                })
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_YIXIALAN_RECAST_CARD,
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                false,
            ))
        }
        // 废墟（mtg 第四批 v103）：弃牌堆发现三选一（挂起时 stage 已
        // 前移到 SelectCard；选项读快照 cards——亮出的牌仍在弃堆）
        HostBehavior::MtgFeixuLand => {
            if stage != HostStage::SelectCard || cards.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                crate::content::host::MTG_FEIXU_DISCOVER,
                ChoiceKind::SelectCards,
                cards.iter().copied().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            ))
        }
        HostBehavior::MtgShuimomuxueLand => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let mut options = hand_card_options(&core.player(owner).hand);
            options.extend(
                core.player(owner)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            Some(ask(
                owner,
                crate::content::host::MTG_SHUIMOMUXUE_DISCARD,
                ChoiceKind::SelectCards,
                options,
                0,
                1,
                true,
            ))
        }
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
        _ => None,
    }
}
