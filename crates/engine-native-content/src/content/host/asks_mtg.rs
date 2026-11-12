//! 宿主机器询问派生：万智牌（mtg）行为。

use super::*;
use crate::catalog::Catalog;
use crate::snap::*;
use crate::state::GameStateCore;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_mtg(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
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
        _ => unreachable!("behavior routed to the wrong ask group"),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_batches(
    behavior: HostBehavior,
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match behavior {
        HostBehavior::U01DddBatchA => super::step::u01_batch_a::active_ask_u01_batch_a(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U01DddBatchB => super::step::u01_batch_b::active_ask_u01_batch_b(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U01DddBatchC => super::step::u01_batch_c::active_ask_u01_batch_c(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::U01DddBatchD => super::step::u01_batch_d::active_ask_u01_batch_d(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::L28ThunderBatchA => super::step::l28_batch_a::ask_l28_batch_a(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::L28ThunderBatchB => super::step::l28_batch_b::ask_l28_batch_b(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::L27BeautifyBatchA => super::step::l27_batch_a::ask_l27_batch_a(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::L27BeautifyBatchB => super::step::l27_batch_b::ask_l27_batch_b(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::L19DoomsdayBatchA => super::step::l19_batch_a::ask_l19_batch_a(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::L19DoomsdayBatchB => super::step::l19_batch_b::ask_l19_batch_b(
            stage, owner, target, targets, card, cards, core, catalog,
        ),
        HostBehavior::NewMeibu => {
            if stage != HostStage::SelectCard {
                return None;
            }
            let t = target?;
            if !core.player(t).alive {
                return None;
            }
            let mut options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .chain(core.player(owner).equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            options.truncate(64);
            Some(ask(
                owner,
                "new_meibu_discard",
                ChoiceKind::SelectCards,
                options,
                1,
                1,
                true,
            ))
        }
        HostBehavior::MbcmQingzheng => {
            let suit_opts = |p: PlayerId| -> Vec<ChoiceOption> {
                let mut suits: Vec<u8> = Vec::new();
                for c in core.player(p).hand.iter().copied() {
                    let s = match catalog.cards[c as usize].suit {
                        crate::catalog::Suit::Spade => 0,
                        crate::catalog::Suit::Heart => 1,
                        crate::catalog::Suit::Club => 2,
                        crate::catalog::Suit::Diamond => 3,
                    };
                    if !suits.contains(&s) {
                        suits.push(s);
                    }
                }
                suits.sort_unstable();
                suits.into_iter().map(ChoiceOption::Tag).collect()
            };
            match stage {
                HostStage::SelectNature => {
                    let options = suit_opts(owner);
                    (!options.is_empty()).then(|| {
                        ask(
                            owner,
                            "mbcmqingzheng_suit",
                            ChoiceKind::PickTag,
                            options,
                            1,
                            1,
                            false,
                        )
                    })
                }
                HostStage::TargetChoice => {
                    let options: Vec<ChoiceOption> = core
                        .alive_players()
                        .filter(|p| *p != owner && !core.player(*p).hand.is_empty())
                        .map(ChoiceOption::Player)
                        .collect();
                    (!options.is_empty()).then(|| {
                        ask(
                            owner,
                            "mbcmqingzheng_target",
                            ChoiceKind::SelectPlayers,
                            options,
                            1,
                            1,
                            true,
                        )
                    })
                }
                HostStage::CompareTargetPick => {
                    let t = target?;
                    let options = suit_opts(t);
                    (!options.is_empty()).then(|| {
                        ask(
                            owner,
                            "mbcmqingzheng_target_suit",
                            ChoiceKind::PickTag,
                            options,
                            1,
                            1,
                            false,
                        )
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}
