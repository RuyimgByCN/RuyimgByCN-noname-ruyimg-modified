use super::*;

pub(super) fn enc_machine(w: &mut W, m: &MachineSnapshot) {
    match m {
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
            w.u32(*card);
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
        MachineSnapshot::BossXiangde { owner } => {
            w.u8(61);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossFutai { owner } => {
            w.u8(62);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLuanchang { owner } => {
            w.u8(63);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZhaohuo { owner } => {
            w.u8(65);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossBuchun {
            owner,
            stage,
            targets,
        } => {
            w.u8(66);
            enc_pid(w, owner);
            w.u8(*stage);
            w.u32(targets.len() as u32);
            for t in targets {
                w.u32(*t);
            }
        }
        MachineSnapshot::BossXingxia {
            owner,
            stage,
            actor,
        } => {
            w.u8(67);
            enc_pid(w, owner);
            w.u8(*stage);
            w.u32(*actor);
        }
        MachineSnapshot::BossQizuo { owner } => {
            w.u8(68);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossXiaoxiong { owner } => {
            w.u8(69);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLingbo { owner } => {
            w.u8(70);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossTashui { owner } => {
            w.u8(71);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZjLeiji { owner } => {
            w.u8(72);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossHeisheng { owner } => {
            w.u8(73);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossLeizhu { owner, emitted } => {
            w.u8(74);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossRenao { owner, emitted } => {
            w.u8(75);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossWanghun { owner } => {
            w.u8(76);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossZhuxin { owner, emitted } => {
            w.u8(77);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossPingdeng { owner, emitted } => {
            w.u8(78);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossHuoxing { owner, emitted } => {
            w.u8(79);
            enc_pid(w, owner);
            w.u8(*emitted);
        }
        MachineSnapshot::BossBingfeng { owner } => {
            w.u8(80);
            enc_pid(w, owner);
        }
        MachineSnapshot::BossWangxiang { owner } => {
            w.u8(81);
            enc_pid(w, owner);
        }
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
        }
        MachineSnapshot::Shedu { owner } => {
            w.u8(64);
            enc_pid(w, owner);
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
        MachineSnapshot::JudgePhase {
            owner,
            queue,
            current,
        } => {
            w.u8(4);
            enc_pid(w, owner);
            w.len(queue.len());
            for c in queue {
                w.u32(*c);
            }
            match current {
                Some(c) => {
                    w.u8(1);
                    enc_judgement_progress(w, c);
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
        MachineSnapshot::SanCai {
            owner,
            stage,
            target,
        } => {
            w.u8(13);
            enc_pid(w, owner);
            w.u8(match stage {
                SanCaiStageSnap::Draw => 0,
                SanCaiStageSnap::Target => 1,
                SanCaiStageSnap::GiveCard => 2,
                SanCaiStageSnap::Finish => 3,
            });
            enc_opt_pid(w, target);
        }
        MachineSnapshot::YiShang { owner, stage } => {
            w.u8(14);
            enc_pid(w, owner);
            w.u8(match stage {
                YiShangStageSnap::Target => 0,
                YiShangStageSnap::Apply => 1,
            });
        }
        MachineSnapshot::SuiDun => w.u8(15),
        MachineSnapshot::SuQing { target } => {
            w.u8(16);
            enc_pid(w, target);
        }
        MachineSnapshot::ShouYi { owner } => {
            w.u8(17);
            enc_pid(w, owner);
        }
        MachineSnapshot::FanKui {
            owner,
            source,
            asked,
        } => {
            w.u8(18);
            enc_pid(w, owner);
            enc_opt_pid(w, source);
            w.boolean(*asked);
        }
        MachineSnapshot::JianXiong { owner, card, asked } => {
            w.u8(19);
            enc_pid(w, owner);
            w.u32(*card);
            w.boolean(*asked);
        }
        MachineSnapshot::BiYue { owner } => {
            w.u8(21);
            enc_pid(w, owner);
        }
        MachineSnapshot::JiZhi { owner } => {
            w.u8(25);
            enc_pid(w, owner);
        }
        MachineSnapshot::ZhiHeng { owner, stage } => {
            w.u8(27);
            enc_pid(w, owner);
            w.u8(match stage {
                ZhiHengStageSnap::Select => 0,
                ZhiHengStageSnap::Draw => 1,
            });
        }
        MachineSnapshot::KuRou { owner, stage } => {
            w.u8(26);
            enc_pid(w, owner);
            w.u8(match stage {
                KuRouStageSnap::Lose => 0,
                KuRouStageSnap::Draw => 1,
            });
        }
        MachineSnapshot::LuoShen { owner } => {
            w.u8(22);
            enc_pid(w, owner);
        }
        MachineSnapshot::YingZi { owner } => {
            w.u8(28);
            enc_pid(w, owner);
        }
        MachineSnapshot::Dismantle {
            card,
            source,
            target,
            asked,
        } => {
            w.u8(29);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*asked);
        }
        MachineSnapshot::MonkeySteal {
            card,
            user,
            holders,
            asked,
        } => {
            // E03 猴子偷桃：使用前取消窗口（tag 44，表尾追加）
            w.u8(44);
            w.u32(*card);
            enc_pid(w, user);
            enc_pids(w, holders);
            w.boolean(*asked);
        }
        MachineSnapshot::XiaoJi { owner } => {
            // tag 43（表尾追加）：tag 29 曾与 Dismantle 冲突（解码分支
            // 不可达，枭姬挂起无法恢复），2026-09-24 经 A0 唯一性检查
            // 定位后修正；旧字节中 XiaoJi 挂起本就无法正确解码。
            w.u8(43);
            enc_pid(w, owner);
        }
        MachineSnapshot::QingNang {
            owner,
            stage,
            healed,
        } => {
            w.u8(30);
            enc_pid(w, owner);
            w.u8(match stage {
                QingNangStageSnap::TargetAsk => 0,
                QingNangStageSnap::TargetRead => 1,
                QingNangStageSnap::CardRead => 2,
            });
            enc_opt_pid(w, healed);
        }
        MachineSnapshot::JiJi {
            owner,
            stage,
            target,
            given,
        } => {
            w.u8(20);
            enc_pid(w, owner);
            w.u8(match stage {
                JiJiStageSnap::Draw => 0,
                JiJiStageSnap::Target => 1,
                JiJiStageSnap::Retarget => 2,
                JiJiStageSnap::GiveCard => 3,
                JiJiStageSnap::GiveChooseCard => 4,
            });
            enc_opt_pid(w, target);
            w.u8(*given);
        }
        MachineSnapshot::GangLie {
            owner,
            source,
            asked,
        } => {
            w.u8(31);
            enc_pid(w, owner);
            enc_pid(w, source);
            w.u8(*asked as u8);
        }
        MachineSnapshot::XiaoGuo {
            owner,
            target,
            stage,
        } => {
            w.u8(45);
            enc_pid(w, owner);
            enc_pid(w, target);
            w.u8(match stage {
                crate::snap::XiaoGuoStageSnap::Ask => 0,
                crate::snap::XiaoGuoStageSnap::Read => 1,
            });
        }
        MachineSnapshot::KuangFu { owner, target } => {
            w.u8(46);
            enc_pid(w, owner);
            enc_pid(w, target);
        }
        MachineSnapshot::YaoWu {
            owner,
            source,
            stage,
        } => {
            w.u8(47);
            enc_pid(w, owner);
            enc_pid(w, source);
            w.u8(match stage {
                crate::snap::YaoWuStageSnap::Ask => 0,
                crate::snap::YaoWuStageSnap::Read => 1,
            });
        }
        MachineSnapshot::ShuShen {
            owner,
            target,
            stage,
        } => {
            w.u8(48);
            enc_pid(w, owner);
            enc_opt_pid(w, target);
            w.u8(match stage {
                ShuShenStageSnap::Ask => 0,
                ShuShenStageSnap::Read => 1,
            });
        }
        MachineSnapshot::ShenZhi { owner } => {
            w.u8(49);
            enc_pid(w, owner);
        }
        MachineSnapshot::XunXun { owner, stage, top } => {
            w.u8(50);
            enc_pid(w, owner);
            w.u8(match stage {
                XunXunStageSnap::Undo => 0,
                XunXunStageSnap::Arrange => 1,
            });
            w.len(top.len());
            for c in top {
                w.u32(*c);
            }
        }
        MachineSnapshot::WangXi { owner, other } => {
            w.u8(51);
            enc_pid(w, owner);
            enc_pid(w, other);
        }
        MachineSnapshot::JiJie { owner, card, stage } => {
            w.u8(52);
            enc_pid(w, owner);
            match card {
                None => w.u8(0),
                Some(c) => {
                    w.u8(1);
                    w.u32(*c);
                }
            }
            w.u8(match stage {
                JiJieStageSnap::Ask => 0,
                JiJieStageSnap::Read => 1,
            });
        }
        MachineSnapshot::JiYuan { owner } => {
            w.u8(53);
            enc_pid(w, owner);
        }
        MachineSnapshot::WangZun { owner, target } => {
            w.u8(54);
            enc_pid(w, owner);
            enc_pid(w, target);
        }
        MachineSnapshot::TongJi { owner, stage } => {
            w.u8(55);
            enc_pid(w, owner);
            w.u8(match stage {
                TongJiStageSnap::Ask => 0,
                TongJiStageSnap::Read => 1,
            });
        }
        MachineSnapshot::Wuzhong { owner } => {
            w.u8(56);
            enc_pid(w, owner);
        }
        MachineSnapshot::Taoyuan { owner } => {
            w.u8(57);
            enc_pid(w, owner);
        }
        MachineSnapshot::Wugu {
            owner,
            cards,
            pickers,
        } => {
            w.u8(58);
            enc_pid(w, owner);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            w.len(pickers.len());
            for p in pickers {
                enc_pid(w, p);
            }
        }
        MachineSnapshot::Jiedao {
            owner,
            holder,
            victim,
            stage,
        } => {
            w.u8(59);
            enc_pid(w, owner);
            enc_pid(w, holder);
            enc_pid(w, victim);
            w.u8(match stage {
                JiedaoStageSnap::Ask => 0,
                JiedaoStageSnap::Read => 1,
            });
        }
        MachineSnapshot::TuXi { owner, stage } => {
            w.u8(32);
            enc_pid(w, owner);
            w.u8(match stage {
                TuXiStageSnap::Ask => 0,
                TuXiStageSnap::Read => 1,
            });
        }
        MachineSnapshot::LuoYi { owner } => {
            w.u8(33);
            enc_pid(w, owner);
        }
        MachineSnapshot::GuanXing { owner, stage, top } => {
            w.u8(34);
            enc_pid(w, owner);
            w.u8(match stage {
                GuanXingStageSnap::Arrange => 0,
                GuanXingStageSnap::Place => 1,
            });
            w.len(top.len());
            for c in top {
                w.u32(*c);
            }
        }
        MachineSnapshot::LianYing { owner } => {
            w.u8(35);
            enc_pid(w, owner);
        }
        MachineSnapshot::Rende {
            owner,
            stage,
            cards,
        } => {
            w.u8(36);
            enc_pid(w, owner);
            w.u8(match stage {
                RendeStageSnap::SelectCardsAsk => 0,
                RendeStageSnap::TargetAsk => 1,
                RendeStageSnap::TargetRead => 2,
            });
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        MachineSnapshot::Jijiang {
            owner,
            stage,
            queue,
            helper,
            sha,
            inner,
        } => {
            w.u8(37);
            enc_pid(w, owner);
            w.u8(match stage {
                JijiangStageSnap::HelperAsk => 0,
                JijiangStageSnap::TargetAsk => 1,
            });
            // 逐人询问队列：None=0，Some=1+len+pid 序列
            match queue {
                None => w.u8(0),
                Some(q) => {
                    w.u8(1);
                    w.len(q.len());
                    for p in q {
                        enc_pid(w, p);
                    }
                }
            }
            enc_opt_pid(w, helper);
            enc_opt_u32(w, *sha);
            match inner {
                Some(m) => {
                    w.u8(1);
                    enc_machine(w, m);
                }
                None => w.u8(0),
            }
        }
        MachineSnapshot::Snatch {
            card,
            source,
            target,
            asked,
        } => {
            w.u8(38);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
            w.boolean(*asked);
        }
        MachineSnapshot::JieYin {
            owner,
            stage,
            cards,
        } => {
            w.u8(39);
            enc_pid(w, owner);
            w.u8(match stage {
                JieYinStageSnap::CardsAsk => 0,
                JieYinStageSnap::TargetAsk => 1,
                JieYinStageSnap::TargetRead => 2,
            });
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        MachineSnapshot::FanJian {
            owner,
            stage,
            target,
            chosen_suit,
        } => {
            w.u8(40);
            enc_pid(w, owner);
            w.u8(match stage {
                FanJianStageSnap::TargetAsk => 0,
                FanJianStageSnap::SuitAsk => 1,
                FanJianStageSnap::PickAsk => 2,
                FanJianStageSnap::Consume => 3,
            });
            match *chosen_suit {
                None => w.u8(0),
                Some(t) => {
                    w.u8(1);
                    w.u8(t);
                }
            }
            enc_opt_pid(w, target);
        }
        MachineSnapshot::LiJian {
            owner,
            stage,
            card,
            inner,
        } => {
            w.u8(41);
            enc_pid(w, owner);
            w.u8(match stage {
                LiJianStageSnap::DiscardAsk => 0,
                LiJianStageSnap::TargetsAsk => 1,
                LiJianStageSnap::Launch => 2,
            });
            enc_opt_u32(w, *card);
            match inner {
                Some(m) => {
                    w.u8(1);
                    enc_machine(w, m);
                }
                None => w.u8(0),
            }
        }
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
