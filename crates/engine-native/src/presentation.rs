//! N10 表现事件与渲染投影（ADR 0761）。
//!
//! 权威日志（`LogEvent`）→ 座位可见的表现事件（`PresentationEvent`）→
//! 确定性渲染帧（`render_table`）。三层均为**纯投影**：不修改状态、
//! 不进入快照与摘要（N10 约定）；可见性规则与 `EventView` 一致——
//! 他人摸牌为暗置信息（仅数量，类型层面无牌 id），其余事件公开。

use crate::ids::{CardId, PlayerId};
use crate::machine::LogEvent;
use crate::view::PlayerView;

/// 座位可见的表现事件：UI/渲染消费的最小事件集（牌已解析为名称）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentationEvent {
    Phase {
        player: PlayerId,
        phase: crate::event::Phase,
        begin: bool,
    },
    TurnEnded {
        player: PlayerId,
    },
    DeckReshuffled,
    /// 摸牌：他人仅数量（`cards: None`，暗置信息不下渗到表现层）；
    /// 本人携带具体牌名。
    Drawn {
        player: PlayerId,
        count: usize,
        cards: Option<Vec<String>>,
    },
    Played {
        player: PlayerId,
        card: String,
        targets: Vec<PlayerId>,
    },
    Responded {
        player: PlayerId,
        card: String,
    },
    Discarded {
        player: PlayerId,
        cards: Vec<String>,
    },
    Given {
        from: PlayerId,
        to: PlayerId,
        cards: Vec<String>,
    },
    Damaged {
        source: Option<PlayerId>,
        target: PlayerId,
        amount: u32,
        by_card: Option<String>,
    },
    DamagePrevented {
        target: PlayerId,
    },
    /// 失去体力（非伤害；逐技对照批三 v77）。
    HpLost {
        player: PlayerId,
        amount: u32,
    },
    Recovered {
        player: PlayerId,
        amount: u32,
    },
    DyingEntered {
        player: PlayerId,
    },
    DyingSaved {
        player: PlayerId,
        card: String,
    },
    Died {
        player: PlayerId,
        killer: Option<PlayerId>,
    },
    Equipped {
        player: PlayerId,
        card: String,
        replaced: Option<String>,
    },
    ChainToggled {
        player: PlayerId,
        linked: bool,
    },
    Nullified {
        card: String,
        player: PlayerId,
    },
    SkillTriggered {
        owner: PlayerId,
        skill: String,
    },
    SkillToggled {
        player: PlayerId,
        skill: String,
        enabled: bool,
    },
    Judged {
        player: PlayerId,
        card: String,
    },
    ArmorJudged {
        player: PlayerId,
        success: bool,
    },
    ChoiceOpened {
        id: u64,
        observer: PlayerId,
        prompt: String,
    },
}

/// 日志 → 表现事件投影（`viewer`：None = 观战者，全部摸牌仅数量）。
pub fn project(
    log: &[LogEvent],
    catalog: &crate::catalog::Catalog,
    viewer: Option<PlayerId>,
) -> Vec<PresentationEvent> {
    let name = |c: CardId| catalog.cards[c as usize].name.to_string();
    let names = |cs: &[CardId]| cs.iter().copied().map(name).collect::<Vec<_>>();
    let mut out = Vec::with_capacity(log.len());
    for e in log {
        out.extend(match e {
            LogEvent::DamageDealt {
                source,
                target,
                amount,
                redirected_from: _,
                card,
            } => vec![PresentationEvent::Damaged {
                source: *source,
                target: *target,
                amount: *amount,
                by_card: card.map(name),
            }],
            LogEvent::DamagePrevented { target } => {
                vec![PresentationEvent::DamagePrevented { target: *target }]
            }
            LogEvent::CardsDrawn { player, cards } => {
                let own = viewer == Some(*player);
                vec![PresentationEvent::Drawn {
                    player: *player,
                    count: cards.len(),
                    cards: own.then(|| names(cards)),
                }]
            }
            LogEvent::CardPlayed {
                card,
                player,
                targets,
            } => vec![PresentationEvent::Played {
                player: *player,
                card: name(*card),
                targets: targets.clone(),
            }],
            LogEvent::CardResponded { card, player } => vec![PresentationEvent::Responded {
                player: *player,
                card: name(*card),
            }],
            LogEvent::CardDiscarded { player, cards } => vec![PresentationEvent::Discarded {
                player: *player,
                cards: names(cards),
            }],
            LogEvent::CardsGiven { from, to, cards } => vec![PresentationEvent::Given {
                from: *from,
                to: *to,
                cards: names(cards),
            }],
            LogEvent::HpRecovered { player, amount } => vec![PresentationEvent::Recovered {
                player: *player,
                amount: *amount,
            }],
            LogEvent::HujiaAbsorbed { player, amount } => {
                vec![PresentationEvent::Recovered {
                    player: *player,
                    amount: *amount,
                }]
            }
            LogEvent::HpLost { player, amount } => vec![PresentationEvent::HpLost {
                player: *player,
                amount: *amount,
            }],
            LogEvent::BossChongzhengTick { player, count: _ } => {
                vec![PresentationEvent::Recovered { player: *player, amount: 0 }]
            }
            LogEvent::BossRevived { player } => vec![PresentationEvent::Recovered { player: *player, amount: 0 }],
            LogEvent::DyingEntered { player } => {
                vec![PresentationEvent::DyingEntered { player: *player }]
            }
            LogEvent::DyingSaved { player, by_card } => vec![PresentationEvent::DyingSaved {
                player: *player,
                card: name(*by_card),
            }],
            LogEvent::PlayerDied { player, killer } => vec![PresentationEvent::Died {
                player: *player,
                killer: *killer,
            }],
            LogEvent::ChoiceOpened { id, observer, prompt } => {
                vec![PresentationEvent::ChoiceOpened {
                    id: *id,
                    observer: *observer,
                    prompt: (*prompt).to_string(),
                }]
            }
            // 应答内容仅进入询问者本人的权威日志语义；表现层统一
            // 不再重复展示选项内容（询问开启事件已足够渲染）。
            LogEvent::ChoiceAnswered { .. } => vec![],
            LogEvent::SkillTriggered { skill, owner } => vec![PresentationEvent::SkillTriggered {
                owner: *owner,
                skill: (*skill).to_string(),
            }],
            LogEvent::SkillToggled {
                player,
                skill,
                enabled,
            } => vec![PresentationEvent::SkillToggled {
                player: *player,
                skill: (*skill).to_string(),
                enabled: *enabled,
            }],
            LogEvent::PhaseChanged {
                player,
                phase,
                begin,
            } => vec![PresentationEvent::Phase {
                player: *player,
                phase: *phase,
                begin: *begin,
            }],
            LogEvent::TurnEnded { player } => {
                vec![PresentationEvent::TurnEnded { player: *player }]
            }
            LogEvent::DeckReshuffled => vec![PresentationEvent::DeckReshuffled],
            LogEvent::Equipped {
                player,
                card,
                replaced,
            } => vec![PresentationEvent::Equipped {
                player: *player,
                card: name(*card),
                replaced: replaced.map(name),
            }],
            LogEvent::Linked { player, linked } => vec![PresentationEvent::ChainToggled {
                player: *player,
                linked: *linked,
            }],
            LogEvent::CardNullified { card, player } => vec![PresentationEvent::Nullified {
                card: name(*card),
                player: *player,
            }],
            LogEvent::Judged { player, card } => vec![PresentationEvent::Judged {
                player: *player,
                card: name(*card),
            }],
            LogEvent::JudgementRewritten { .. } => vec![], // 改判细节属判定区内部流转
            LogEvent::JudgementFinalized { player, card } => vec![PresentationEvent::Judged {
                player: *player,
                card: name(*card),
            }],
            LogEvent::JudgementGained { player, card } => {
                // 天妒收判定牌：公开的获得动作
                vec![PresentationEvent::Given {
                    from: PlayerId::MAX,
                    to: *player,
                    cards: vec![name(*card)],
                }]
            }
            LogEvent::ArmorJudged {
                player,
                card: _,
                success,
            } => vec![PresentationEvent::ArmorJudged {
                player: *player,
                success: *success,
            }],
        });
    }
    out
}

/// 确定性渲染帧：座位行（体力/手牌数/装备/双将）+ 最近表现事件。
/// 纯函数、无时间与随机源——同输入逐字节同输出。
pub fn render_table(view: &PlayerView, events: &[PresentationEvent], width: usize) -> String {
    let w = width.max(40);
    let mut s = String::new();
    let line = |txt: &str, s: &mut String| {
        s.push_str("| ");
        s.push_str(txt);
        for _ in txt.chars().count() + 2..w {
            s.push(' ');
        }
        s.push_str("|\n");
    };
    let rule = |s: &mut String| {
        s.push('+');
        for _ in 0..w {
            s.push('-');
        }
        s.push_str("+\n");
    };
    let phase_name = |p: crate::event::Phase| match p {
        crate::event::Phase::Judge => "判定",
        crate::event::Phase::Draw => "摸牌",
        crate::event::Phase::Play => "出牌",
        crate::event::Phase::Discard => "弃牌",
        crate::event::Phase::End => "结束",
    };
    rule(&mut s);
    line(
        &format!(
            "回合:{} 阶段:{} 第{}轮 规则v{}",
            view.turn,
            phase_name(view.phase),
            view.round,
            view.ruleset_version
        ),
        &mut s,
    );
    rule(&mut s);
    for p in &view.players {
        let hand = match &p.hand {
            crate::view::HandView::Own(_) => p.hand_count.to_string(),
            crate::view::HandView::Hidden => format!("{}(暗)", p.hand_count),
        };
        let generals = p
            .generals
            .iter()
            .enumerate()
            .filter_map(|(slot, g)| {
                g.as_ref()
                    .map(|g| format!("{}{}", g.name, if slot == 0 { "" } else { "/副" }))
            })
            .collect::<Vec<_>>()
            .join("+");
        let generals = if generals.is_empty() {
            "无将".to_string()
        } else {
            generals
        };
        let status = if p.alive { "" } else { " [阵亡]" };
        line(
            &format!(
                "座位{} 体力{}/{} 手牌{} 将:{}{}",
                p.seat, p.hp, p.max_hp, hand, generals, status
            ),
            &mut s,
        );
    }
    rule(&mut s);
    let tail: Vec<&PresentationEvent> = events.iter().rev().take(6).collect();
    if tail.is_empty() {
        line("（暂无表现事件）", &mut s);
    }
    for e in tail.iter().rev() {
        let txt = match e {
            PresentationEvent::Phase {
                player,
                phase,
                begin,
            } => format!(
                "座位{} {}阶段{}",
                player,
                phase_name(*phase),
                if *begin { "开始" } else { "结束" }
            ),
            PresentationEvent::TurnEnded { player } => format!("座位{} 回合结束", player),
            PresentationEvent::HpLost { player, amount } => {
                format!("座位{} 失去{}点体力", player, amount)
            }
            PresentationEvent::DeckReshuffled => "牌库洗回".to_string(),
            PresentationEvent::Drawn {
                player,
                count,
                cards,
            } => match cards {
                Some(cs) => format!("座位{} 摸牌 {:?} ({}张)", player, cs, count),
                None => format!("座位{} 摸牌 {}张(暗)", player, count),
            },
            PresentationEvent::Played {
                player,
                card,
                targets,
            } => format!(
                "座位{} 使用{} → {:?}",
                player,
                card,
                targets.iter().map(|t| format!("座位{}", t)).collect::<Vec<_>>()
            ),
            PresentationEvent::Responded { player, card } => {
                format!("座位{} 打出{}", player, card)
            }
            PresentationEvent::Discarded { player, cards } => {
                format!("座位{} 弃置 {:?}", player, cards)
            }
            PresentationEvent::Given { from, to, cards } => {
                format!("座位{} → 座位{} 交付 {:?}", from, to, cards)
            }
            PresentationEvent::Damaged {
                source,
                target,
                amount,
                by_card,
            } => format!(
                "座位{} 受 {} 点伤害（来源：{}，牌：{}）",
                target,
                amount,
                source.map(|s| format!("座位{}", s)).unwrap_or_else(|| "无".into()),
                by_card.clone().unwrap_or_else(|| "无".into())
            ),
            PresentationEvent::DamagePrevented { target } => {
                format!("座位{} 伤害被防止", target)
            }
            PresentationEvent::Recovered { player, amount } => {
                format!("座位{} 回复 {} 点体力", player, amount)
            }
            PresentationEvent::DyingEntered { player } => format!("座位{} 濒死", player),
            PresentationEvent::DyingSaved { player, card } => {
                format!("座位{} 被救（{}）", player, card)
            }
            PresentationEvent::Died { player, killer } => format!(
                "座位{} 阵亡（凶手：{}）",
                player,
                killer.map(|s| format!("座位{}", s)).unwrap_or_else(|| "无".into())
            ),
            PresentationEvent::Equipped {
                player,
                card,
                replaced,
            } => format!(
                "座位{} 装备{}{}",
                player,
                card,
                replaced
                    .as_ref()
                    .map(|r| format!("（替换{}）", r))
                    .unwrap_or_default()
            ),
            PresentationEvent::ChainToggled { player, linked } => {
                format!("座位{} {}", player, if *linked { "横置" } else { "重置" })
            }
            PresentationEvent::Nullified { card, player } => {
                format!("{} 被 座位{} 无懈抵消", card, player)
            }
            PresentationEvent::SkillTriggered { owner, skill } => {
                format!("座位{} 发动【{}】", owner, skill)
            }
            PresentationEvent::SkillToggled {
                player,
                skill,
                enabled,
            } => format!(
                "座位{} 技能【{}】{}",
                player,
                skill,
                if *enabled { "启用" } else { "停用" }
            ),
            PresentationEvent::Judged { player, card } => {
                format!("座位{} 判定 {}", player, card)
            }
            PresentationEvent::ArmorJudged { player, success } => {
                format!("座位{} 八卦阵判定{}", player, if *success { "成功" } else { "失败" })
            }
            PresentationEvent::ChoiceOpened { observer, prompt, .. } => {
                format!("座位{} 待选择（{}）", observer, prompt)
            }
        };
        line(&txt, &mut s);
    }
    rule(&mut s);
    if let Some(o) = &view.outcome {
        let who = match o {
            crate::state::Winner::Player(p) => format!("座位{}", p),
            crate::state::Winner::LordSide => "主公方".to_string(),
            crate::state::Winner::Rebels => "反贼".to_string(),
            crate::state::Winner::Traitor => "内奸".to_string(),
            crate::state::Winner::Faction(f) => format!("{:?}", f),
            crate::state::Winner::Versus(t) => format!("{}队", if *t == 0 { "一" } else { "二" }),
            crate::state::Winner::Doudizhu { lord_won } => {
                if *lord_won { "地主".to_string() } else { "农民".to_string() }
            }
            crate::state::Winner::Boss { boss_won } => {
                if *boss_won { "Boss".to_string() } else { "挑战者".to_string() }
            }
        };
        line(&format!("终局：{} 胜", who), &mut s);
        rule(&mut s);
    }
    s
}

impl crate::engine::Engine {
    /// 座位可见的表现事件流（`None` = 观战者）。
    pub fn presentation_for(&self, seat: Option<PlayerId>) -> Vec<PresentationEvent> {
        crate::presentation::project(self.log(), self.catalog(), seat)
    }

    /// 座位视角的确定性渲染帧（视图 + 表现事件）。
    pub fn render_frame(&self, seat: Option<PlayerId>, width: usize) -> String {
        let view = match seat {
            Some(s) => self.view_for(s),
            None => self.view_spectator(),
        };
        let events = self.presentation_for(seat);
        crate::presentation::render_table(&view, &events, width)
    }
}
