//! Decision 34 accurate trigger classification for 9 extension sources
//! (`U01`, `U05`, `L08`, `L14`, `L19`, `L20`, `L24`, `L25`, `L27`) plus `U07`.
//!
//! Eliminates coarse 7-window grouping and eliminates arbitrary `PhaseBegin` fallback.

use engine_native::event::EventKind;
use engine_native::rvm_bridge::VmFilterPolicy;

/// Classifies skill trigger to authentic upstream event and filter policy.
pub fn classify_skill_trigger(name: &str) -> Option<(EventKind, VmFilterPolicy)> {
    if name.ends_with("__ref_busi") {
        return Some((EventKind::Dying, VmFilterPolicy::OwnerIsTarget));
    }
    if let Some(res) = crate::classify_l11_npc_skill_trigger(name) {
        return Some(res);
    }
    if let Some(res) = crate::classify_l11_taixu_skill_trigger(name) {
        return Some(res);
    }
    match name {
        // === 1. Dying Window ===
        "busi"
        | "ssr_newbuqu"
        | "_zuanidentitydyingskill"
        | "_bingleDying"
        | "yihun"
        | "huimeng" => Some((EventKind::Dying, VmFilterPolicy::OwnerIsTarget)),

        // === 2. Death Window ===
        "dddqianlong"
        | "shouge"
        | "zihui"
        | "_decadeUI_dieKillEffect"
        | "_kill_framebg"
        | "_dieNoTuoguan"
        | "_jd_wjsw"
        | "_check_die_yh"
        | "_pfqh_check_jisha"
        | "_killskill"
        | "_extradieskill"
        | "_dieskill"
        | "_guozhan_diebiaoji_"
        | "_shoushajisha_"
        | "_jiangshi1"
        | "_jiangshi2"
        | "_jiangshi3" => Some((EventKind::Death, VmFilterPolicy::OwnerIsTarget)),

        // === 3. Damage Taken (OwnerIsTarget) ===
        "ycndiyjianxiong"
        | "cyxianjiang"
        | "ssr_newganglie"
        | "ssr_newtianxiang"
        | "ssr_newjiyuan"
        | "ssr_guimou"
        | "ssr_newduanxian"
        | "ssr_wangxi"
        | "txhj_dundunSkill2"
        | "txhj_yayaSkill2"
        | "txhj_piliche"
        | "_pfqh_check_damage"
        | "_pfqh_check_damage_times"
        | "_pfqh_check_hp_change"
        | "_zuan_damageSendEgg"
        | "_zuan_damageSendFlower"
        | "_extradamageskill"
        | "_wujinmoshiDamage"
        | "_wujinmoshiDamageE"
        | "_losehp_"
        | "_mianshang_"
        | "baonvezhi"
        | "dddtuoji"
        | "ddddiedang"
        | "dddxianmou"
        | "dddzhangcai"
        | "dddposu"
        | "dddcuirui"
        | "choudu"
        | "shanhui"
        | "pal_bingxin" => Some((EventKind::Damage, VmFilterPolicy::OwnerIsTarget)),

        // === 4. Damage Dealt (OwnerIsSource) ===
        "dujiang" => Some((EventKind::Damage, VmFilterPolicy::OwnerIsSource)),

        // === 5. Card Used / Responded (OwnerIsSource) ===
        "tanlin"
        | "feizhua"
        | "tianjian"
        | "shanguang"
        | "ssr_wushuang"
        | "ssr_baonue"
        | "ssr_newfabing"
        | "ssr_newjilue"
        | "ssr_tieji"
        | "ssr_liegong"
        | "txhj_taijifuchen"
        | "gx_taijifuchen"
        | "pal_lingli"
        | "pal_jianqi"
        | "dddqiaoduan"
        | "dddhuishi"
        | "dddmubing"
        | "dddyongjue"
        | "dddqinzheng"
        | "dddweidai"
        | "dddyanlin"
        | "dddzhanyi"
        | "_usecard"
        | "_decadeUI_usecardBegin"
        | "_changeEquipExt"
        | "_tsx"
        | "_gj"
        | "_ts"
        | "_chupaiA"
        | "_chupaiB"
        | "_chupaiC"
        | "_chupaiD"
        | "_chupaiE"
        | "_chupaiF"
        | "_chupaiG"
        | "_chupaiH"
        | "_useskill"
        | "_extrauseskill"
        | "_shiyongkapaitexiao_" => Some((EventKind::CardUsed, VmFilterPolicy::OwnerIsSource)),

        // === 6. Card Targeted / Responded (OwnerIsTarget) ===
        "jianbi" | "_jd_shiyongshanD" | "_checkDcdShan" | "_shanskill" => {
            Some((EventKind::CardUsed, VmFilterPolicy::OwnerIsTarget))
        }

        // === 7. Cards Lost / Discarded ===
        "dddcangji" => Some((EventKind::CardsLost, VmFilterPolicy::OwnerIsTarget)),

        // === 8. Draw Phase Replacement / Augmentation ===
        "dddlianer" => Some((EventKind::DrawPhaseBegin1, VmFilterPolicy::OwnerIsTarget)),

        // === 9. Phase End (phaseJieshuBegin & phaseDiscardBegin) ===
        "leiyu"
        | "zhengsu"
        | "ssr_biyue"
        | "ssr_jushou"
        | "ssr_zhuhai"
        | "ssr_shendao"
        | "dddyeshen"
        | "dddchengjing"
        | "dddzhijie"
        | "_pfqh_check_phaseEnd"
        | "_jd_hhjs"
        | "_jd_phaseAfter"
        | "_phaseendskill" => Some((EventKind::PhaseEnd, VmFilterPolicy::OwnerIsTarget)),

        // === 10. Turn / Round Begin ===
        "dddbailei"
        | "dddqice"
        | "dddxuyu"
        | "_pfqh_check_roundCount"
        | "_zuanRoundTX"
        | "_chooseCharacterTwo"
        | "_tuizhi"
        | "_tuizhi2"
        | "_tuizhi3" => Some((EventKind::TurnBegin, VmFilterPolicy::OwnerIsTarget)),

        // === 11. Upstream Active PhaseUse / Choice Skills (enable: "phaseUse" | "chooseToUse") ===
        "_anshaskill" | "_anyeskill" | "_aozhan_judge" | "_basicjuedgeskill" | "_bingleAddJudge" | "_bingleBigScreen"
        | "_bingleLoseJudge" | "_binglediecls" | "_bingletexiao" | "_bingletexiao1" | "_bingletexiao2" | "_bingletexiaobingliang"
        | "_bingletexiaochangehp" | "_bingletexiaofulei" | "_bingletexiaohongshui" | "_bingletexiaohuoshan" | "_bingletexiaole" | "_bingletexiaoshandian"
        | "_binglintu" | "_checkDcdChuChang" | "_chongzhu" | "_chupaiI" | "_chupaiJ" | "_chupaiK"
        | "_chupaiL" | "_chupaiM" | "_chupaiMX" | "_chupaiN" | "_chupaiNX" | "_chupaiO"
        | "_chupaiP" | "_clearBackground" | "_clearthunder" | "_comparesayskill" | "_damagesay" | "_damsayskill"
        | "_diankuangtulu_gua" | "_diesay" | "_dietime" | "_discard" | "_diycharacterskill" | "_diyskill"
        | "_dysp" | "_emojiRebate1" | "_emojiRebate2" | "_equipuseskill" | "_exp2skill" | "_expskill"
        | "_extracompareskill" | "_extradlosehpskill" | "_extragainskill" | "_extrajudgeskill" | "_extralinkskill" | "_extraloseCardskill"
        | "_extraloseskill" | "_extraphaseskill" | "_extraturnoverskill" | "_fix_phase_yh" | "_fix_yh" | "_fromsay"
        | "_gainSkill" | "_goodluckjuedgeskill" | "_guandieskill" | "_guanmodeskill" | "_guixinmodeskill" | "_guozhanChooseControlCHG"
        | "_guozhan_duiweibiaoji_gua_" | "_guozhan_duiweibiaoji_gua_gua_" | "_guozhan_marks" | "_guozhan_quanjubiaoji_guagua" | "_guozhanhalfchghp" | "_haoyunlai"
        | "_helasisyskill" | "_huanhaiskill" | "_huzhu" | "_jd_cpjd" | "_jd_ddxyA" | "_jd_ddxyAll"
        | "_jd_ddxyB" | "_jd_ddxyC" | "_jd_ddxyD" | "_jd_ddxyE" | "_jd_ddxyF" | "_jd_dfsk"
        | "_jd_hhks" | "_jd_mpjd" | "_jd_pdjd" | "_jd_qpjd" | "_jd_swxs" | "_jianghunskill"
        | "_jiangshi" | "_jiangshi4" | "_jiangshiTx" | "_jiangshiTx2" | "_jiangshiTx3" | "_jindutiao"
        | "_jindutiaoA" | "_jindutiaoKS" | "_jindutiaoMB" | "_jindutiaoO" | "_jindutiao_close" | "_jindutiaopl"
        | "_jingchengadddie" | "_jisuangailv" | "_jiubuff_" | "_jiubuff_xiaoshi_" | "_jszsx" | "_judgeskill"
        | "_keepideskill" | "_levelskill" | "_linkskill" | "_lunjianskill" | "_menuCheat" | "_newBackground"
        | "_newBackgroundAtart" | "_newBackgroundUse" | "_newZuanPhaseTX" | "_newbiankuang" | "_o_o_miaoshouhuichun" | "_o_o_recovertrigger"
        | "_o_o_yishugaochao" | "_pfqh_check_changeGroup" | "_pfqh_check_phaseBegin" | "_pfqh_reapply_on_init" | "_pfqh_reapply_saved_params" | "_phasebeginskill"
        | "_playAudioToQueue" | "_player_baojiA" | "_player_baojiB" | "_player_baojiC" | "_player_baojiD" | "_player_baojiE"
        | "_player_baojiF" | "_player_baojiG" | "_player_baojiH" | "_player_daojianfu_xxgg_fuzi_" | "_player_game_guagua" | "_player_xxgg_zhishixian2_xxxxx_ggg"
        | "_player_xxgg_zhishixian3_xxxxx_ggg" | "_player_xxgg_zhishixian_xxxxx_ggg" | "_qhlyChangeGroupSkin" | "_qhly_addButton" | "_qhly_autoc" | "_qunxiongzhengbaskill"
        | "_rarity" | "_recoversay" | "_relifeskill" | "_saveskill" | "_seadreammodeskill" | "_selfsay"
        | "_shanghaipaishiyongtexiao_" | "_shiqihujia_" | "_shiyongkapaitexiao_xi_gua_" | "_shoushajisha_delete_" | "_shuohuaSayWordphase" | "_skilltip_closeB"
        | "_skin_switch_hook" | "_ssrchangehpskill" | "_ssrnohuahensskill" | "_startgamesay" | "_strengthencardskill" | "_strengthenstandardskill"
        | "_tamodeskill" | "_taohuaskill" | "_thunderjudgemodeskill" | "_thundermodeskill" | "_thundertexiao" | "_thundertexiao2"
        | "_tiesuo_ying" | "_tosay" | "_ui_auto_cheat" | "_useCardsayskill" | "_wanjunqushou_gua" | "_wmkzSayChange"
        | "_wujinmoshiDraw" | "_wujinmoshiDrawE" | "_wujinmoshiMaxHandCard" | "_wujinmoshiMaxHandCardE" | "_wujinmoshiRE" | "_wujinmoshiRecover"
        | "_wujinmoshiRecoverE" | "_xigua_feiyanbahu_" | "_xigua_mopaifanbei_" | "_xigua_xueliangfanbei_" | "_xigua_zhuwanzhongji_" | "_yinyang_mark_add"
        | "_yuanqiskill" | "_zhongfandieskill" | "_zhongzhubuff" | "_zhudieskill" | "_zhulianbihe_mark_tao" | "_zuanCHANGEHPsayskill"
        | "_zuanSTARTsayskill" | "_zuanUSECARDsayskill" | "_zuan_killSendFlower" | "_zuan_sendWine" | "_zuanattacklinecheck" | "_zuanboss"
        | "_zuanidentitychgneiskill" | "_zuanidentitydamskill" | "_zuanidentitydieafterskill" | "_zuanidentityrecskill" | "_zuanidentityshowskill" | "_zuanidentityskill"
        | "_zuanidentitytodieskill" | "_zuanidentityuseskill" | "_zuanphaseTX" | "_zuansaidskill" | "_zuanwashhandcard" | "aqianghua"
        | "baihe" | "baonvezhi_mark" | "baozou" | "batu" | "bfengshi" | "biaoqi"
        | "bingqiang" | "bingsheng" | "bolan" | "bolehuiyan" | "bolehuiyan2" | "bolehuiyan3"
        | "chajue2" | "chanceIdentity" | "chanceIdentity0" | "chanceIdentity10" | "chanceIdentity11" | "chanceIdentity12"
        | "chanceIdentity13" | "chanceIdentity14" | "chanceIdentity15" | "chanceIdentity2" | "chanceIdentity3" | "chanceIdentity4"
        | "chanceIdentity5" | "chanceIdentity6" | "chanceIdentity7" | "chanceIdentity8" | "chanceIdentity9" | "chezhen"
        | "chongzhuang" | "chujia" | "cike" | "ciqiu" | "ciqiu_destroy" | "ciqiu_dying"
        | "clandianzhan" | "danqing" | "dddanliu" | "dddanzhi" | "dddbingjian" | "dddchashi"
        | "ddddanggu" | "ddddongcha" | "dddduanbing" | "dddfengzheng" | "dddfenye" | "dddfushi"
        | "dddfusi" | "dddfusi_global" | "dddfuyi" | "dddfuzong" | "dddguiying" | "dddjijian"
        | "dddjijian_old" | "dddjinggou" | "dddjungui" | "dddjuxian" | "dddlanghuai" | "dddlangzhi"
        | "dddlingyong" | "dddmiaoxing" | "dddmoyan" | "dddqiahua" | "dddquche" | "dddshichao"
        | "dddshijian" | "dddshijian_old" | "dddshilie" | "dddshixing" | "dddtaisi" | "dddtongyu"
        | "dddweiqiu" | "dddxiaheng" | "dddxianxi" | "dddxiaoxing" | "dddxiaxing" | "dddxuanlun"
        | "dddxunxun" | "dddxuxiao" | "dddxuyu_old" | "dddyouxue" | "dddyuanzhen" | "dddyulv"
        | "dddyulv_old" | "dddzhengjun" | "dddzhilian" | "dddzhishu" | "dddzhuanshe" | "dianji"
        | "diaobingqianjiang" | "diaohulishan" | "dinghan" | "diyjuntun" | "diyjuntun2" | "drlt_duorui"
        | "ducai" | "ducai2" | "ducai3" | "duijue" | "duoqi" | "duxinshu"
        | "fanpu" | "feigong" | "feixia" | "fenghuo" | "fengliu" | "fengyan"
        | "fengyi" | "fenxing" | "fuchou" | "fuchou2" | "fuchou3" | "fz_gzpaoxiao"
        | "fz_liegong" | "fz_new_longdan" | "fz_new_rewusheng" | "fz_new_tieji" | "fz_xinkuanggu" | "g_du"
        | "g_mtg_cangbaohaiwan" | "g_mtg_duzhao" | "g_pozhenjue" | "g_youdishenru" | "ganran" | "ganran2"
        | "geju" | "guifu" | "guimian" | "hanbei" | "hanqiang" | "heqin"
        | "heqin2" | "heqin3" | "huamu" | "huli" | "huoyu" | "huxi"
        | "jianai" | "jiangshifz" | "jiangshinj" | "jianzhen" | "jiean" | "jieyong"
        | "jieyong2" | "jieyong3" | "jimin" | "jinghe" | "jinyan" | "jishe"
        | "jishe2" | "juedao" | "juntun" | "juntun_global" | "kangyin" | "kangyin2"
        | "kongju" | "kuangcai" | "kuangchan" | "kuanshi2" | "kuwu" | "lguiyin"
        | "liangce" | "lichang" | "liduan" | "liduan2" | "liebo" | "lingdi"
        | "liudan" | "longxi" | "lshengong" | "luobi" | "lyuxue" | "lyuxue2"
        | "lzhangyi" | "maichong" | "maoding" | "mbcmqingzheng" | "mbmowang" | "mbwuling"
        | "mianzhen" | "miaobi" | "miaobix" | "mibei" | "moshou" | "mtg_cangbaohaiwan"
        | "mtg_duzhao" | "new_guixin" | "new_meibu" | "new_zhixi" | "new_zhixi_recover" | "nichang"
        | "nichang2" | "nsshuaiyan" | "nvquan" | "nvquan1" | "nvquan2" | "nvquan3"
        | "oldbolehuiyan" | "oldbolehuiyan2" | "potzhuangshi" | "pozhenjue" | "pro_drlt_duorui" | "qiandu"
        | "qiangyun" | "qijian" | "qimou" | "qingyu" | "qinru" | "qiongguang"
        | "qixia" | "quanji" | "rechuhai" | "regushe" | "rehuashen_mark" | "rekuanshi"
        | "reqimou" | "rexinsheng" | "sanbanfu" | "sanbanfu2" | "sanbanfu3" | "sbduanliang"
        | "sbjianxiong" | "sbqingzheng" | "sbtieji" | "sbtongye" | "sbzhaxiang" | "seyou"
        | "shanxian" | "shentan" | "sheshi" | "sheshu" | "shiqin" | "shishi"
        | "shoujia" | "shouxi" | "shouye" | "siji" | "spyanjiao" | "ssr_caiwenji"
        | "ssr_caocao" | "ssr_caoren" | "ssr_daqiao" | "ssr_dengai" | "ssr_dianwei" | "ssr_diaochan"
        | "ssr_dongzhuo" | "ssr_fazheng" | "ssr_ganning" | "ssr_gongsunzan" | "ssr_guanyu" | "ssr_guojia"
        | "ssr_huanggai" | "ssr_huangyueying" | "ssr_huangzhong" | "ssr_huatuo" | "ssr_huaxiong" | "ssr_jiangwei"
        | "ssr_jiaxu" | "ssr_liubei" | "ssr_lusu" | "ssr_luxun" | "ssr_lvbu" | "ssr_lvmeng"
        | "ssr_machao" | "ssr_menghuo" | "ssr_pangtong" | "ssr_re_lidian" | "ssr_simayi" | "ssr_sunce"
        | "ssr_sunjian" | "ssr_sunquan" | "ssr_sunshangxiang" | "ssr_taishici" | "ssr_xiahoudun" | "ssr_xiahouyuan"
        | "ssr_xiaoqiao" | "ssr_xuhuang" | "ssr_xushu" | "ssr_xuzhu" | "ssr_yiji" | "ssr_yuanshao"
        | "ssr_yuanshu" | "ssr_yuji" | "ssr_yujin" | "ssr_zhangbao" | "ssr_zhangfei" | "ssr_zhanghe"
        | "ssr_zhangjiao" | "ssr_zhangliang" | "ssr_zhangliao" | "ssr_zhangzhang" | "ssr_zhaoyun" | "ssr_zhenji"
        | "ssr_zhoutai" | "ssr_zhouyu" | "ssr_zhugeliang" | "ssr_zhurong" | "ssr_zuoci" | "taiji"
        | "tanhua" | "taolue" | "tiandan" | "tianwu" | "tiewan" | "tongling"
        | "tongyu" | "tspowei" | "tuijin" | "tuoqiao" | "tuqiang" | "twfengji"
        | "twxingzhui" | "twzhouzu" | "wange" | "wange2" | "wengua" | "wengua5"
        | "wlianhuan" | "wluoyan" | "woliu" | "wujinmoshiRE1" | "wujinmoshiRE2" | "wumou"
        | "wumu" | "xfenxin" | "xiangma" | "xianqu" | "xianqu_mark" | "xiaoyue"
        | "xiefang" | "xieling" | "xinfu_jingxie" | "xingchong" | "xiushen" | "xjzongyu"
        | "xuanning" | "xumou" | "xunmeng" | "yanshi" | "yanyi" | "yanzhan"
        | "yaoji" | "yaoyi" | "ycnxinfu_qiaosi" | "yexinjia_mark" | "yinyang" | "yinyang_mark"
        | "yixin" | "yizhuang" | "yizhuang2" | "youdishenru" | "youzhan" | "ysheshen"
        | "yujia" | "ywuhun" | "yxs_fanji2" | "yxs_menshen" | "yxs_menshen2" | "yxs_menshen3"
        | "yxschangnian" | "yxschangnian2" | "yxsciqiu" | "yxsdili" | "yxsweiyi" | "zaibian"
        | "zbudao" | "zbudao2" | "zhanlong" | "zhengsu_bianzhen" | "zhengsu_leijin" | "zhengsu_mingzhi"
        | "zhensha" | "zhenyi_spade" | "zhexian" | "zhijie" | "zhongdun" | "zhucheng"
        | "zhulianbihe_mark" | "zhulu" | "zhuxin" | "zuoxing" | "zyhufu" => Some((EventKind::PhaseBegin, VmFilterPolicy::OwnerIsTarget)),

        // === 12. Package Data Whitelist: Active skills from implemented pack9 sources ===
        other => {
            if crate::shahai_attach::shahai_implemented_skills::is_implemented_u05_vm_skill(other)
                || crate::lookup_upstream_host_behavior(other).is_some()
                || crate::decision34_pack9_data::U01_DDD_SKILLS.contains(&other)
                || crate::decision34_pack9_data::U07_WUDIAN_ALL_SKILLS.contains(&other)
                || crate::decision34_pack9_data::U08_YXS_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L08_DECADE_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L14_SHOUSHA_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L19_DOOMSDAY_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L20_MARK_SUPPLEMENT_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L24_SKIN_SWITCH_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L25_ZUAN_SKILLS.contains(&other)
                || crate::decision34_pack9_data::L27_SELECT_BEAUTIFY_SKILLS.contains(&other)
                || crate::decision34_closure::L10_SCS_COMPANION_SKILLS.contains(&other)
                || crate::decision34_closure::L28_THUNDER_EXTRA_SKILLS.contains(&other)
                || crate::zuan_data::L25_SSR_GENERAL_SKILLS.contains(&other)
                || crate::zuan_data::L25_ALL_RULE_SKILLS.contains(&other)
                || matches!(other, "ssr_yongsi" | "ssr_yongsi_yuxi_card" | "l20_mark_dispatch" | "_chooseCharacterTwo" | "xueji" | "wansha" | "paoxiao" | "dcxiaoxi" | "rehuashen" | "twxiafeng" | "k_danggu" | "k_mowang" | "th_dunshi" | "th_taipingyaoshu")
            {
                Some((EventKind::PhaseBegin, VmFilterPolicy::OwnerIsTarget))
            } else {
                None
            }
        }
    }
}
