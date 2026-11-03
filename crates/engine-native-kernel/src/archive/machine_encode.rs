use super::damage::enc_native_damage_skill;
use super::*;

// R05-F2 行数门禁拆分：按机族把编码臂外移子模块（字节序与公开接口零
// 变化；分派臂在此保持编译期穷尽检查）。
mod boss;
mod skills;

pub(super) fn enc_machine(w: &mut W, m: &MachineSnapshot) {
    match m {
        MachineSnapshot::JudgeContinuation { .. } | MachineSnapshot::JudgePhase { .. } => {
            super::judge::enc_judge_machine(w, m)
        }

        MachineSnapshot::NativeInvocation {
            owner,
            def,
            window,
            progress,
        } => {
            w.u8(86);
            w.u8(*owner);
            w.u16(*def);
            w.u8(window.tag());
            super::invocation::enc_progress(w, progress);
        }
        MachineSnapshot::NativeDamageSkill { owner, def, window } => {
            enc_native_damage_skill(w, *owner, *def, *window)
        }
        MachineSnapshot::DrawPhase { root } => {
            // R05：tag 90。
            w.u8(90);
            w.u32(*root);
        }
        MachineSnapshot::Draw { root } => {
            // R05：tag 91。
            w.u8(91);
            w.u32(*root);
        }
        MachineSnapshot::Gain { root } => {
            // R05：tag 92。
            w.u8(92);
            w.u32(*root);
        }
        MachineSnapshot::DamageFlow { root } => {
            w.u8(84);
            w.u32(*root);
        }
        MachineSnapshot::TriggerOrder {
            frame,
            owner,
            candidates,
            asked,
            cursor,
            done,
            owners,
            normal_owners,
        } => {
            w.u8(83);
            w.u32(*frame);
            enc_pid(w, owner);
            w.len(candidates.len());
            for def in candidates {
                w.u16(*def);
            }
            w.boolean(*asked);
            w.u16(*cursor);
            w.len(done.len());
            for (owner, def) in done {
                enc_pid(w, owner);
                w.u16(*def);
            }
            w.len(owners.len());
            for owner in owners {
                enc_pid(w, owner);
            }
            w.u16(*normal_owners);
        }
        MachineSnapshot::Sha {
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
            asking_sadeng,
            asking_sadeng_shan,
            shoulie_direct,
            asking_lueying,
            lueying_victim,
            lueying_declined,
            asking_fuchou,
            fuchou_card,
        } => {
            w.u8(0);
            w.u32(card.unwrap_or(u32::MAX));
            enc_pid(w, source);
            enc_pid(w, target);
            enc_damage_nature(w, nature);
            w.boolean(*asked);
            w.boolean(*armor_judged);
            w.u8(*dodges_used);
            w.u8(match tieji_red {
                None => 0,
                Some(false) => 1,
                Some(true) => 2,
            });
            // N05（ADR 0774/0776）：流离/护驾状态
            w.boolean(*liuli_done);
            w.u8(*liuli_stage);
            match liuli_card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            // 护驾逐人询问队列：None=0，Some=1+len+pid 序列
            match hujia_queue {
                None => w.u8(0),
                Some(q) => {
                    w.u8(1);
                    w.len(q.len());
                    for p in q {
                        enc_pid(w, p);
                    }
                }
            }
            enc_opt_pid(w, hujia_holder);
            w.boolean(*jiu_bonus);
            w.boolean(*asking_qilin);
            match qilin_horse {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.boolean(*asking_hanbing);
            w.boolean(*asking_fengxueren);
            w.boolean(*asking_guanshi);
            w.len(guanshi_cards.len());
            for c in guanshi_cards {
                w.u32(*c);
            }
            w.boolean(*asking_qinglong);
            w.boolean(*asking_cixiong);
            w.boolean(*cixiong_done);
            w.len(extra_targets.len());
            for c in extra_targets {
                w.u32(*c);
            }
            w.u8(*hanbing_pending);
            w.boolean(*asking_xiandan);
            w.boolean(*xiandan_done);
            w.boolean(*xiandan_bonus);
            w.boolean(*asking_lueying);
            match lueying_victim {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.boolean(*lueying_declined);
            w.boolean(*asking_fuchou);
            match fuchou_card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            // v75（swd 卡牌面窗口机批）：萨登荆环两询问挂起旗（尾部追加）
            w.boolean(*asking_sadeng);
            w.boolean(*asking_sadeng_shan);
            // v78（swd deferred 消解批 95）：狩猎不可闪避旗（尾部追加）
            w.boolean(*shoulie_direct);
        }
        MachineSnapshot::Zhangba {
            owner,
            cards,
            stage,
        } => {
            w.u8(60);
            enc_pid(w, owner);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.u8(match stage {
                ZhangbaStageSnap::Cards => 0,
                ZhangbaStageSnap::Target => 1,
                ZhangbaStageSnap::TargetRead => 2,
            });
        }
        // Boss 机族（拆至 machine_encode/boss.rs；字节序零变化）
        MachineSnapshot::BossXiangde { .. }
            | MachineSnapshot::BossFutai { .. }
            | MachineSnapshot::BossLuanchang { .. }
            | MachineSnapshot::BossZhaohuo { .. }
            | MachineSnapshot::BossBuchun { .. }
            | MachineSnapshot::BossXingxia { .. }
            | MachineSnapshot::BossQizuo { .. }
            | MachineSnapshot::BossXiaoxiong { .. }
            | MachineSnapshot::BossLingbo { .. }
            | MachineSnapshot::BossTashui { .. }
            | MachineSnapshot::BossZjLeiji { .. }
            | MachineSnapshot::BossHeisheng { .. }
            | MachineSnapshot::BossLeizhu { .. }
            | MachineSnapshot::BossRenao { .. }
            | MachineSnapshot::BossWanghun { .. }
            | MachineSnapshot::BossZhuxin { .. }
            | MachineSnapshot::BossPingdeng { .. }
            | MachineSnapshot::BossHuoxing { .. }
            | MachineSnapshot::BossBingfeng { .. }
            | MachineSnapshot::BossWangxiang { .. }
            | MachineSnapshot::Shedu { .. } => boss::enc(w, m),
        MachineSnapshot::Rvm {
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
            w.u8(82);
            enc_pid(w, owner);
            w.bytes(module);
            match paused {
                Some(p) => {
                    w.u8(1);
                    w.bytes(&p.frames);
                    w.u32(p.pc);
                    w.u32(p.fuel_remaining);
                    w.u64(p.id);
                    w.bytes(&p.module_hash);
                    w.u32(p.options.len() as u32);
                    for t in &p.options {
                        w.u32(*t);
                    }
                    w.bytes(&p.pending);
                }
                None => w.u8(0),
            }
            // B7（规则 v324）：技能身份/触发语境/限次/命中旗标（尾部追加——
            // 旧格式存档在此读完即止，新解码器读到 EOF 拒绝）。
            w.bytes(skill_name.as_bytes());
            enc_event_data(w, event);
            w.u32(consume.len() as u32);
            for (key, scope) in consume {
                w.bytes(key.as_bytes());
                w.u8(*scope);
            }
            w.boolean(*count_hits);
            // J1（规则 v328）：提交期挂起（尾部追加——旧格式档在此读完即止）。
            w.bytes(commit_pending);
            match commit_pick {
                None => w.u8(0),
                Some((player, count)) => {
                    w.u8(1);
                    w.u8(*player);
                    w.u32(*count);
                }
            }
            w.u32(following.len() as u32);
            for next in following {
                enc_machine(w, next);
            }
            w.u32(commit_state.len() as u32);
            for value in commit_state {
                w.u32(*value);
            }
            w.boolean(*committing);
        }
        MachineSnapshot::DyingSave {
            dying,
            source,
            responders,
            asked,
            round_used,
            guihan_asked,
        } => {
            w.u8(1);
            enc_pid(w, dying);
            enc_opt_pid(w, source);
            enc_pids(w, responders);
            w.boolean(*asked);
            w.boolean(*round_used);
            w.boolean(*guihan_asked);
        }
        MachineSnapshot::Duel {
            card,
            source,
            target,
            on_target,
            asked,
            second_pending,
        } => {
            w.u8(2);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*on_target);
            w.boolean(*asked);
            w.boolean(*second_pending);
        }
        MachineSnapshot::NullifyWindow {
            card,
            source,
            nullified,
            queue,
            asked,
            window_open,
            owns_disposal,
            effect,
        } => {
            w.u8(3);
            w.u32(*card);
            enc_pid(w, source);
            w.boolean(*nullified);
            enc_pids(w, queue);
            w.boolean(*asked);
            w.boolean(*window_open);
            w.boolean(*owns_disposal);
            match effect {
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
                None => w.u8(0),
            }
        }

        MachineSnapshot::Noop => w.u8(5),
        MachineSnapshot::PlaceDelayed {
            card,
            target,
            placed,
        } => {
            w.u8(6);
            w.u32(*card);
            enc_pid(w, target);
            w.boolean(*placed);
        }
        MachineSnapshot::Aoe {
            card,
            source,
            need,
            targets,
            idx,
            inner,
            nature,
        } => {
            w.u8(7);
            w.u32(*card);
            enc_pid(w, source);
            // need: CardKind 按数值编码
            w.u32(card_kind_to_num(need));
            w.len(targets.len());
            for t in targets {
                w.u8(*t);
            }
            w.u64(*idx as u64);
            match inner {
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
                None => w.u8(0),
            }
            // AOE 伤害属性（批⑦）
            w.u8(*nature as u8);
        }
        MachineSnapshot::AoeTarget {
            source,
            target,
            need,
            asked,
            card,
            nature,
        } => {
            w.u8(8);
            enc_pid(w, source);
            enc_pid(w, target);
            w.u32(card_kind_to_num(need));
            w.boolean(*asked);
            w.u32(*card);
            w.u8(*nature as u8);
        }
        MachineSnapshot::Chain {
            card,
            source,
            targets,
            idx,
            inner,
            dispatcher,
        } => {
            w.u8(9);
            w.u32(*card);
            enc_pid(w, source);
            w.len(targets.len());
            for t in targets {
                w.u8(*t);
            }
            w.u64(*idx as u64);
            match inner {
                None => w.u8(0),
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
            }
            w.boolean(*dispatcher);
        }
        MachineSnapshot::DiscardToLimit {
            player,
            need,
            asked,
        } => {
            w.u8(10);
            enc_pid(w, player);
            w.u64(*need as u64);
            w.boolean(*asked);
        }
        MachineSnapshot::Death { player, killer } => {
            w.u8(11);
            enc_pid(w, player);
            enc_opt_pid(w, killer);
        }
        MachineSnapshot::OptionalGate {
            owner,
            def,
            event,
            asked,
            inner,
        } => {
            w.u8(12);
            enc_pid(w, owner);
            enc_skill(w, def);
            enc_event_data(w, event);
            w.boolean(*asked);
            match inner {
                Some(e) => {
                    w.u8(1);
                    enc_machine(w, e);
                }
                None => w.u8(0),
            }
        }
        // 经典技能机族（拆至 machine_encode/skills.rs；字节序零变化）
        MachineSnapshot::SuiDun
            | MachineSnapshot::SanCai { .. }
            | MachineSnapshot::YiShang { .. }
            | MachineSnapshot::SuQing { .. }
            | MachineSnapshot::ShouYi { .. }
            | MachineSnapshot::FanKui { .. }
            | MachineSnapshot::JianXiong { .. }
            | MachineSnapshot::BiYue { .. }
            | MachineSnapshot::JiZhi { .. }
            | MachineSnapshot::ZhiHeng { .. }
            | MachineSnapshot::KuRou { .. }
            | MachineSnapshot::LuoShen { .. }
            | MachineSnapshot::YingZi { .. }
            | MachineSnapshot::Dismantle { .. }
            | MachineSnapshot::MonkeySteal { .. }
            | MachineSnapshot::XiaoJi { .. }
            | MachineSnapshot::QingNang { .. }
            | MachineSnapshot::JiJi { .. }
            | MachineSnapshot::GangLie { .. }
            | MachineSnapshot::XiaoGuo { .. }
            | MachineSnapshot::KuangFu { .. }
            | MachineSnapshot::YaoWu { .. }
            | MachineSnapshot::ShuShen { .. }
            | MachineSnapshot::ShenZhi { .. }
            | MachineSnapshot::XunXun { .. }
            | MachineSnapshot::WangXi { .. }
            | MachineSnapshot::JiJie { .. }
            | MachineSnapshot::JiYuan { .. }
            | MachineSnapshot::WangZun { .. }
            | MachineSnapshot::TongJi { .. }
            | MachineSnapshot::Wuzhong { .. }
            | MachineSnapshot::Taoyuan { .. }
            | MachineSnapshot::Wugu { .. }
            | MachineSnapshot::Jiedao { .. }
            | MachineSnapshot::TuXi { .. }
            | MachineSnapshot::LuoYi { .. }
            | MachineSnapshot::GuanXing { .. }
            | MachineSnapshot::LianYing { .. }
            | MachineSnapshot::Rende { .. }
            | MachineSnapshot::Jijiang { .. }
            | MachineSnapshot::Snatch { .. }
            | MachineSnapshot::JieYin { .. }
            | MachineSnapshot::FanJian { .. }
            | MachineSnapshot::LiJian { .. } => skills::enc(w, m),
        MachineSnapshot::Host {
            def_key,
            behavior,
            stage,
            owner,
            target,
            card,
            cards,
            targets,
        } => {
            w.u8(42);
            // 定义键：有界字符串（解码侧限长 64）
            w.str(def_key);
            w.u16(*behavior);
            w.u8(*stage);
            enc_pid(w, owner);
            enc_opt_pid(w, target);
            enc_opt_u32(w, *card);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.len(targets.len());
            for p in targets {
                enc_pid(w, p);
            }
        }
    }
}
