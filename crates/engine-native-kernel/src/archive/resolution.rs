use super::*;

// ---- 选择 ----

pub(super) fn enc_choice_option(w: &mut W, o: &ChoiceOption) {
    match o {
        ChoiceOption::HiddenCard(slot) => {
            w.u8(8);
            w.u32(*slot);
        }
        ChoiceOption::Confirm => w.u8(0),
        ChoiceOption::Pass => w.u8(1),
        ChoiceOption::Card(c) => {
            w.u8(2);
            w.u32(*c);
        }
        ChoiceOption::Player(p) => {
            w.u8(3);
            w.u8(*p);
        }
        ChoiceOption::Tag(t) => {
            w.u8(4);
            w.u8(*t);
        }
        ChoiceOption::Control(option) => {
            w.u8(7);
            w.u8(option.tag());
        }
        ChoiceOption::Skill(s) => {
            w.u8(6);
            w.u16(*s);
        }
        ChoiceOption::Kind(k) => {
            w.u8(5);
            w.u32(u32::from(*k));
        }
    }
}

pub(super) fn dec_choice_option(r: &mut R<'_>) -> Result<ChoiceOption, ArchiveError> {
    Ok(match r.u8()? {
        8 => ChoiceOption::HiddenCard(r.u32()?),
        0 => ChoiceOption::Confirm,
        1 => ChoiceOption::Pass,
        2 => ChoiceOption::Card(r.u32()?),
        3 => ChoiceOption::Player(r.u8()?),
        4 => ChoiceOption::Tag(r.u8()?),
        7 => ChoiceOption::Control(
            crate::event::YuanceOption::from_tag(r.u8()?)
                .ok_or(ArchiveError::BadTag("yuance_control", 0))?,
        ),
        6 => ChoiceOption::Skill(r.u16()?),
        5 => {
            let code = r.u32()?;
            // 码位必须能反解为目录牌种（未知码=坏档）
            card_kind_from_num(code)?;
            ChoiceOption::Kind(code as u16)
        }
        t => return Err(ArchiveError::BadTag("choice_option", t)),
    })
}

pub(super) fn enc_open_choice(w: &mut W, o: &OpenChoice) {
    w.u64(o.id);
    enc_pid(w, &o.observer);
    let spec = &o.spec;
    // v2：kind 显式编码（内核机器的种类是权威，不再由提示词静态推导）。
    enc_choice_kind(w, spec.kind);
    w.str(spec.prompt);
    w.len(spec.options.len());
    for o in &spec.options {
        enc_choice_option(w, o);
    }
    w.u64(spec.min as u64);
    w.u64(spec.max as u64);
    w.boolean(spec.allow_cancel);
}

/// N11-b（ADR 0774）：固定提示词注册表——零分配，未知 prompt
/// 直接拒绝。所有引擎生成的 prompt 均为编译期常量。
pub(super) const KNOWN_PROMPTS: &[&str] = &[
    "zhenlie_discard",
    "yihua_discard",
    "discard_to_limit",
    "respond_shan",
    "dying_save",
    "gui_dao_rewrite",
    "wudian_fuchou_cancel",
    "wudian_diyjuntun_recast",
    "guanxing_arrange",
    "tuxi_target",
    "rende_cards",
    "rende_target",
    "jieyin_cards",
    "jieyin_target",
    "fanjian_target",
    "fanjian_pick",
    "lijian_discard",
    "lijian_targets",
    "jijiang_helper",
    "jijiang_respond",
    "jijiang_target",
    "liuli_transfer",
    "hujia_respond",
    "qingnang_target",
    "qingnang_card",
    "sancai_target",
    "sancai_card",
    "yishang_target",
    "zhiheng_select",
    "trigger_confirm",
    "respond_sha",
    "aoe_response",
    "nullify_window",
    "duel_sha",
    "jiji_card",
    "jiji_target",
    "tian_du_gain",
    "host_target_discard_equip",
    "wuxing_select_card",
    "wuxing_select_nature",
    "d3_jinggou_target",
    "wd_sanjian_discard",
    "wd_sanjian_target",
    "wd_fudi_target_pick",
    "wd_fudi_player_pick",
    "wd_fudi_compare",
    "wd_touliang_own",
    "wd_touliang_target",
    "yc_gean_t1_pick",
    "yc_gean_t2_pick",
    "wd_caochuan_branch",
    "gj_shujinsan_discard",
    "gj_feibiao_discard",
    "gj_jinlianzhu_confirm",
    "gj_xiajiao_discard",
    "gj_luyugeng_cost",
    "gj_dinvxuanshuang_rider",
    "hh_dunpai_discard",
    "hh_yuansu_discard",
    "gw_poxiao_branch",
    // T8 增量 2/7：rule-vm 技能选择挂起（rvm_bridge）
    "rvm_choice",
    "huanren_pick",
    "d3_moyan_target",
    "d3_moyan_gain",
    // J1（ADR 0786 §A.3，规则 v328）：VM 提交期选择窗挂起的受限 prompt 键
    // （ADR 预期的「rvm_effect_pick 类受限 prompt」；复用既有 ChoiceKind::
    // SelectCards 与 Answer 通道，无新增变体/载荷）。
    // 静态解码白名单；提示按字符串编码，尾追加保留历史登记。
    "rvm_discard_pick",
    "xunxun_arrange",
    "jijie_give_target",
    "gaiming_pick",
    "haotianta_pick",
    "liaoyuan_discard",
    "fanjian_suit",
    "rvm_danshou_option",
    "rvm_danshou_target",
    "rvm_danshou_card",
    "rvm_danshou_discard",
    "rvm_fenghe_card",
    "rvm_fenghe_destination",
    "rvm_pingkou_targets",
    "trigger_order",
    "c331_yuance",
    "c331_jidian_target",
    "c331_zhangwu_holder",
    "c331_zhangwu_source",
    "xuanying_target",
    "c331_qinglong_card",
    "judge_rewrite_provider",
    "rvm_tiqi_confirm",
    "rvm_tiqi_sign",
    "rvm_baoshu_cost",
    "rvm_dcbihuo_target",
    "cangchizhibi_target",
    "aojian_discard", "aojian_targets", "baishouzhihu_target", "beige_discard",
    "bingfeng_targets", "bingjian_discard", "bingjian_target", "buchun_target",
    "cixiong_discard_choose", "cyqiaoxie_card", "cyqiaoxie_target", "cyxianjiang_pick",
    "daixing_discard", "dangping_discard", "dangping_target", "donghuangzhong_cost",
    "donghuangzhong_target", "duanxing_target", "duanyi_discard", "duanyi_target",
    "duanyue_card", "duanyue_target", "duishi_discard", "duishi_discard_same_suit",
    "duishi_target", "dunxing_discard", "fengming_discard", "fengming_target",
    "fengqi_card", "fengqi_targets", "fengxueren_flip_confirm", "funiao_give",
    "funiao_target", "fuyan_target", "fz_xinkuanggu_choice", "fzhenwei_target",
    "gaizao_slot", "gj_bingpo_show_own", "gj_bingpo_show_target", "gj_longxu_pick",
    "gongshen_discard", "gongxin_card", "gongxin_choice", "gongxin_target",
    "guanhu_discard", "guanshi_force_damage", "guiyan_take", "guiyan_target",
    "guozao_target", "guxing_targets", "gw_baoxue_discard", "gxianyin_gain_suit",
    "gxianyin_suit", "hanbing_prevent_confirm", "hanbing_target_discard", "heihuo_discard",
    "huajian_discard", "huajian_target", "huajin_discard", "huangyu_discard",
    "huanhun_discard", "huanle_monkey_confirm", "huanle_xuelunyang_skill", "huanle_xuelunyang_target",
    "huanling_target", "huanxing_discard", "huanxing_target", "huiqi_target",
    "huodan_pick", "huodan_target", "hutian_cards", "hutian_target",
    "hwendao_discard_suit", "jianji_pick", "jianji_target", "jiedao_sha_or_give",
    "jikong_target", "jilve_card", "jilve_targets", "jinlin_targets",
    "juxi_choice", "juxi_target", "kunlunjingc_pick", "kunlunjingc_replace",
    "lianda_discard", "lianhua_pick", "lieren_player_pick", "lieren_target_pick",
    "lingfeng_damage_or_hujia", "lingfeng_target", "linyun_pick_two", "linyun_sha_target",
    "liuli_discard_choose", "lmazui_pick", "lmazui_target", "luanji_discard",
    "lueying_choose_victim", "lueying_confirm", "lxianglong_target", "mailun_mode",
    "mbcmqingzheng_suit", "mbcmqingzheng_target", "mbcmqingzheng_target_suit", "meihuo_pick",
    "meihuo_target", "miedao_discard", "mingfu_pick", "mingfu_target",
    "moyan_cards", "moyan_targets", "mtg_bingheyaosai_discard", "mtg_feixu_discover",
    "mtg_haidao_discard", "mtg_lindixiliu_discard", "mtg_linzhongjianta_card", "mtg_linzhongjianta_discover",
    "mtg_linzhongjianta_target", "mtg_longshanfeng_gain", "mtg_shamolvzhou_discover", "mtg_shuimomuxue_discard",
    "mtg_yixialan_gain", "mtg_yixialan_recast_card", "mtg_youlin_pick", "mtg_youlin_trick",
    "mufeng_discover", "new_meibu_discard", "nigong_target", "ningshuang_discard",
    "ningxian_discard", "ningxian_targets", "nlianji_t0_pick", "nlianji_target0",
    "nlianji_target1", "nlianji_throw", "polang_discard_equip", "pozhen_discard",
    "pozhen_discard_source", "pozhou_target", "qilin_discard_horse", "qinglong_replay_sha",
    "qinglongzhigui_discard", "rangjie_choice", "rangjie_move_card", "rangjie_move_target",
    "rangjie_type", "reguhuo_card", "reguhuo_name", "reguhuo_targets",
    "rumeng_basic_or_skip", "rumeng_discard", "sadengjinhuan_extra_shan", "sadengjinhuan_judge_confirm",
    "shangxi_discard", "shangxi_target", "shedu_discard_or_lose", "shehun_discard",
    "shehun_target", "shehun_target_discard", "shennongding_cost", "shoulie_discard",
    "shouna_cost", "shouna_target", "shuchui_target", "shushen_target",
    "songci_discard", "songci_target", "suiyan_discard", "susheng_discard",
    "swd_jiefen_give", "swd_jiefen_target", "swd_jiguanfeng_shan", "swd_jiguantong_discard",
    "swd_jiguanyuan_choose", "swd_liuxing_discard", "swd_niaoxiangzhen_shan", "swd_qixingzhen_discard",
    "swd_qixingzhen_shan", "swd_shentou_cost", "swd_tianfuzhen_discard", "swd_yangpijuan_category",
    "swd_yangpijuan_pick", "swd_yufulu_target", "swd_yufulu_weapon", "swdtianshu_card",
    "swdtianshu_skill", "swdtianshu_target", "sxianjing_pick", "tianlun_rewrite",
    "tianxiang_heart", "tianxiang_target", "tongji_transfer_discard", "toudan_pick",
    "toudan_target", "wangchen_target", "wudian_baoxue_targets", "wudian_feidan_discard",
    "wudian_feitiao_cards", "wudian_feitiao_target", "wudian_guangshu_card", "wudian_guangshu_target",
    "wudian_huhuan_cards", "wudian_kangyin_discard", "wudian_kangyin_draw", "wudian_kangyin_target",
    "wudian_kuoyin_cards", "wudian_liangou_cards", "wudian_liangou_target", "wudian_mujing_card",
    "wudian_mujing_target", "wudian_paotai_card", "wudian_pingzhang_cards", "wudian_qianggu_cards",
    "wudian_qianggu_guard_ask", "wudian_shouhu_card", "wudian_shouhu_target", "wudian_shuaiyan_choice",
    "wudian_shuaiyan_discard", "wudian_xj_yuexing_target", "wudian_yinbo_cards", "wudian_yuedong_targets",
    "wudian_zhencha_choice", "wudian_zhucheng_guard", "wudian_ziyu_choice", "wugu_pick",
    "xiandan_discard", "xianghui_discard", "xiaoguo_discard_basic", "xielei_discard",
    "xielei_target", "xingdian_card", "xingxia_discard", "xingxia_enemy",
    "xingzhui_discard", "xingzhui_same_type", "xingzhui_target", "xiuluo_hand",
    "xiuluo_judge", "xj_changnian_target", "xj_ctianfu_card", "xj_ctianfu_target",
    "xj_diewu_cards", "xj_diewu_target", "xj_duci_target", "xj_fenglue_target",
    "xj_guanri_cards", "xj_guanri_target", "xj_guijin_cards", "xj_guijin_target",
    "xj_guiyuan_cards", "xj_huahu_targets", "xj_husha_mode", "xj_husha_target",
    "xj_liguang_cards", "xj_lingquan_cards", "xj_lingyu_target", "xj_luanjian_cards",
    "xj_luanjian_targets", "xj_minsha_cards", "xj_minsha_target", "xj_poyun_cards",
    "xj_qianfang_cards", "xj_runxin_target", "xj_sajin_cards", "xj_sajin_targets",
    "xj_shuiyun5_cards", "xj_shuiyun_cards", "xj_tianjian_cards", "xj_wangyou_cards",
    "xj_xiepan_cards", "xj_xtanxi_cards", "xj_xuanmo_cards", "xj_zhaoyao_cards",
    "xj_zhimeng_target", "xj_zhuyue_cards", "xj_zhuyue_targets", "xuanyuan_discard",
    "xuanzhou_target", "xuanzhou_trick", "yaowu_draw_or_recover", "yiesheng_discard",
    "yijue_player_pick", "yijue_target", "yijue_target_pick", "yiluan_target",
    "yinguo_discard", "yuchen_target", "yudun_pick_two", "yudun_sha_target",
    "yulin_discard", "yunchou_discard", "yunchou_target", "yxs_fanji_sha",
    "yxs_menshen_target", "zhangba_pick_two", "zhangba_sha_target", "zhanlu_discard",
    "zhanlu_targets", "zhanxing_discard", "zhanxing_target", "zhenjiu_pick",
    "zhenjiu_target", "zhuhai_sha", "zhuyu_discard",
    "boss_chuanyun_target", "boss_tianyun_target", "dddxiaxing_card", "kuwu_discard", "shengke_discard",
    "wudian_duoqi_card", "wudian_youzhan_discard", "wudian_choudu_card", "wudian_choudu_target",
    "wudian_diaobing_swap", "wudian_liduan_give", "wudian_youdishenru_sha",
    "mbcmjiushi_gain_confirm", "mbcmjiushi_turnback_confirm", "mbcmfangzhu_target", "mbcmfangzhu_option",
    "k_danggu_partner",
    "dddyuanzhen_discard", "dddzhishu_card", "dddzhishu_target", "dddjuxian_mode", "dddjuxian_card",
    "dddjuxian_target", "dddjungui_target", "dddjungui_reduce", "dddjungui_discard", "dddshilie_cards",
    "dddshilie_target", "dddyeshen_confirm", "dddyeshen_target", "dddqiaoduan_cards", "dddqiaoduan_recover_target",
    "dddqiaoduan_draw_targets", "ddddiedang_discard", "dddanliu_target", "dddanliu_pick_target", "dddanliu_pick_owner",
    "dddanliu_swap_confirm", "dddguiying_confirm", "dddbingjian_discard", "dddbingjian_partner", "dddtongyu_cards",
    "dddtongyu_target_choice",
    "dddzhilian_mode", "dddjijian_sha", "dddjijian_target", "dddjijian_show", "dddjijian_old_target",
    "dddjijian_old_show", "dddjijian_old_give_sha", "dddzhengjun_hp_choice", "dddzhengjun_hs_choice",
    "dddzhengjun_es_confirm", "dddxianxi_extra_target", "dddxianxi_discard", "dddlingyong_confirm",
    "dddlingyong_repeat", "dddxuxiao_card", "dddxuxiao_target", "dddxuyu_discard", "dddxuyu_confirm",
    "dddshijian_give", "dddshijian_mode", "dddxuyu_old_confirm", "dddxuyu_old_discard", "dddshijian_old_confirm",
    "dddtaisi_target", "dddtaisi_card", "dddquche_red_card", "dddquche_give_target", "dddquche_black_sha",
    "ddddongcha_owner_put", "ddddongcha_target_put", "ddddongcha_owner_take", "dddzhijie_cards",
    "dddqiahua_show", "dddxunxun_top", "dddfusi_pick", "dddfusi_agree", "dddtuoji_confirm", "dddchashi_discard",
    "dddqice_confirm", "dddmiaoxing_pile", "dddfushi_pile", "dddfushi_male_target", "dddfenji_trick",
    "dddfenji_target", "dddxiaheng_discard_target", "dddxiaheng_damage_target", "dddfengzheng_cards",
    "dddfengzheng_give_target", "dddyulv_put", "dddyulv_mode", "dddyulv_old_declare", "dddyulv_old_mode",
    "dddfenye_target", "dddfenye_owner_card", "dddfenye_target_card", "dddshichao_target",
    "dddyouxue_target", "dddchengjing_card", "dddchengjing_target", "dddduanbing_card", "dddduanbing_target",
    "dddduanbing_transfer", "dddlangzhi_mode", "dddlangzhi_own_card", "dddfuyi_confirm", "dddzhuanshe_give",
    "dddzhuanshe_damage", "dddlianer_confirm", "dddanzhi_target", "ddddanggu_targets", "dddlanghuai_confirm",
    "dddxuanlun_confirm", "dddxuanlun_end_mode", "dddxuanlun_cards", "dddjiexing_confirm", "dddbailei_target",
    "dddbailei_mode",
    "th_dunshi:choose_basic", "th_dunshi:choose_sha_target", "th_dunshi:respond_shan", "th_dunshi:damage_choice",
    "th_bushi:choose_posture", "th_caiyi:choose_target", "th_caiyi:choose_count", "th_chongxu:choose_action",
    "miaojian:choose_target", "th_shengong:discard_card", "th_shengong:choose_target", "olqisi:choose_slot",
    "th_pingcai:choose_relic", "th_pingcai:choose_target", "th_zhengjing:choose_count", "th_zhengjing:choose_target",
    "th_chengye:choose_target", "th_buxu:discard_cards", "th_buxu:choose_chamber", "th_yufeng:choose_score",
    "th_yufeng:choose_target", "th_tianshu:discard_card", "th_tianshu:choose_target",
    "th_jingxie:choose_card", "th_qiaosi:choose_count", "th_qiaosi:choose_mode", "th_qiaosi:choose_target",
    "th_tiansuan:choose_lot", "th_tiansuan:choose_target", "th_gushe:choose_targets", "rejici:choose_option",
    "th_geyuan:choose_action", "th_jieshu:choose_card", "th_yeyan:choose_mode", "th_yeyan:choose_targets",
    "th_yeyan:discard_cards", "qinyin:choose_mode",
    "th_dunshi:choose_skill", "th_caiyi:choose_option", "th_caiyi:discard_cards", "th_guili:choose_target",
    "th_shengong:choose_forge", "th_shengong:assist",
    "th_tiansuan:gain_card", "th_gushe:pindian_card", "th_gushe:target_pindian_card", "th_gushe:target_discard",
    "th_gushe:owner_discard", "rejici:killer_discard", "th_geyuan:magica_targets", "th_geyuan:magica_discard",
    "regushe:choose_targets", "regushe:pindian_card", "regushe:target_pindian_card", "regushe:target_discard",
    "regushe:owner_discard", "regushe:confirm_draw", "sbtongye:predict", "rehuashen:choose_skill", "rehuashen:choose_action",
    "taoluan:choose_card", "taoluan:choose_virtual", "taoluan:choose_target", "taoluan:choose_helper",
    "taoluan:helper_give", "shouxi:declare_kind", "shouxi:attacker_discard", "dinghan:adjust_record",
    "quanji:confirm", "quanji:put_card", "potzhuangshi:discard_cards", "potzhuangshi:lose_hp",
    "qimou:choose_hp", "reqimou:choose_hp", "sbjianxiong:init_marks", "sbjianxiong:confirm",
    "sbjianxiong:remove_mark", "jinghe:show_cards", "jinghe:choose_targets", "jinghe:choose_skill",
    "sbqingzheng:choose_suit", "sbqingzheng:choose_target", "sbqingzheng:choose_target_suit",
    "sbqingzheng:gain_mark", "sbqingzheng:confirm", "spyanjiao:choose_suit", "spyanjiao:choose_target",
    "spyanjiao:confirm", "twxiafeng:choose_marks", "twxiafeng:choose_count", "twxiafeng:confirm",
    "twxingzhui:choose_turns", "twxingzhui:choose_count", "twxingzhui:choose_target", "twxingzhui:confirm",
    "xingchong:choose_draw", "xingchong:choose_count", "xingchong:choose_cards", "xingchong:confirm",
    "dcxiaoxi:choose_maxhp", "dcxiaoxi:choose_count", "dcxiaoxi:choose_target", "dcxiaoxi:choose_mode",
    "dcxiaoxi:choose_option", "dcxiaoxi:confirm", "twfengji:choose_card", "twfengji:discard_card",
    "twfengji:choose_turns", "twfengji:choose_count", "twfengji:confirm", "twharvestinori:choose_card",
    "twharvestinori:discard_card", "twharvestinori:choose_turns", "twharvestinori:choose_count",
    "twharvestinori:confirm", "twzhouhu:choose_card", "twzhouhu:discard_card", "twzhouhu:choose_turns",
    "twzhouhu:choose_count", "twzhouhu:confirm", "twzuhuo:choose_card", "twzuhuo:discard_card",
    "twzuhuo:choose_turns", "twzuhuo:choose_count", "twzuhuo:confirm", "twzhouzu:choose_target",
    "twzhouzu:choose_turns", "twzhouzu:choose_count", "twzhouzu:target_discard", "twzhouzu:confirm",
    "bolan:choose_skill", "bolan:confirm", "bolan_g:confirm", "bolan_g:choose_skill", "bolan_g:choose_target",
    "drlt_duorui:confirm", "drlt_duorui:choose_slot", "drlt_duorui:choose_skill", "drlt_duorui:choose_target",
    "pro_drlt_duorui:confirm", "pro_drlt_duorui:choose_slot", "pro_drlt_duorui:choose_skill",
    "pro_drlt_duorui:choose_target", "xinfu_xionghuo:choose_target", "xinfu_xionghuo:confirm",
    "mbwuling:choose_target", "mbwuling:choose_order", "mbwuling:choose_spirit",
    "mbwuling:choose_equip_target", "mbwuling:confirm", "jishe2:choose_targets",
    "sbqingzheng:choose_suits", "xingchong:show_cards", "twharvestinori:discard_black",
    "twzhouhu:discard_red", "twzuhuo:discard_nonbasic",
];

pub(super) fn intern_prompt(bytes: &[u8]) -> Result<&'static str, ArchiveError> {
    let s = std::str::from_utf8(bytes).map_err(|_| ArchiveError::BadTag("prompt_utf8", 0))?;
    KNOWN_PROMPTS
        .iter()
        .find(|k| **k == s)
        .copied()
        .ok_or(ArchiveError::BadTag("prompt_unknown", 0))
}

pub(super) fn dec_open_choice(r: &mut R<'_>) -> Result<OpenChoice, ArchiveError> {
    let id = r.u64()?;
    let observer = dec_pid(r)?;
    let kind = dec_choice_kind(r)?;
    let prompt_bytes = r.bytes()?;
    if prompt_bytes.len() > 128 {
        return Err(ArchiveError::BadTag("prompt_too_long", 0));
    }
    let prompt: &'static str = intern_prompt(&prompt_bytes)?;
    let options = r.vec(dec_choice_option)?;
    let min = r.u64()? as usize;
    let max = r.u64()? as usize;
    let allow_cancel = r.boolean()?;
    Ok(OpenChoice {
        id,
        observer,
        spec: ChoiceSpec {
            kind,
            prompt,
            options,
            min,
            max,
            allow_cancel,
        },
    })
}

/// v2：种类按枚举声明序显式编码，未知标签拒绝。
pub(super) fn enc_choice_kind(w: &mut W, k: ChoiceKind) {
    w.u8(match k {
        ChoiceKind::Confirm => 0,
        ChoiceKind::SelectCards => 1,
        ChoiceKind::SelectPlayers => 2,
        ChoiceKind::RespondCard => 3,
        ChoiceKind::PickTag => 4,
        ChoiceKind::Control => 5,
    });
}

pub(super) fn dec_choice_kind(r: &mut R<'_>) -> Result<ChoiceKind, ArchiveError> {
    match r.u8()? {
        0 => Ok(ChoiceKind::Confirm),
        1 => Ok(ChoiceKind::SelectCards),
        2 => Ok(ChoiceKind::SelectPlayers),
        3 => Ok(ChoiceKind::RespondCard),
        4 => Ok(ChoiceKind::PickTag),
        5 => Ok(ChoiceKind::Control),
        t => Err(ArchiveError::BadTag("choice_kind", t)),
    }
}

// ---- 解析与执行位置 ----

pub(super) fn enc_resolution(w: &mut W, res: &ResolutionSnapshot) {
    w.u32(res.next_event_id);
    w.u64(res.next_choice_id);
    // 栈帧
    w.len(res.stack.len());
    for f in &res.stack {
        w.u32(f.id);
        match f.parent {
            Some(p) => {
                w.u8(1);
                w.u32(p);
            }
            None => w.u8(0),
        }
        enc_event_kind(w, &f.kind);
        enc_event_data(w, &f.data);
        enc_stage(w, &f.stage);
        w.len(f.queue.len());
        for q in &f.queue {
            enc_skill(w, &q.def);
            enc_pid(w, &q.owner);
            w.u8(q.order.0);
            w.i32(q.order.1);
            w.u8(q.order.2);
            w.u8(q.order.3);
            w.u16(q.order.4);
        }
        w.boolean(f.planned);
        w.boolean(f.intrinsic_done);
    }
    // 挂起机器
    w.len(res.parks.len());
    for p in &res.parks {
        w.u64(p.resume_depth as u64);
        match p.slot {
            Slot::Intrinsic { frame } => {
                w.u8(0);
                w.u32(frame);
            }
            Slot::Trigger { frame } => {
                w.u8(1);
                w.u32(frame);
            }
        }
        enc_machine(w, &p.machine);
    }
    // 开放选择与应答
    match &res.open_choice {
        Some(o) => {
            w.u8(1);
            enc_open_choice(w, o);
        }
        None => w.u8(0),
    }
    match &res.answer {
        Some(sel) => {
            w.u8(1);
            w.len(sel.len());
            for o in sel {
                enc_choice_option(w, o);
            }
        }
        None => w.u8(0),
    }
    w.u32(res.budget);
}

pub(super) fn dec_resolution(r: &mut R<'_>) -> Result<ResolutionSnapshot, ArchiveError> {
    let next_event_id = r.u32()?;
    let next_choice_id = r.u64()?;
    let stack = r.vec(|r| {
        let id = r.u32()?;
        let parent = match r.u8()? {
            0 => None,
            1 => Some(r.u32()?),
            t => return Err(ArchiveError::BadTag("opt_event_id", t)),
        };
        let kind = dec_event_kind(r)?;
        let data = dec_event_data(r)?;
        let stage = dec_stage(r)?;
        let queue = r.vec(|r| {
            let def = r.u16()?;
            let owner = r.u8()?;
            let o0 = r.u8()?;
            let o1 = r.i32()?;
            let o2 = r.u8()?;
            let o3 = r.u8()?;
            let o4 = r.u16()?;
            Ok(QueuedTrigger {
                def,
                owner,
                order: (o0, o1, o2, o3, o4),
            })
        })?;
        let planned = r.boolean()?;
        let intrinsic_done = r.boolean()?;
        Ok(crate::event::Frame {
            id,
            parent,
            kind,
            data,
            stage,
            queue,
            planned,
            intrinsic_done,
        })
    })?;
    let parks = r.vec(|r| {
        let resume_depth = r.u64()? as usize;
        let slot = match r.u8()? {
            0 => Slot::Intrinsic { frame: r.u32()? },
            1 => Slot::Trigger { frame: r.u32()? },
            t => return Err(ArchiveError::BadTag("slot", t)),
        };
        let machine = dec_machine(r)?;
        Ok(ParkSnapshot {
            resume_depth,
            slot,
            machine,
        })
    })?;
    let open_choice = match r.u8()? {
        0 => None,
        1 => Some(dec_open_choice(r)?),
        t => return Err(ArchiveError::BadTag("opt_open_choice", t)),
    };
    let answer = match r.u8()? {
        0 => None,
        1 => Some(r.vec(dec_choice_option)?),
        t => return Err(ArchiveError::BadTag("opt_answer", t)),
    };
    let budget = r.u32()?;
    Ok(ResolutionSnapshot {
        next_event_id,
        next_choice_id,
        stack,
        parks,
        open_choice,
        answer,
        budget,
    })
}
