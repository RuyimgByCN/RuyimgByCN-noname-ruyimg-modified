//! 挂起机器快照（MachineSnapshot）的规范哈希编码。

use super::native_invocation::push_native_invocation;
use super::*;

/// 机器规范快照的摘要编码：判别符 + 全字段；嵌套机器递归。
pub(super) fn push_machine(h: &mut HashSink, s: &crate::snap::MachineSnapshot) {
    use crate::snap::MachineSnapshot as M;
    match s {
        M::JudgeContinuation {
            owner,
            queue,
            current,
            ordinary,
            started,
            payment,
        } => {
            push_u8(h, 87);
            push_machine(
                h,
                &M::JudgePhase {
                    owner: *owner,
                    queue: queue.clone(),
                    current: current.clone(),
                },
            );
            push_u32(h, ordinary.unwrap_or(u32::MAX));
            push_u8(h, u8::from(*started));
            if let Some(p) = payment {
                push_u8(h, 1);
                push_u8(h, p.player);
                push_u32(h, p.old);
                push_u8(h, u8::from(p.discard_old));
                push_u16(h, p.provider);
            } else {
                push_u8(h, 0);
            }
        }
        M::NativeInvocation {
            owner,
            def,
            window,
            progress,
        } => {
            push_u8(h, 86);
            push_u8(h, *owner);
            push_u16(h, *def);
            push_u8(h, window.tag());
            push_native_invocation(h, progress);
        }
        M::NativeDamageSkill { owner, def, window } => {
            push_u8(h, 85);
            push_u8(h, *owner);
            push_u16(h, *def);
            push_u8(h, window.tag());
        }
        M::DamageFlow { root } => {
            push_u8(h, 84);
            push_u32(h, *root);
        }
        M::TriggerOrder {
            frame,
            owner,
            candidates,
            asked,
            cursor,
            done,
            owners,
            normal_owners,
        } => {
            push_u8(h, 83);
            push_u32(h, *frame);
            push_u8(h, *owner);
            push_u64(h, candidates.len() as u64);
            for def in candidates {
                push_u32(h, u32::from(*def));
            }
            push_u8(h, u8::from(*asked));
            push_u32(h, u32::from(*cursor));
            push_u64(h, done.len() as u64);
            for (owner, def) in done {
                push_u8(h, *owner);
                push_u32(h, u32::from(*def));
            }
            push_u32(h, owners.len() as u32);
            for owner in owners {
                push_u8(h, *owner);
            }
            push_u32(h, u32::from(*normal_owners));
        }
        M::Sha {
            card,
            source,
            target,
            nature,
            asked,
            armor_judged,
            dodges_used,
            tieji_red,
            liuli_done,
            liuli_stage,
            liuli_card,
            hujia_queue,
            hujia_holder,
            jiu_bonus,
            asking_qilin,
            qilin_horse,
            asking_hanbing,
            asking_fengxueren,
            asking_guanshi,
            guanshi_cards,
            asking_qinglong,
            asking_cixiong,
            cixiong_done,
            extra_targets,
            hanbing_pending,
            asking_xiandan,
            xiandan_done,
            xiandan_bonus,
            asking_lueying,
            lueying_victim,
            lueying_declined,
            asking_fuchou,
            fuchou_card,
            asking_sadeng,
            asking_sadeng_shan,
            shoulie_direct: _,
        } => {
            push_u8(h, 1);
            push_u32(h, card.unwrap_or(u32::MAX));
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *nature as u8);
            push_u8(h, *asked as u8);
            // 八卦阵判定标志入哈希（防篡改完整性）
            push_u8(h, *armor_judged as u8);
            // 无双已用闪数入哈希
            push_u8(h, *dodges_used as u8);
            // 铁骑判定结果入哈希
            push_u8(
                h,
                match tieji_red {
                    None => 0,
                    Some(false) => 1,
                    Some(true) => 2,
                },
            );
            // 流离/护驾状态入哈希（ADR 0774/0776 消除恢复分歧）
            push_u8(h, *liuli_done as u8);
            push_u8(h, *liuli_stage);
            match liuli_card {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, *c);
                }
            }
            // 护驾逐人询问队列入哈希
            match hujia_queue {
                None => push_u8(h, 0),
                Some(q) => {
                    push_u8(h, 1);
                    push_players(h, q);
                }
            }
            push_u8(h, hujia_holder.map(|p| p + 1).unwrap_or(0));
            // 酒效果入哈希（防篡改完整性）
            push_u8(h, (*jiu_bonus) as u8);
            // 麒麟弓弃马询问状态入哈希
            push_u8(h, (*asking_qilin) as u8);
            match qilin_horse {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, *c);
                }
            }
            // 方天画戟剩余目标队列入哈希
            push_u32(h, extra_targets.len() as u32);
            for c in extra_targets {
                push_u32(h, *c);
            }
            // 寒冰剑防止伤害确认状态入哈希
            push_u8(h, (*asking_hanbing) as u8);
            // 风雪刃翻面+摸牌询问状态入哈希（规则 v95）
            push_u8(h, (*asking_fengxueren) as u8);
            // 贯石斧弃二强伤询问状态入哈希
            push_u8(h, (*asking_guanshi) as u8);
            push_u32(h, guanshi_cards.len() as u32);
            for c in guanshi_cards {
                push_u32(h, *c);
            }
            // 青龙再杀询问状态入哈希
            push_u8(h, (*asking_qinglong) as u8);
            // 雌雄双股剑状态入哈希
            push_u8(h, (*asking_cixiong) as u8);
            push_u8(h, (*cixiong_done) as u8);
            // 寒冰剑目标自选弃牌剩余次数入哈希（逐技对照批一 v75）
            push_u8(h, *hanbing_pending);
            push_u8(h, *asking_xiandan as u8);
            push_u8(h, *xiandan_done as u8);
            push_u8(h, *xiandan_bonus as u8);
            push_u8(h, *asking_lueying as u8);
            match *lueying_victim {
                None => push_u8(h, 255),
                Some(t) => push_u8(h, t as u8),
            }
            push_u8(h, *lueying_declined as u8);
            push_u8(h, *asking_fuchou as u8);
            push_u8(h, fuchou_card.is_some() as u8);
            match fuchou_card {
                Some(c) => push_u32(h, *c),
                None => {}
            }
            push_u8(h, *asking_sadeng as u8);
            push_u8(h, *asking_sadeng_shan as u8);
        }
        M::Zhangba {
            owner,
            cards,
            stage,
        } => {
            push_u8(h, 61);
            push_u8(h, *owner);
            push_u32(h, cards.len() as u32);
            for c in cards {
                push_u32(h, *c);
            }
            push_u8(h, *stage as u8);
        }
        M::BossXiangde { owner } => {
            push_u8(h, 62);
            push_u8(h, *owner);
        }
        M::BossFutai { owner } => {
            push_u8(h, 63);
            push_u8(h, *owner);
        }
        M::BossLuanchang { owner } => {
            push_u8(h, 64);
            push_u8(h, *owner);
        }
        M::BossZhaohuo { owner } => {
            push_u8(h, 66);
            push_u8(h, *owner);
        }
        M::BossQizuo { owner } => {
            push_u8(h, 68);
            push_u8(h, *owner);
        }
        M::BossXiaoxiong { owner } => {
            push_u8(h, 69);
            push_u8(h, *owner);
        }
        M::BossLingbo { owner } => {
            push_u8(h, 70);
            push_u8(h, *owner);
        }
        M::BossTashui { owner } => {
            push_u8(h, 71);
            push_u8(h, *owner);
        }
        M::BossZjLeiji { owner } => {
            push_u8(h, 72);
            push_u8(h, *owner);
        }
        M::BossHeisheng { owner } => {
            push_u8(h, 73);
            push_u8(h, *owner);
        }
        M::BossLeizhu { owner, emitted } => {
            push_u8(h, 74);
            push_u8(h, *owner);
            push_u8(h, *emitted);
        }
        M::BossRenao { owner, emitted } => {
            push_u8(h, 75);
            push_u8(h, *owner);
            push_u8(h, *emitted);
        }
        M::BossWanghun { owner } => {
            push_u8(h, 76);
            push_u8(h, *owner);
        }
        M::BossZhuxin { owner, emitted } => {
            push_u8(h, 77);
            push_u8(h, *owner);
            push_u8(h, *emitted);
        }
        M::BossPingdeng { owner, emitted } => {
            push_u8(h, 78);
            push_u8(h, *owner);
            push_u8(h, *emitted);
        }
        M::BossHuoxing { owner, emitted } => {
            push_u8(h, 79);
            push_u8(h, *owner);
            push_u8(h, *emitted);
        }
        M::BossBingfeng { owner } => {
            push_u8(h, 80);
            push_u8(h, *owner);
        }
        M::BossWangxiang { owner } => {
            push_u8(h, 81);
            push_u8(h, *owner);
        }
        M::Rvm {
            owner,
            module,
            paused,
            skill_name,
            event,
            consume,
            count_hits,
            commit_pending,
            commit_pick,
            following,
            commit_state,
            committing,
        } => {
            push_u8(h, 82);
            push_u8(h, *owner);
            push_u32(h, module.len() as u32);
            h.update(module);
            match paused {
                Some(p) => {
                    push_u8(h, 1);
                    push_u32(h, p.frames.len() as u32);
                    h.update(&p.frames);
                    push_u32(h, p.pc);
                    push_u32(h, p.fuel_remaining);
                    push_u64(h, p.id);
                    h.update(&p.module_hash);
                    push_u32(h, p.options.len() as u32);
                    for t in &p.options {
                        push_u32(h, *t);
                    }
                    push_u32(h, p.pending.len() as u32);
                    h.update(&p.pending);
                }
                None => push_u8(h, 0),
            }
            // B7（规则 v324）：技能身份/触发语境/限次/命中旗标入摘要——
            // 此前两个仅声明技能名或限次清单不同的挂起机得到相同摘要。
            push_u32(h, skill_name.len() as u32);
            h.update(skill_name.as_bytes());
            push_event_data(h, event);
            push_u32(h, consume.len() as u32);
            for (key, scope) in consume {
                push_u32(h, key.len() as u32);
                h.update(key.as_bytes());
                push_u8(h, *scope);
            }
            push_u8(h, *count_hits as u8);
            // J1（规则 v328）：提交期挂起字段入摘要——两个仅剩余队列或开窗
            // 参数不同的提交期挂起机必须可区分（GT-011 同款身份要求）。
            push_u32(h, commit_pending.len() as u32);
            h.update(commit_pending);
            match commit_pick {
                None => push_u8(h, 0),
                Some((player, count)) => {
                    push_u8(h, 1);
                    push_u8(h, *player);
                    push_u32(h, *count);
                }
            }
            push_u32(h, following.len() as u32);
            for next in following {
                push_machine(h, next);
            }
            push_u32(h, commit_state.len() as u32);
            for value in commit_state {
                push_u32(h, *value);
            }
            push_u8(h, *committing as u8);
        }
        M::BossBuchun {
            owner,
            stage,
            targets,
        } => {
            push_u8(h, 67);
            push_u8(h, *owner);
            push_u8(h, *stage);
            for t in targets {
                push_u8(h, *t as u8);
            }
            push_u8(h, 0);
        }
        M::BossXingxia {
            owner,
            stage,
            actor,
        } => {
            push_u8(h, 68);
            push_u8(h, *owner);
            push_u8(h, *stage);
            push_u32(h, *actor);
        }
        M::Shedu { owner } => {
            push_u8(h, 65);
            push_u8(h, *owner);
        }
        M::DyingSave {
            dying,
            source,
            responders,
            asked,
            round_used,
            guihan_asked: _,
        } => {
            push_u8(h, 2);
            push_u8(h, *dying);
            push_u8(h, source.map(|p| p + 1).unwrap_or(0));
            push_players(h, responders);
            push_u8(h, *asked as u8);
            // 轮次用桃标志入哈希（防篡改完整性）
            push_u8(h, *round_used as u8);
        }
        M::Duel {
            card,
            source,
            target,
            on_target,
            asked,
            second_pending,
        } => {
            push_u8(h, 3);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *on_target as u8);
            push_u8(h, *asked as u8);
            push_u8(h, *second_pending as u8);
        }
        M::NullifyWindow {
            card,
            source,
            nullified,
            queue,
            asked,
            window_open,
            owns_disposal,
            effect,
        } => {
            push_u8(h, 4);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *nullified as u8);
            push_players(h, queue);
            push_u8(h, *asked as u8);
            push_u8(h, *window_open as u8);
            push_u8(h, *owns_disposal as u8);
            match effect {
                None => push_u8(h, 0),
                Some(e) => {
                    push_u8(h, 1);
                    push_machine(h, e);
                }
            }
        }
        M::JudgePhase {
            owner,
            queue,
            current,
        } => {
            push_u8(h, 5);
            push_u8(h, *owner);
            push_cards(h, queue);
            match current {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, c.trick.unwrap_or(u32::MAX));
                    push_u32(h, c.judge_card);
                    push_players(h, &c.rewriters);
                    if c.rewrite_provider.is_some() || !c.used_rewrite_providers.is_empty() {
                        push_u8(h, 255);
                        push_u16(h, c.rewrite_provider.unwrap_or(u16::MAX));
                        for (p, d) in &c.used_rewrite_providers {
                            push_u8(h, *p);
                            push_u16(h, *d);
                        }
                    }
                    push_u8(h, c.asked as u8);
                    push_players(h, &c.tian_du);
                    push_u8(h, c.gained as u8);
                    push_u8(h, c.in_rewrite as u8);
                    // 判定确定/机器持牌标志入哈希：防篡改完整性（N08-d）。
                    push_u8(h, c.finalized as u8);
                    push_u8(h, c.judge_held as u8);
                }
            }
        }
        M::Noop => push_u8(h, 18),
        M::Aoe {
            card,
            source,
            need,
            targets,
            idx,
            inner,
            nature,
        } => {
            push_u8(h, 6);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *need as u8);
            push_players(h, targets);
            push_usize(h, *idx);
            match inner {
                None => push_u8(h, 0),
                Some(m) => {
                    push_u8(h, 1);
                    push_machine(h, m);
                }
            }
            push_u8(h, *nature as u8);
        }
        M::AoeTarget {
            source,
            target,
            need,
            asked,
            card,
            nature,
        } => {
            push_u8(h, 15);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *need as u8);
            push_u8(h, *asked as u8);
            push_u32(h, *card);
            push_u8(h, *nature as u8);
        }
        M::Chain {
            card,
            source,
            targets,
            idx,
            inner,
            dispatcher,
        } => {
            push_u8(h, 16);
            push_u32(h, *card);
            push_u8(h, *source);
            push_players(h, targets);
            push_usize(h, *idx);
            match inner {
                None => push_u8(h, 0),
                Some(m) => {
                    push_u8(h, 1);
                    push_machine(h, m);
                }
            }
            push_u8(h, *dispatcher as u8);
        }
        M::PlaceDelayed {
            card,
            target,
            placed,
        } => {
            push_u8(h, 17);
            push_u32(h, *card);
            push_u8(h, *target);
            push_u8(h, *placed as u8);
        }
        M::DiscardToLimit {
            player,
            need,
            asked,
        } => {
            push_u8(h, 7);
            push_u8(h, *player);
            push_usize(h, *need);
            push_u8(h, *asked as u8);
        }
        M::Death { player, killer } => {
            push_u8(h, 8);
            push_u8(h, *player);
            push_u8(h, killer.map(|p| p + 1).unwrap_or(0));
        }
        M::OptionalGate {
            owner,
            def,
            event,
            asked,
            inner,
        } => {
            push_u8(h, 9);
            push_u8(h, *owner);
            push_u16(h, *def);
            push_event_data(h, event);
            push_u8(h, *asked as u8);
            match inner {
                None => push_u8(h, 0),
                Some(m) => {
                    push_u8(h, 1);
                    push_machine(h, m);
                }
            }
        }
        M::SanCai {
            owner,
            stage,
            target,
        } => {
            push_u8(h, 10);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
        }
        M::YiShang { owner, stage } => {
            push_u8(h, 11);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::SuiDun => push_u8(h, 12),
        M::SuQing { target } => {
            push_u8(h, 13);
            push_u8(h, *target);
        }
        M::ShouYi { owner } => {
            push_u8(h, 14);
            push_u8(h, *owner);
        }
        M::FanKui {
            owner,
            source,
            asked,
        } => {
            push_u8(h, 15);
            push_u8(h, *owner);
            push_u8(h, source.map(|p| p + 1).unwrap_or(0));
            push_u8(h, *asked as u8);
        }
        M::JianXiong { owner, card, asked } => {
            push_u8(h, 21);
            push_u8(h, *owner);
            push_u32(h, *card);
            push_u8(h, *asked as u8);
        }
        M::BiYue { owner } => {
            push_u8(h, 23);
            push_u8(h, *owner);
        }
        M::JiZhi { owner } => {
            push_u8(h, 25);
            push_u8(h, *owner);
        }
        M::KuRou { owner, stage } => {
            push_u8(h, 26);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::ZhiHeng { owner, stage } => {
            push_u8(h, 27);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::LuoShen { owner } => {
            push_u8(h, 24);
            push_u8(h, *owner);
        }
        M::YingZi { owner } => {
            push_u8(h, 28);
            push_u8(h, *owner);
        }
        M::MonkeySteal {
            card,
            user,
            holders,
            asked,
        } => {
            push_u8(h, 44);
            push_u32(h, *card);
            push_u8(h, *user);
            push_players(h, holders);
            push_u8(h, *asked as u8);
        }
        M::Dismantle {
            card,
            source,
            target,
            asked,
        } => {
            push_u8(h, 29);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *asked as u8);
        }
        M::XiaoJi { owner } => {
            push_u8(h, 29);
            push_u8(h, *owner);
        }
        M::QingNang {
            owner,
            stage,
            healed,
        } => {
            push_u8(h, 30);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, healed.map(|p| p + 1).unwrap_or(0));
        }
        M::JiJi {
            owner,
            stage,
            target,
            given,
        } => {
            push_u8(h, 22);
            push_u8(h, *owner);
            push_u8(
                h,
                match stage {
                    JiJiStageSnap::Draw => 0,
                    JiJiStageSnap::Target => 1,
                    JiJiStageSnap::Retarget => 2,
                    JiJiStageSnap::GiveCard => 3,
                    JiJiStageSnap::GiveChooseCard => 4,
                },
            );
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, *given);
        }
        M::XiaoGuo {
            owner,
            target,
            stage,
        } => {
            push_u8(h, 45);
            push_u8(h, *owner);
            push_u8(h, *target);
            push_u8(h, *stage as u8);
        }
        M::KuangFu { owner, target } => {
            push_u8(h, 46);
            push_u8(h, *owner);
            push_u8(h, *target);
        }
        M::YaoWu {
            owner,
            source,
            stage,
        } => {
            push_u8(h, 47);
            push_u8(h, *owner);
            push_u8(h, *source);
            push_u8(h, *stage as u8);
        }
        M::ShuShen {
            owner,
            target,
            stage,
        } => {
            push_u8(h, 48);
            push_u8(h, *owner);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, *stage as u8);
        }
        M::ShenZhi { owner } => {
            push_u8(h, 49);
            push_u8(h, *owner);
        }
        M::XunXun { owner, stage, top } => {
            push_u8(h, 50);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u32(h, top.len() as u32);
            for c in top {
                push_u32(h, *c);
            }
        }
        M::WangXi { owner, other } => {
            push_u8(h, 51);
            push_u8(h, *owner);
            push_u8(h, *other);
        }
        M::JiJie { owner, card, stage } => {
            push_u8(h, 52);
            push_u8(h, *owner);
            match card {
                None => push_u8(h, 0),
                Some(c) => {
                    push_u8(h, 1);
                    push_u32(h, *c);
                }
            }
            push_u8(h, *stage as u8);
        }
        M::JiYuan { owner } => {
            push_u8(h, 53);
            push_u8(h, *owner);
        }
        M::WangZun { owner, target } => {
            push_u8(h, 54);
            push_u8(h, *owner);
            push_u8(h, *target);
        }
        M::TongJi { owner, stage } => {
            push_u8(h, 55);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::Wuzhong { owner } => {
            push_u8(h, 56);
            push_u8(h, *owner);
        }
        M::Taoyuan { owner } => {
            push_u8(h, 57);
            push_u8(h, *owner);
        }
        M::Wugu {
            owner,
            cards,
            pickers,
        } => {
            push_u8(h, 58);
            push_u8(h, *owner);
            push_u32(h, cards.len() as u32);
            for c in cards {
                push_u32(h, *c);
            }
            push_u32(h, pickers.len() as u32);
            for p in pickers {
                push_u8(h, *p);
            }
        }
        M::Jiedao {
            owner,
            holder,
            victim,
            stage,
        } => {
            push_u8(h, 59);
            push_u8(h, *owner);
            push_u8(h, *holder);
            push_u8(h, *victim);
            push_u8(h, *stage as u8);
        }
        M::GangLie {
            owner,
            source,
            asked,
        } => {
            push_u8(h, 31);
            push_u8(h, *owner);
            push_u8(h, *source);
            push_u8(h, *asked as u8);
        }
        M::TuXi { owner, stage } => {
            push_u8(h, 32);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
        }
        M::LuoYi { owner } => {
            push_u8(h, 33);
            push_u8(h, *owner);
        }
        M::GuanXing { owner, stage, top } => {
            push_u8(h, 34);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            for c in top {
                push_u32(h, *c);
            }
        }
        M::LianYing { owner } => {
            push_u8(h, 35);
            push_u8(h, *owner);
        }
        M::Rende {
            owner,
            stage,
            cards,
        } => {
            push_u8(h, 36);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            for c in cards {
                push_u32(h, *c);
            }
        }
        M::Jijiang {
            owner,
            stage,
            queue,
            helper,
            sha,
            inner,
        } => {
            push_u8(h, 37);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            // 逐人询问队列入哈希
            match queue {
                None => push_u8(h, 0),
                Some(q) => {
                    push_u8(h, 1);
                    push_players(h, q);
                }
            }
            push_u8(h, helper.map(|p| p + 1).unwrap_or(0));
            push_u8(h, sha.map(|c| (c % 251) as u8 + 1).unwrap_or(0));
            push_u8(h, inner.is_some() as u8);
        }
        M::Host {
            def_key,
            behavior,
            stage,
            owner,
            target,
            card,
            cards,
            targets,
        } => {
            push_u8(h, 42);
            // 定义键入哈希（有界）：逐字节 FNV-1a
            let mut hv: u64 = 0xcbf2_9ce4_8422_2325;
            for b in def_key.as_bytes() {
                hv ^= *b as u64;
                hv = hv.wrapping_mul(0x1000_0000_01b3);
            }
            push_u64(h, hv);
            push_u16(h, *behavior);
            push_u8(h, *stage);
            push_u8(h, *owner);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, card.map(|c| (c % 251) as u8 + 1).unwrap_or(0));
            for c in cards {
                push_u32(h, *c);
            }
            for p in targets {
                push_u8(h, *p);
            }
        }
        M::Snatch {
            card,
            source,
            target,
            asked,
        } => {
            push_u8(h, 38);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
            push_u8(h, *asked as u8);
        }
        M::JieYin {
            owner,
            stage,
            cards,
        } => {
            push_u8(h, 39);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            for c in cards {
                push_u32(h, *c);
            }
        }
        M::FanJian {
            owner,
            stage,
            target,
            chosen_suit,
        } => {
            push_u8(h, 40);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, target.map(|p| p + 1).unwrap_or(0));
            push_u8(h, chosen_suit.map(|t| t + 1).unwrap_or(0));
        }
        M::LiJian {
            owner,
            stage,
            card,
            inner,
        } => {
            push_u8(h, 41);
            push_u8(h, *owner);
            push_u8(h, *stage as u8);
            push_u8(h, card.map(|c| (c % 251) as u8 + 1).unwrap_or(0));
            push_u8(h, inner.is_some() as u8);
        }
    }
}
