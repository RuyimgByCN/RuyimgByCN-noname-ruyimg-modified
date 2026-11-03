use super::*;

// ---- 顶层快照 ----

pub(super) fn enc_snapshot(w: &mut W, s: &StateSnapshot) {
    // core
    w.u64(s.core.revision);
    w.u64(s.core.ruleset_version);
    match &s.core.mode {
        GameMode::Prototype => w.u8(0),
        GameMode::Identity { roles } => {
            w.u8(1);
            w.len(roles.len());
            for r in roles {
                enc_role(w, r);
            }
        }
        GameMode::Kingdoms => w.u8(2),
        GameMode::Doudizhu { lord } => {
            w.u8(4);
            enc_pid(w, lord);
        }
        GameMode::Boss {
            boss,
            chongzheng,
            fellows,
            stage,
        } => {
            w.u8(5);
            enc_pid(w, boss);
            w.u8(*chongzheng);
            w.len(fellows.len());
            for f in fellows {
                enc_pid(w, f);
            }
            // v79：BossStageState
            enc_boss_stage(w, stage);
        }
        GameMode::Kof { pools, wins, games } => {
            w.u8(6);
            for p in pools {
                w.len(p.len());
                for g in p {
                    w.u16(*g);
                }
            }
            w.u32(wins[0]);
            w.u32(wins[1]);
            w.u32(*games);
        }
        GameMode::Versus { teams, substitutes } => {
            w.u8(3);
            w.len(teams.len());
            for t in teams {
                w.len(t.len());
                for p in t {
                    enc_pid(w, p);
                }
            }
            // 替补武将池（v206；与队伍同队序，可为空）
            w.len(substitutes.len());
            for pool in substitutes {
                w.len(pool.len());
                for g in pool {
                    w.u16(*g);
                }
            }
        }
    }
    // 座位环（T19b/v46）：恒等排列亦显式编码（简单且自校验）
    w.len(s.core.seat_ring.len());
    for pid in &s.core.seat_ring {
        w.u8(*pid);
    }
    w.len(s.core.players.len());
    for p in &s.core.players {
        w.boolean(p.alive);
        w.i32(p.hp);
        w.u32(p.max_hp);
        w.len(p.hand.len());
        for c in &p.hand {
            w.u32(*c);
        }
        w.len(p.equips.len());
        for c in &p.equips {
            w.u32(*c);
        }
        w.len(p.judge.len());
        for c in &p.judge {
            w.u32(*c);
        }
        w.boolean(p.dying);
        w.boolean(p.linked);
        // 国战双将槽 + 明置状态（随玩家编码）
        for slot in 0..2 {
            w.opt_u16(p.general_slots[slot]);
        }
        let extended = p.kingdom_identity != crate::state::KingdomIdentity::Unknown
            || !p.yexinjia_friends.is_empty();
        w.u8(u8::from(p.revealed_generals[0]) | if extended { 2 } else { 0 });
        w.boolean(p.revealed_generals[1]);
        if extended {
            match &p.kingdom_identity {
                crate::state::KingdomIdentity::Unknown => w.u8(0),
                crate::state::KingdomIdentity::Ye => w.u8(1),
                crate::state::KingdomIdentity::Declared(name) => {
                    w.u8(2);
                    w.str(name);
                }
            }
            enc_pids(w, &p.yexinjia_friends);
        }
        // 五行属性（G1 wuxing）
        w.opt_u8(p.wunature);
        w.opt_u8(p.ctianfu_source);
        match p.ctianfu_card {
            None => w.u8(0),
            Some(v) => {
                w.u8(1);
                w.u32(v);
            }
        }
        w.u32(p.gouhunluo_rounds);
        w.opt_u8(p.gouhunluo_source);
        // 护甲（G1 扩展）
        w.u8(p.hujia);
        // E03 翻面原语（v4）：武将牌朝向
        w.boolean(p.face_down);
        w.u8(p.boss_chongzheng);
        // 筑城标记（v5，U07 刘馥）：跨回合持久标记牌
        w.len(p.zhucheng.len());
        for c in &p.zhucheng {
            w.u32(*c);
        }
        // 机关怨（v43，U05 swd 第二十一批）：待获得牌实体
        w.len(p.jiguanyuan_cards.len());
        for c in &p.jiguanyuan_cards {
            w.u32(*c);
        }
        // 灵剑状态（v44，U05 swd 机关术批）：冷冻/嘲讽/星体改造/隐秘
        w.boolean(p.jileng_active);
        w.boolean(p.chaofeng_active);
        w.u8(p.xingtigaizao_count);
        w.boolean(p.yinmi_active);
        // 卡牌实例改造表（v45，机关术批/ADR 0780）：len+（装备,附着灵剑）
        w.len(p.card_mods.len());
        for (card, attached) in &p.card_mods {
            w.u32(*card);
            w.u32(*attached);
        }
        // 炮台等级（v6，U05 ow 托比昂）：跨回合持久
        w.u8(p.paotai_lv);
        w.u8(p.baolei_mode);
        // 暴雪/急冻（v8，U05 ow 美）：limited 已发动 + 冰封标记
        w.boolean(p.baoxue_used);
        w.boolean(p.guihan_used);
        // 雪煌 limited（swd 芝茵，批 26）
        w.boolean(p.xuehuang_used);
        // 雪霁冰雹标记（swd 卡牌面补切批②）
        w.u8(p.xuejibingbao);
        // 炼妖壶收纳仓（swd 卡牌面补切批③）
        w.len(p.lianyaohu_store.len());
        for c in &p.lianyaohu_store {
            w.u32(*c);
        }
        // 逆攻标记（swd 龟灵之桃，批⑧）
        w.u8(p.nigong_lv);
        w.boolean(p.jidong_active);
        // 强固（v10，U05 ow 奥丽莎）：生效标记
        w.boolean(p.qianggu_active);
        // 屏障（v12，U05 ow 查莉娅）：两标志+上回合次数
        w.boolean(p.pingzhang_self_used);
        w.boolean(p.pingzhang_other_used);
        w.u8(p.pingzhang_count);
        // 潜行（v13，U05 ow 黑影）：生效标记
        w.boolean(p.yinshen_active);
        // 机甲血（v15，U05 ow D.Va）
        w.u8(p.jijia_hp);
        // 风雅颂计数器（v17，U05 xianjian 雷元戈）
        w.u8(p.feng_count);
        w.u8(p.ya_count);
        w.u8(p.song_count);
        // 蛊舞剩余回合数（v21，U05 xianjian 阿奴）
        w.u8(p.anwugu_count);
        // 龙火已觉醒（v23，U05 xianjian 姜城）
        w.boolean(p.jiangcheng_longhuo);
        // 月行对偶（v25，U05 xianjian 龙幽）
        w.opt_u8(p.yuexing_pair);
        // 明绣三字段（v26，U05 xianjian 明绣：回合伤害/灵火/归烬/跳摸）
        w.u8(p.turn_damage);
        w.u32(p.linghuo_ready_round);
        w.u32(p.guijin_ready_round);
        w.boolean(p.skip_next_draw);
        // 贯日已发动（v27，U05 xianjian 云天河）
        w.boolean(p.guanri_used);
        // 离场区+水云计数+忘忧戳（v28，U05 xianjian 紫萱）
        w.len(p.off_court.len());
        for c in &p.off_court {
            w.u32(*c);
        }
        w.u8(p.shuiyun_count);
        w.u32(p.wangyou3_round);
        // 小蛮三字段（v29，U05 xianjian 小蛮：全局用牌/双觉醒）
        w.u32(p.total_used);
        w.boolean(p.lingquan_awakened);
        w.boolean(p.shenwu_awakened);
        // 诚信免伤+冷却（v30，U05 xianjian 明绣）
        w.boolean(p.chengxin_active);
        w.u32(p.chengxin_ready_round);
        // 虎煞标记（v32，U05 xianjian 王小虎）
        w.u8(p.husha_mark);
        w.u8(p.hyunshen_count);
        w.u8(p.daixing_count);
        w.u8(p.juxi_count);
        w.u8(p.pozhou_count);
        w.len(p.sxianjing_traps.len());
        for c in &p.sxianjing_traps {
            w.u32(*c);
        }
        // 死亡拦截+适盈一次性（v33，U05 xianjian 月琦）
        w.boolean(p.death_cancelled);
        w.boolean(p.shiying_used);
        // 花好一次性（v34，U05 xianjian 苏媚）
        w.boolean(p.huahu_used);
        // 玄砚计数（v35，U05 xianjian 慕容紫英）
        w.u32(p.xuanning_mark);
        // 蛇毒标记（v223）
        w.u8(p.shedu);
        // 巨源记录（v226）
        w.i32(p.juyuan_storage);
        // 封印（v228）
        w.boolean(p.fengyin_active);
        // 灵渠计数（v237）
        w.u8(p.lingqu_count);
        // 星囚交替开关（v239）
        w.boolean(p.xingqiu_toggled);
        // 反噬 terra 旗（v269）
        w.boolean(p.zlfanshi_terra);
        // 往事封禁（v270）
        w.boolean(p.wangshi_ban);
        // 复仇标记（玩点论杀批 14）
        w.len(p.fuchou_marked.len());
        for src in &p.fuchou_marked {
            w.u8(*src);
        }
        // 不臣上次发动轮次（v240）
        w.u32(p.buchun_used_round);
        // 星夏上次发动轮次（v242）
        w.u32(p.xingxia_used_round);
        // 形态轮换交替旗标（v249）
        w.boolean(p.nbianshen_toggle);
        // 混乱标记（v60，Boss 郭嘉鬼谋）
        w.boolean(p.boss_mad);
        // 锦鳞一次性发动（v87）
        w.boolean(p.jinlin_used);
        // hlss 星系统（v88）：星数/计数/已受伤类型位图
        w.u8(p.hlss_stars);
        w.u8(p.hlss_count);
        w.u8(p.hlss_damage);
        // 起风已用牌种表（v89）
        w.len(p.hlss_fengqi_used.len());
        for k in &p.hlss_fengqi_used {
            w.u32(u32::from(*k));
        }
        // 潜龙道心值（v90）
        w.u8(p.mbqianlong_marks);
        // 涅槃限定技（v91）
        w.boolean(p.niepan_used);
        // 神焰限定技 + 待蔓延目标（v93）
        w.boolean(p.shenyan_used);
        w.len(p.shenyan_pending.len());
        for t in &p.shenyan_pending {
            w.u8(*t);
        }
        // 入梦跳过弃牌阶段（v94）
        w.boolean(p.skip_discard_phase);
        // 义山最近失去的牌（v95）
        w.len(p.yishan_lost.len());
        for c in &p.yishan_lost {
            w.u32(*c);
        }
        // 守印限定位（v96）
        w.boolean(p.shouyin_used);
        // 颂词已发动目标（v97）
        w.len(p.songci_targets.len());
        for t in &p.songci_targets {
            w.u8(*t);
        }
        // 极略已用次数（v99）
        w.u8(p.jilve_used);
        // 天书两字段（v101）
        match p.swdtianshu_from {
            None => w.u8(0),
            Some(t) => {
                w.u8(1);
                w.u8(t);
            }
        }
        match p.swdtianshu_granted {
            None => w.u8(0),
            Some(d) => {
                w.u8(1);
                w.u16(d);
            }
        }
        // 脉轮三字段（v100）
        w.u8(p.mailun_mode);
        w.boolean(p.mailun_out_used);
        w.boolean(p.mailun_in_used);
        // 护天四字段（v98）
        match p.hutian_deposit {
            None => w.u8(0),
            Some(t) => {
                w.u8(1);
                w.u8(t);
            }
        }
        w.boolean(p.hutian_just_given);
        w.len(p.hutian_hold.len());
        for c in &p.hutian_hold {
            w.u32(*c);
        }
        match p.hutian_owner {
            None => w.u8(0),
            Some(t) => {
                w.u8(1);
                w.u8(t);
            }
        }
    }
    let _ = &s.core.turn;
    w.len(s.core.deck.len());
    for c in &s.core.deck {
        w.u32(*c);
    }
    w.len(s.core.discard.len());
    for c in &s.core.discard {
        w.u32(*c);
    }

    w.len(s.core.table.len());
    for c in &s.core.table {
        w.u32(*c);
    }
    // R05：detached 实体区暂不入档——随存档 107 与 D 机接入同批尾追
    // （此时尚无生产者，106 布局保持逐字节不变）。
    // rng
    let (rng_state, rng_calls) = s.core.rng.raw_parts();
    w.u64(rng_state);
    w.u64(rng_calls);
    // turn
    let t = &s.core.turn;
    enc_pid(w, &t.current);
    enc_phase(w, &t.phase);
    w.u32(t.round);
    w.boolean(t.sha_used_play_phase);
    w.boolean(t.mufeng_used_basic);
    w.boolean(t.wangchen_discard_basic);
    // v75（swd 卡牌面窗口机批）：神秘果/鼠槌三旗（尾部追加）
    w.boolean(t.shenmiguo_used);
    w.boolean(t.shuchui_armed);
    w.boolean(t.shuchui_hit);
    // v76（swd 武将面批㉔）：连打每回合旗（尾部追加）
    w.boolean(t.lianda_used);
    w.boolean(t.land_used);
    w.boolean(t.land_sha_used);
    w.u32(t.discard_phase_discarded);
    w.len(t.ciqiu_dying.len());
    for p in &t.ciqiu_dying {
        w.u8(*p);
    }
    match t.kangyin2 {
        None => w.u8(0),
        Some((o, x)) => {
            w.u8(1);
            w.u8(o);
            w.u32(x);
        }
    }
    // 还侠跟踪（swd 屈娴，批 27）
    w.len(t.huanxia_track.len());
    for (c, caused) in &t.huanxia_track {
        w.u32(*c);
        w.boolean(*caused);
    }
    w.u32(t.cards_used_play_phase);
    // v92（swd 灵武批 120）：本回合已用牌数 + 额外阶段队列 + 灵武插入位
    w.u32(t.cards_used_turn);
    // 燎原计数（v103，swd 批 145）
    w.u8(t.liaoyuan_count);
    // 轩辕剑加伤标记（v104，swd 批 146）
    w.boolean(t.xuanyuanjian_marked);
    // 昆仑镜快照（v105，swd 批 149）：逐玩家（手牌/装备/判定区）+ 体力
    w.len(t.kunlunjing_snapshot.len());
    for (p, hand, equips, judge) in &t.kunlunjing_snapshot {
        w.u8(*p);
        w.len(hand.len());
        for c in hand {
            w.u32(*c);
        }
        w.len(equips.len());
        for c in equips {
            w.u32(*c);
        }
        w.len(judge.len());
        for c in judge {
            w.u32(*c);
        }
    }
    w.u32(t.kunlunjing_hp as u32);
    w.len(t.extra_phases.len());
    for p in &t.extra_phases {
        w.u8(*p);
    }
    w.boolean(t.extra_phase_lingwu);
    w.u32(t.bte_used_play_phase);
    w.u32(t.phase_number);
    w.u8(t.rende_given);
    w.boolean(t.jiu_used);
    w.boolean(t.jiu_active);
    w.boolean(t.turn_opened);
    w.boolean(t.skip_draw_phase);
    w.boolean(t.skip_play_phase);
    w.boolean(t.guxing_armed);
    w.u8(t.guxing_count);
    // 猛态双旗标（v248）
    w.boolean(t.mengtai_draw_pending);
    w.boolean(t.mengtai_use_pending);
    w.opt_u8(t.zhufangshenshi_target.map(|p| p as u8));
    // 乐动三角（U07 ow 卢西奥，规则 v126）：目标数/摸回量/回复模式
    w.u8(t.yuedong_num);
    w.u8(t.yuedong_eff);
    w.boolean(t.yuedong_recover);
    // 链钩（U05 ow 路霸，规则 v128）：被钩者/伤害增幅已消耗
    w.opt_u8(t.liangou_hook);
    w.boolean(t.liangou_bonus_used);
    // 跳跃（U05 ow 温斯顿，规则 v137）：目标
    w.opt_u8(t.feitiao_target);
    // 返灵（U05 xianjian 瑕，规则 v145）：本回合已发动
    w.boolean(t.fanling_used);
    // 御风（U05 xianjian 李逍遥，规则 v151）：本回合已发动次数
    w.u8(t.xjyufeng_used);
    // 蝶舞（U05 xianjian 龙葵，规则 v153）：本回合已摸牌
    w.boolean(t.diewu_draw_used);
    // 掠影（U05 xianjian 慕长兰，规则 v154）：本回合已发动
    w.boolean(t.lueying_used);
    // 甜香去（U05 xianjian 唐雨柔，规则 v156）：本回合已发动
    w.boolean(t.txianqu_used);
    // 聚宝已回收（U05 xianjian 景天，规则 v158）：本回合已发动
    w.boolean(t.jtjubao_used);
    // 逐月目标（U05 xianjian 云天河，规则 v161）：本出牌阶段白名单
    w.len(t.zhuyue_targets.len());
    for p in &t.zhuyue_targets {
        w.u8(*p);
    }
    // 摄灵本回合次数（U05 xianjian 南宫煌，规则 v168）
    w.u8(t.sheling_used);
    // 射影本回合已发动（U05 xianjian 苏媚，规则 v175）
    w.boolean(t.sheying_used);
    // 张目本回合已发动（U05 xianjian 南宫煌，规则 v181）
    w.boolean(t.zhangmu_used);
    w.boolean(t.yingfeng_chain);
    w.boolean(t.mujiaren_used);
    match t.luanjian_card {
        None => w.u8(0),
        Some(v) => {
            w.u8(1);
            w.u32(v);
        }
    }
    // 冥顽锁定集（v219）
    w.len(t.minwan_targets.len());
    for p in &t.minwan_targets {
        enc_pid(w, p);
    }
    // 炎狱计数（v229）
    w.u8(t.yanyu_count);
    // 奇佐副本载体槽（v60，Boss 郭嘉）
    match t.qizuo_copy {
        None => w.u8(0),
        Some(v) => {
            w.u8(1);
            w.u32(v);
        }
    }
    // 极点链挡旗标（v61，Boss 张角）
    w.boolean(t.jidian_chain);
    // skills（BTreeMap：按键序确定性遍历）
    w.len(s.core.skills.len());
    for ((owner, def), inst) in &s.core.skills {
        enc_pid(w, owner);
        enc_skill(w, def);
        enc_pid(w, &inst.owner);
        w.boolean(inst.enabled);
        w.boolean(inst.once_used);
        w.boolean(inst.expires_end_of_turn);
        w.u8(inst.rounds);
    }
    // outcome
    // view-as 声明牌种表（BTreeMap 确定性遍历）
    // 装备栏改写（v102，swd 批 143 改造）
    w.len(s.core.declared_slots.len());
    for (card, slot) in &s.core.declared_slots {
        w.u32(*card);
        w.u8(match slot {
            crate::catalog::EquipSlot::Weapon => 0,
            crate::catalog::EquipSlot::Armor => 1,
            crate::catalog::EquipSlot::HorsePlus => 2,
            crate::catalog::EquipSlot::HorseMinus => 3,
            crate::catalog::EquipSlot::Treasure => 4,
        });
    }
    w.len(s.core.declared_kinds.len());
    for (card, kind) in &s.core.declared_kinds {
        w.u32(*card);
        w.u8(card_kind_to_num(kind) as u8);
    }
    // 五行属性表（G1 wuxing，BTreeMap 确定性遍历）
    w.len(s.core.card_natures.len());
    for (card, nature) in &s.core.card_natures {
        w.u32(*card);
        w.u8(*nature);
    }
    // 当前地（mtg land，规则 v100）
    match s.core.current_land {
        None => w.u8(0),
        Some(kind) => {
            w.u8(1);
            w.u8(card_kind_to_num(&kind) as u8);
        }
    }
    match &s.core.outcome {
        Some(o) => {
            w.u8(1);
            enc_winner(w, &o.winner);
        }
        None => w.u8(0),
    }
    enc_vm_aux(w, &s.core.vm_aux);
    // 会话数据
    w.u64(s.last_dedup);
    w.boolean(s.segment_open);
    // N11-a：令牌代次随快照延续
    w.u64(s.auth_generation);
    // resolution
    enc_resolution(w, &s.resolution);
    // 段检查点（递归；由 N04 校验保证不嵌套）
    match &s.segment_checkpoint {
        Some(c) => {
            w.u8(1);
            enc_snapshot(w, c);
        }
        None => w.u8(0),
    }
}
