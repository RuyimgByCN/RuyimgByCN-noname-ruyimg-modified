use super::*;
mod machine;
use machine::*;

/// PA03 缓冲摘要写入器：先拼缓冲再一次 blake3 update。blake3 流式与
/// 一次性对同一字节序列产出**相同摘要**，故字节序与既有 golden/跨目标
/// 对照完全一致；逐字段约 800 次小块 `Hasher::update` 比缓冲后一次
/// 慢约 1.4–1.7 倍（内核复核表 C 与哈希微基准）。
pub(super) struct HashSink {
    buf: Vec<u8>,
}

impl HashSink {
    fn with_state_capacity() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
        }
    }

    #[inline]
    fn update(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }
}

// ---------------------------------------------------------------------------
// 确定性摘要：blake3 编码全部状态字段。执行位置含开放选择、挂起元数据与
// 每台挂起机器的**规范快照**（MachineSnapshot 全字段——不同机器的局部状态
// 可区分）；摘要以 ruleset_version 开头（规则版本绑定：跨规则版本的哈希
// 不互认）。重放一致性以静止点哈希核对。
// ---------------------------------------------------------------------------

pub(super) fn push_vm_aux(h: &mut HashSink, a: &crate::state::VmAux) {
    if a.is_empty() {
        return;
    }
    push_u8(h, 0xA8);
    push_u32(h, a.marks.len() as u32);
    for ((p, id), n) in &a.marks {
        push_u8(h, *p);
        push_u32(h, *id);
        push_i32(h, *n);
    }
    push_u32(h, a.piles.len() as u32);
    for ((p, name), cards) in &a.piles {
        push_u8(h, *p);
        push_static_owned(h, name);
        push_u32(h, cards.len() as u32);
        for c in cards {
            push_u32(h, *c);
        }
    }
    push_u32(h, a.lost.len() as u32);
    for (p, cards) in &a.lost {
        push_u8(h, *p);
        push_u32(h, cards.len() as u32);
        for c in cards {
            push_u32(h, *c);
        }
    }
    push_u32(h, a.once.len() as u32);
    for (p, key, scope) in &a.once {
        push_u8(h, *p);
        push_static_owned(h, key);
        push_u8(h, *scope);
    }
    push_u32(h, a.flags.len() as u32);
    for ((p, key), n) in &a.flags {
        push_u8(h, *p);
        push_static_owned(h, key);
        push_u32(h, *n);
    }
    push_u32(h, a.hits.len() as u32);
    for ((p, key), n) in &a.hits {
        push_u8(h, *p);
        push_static_owned(h, key);
        push_u32(h, *n);
    }
    push_u32(h, a.drawn.len() as u32);
    for (p, n) in &a.drawn {
        push_u8(h, *p);
        push_u32(h, *n);
    }
    push_u32(h, a.storage.len() as u32);
    for (key, value) in &a.storage {
        push_static_owned(h, key);
        push_u32(h, *value);
    }
}

pub(super) fn push_static_owned(h: &mut HashSink, s: &str) {
    push_u32(h, s.len() as u32);
    h.update(s.as_bytes());
}

pub(super) fn canonical_hash(state: &GameState) -> [u8; 32] {
    let mut h = HashSink::with_state_capacity();
    let c = &state.core;
    // NC07：座位环（换位结果）进入规范摘要——此前未参与，两个仅座位次序不同
    // 的状态会得到相同摘要（漂移源）。
    push_u32(&mut h, c.seat_ring.len() as u32);
    for p in &c.seat_ring {
        push_u32(&mut h, *p as u32);
    }
    push_u64(&mut h, c.revision);
    push_u64(&mut h, c.ruleset_version);
    match &c.mode {
        GameMode::Prototype => push_u8(&mut h, 1),
        GameMode::Kingdoms => push_u8(&mut h, 3),
        GameMode::Versus { teams, substitutes } => {
            push_u8(&mut h, 4);
            push_u32(&mut h, teams.len() as u32);
            for t in teams {
                push_players(&mut h, t);
            }
            // 替补武将池（v206）入确定性摘要
            push_u32(&mut h, substitutes.len() as u32);
            for pool in substitutes {
                push_u32(&mut h, pool.len() as u32);
                for g in pool {
                    push_u32(&mut h, *g as u32);
                }
            }
        }
        GameMode::Doudizhu { lord } => {
            push_u8(&mut h, 5);
            push_u8(&mut h, *lord);
        }
        GameMode::Boss {
            boss,
            chongzheng,
            fellows,
            stage,
        } => {
            push_u8(&mut h, 6);
            push_u8(&mut h, *boss);
            push_u8(&mut h, *chongzheng);
            push_players(&mut h, fellows);
            // D4/B0：关卡流程态入确定性摘要（规则 v291）
            push_u8(&mut h, stage.kind as u8);
            push_u8(&mut h, stage.level);
            push_u8(&mut h, stage.mengpo_spawned as u8);
            push_u8(&mut h, stage.mengpo_died as u8);
            push_u32(&mut h, stage.banned_generals.len() as u32);
            for g in &stage.banned_generals {
                push_u16(&mut h, *g);
            }
            push_u8(&mut h, stage.wing_armed);
            // 浴火关卡循环型（批 118，规则 v306）
            push_u8(&mut h, stage.loop_type);
            match &stage.pending {
                None => push_u8(&mut h, 0),
                Some(crate::state::BossStagePending::SelectGeneral { candidates }) => {
                    push_u8(&mut h, 1);
                    push_u32(&mut h, candidates.len() as u32);
                    for g in candidates {
                        push_u16(&mut h, *g);
                    }
                }
            }
        }
        GameMode::Kof { pools, wins, games } => {
            push_u8(&mut h, 7);
            for p in pools {
                push_u32(&mut h, p.len() as u32);
                for g in p {
                    push_u32(&mut h, *g as u32);
                }
            }
            push_u32(&mut h, wins[0]);
            push_u32(&mut h, wins[1]);
            push_u32(&mut h, *games);
        }
        GameMode::Identity { roles } => {
            push_u8(&mut h, 2);
            push_u32(&mut h, roles.len() as u32);
            for r in roles {
                push_u8(
                    &mut h,
                    match r {
                        crate::state::Role::Lord => 1,
                        crate::state::Role::Loyalist => 2,
                        crate::state::Role::Rebel => 3,
                        crate::state::Role::Traitor => 4,
                    },
                );
            }
        }
    }
    for p in &c.players {
        push_u8(&mut h, p.alive as u8);
        push_i32(&mut h, p.hp);
        push_u32(&mut h, p.max_hp);
        push_u8(&mut h, p.dying as u8);
        push_u8(&mut h, p.linked as u8);
        // 国战双将槽/明置状态入哈希（None 编码为 0）
        for slot in 0..2 {
            match p.general_slots[slot] {
                Some(g) => {
                    push_u8(&mut h, 1);
                    push_u16(&mut h, g);
                }
                None => push_u8(&mut h, 0),
            }
        }
        push_u8(&mut h, p.revealed_generals[0] as u8);
        push_u8(&mut h, p.revealed_generals[1] as u8);
        push_u8(&mut h, p.wunature.unwrap_or(255));
        // 天符挂附（规则 v188）
        push_u8(&mut h, p.ctianfu_source.unwrap_or(255));
        // 勾魂锣挂附（规则 v191）
        push_u32(&mut h, p.gouhunluo_rounds);
        push_u8(&mut h, p.gouhunluo_source.unwrap_or(255));
        push_u8(&mut h, p.ctianfu_card.map(|_| 1).unwrap_or(0));
        if let Some(v) = p.ctianfu_card {
            push_u32(&mut h, v);
        }
        push_u8(&mut h, p.hujia);
        // E03 翻面原语：武将牌朝向入摘要
        push_u8(&mut h, p.face_down as u8);
        push_u8(&mut h, p.boss_chongzheng);
        push_u32(&mut h, p.zhucheng.len() as u32);
        for c in &p.zhucheng {
            push_u32(&mut h, *c);
        }
        // NC07（规则 v325）：炼妖壶壶仓顺序入摘要——炼化选项与丢弃序按壶序
        // 生成，此前两种壶序摘要相同（GT-012 反例）。
        push_u32(&mut h, p.lianyaohu_store.len() as u32);
        for c in &p.lianyaohu_store {
            push_u32(&mut h, *c);
        }
        // NC07（规则 v327，裁决第 11 项）：字段审计所列 9 个「入档未入摘要」
        // 玩家字段全部入摘要（编码序与 state_encode.rs 一致）。它们都在规则
        // 路径被读取（目标合法性/摸牌数/手牌上限/装备改造/回合旗标），排除
        // 名单为空；结构守卫 nc07_archive_hash_guard 要求入档字段要么入摘要、
        // 要么登记排除理由。逐字段反例见 nc07_archive_hash_fields。
        push_u32(&mut h, p.jiguanyuan_cards.len() as u32);
        for c in &p.jiguanyuan_cards {
            push_u32(&mut h, *c);
        }
        push_u8(&mut h, p.jileng_active as u8);
        push_u8(&mut h, p.chaofeng_active as u8);
        push_u8(&mut h, p.xingtigaizao_count);
        push_u8(&mut h, p.yinmi_active as u8);
        push_u8(&mut h, p.xuehuang_used as u8);
        push_u8(&mut h, p.xuejibingbao);
        push_u8(&mut h, p.nigong_lv);
        push_u32(&mut h, p.card_mods.len() as u32);
        for (card, attached) in &p.card_mods {
            push_u32(&mut h, *card);
            push_u32(&mut h, *attached);
        }
        push_u8(&mut h, p.paotai_lv);
        push_u8(&mut h, p.baolei_mode);
        // 暴雪/急冻（U05 ow 美，规则 v127）
        push_u8(&mut h, p.baoxue_used as u8);
        push_u8(&mut h, p.guihan_used as u8);
        push_u8(&mut h, p.jidong_active as u8);
        // 强固（U05 ow 奥丽莎，规则 v136）
        push_u8(&mut h, p.qianggu_active as u8);
        // 屏障（U05 ow 查莉娅，规则 v138）
        push_u8(&mut h, p.pingzhang_self_used as u8);
        push_u8(&mut h, p.pingzhang_other_used as u8);
        push_u8(&mut h, p.pingzhang_count);
        // 潜行（U05 ow 黑影，规则 v139）
        push_u8(&mut h, p.yinshen_active as u8);
        // 机甲血（U05 ow D.Va，规则 v141）
        push_u8(&mut h, p.jijia_hp);
        // 风雅颂计数器（U05 xianjian 雷元戈，规则 v150）
        push_u8(&mut h, p.feng_count);
        push_u8(&mut h, p.ya_count);
        push_u8(&mut h, p.song_count);
        // 蛊舞剩余回合数（U05 xianjian 阿奴，规则 v154）
        push_u8(&mut h, p.anwugu_count);
        // 龙火已觉醒（U05 xianjian 姜城，规则 v157）
        push_u8(&mut h, p.jiangcheng_longhuo as u8);
        // 月行对偶（U05 xianjian 龙幽，规则 v159）
        push_u8(&mut h, p.yuexing_pair.map(|x| x + 1).unwrap_or(0));
        // 明绣（U05 xianjian 明绣，规则 v160）
        push_u8(&mut h, p.turn_damage);
        push_u32(&mut h, p.linghuo_ready_round);
        push_u32(&mut h, p.guijin_ready_round);
        push_u8(&mut h, p.skip_next_draw as u8);
        // 贯日（U05 xianjian 云天河，规则 v161）
        push_u8(&mut h, p.guanri_used as u8);
        // 离场区+水云计数+忘忧戳（U05 xianjian 紫萱，规则 v162）
        push_u32(&mut h, p.off_court.len() as u32);
        for c in &p.off_court {
            push_u32(&mut h, *c);
        }
        push_u8(&mut h, p.shuiyun_count);
        push_u32(&mut h, p.wangyou3_round);
        // 小蛮（U05 xianjian 小蛮，规则 v163）
        push_u32(&mut h, p.total_used);
        push_u8(&mut h, p.lingquan_awakened as u8);
        push_u8(&mut h, p.shenwu_awakened as u8);
        // 诚信（U05 xianjian 明绣，规则 v164）
        push_u8(&mut h, p.chengxin_active as u8);
        push_u32(&mut h, p.chengxin_ready_round);
        // 虎煞（U05 xianjian 王小虎，规则 v169）
        push_u8(&mut h, p.husha_mark);
        push_u8(&mut h, p.hyunshen_count);
        push_u8(&mut h, p.daixing_count);
        push_u8(&mut h, p.juxi_count);
        push_u8(&mut h, p.pozhou_count);
        push_u32(&mut h, p.sxianjing_traps.len() as u32);
        for c in &p.sxianjing_traps {
            push_u32(&mut h, *c);
        }
        // 适盈（U05 xianjian 月琦，规则 v171）
        push_u8(&mut h, p.death_cancelled as u8);
        push_u8(&mut h, p.shiying_used as u8);
        // 花好（U05 xianjian 苏媚，规则 v175）
        push_u8(&mut h, p.huahu_used as u8);
        // 玄砚计数（U05 xianjian 慕容紫英，规则 v179）
        push_u32(&mut h, p.xuanning_mark);
        push_u8(&mut h, p.shedu);
        push_u32(&mut h, p.juyuan_storage as u32);
        push_u8(&mut h, p.fengyin_active as u8);
        push_u8(&mut h, p.lingqu_count);
        push_u8(&mut h, p.xingqiu_toggled as u8);
        // 反噬 terra 旗（Boss 转轮王，规则 v269）
        push_u8(&mut h, p.zlfanshi_terra as u8);
        // 往事封禁（Boss 孟婆，规则 v270）
        push_u8(&mut h, p.wangshi_ban as u8);
        // 复仇标记（玩点论杀批 14）：跨回合来源列表
        push_u32(&mut h, p.fuchou_marked.len() as u32);
        for src in &p.fuchou_marked {
            push_u8(&mut h, *src);
        }
        push_u32(&mut h, p.buchun_used_round);
        push_u32(&mut h, p.xingxia_used_round);
        push_u8(&mut h, p.nbianshen_toggle as u8);
        // 混乱标记（Boss 郭嘉鬼谋，规则 v259）
        push_u8(&mut h, p.boss_mad as u8);
        // 锦鳞一次性发动（swd 司空蔷，规则 v302）
        push_u8(&mut h, p.jinlin_used as u8);
        // hlss 星系统（T11 Boss 诸葛亮族，规则 v303）
        push_u8(&mut h, p.hlss_stars);
        push_u8(&mut h, p.hlss_count);
        push_u8(&mut h, p.hlss_damage);
        // 起风已用牌种表（批 116，规则 v303 同批）
        push_u32(&mut h, p.hlss_fengqi_used.len() as u32);
        for k in &p.hlss_fengqi_used {
            push_u32(&mut h, u32::from(*k));
        }
        // 潜龙道心值（D5 批 117，规则 v305）
        push_u8(&mut h, p.mbqianlong_marks);
        // 涅槃限定技（T11 Boss 庞统族批 118，规则 v306）
        push_u8(&mut h, p.niepan_used as u8);
        // 神焰限定技 + 待蔓延目标（swd 批 121，规则 v308）
        push_u8(&mut h, p.shenyan_used as u8);
        push_u32(&mut h, p.shenyan_pending.len() as u32);
        for t in &p.shenyan_pending {
            push_u8(&mut h, *t);
        }
        // 入梦跳过弃牌阶段（swd 批 122，规则 v309）
        push_u8(&mut h, p.skip_discard_phase as u8);
        // 义山最近失去的牌（swd 批 123，规则 v310）
        push_u32(&mut h, p.yishan_lost.len() as u32);
        for c in &p.yishan_lost {
            push_u32(&mut h, *c);
        }
        // 守印限定位（swd 批 124，规则 v311）
        push_u8(&mut h, p.shouyin_used as u8);
        // 颂词已发动目标（swd 批 129，规则 v313）
        push_u32(&mut h, p.songci_targets.len() as u32);
        for t in &p.songci_targets {
            push_u8(&mut h, *t);
        }
        // 极略已用次数（swd 批 136，规则 v315）
        push_u8(&mut h, p.jilve_used);
        // 天书两字段（swd 批 142，规则 v317）
        match p.swdtianshu_from {
            None => push_u8(&mut h, 0),
            Some(t) => {
                push_u8(&mut h, 1);
                push_u8(&mut h, t);
            }
        }
        match p.swdtianshu_granted {
            None => push_u8(&mut h, 0),
            Some(d) => {
                push_u8(&mut h, 1);
                push_u16(&mut h, d);
            }
        }
        // 脉轮三字段（swd 批 137，规则 v316）
        push_u8(&mut h, p.mailun_mode);
        push_u8(&mut h, p.mailun_out_used as u8);
        push_u8(&mut h, p.mailun_in_used as u8);
        // 护天四字段（swd 批 131，规则 v314）
        match p.hutian_deposit {
            None => push_u8(&mut h, 0),
            Some(t) => {
                push_u8(&mut h, 1);
                push_u8(&mut h, t);
            }
        }
        push_u8(&mut h, p.hutian_just_given as u8);
        push_u32(&mut h, p.hutian_hold.len() as u32);
        for c in &p.hutian_hold {
            push_u32(&mut h, *c);
        }
        match p.hutian_owner {
            None => push_u8(&mut h, 0),
            Some(t) => {
                push_u8(&mut h, 1);
                push_u8(&mut h, t);
            }
        }
        push_cards(&mut h, &p.hand);
        push_cards(&mut h, &p.equips);
        push_cards(&mut h, &p.judge);
    }
    push_cards(&mut h, &c.deck);
    push_cards(&mut h, &c.discard);
    push_cards(&mut h, &c.table);
    let (rs, rc) = c.rng.state();
    push_u64(&mut h, rs);
    push_u64(&mut h, rc);
    push_u8(&mut h, c.turn.current);
    push_u8(&mut h, c.turn.phase as u8);
    push_u32(&mut h, c.turn.round);
    push_u8(&mut h, c.turn.sha_used_play_phase as u8);
    push_u8(&mut h, c.turn.mufeng_used_basic as u8);
    push_u8(&mut h, c.turn.wangchen_discard_basic as u8);
    push_u8(&mut h, c.turn.shenmiguo_used as u8);
    push_u8(&mut h, c.turn.shuchui_armed as u8);
    push_u8(&mut h, c.turn.shuchui_hit as u8);
    push_u8(&mut h, c.turn.lianda_used as u8);
    push_u8(&mut h, c.turn.rende_given);
    push_u8(&mut h, c.turn.jiu_used as u8);
    // 木甲人限次（规则 v190）
    push_u8(&mut h, c.turn.mujiaren_used as u8);
    push_u8(&mut h, c.turn.jiu_active as u8);
    push_u8(&mut h, c.turn.land_used as u8);
    push_u8(&mut h, c.turn.land_sha_used as u8);
    push_u32(&mut h, c.turn.discard_phase_discarded);
    push_u8(&mut h, c.turn.yanyu_count);
    // 奇佐副本载体槽（Boss 郭嘉，规则 v259）
    match c.turn.qizuo_copy {
        None => push_u8(&mut h, 0),
        Some(v) => {
            push_u8(&mut h, 1);
            push_u32(&mut h, v);
        }
    }
    // 极点链挡旗标（Boss 张角，规则 v263）
    push_u8(&mut h, c.turn.jidian_chain as u8);

    push_u32(&mut h, c.turn.ciqiu_dying.len() as u32);
    for p in &c.turn.ciqiu_dying {
        push_u8(&mut h, *p);
    }
    match c.turn.kangyin2 {
        None => push_u8(&mut h, 0),
        Some((o, x)) => {
            push_u8(&mut h, 1);
            push_u8(&mut h, o);
            push_u32(&mut h, x);
        }
    }
    push_u32(&mut h, c.turn.huanxia_track.len() as u32);
    for (card, caused) in &c.turn.huanxia_track {
        push_u32(&mut h, *card);
        push_u8(&mut h, *caused as u8);
    }
    push_u32(&mut h, c.turn.cards_used_play_phase);
    // swd 灵武批 120（规则 v307）：本回合已用牌数 + 额外阶段队列 + 插入位
    push_u32(&mut h, c.turn.cards_used_turn);
    // 燎原计数（swd 批 145，规则 v319）
    push_u8(&mut h, c.turn.liaoyuan_count);
    // 轩辕剑加伤标记（swd 批 146，规则 v320）
    push_u8(&mut h, c.turn.xuanyuanjian_marked as u8);
    // 昆仑镜快照（swd 批 149，规则 v321）
    push_u32(&mut h, c.turn.kunlunjing_snapshot.len() as u32);
    for (p, hand, equips, judge) in &c.turn.kunlunjing_snapshot {
        push_u8(&mut h, *p);
        for list in [hand, equips, judge] {
            push_u32(&mut h, list.len() as u32);
            for card in list {
                push_u32(&mut h, *card);
            }
        }
    }
    push_u32(&mut h, c.turn.kunlunjing_hp as u32);
    push_u32(&mut h, c.turn.extra_phases.len() as u32);
    for p in &c.turn.extra_phases {
        push_u8(&mut h, *p);
    }
    push_u8(&mut h, c.turn.extra_phase_lingwu as u8);
    push_u32(&mut h, c.turn.bte_used_play_phase);
    // NC07（规则 v327，裁决第 11 项）：悯婉目标集入摘要——伤害结算收集、
    // 事件分派按集合判读（审计所列唯一 TurnState 遗漏；反例见
    // nc07_archive_hash_fields::minwan_targets_enters_hash）。
    push_u32(&mut h, c.turn.minwan_targets.len() as u32);
    for p in &c.turn.minwan_targets {
        push_u8(&mut h, *p);
    }
    push_u32(&mut h, c.turn.phase_number);
    // 贴房绅视：本回合无视距离目标（规则 v94）
    match c.turn.zhufangshenshi_target {
        None => push_u8(&mut h, 0),
        Some(t) => {
            push_u8(&mut h, 1);
            push_u8(&mut h, t);
        }
    }
    push_u8(&mut h, c.turn.turn_opened as u8);
    push_u8(&mut h, c.turn.skip_draw_phase as u8);
    push_u8(&mut h, c.turn.skip_play_phase as u8);
    push_u8(&mut h, c.turn.guxing_armed as u8);
    push_u8(&mut h, c.turn.guxing_count);
    push_u8(&mut h, c.turn.mengtai_draw_pending as u8);
    push_u8(&mut h, c.turn.mengtai_use_pending as u8);
    // 乐动三角（U07 ow 卢西奥，规则 v126）
    push_u8(&mut h, c.turn.yuedong_num);
    push_u8(&mut h, c.turn.yuedong_eff);
    push_u8(&mut h, c.turn.yuedong_recover as u8);
    // 链钩（U05 ow 路霸，规则 v128）
    match c.turn.liangou_hook {
        None => push_u8(&mut h, 255),
        Some(t) => push_u8(&mut h, t),
    }
    push_u8(&mut h, c.turn.liangou_bonus_used as u8);
    // 跳跃（U05 ow 温斯顿，规则 v137）
    match c.turn.feitiao_target {
        None => push_u8(&mut h, 255),
        Some(t) => push_u8(&mut h, t),
    }
    // 返灵（U05 xianjian 瑕，规则 v145）
    push_u8(&mut h, c.turn.fanling_used as u8);
    // 御风（U05 xianjian 李逍遥，规则 v151）
    push_u8(&mut h, c.turn.xjyufeng_used);
    // 蝶舞（U05 xianjian 龙葵，规则 v153）
    push_u8(&mut h, c.turn.diewu_draw_used as u8);
    // 掠影（U05 xianjian 慕长兰，规则 v154）
    push_u8(&mut h, c.turn.lueying_used as u8);
    // 甜香去（U05 xianjian 唐雨柔，规则 v156）
    push_u8(&mut h, c.turn.txianqu_used as u8);
    // 聚宝（U05 xianjian 景天，规则 v158）
    push_u8(&mut h, c.turn.jtjubao_used as u8);
    // 逐月（U05 xianjian 云天河，规则 v161）
    push_u32(&mut h, c.turn.zhuyue_targets.len() as u32);
    for p in &c.turn.zhuyue_targets {
        push_u8(&mut h, *p);
    }
    // 摄灵（U05 xianjian 南宫煌，规则 v168）
    push_u8(&mut h, c.turn.sheling_used);
    // 射影（U05 xianjian 苏媚，规则 v175）
    push_u8(&mut h, c.turn.sheying_used as u8);
    // 张目（U05 xianjian 南宫煌，规则 v181）
    push_u8(&mut h, c.turn.zhangmu_used as u8);
    // 影锋链挡旗标（规则 v185）
    push_u8(&mut h, c.turn.yingfeng_chain as u8);
    // 乱剑载体锚定（规则 v187）
    match c.turn.luanjian_card {
        None => push_u8(&mut h, 0),
        Some(v) => {
            push_u8(&mut h, 1);
            push_u32(&mut h, v);
        }
    }
    for ((owner, def), inst) in &c.skills {
        push_u8(&mut h, *owner);
        push_u16(&mut h, *def);
        push_u8(&mut h, inst.enabled as u8);
        push_u8(&mut h, inst.once_used as u8);
        push_u8(&mut h, inst.expires_end_of_turn as u8);
        push_u8(&mut h, inst.rounds);
    }
    // view-as 声明牌种表入哈希（防篡改完整性，BTreeMap 键序确定性）
    for (card, slot) in &c.declared_slots {
        push_u32(&mut h, *card);
        push_u8(
            &mut h,
            match slot {
                crate::catalog::EquipSlot::Weapon => 0,
                crate::catalog::EquipSlot::Armor => 1,
                crate::catalog::EquipSlot::HorsePlus => 2,
                crate::catalog::EquipSlot::HorseMinus => 3,
                crate::catalog::EquipSlot::Treasure => 4,
            },
        );
    }
    for (card, kind) in &c.declared_kinds {
        push_u32(&mut h, *card);
        push_u8(&mut h, *kind as u8);
    }
    // 五行属性表入哈希（G1 wuxing，BTreeMap 键序确定性）
    for (card, nature) in &c.card_natures {
        push_u32(&mut h, *card);
        push_u8(&mut h, *nature);
    }
    // 当前地（mtg land，规则 v100）：仅编码牌种枚举序（与 archive 的
    // card_kind_to_num 同表，哈希口径只需引擎内自洽）
    match c.current_land {
        None => push_u8(&mut h, 0),
        Some(kind) => {
            push_u8(&mut h, 1);
            push_u32(&mut h, (kind as usize) as u32);
        }
    }
    match &c.outcome {
        None => push_u8(&mut h, 0),
        Some(o) => {
            push_u8(&mut h, 1);
            match o.winner {
                crate::state::Winner::Player(w) => {
                    push_u8(&mut h, 1);
                    push_u8(&mut h, w);
                }
                crate::state::Winner::LordSide => push_u8(&mut h, 2),
                crate::state::Winner::Rebels => push_u8(&mut h, 3),
                crate::state::Winner::Faction(f) => {
                    push_u8(&mut h, 4);
                    push_u8(&mut h, f.clone() as u8);
                }
                crate::state::Winner::Versus(t) => {
                    push_u8(&mut h, 6);
                    push_u8(&mut h, t as u8);
                }
                crate::state::Winner::Doudizhu { lord_won } => {
                    push_u8(&mut h, 7);
                    push_u8(&mut h, lord_won as u8);
                }
                crate::state::Winner::Boss { boss_won } => {
                    push_u8(&mut h, 8);
                    push_u8(&mut h, boss_won as u8);
                }
                crate::state::Winner::Traitor => push_u8(&mut h, 5),
            }
        }
    }
    // T8 原语面：空表不写字节，标准局面摘要与加字段前一致。
    push_vm_aux(&mut h, &c.vm_aux);
    let r = &state.resolution;
    push_u32(&mut h, r.next_event_id);
    push_u64(&mut h, r.next_choice_id);
    push_u32(&mut h, r.budget);
    for f in &r.stack {
        push_u32(&mut h, f.id);
        push_u32(&mut h, f.parent.unwrap_or(0));
        push_u8(&mut h, f.parent.is_none() as u8);
        push_u8(&mut h, f.kind as u8);
        push_u8(&mut h, f.stage as u8);
        push_u8(&mut h, f.planned as u8);
        push_u8(&mut h, f.intrinsic_done as u8);
        push_u32(&mut h, f.queue.len() as u32);
        for q in &f.queue {
            push_u16(&mut h, q.def);
            push_u8(&mut h, q.owner);
            push_u8(&mut h, q.order.0);
            push_i32(&mut h, q.order.1);
            push_u8(&mut h, q.order.2);
            push_u8(&mut h, q.order.3);
        }
        push_event_data(&mut h, &f.data);
    }
    match &r.open_choice {
        None => push_u8(&mut h, 0),
        Some(o) => {
            push_u8(&mut h, 1);
            push_u64(&mut h, o.id);
            push_u8(&mut h, o.observer);
            push_u8(&mut h, o.spec.kind as u8);
            push_static(&mut h, o.spec.prompt);
            push_u8(&mut h, o.spec.min as u8);
            push_u8(&mut h, o.spec.max as u8);
            push_u8(&mut h, o.spec.allow_cancel as u8);
            push_options(&mut h, &o.spec.options);
        }
    }
    match &r.answer {
        None => push_u8(&mut h, 0),
        Some(a) => {
            push_u8(&mut h, 1);
            push_options(&mut h, a);
        }
    }
    for p in &r.parks {
        push_usize(&mut h, p.resume_depth);
        match p.slot {
            crate::resolution::Slot::Intrinsic { frame } => {
                push_u8(&mut h, 1);
                push_u32(&mut h, frame);
            }
            crate::resolution::Slot::Trigger { frame } => {
                push_u8(&mut h, 2);
                push_u32(&mut h, frame);
            }
        }
        push_machine(&mut h, &p.machine.snapshot());
    }
    let mut hasher = Hasher::new();
    hasher.update(&h.buf);
    hasher.finalize().into()
}

pub(super) fn push_event_data(h: &mut HashSink, data: &EventData) {
    match data {
        EventData::Damage(d) => {
            push_u8(h, 1);
            push_u8(h, d.source.map(|s| s + 1).unwrap_or(0));
            push_u8(h, d.target);
            push_u32(h, d.amount);
            push_players(h, &d.redirect_chain);
            push_u8(h, d.prevented as u8);
            push_u8(h, d.nature as u8);
        }
        EventData::Recover {
            player,
            amount,
            by_card,
        } => {
            push_u8(h, 2);
            push_u8(h, *player);
            push_u32(h, *amount);
            push_u32(h, by_card.unwrap_or(u32::MAX));
        }
        EventData::CardsDrawn { player, cards } => {
            push_u8(h, 3);
            push_u8(h, *player);
            push_cards(h, cards);
        }
        EventData::CardsMoved { cards, from, to } => {
            push_u8(h, 4);
            push_cards(h, cards);
            push_u8(h, *from);
            push_u8(h, *to);
        }
        EventData::CardUsed(u) => {
            push_u8(h, 5);
            push_u32(h, u.card);
            push_u8(h, u.player);
            push_players(h, &u.targets);
            push_u8(h, u.nature as u8);
        }
        EventData::CardResponded { card, player } => {
            push_u8(h, 6);
            push_u32(h, *card);
            push_u8(h, *player);
        }
        EventData::Dying { player, source } => {
            push_u8(h, 7);
            push_u8(h, *player);
            push_u8(h, source.map(|s| s + 1).unwrap_or(0));
        }
        EventData::Death { player, killer } => {
            push_u8(h, 8);
            push_u8(h, *player);
            push_u8(h, killer.map(|k| k + 1).unwrap_or(0));
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            push_u8(h, 9);
            push_u8(h, *player);
            push_u8(h, *phase as u8);
            push_u8(h, *begin as u8);
        }
        EventData::TurnEnded { player } => {
            push_u8(h, 10);
            push_u8(h, *player);
        }
        EventData::TurnOverBefore { player, cancelled } => {
            push_u8(h, 20);
            push_u8(h, *player);
            push_u8(h, *cancelled as u8);
        }
        EventData::TurnOvered { player, face_down } => {
            push_u8(h, 21);
            push_u8(h, *player);
            push_u8(h, *face_down as u8);
        }
        EventData::LoseHp { player, amount } => {
            push_u8(h, 22);
            push_u8(h, *player);
            push_u32(h, *amount);
        }
        EventData::ShaDodged {
            card,
            source,
            target,
        } => {
            // swd 批 141（判别 16）
            push_u8(h, 16);
            push_u32(h, *card);
            push_u8(h, *source);
            push_u8(h, *target);
        }
        EventData::DyingSaved { player, source } => {
            push_u8(h, 23);
            push_u8(h, *player);
            push_u8(h, source.map(|k| k + 1).unwrap_or(0));
        }
        EventData::CardsGiven {
            giver,
            recipient,
            cards,
        } => {
            // NC04 未映射事件批（判别 24）
            push_u8(h, 24);
            push_u8(h, *giver);
            push_u8(h, *recipient);
            push_u32(h, cards.len() as u32);
            for c in cards {
                push_u32(h, *c);
            }
        }
        EventData::SkillToggled {
            player,
            skill,
            enabled,
            temporary,
        } => {
            push_u8(h, 11);
            push_u8(h, *player);
            push_u16(h, *skill);
            push_u8(h, *enabled as u8);
            push_u8(h, *temporary as u8);
        }
        EventData::Equipped {
            player,
            card,
            replaced,
        } => {
            push_u8(h, 12);
            push_u8(h, *player);
            push_u32(h, *card);
            push_u32(h, replaced.unwrap_or(u32::MAX));
        }
        EventData::Linked { player, linked } => {
            push_u8(h, 13);
            push_u8(h, *player);
            push_u8(h, *linked as u8);
        }
        EventData::CardsDiscarded { player, cards } => {
            push_u8(h, 14);
            push_u8(h, *player);
            push_cards(h, cards);
        }
        EventData::CardsLost {
            player,
            cards,
            origins,
        } => {
            // swd 批 125（判别 15）
            push_u8(h, 15);
            push_u8(h, *player);
            push_cards(h, cards);
            for o in origins {
                push_u8(
                    h,
                    match o {
                        crate::event::LostOrigin::Hand => 0,
                        crate::event::LostOrigin::Equip => 1,
                        crate::event::LostOrigin::Judge => 2,
                    },
                );
            }
        }
    }
}

pub(super) fn push_u8(h: &mut HashSink, v: u8) {
    h.update(&[v]);
}
pub(super) fn push_u16(h: &mut HashSink, v: u16) {
    h.update(&v.to_le_bytes());
}
pub(super) fn push_u32(h: &mut HashSink, v: u32) {
    h.update(&v.to_le_bytes());
}
pub(super) fn push_u64(h: &mut HashSink, v: u64) {
    h.update(&v.to_le_bytes());
}
pub(super) fn push_i32(h: &mut HashSink, v: i32) {
    h.update(&v.to_le_bytes());
}
pub(super) fn push_usize(h: &mut HashSink, v: usize) {
    h.update(&(v as u64).to_le_bytes());
}
pub(super) fn push_cards(h: &mut HashSink, cards: &[CardId]) {
    push_u32(h, cards.len() as u32);
    for c in cards {
        push_u32(h, *c);
    }
}
pub(super) fn push_players(h: &mut HashSink, players: &[PlayerId]) {
    push_u32(h, players.len() as u32);
    for p in players {
        push_u8(h, *p);
    }
}
pub(super) fn push_options(h: &mut HashSink, opts: &[ChoiceOption]) {
    push_u32(h, opts.len() as u32);
    for o in opts {
        match o {
            ChoiceOption::Confirm => push_u8(h, 1),
            ChoiceOption::Pass => push_u8(h, 2),
            ChoiceOption::Card(c) => {
                push_u8(h, 3);
                push_u32(h, *c);
            }
            ChoiceOption::Player(p) => {
                push_u8(h, 4);
                push_u8(h, *p);
            }
            ChoiceOption::Tag(t) => {
                push_u8(h, 5);
                push_u8(h, *t);
            }
            ChoiceOption::Kind(k) => {
                push_u8(h, 6);
                push_u32(h, u32::from(*k));
            }
        }
    }
}
pub(super) fn push_static(h: &mut HashSink, s: &'static str) {
    push_u32(h, s.len() as u32);
    h.update(s.as_bytes());
}
