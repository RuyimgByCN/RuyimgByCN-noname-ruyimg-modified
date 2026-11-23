//! Authoritative lookup of existing `HostBehavior` and `standard_catalog` skill bindings
//! so extension loaders never shadow native Rust `HostBehavior` or standard/boss skills.

use std::collections::BTreeSet;

use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::names;

#[path = "host_skill_bindings_std.rs"]
mod host_skill_bindings_std;
pub use host_skill_bindings_std::lookup_standard_catalog_skill_zh;

/// Look up the native `HostBehavior` for an upstream extension skill symbol, if implemented.
pub fn lookup_upstream_host_behavior(skill_name: &str) -> Option<HostBehavior> {
    match skill_name {
        "shahai-gujian__ref_dangping" | "shahai-gujian__ref_xuanning" => {
            return Some(HostBehavior::U05GujianCharBatchC);
        }
        _ => {}
    }
    let clean = skill_name
        .rsplit_once("__ref_")
        .map(|(_, s)| s)
        .unwrap_or(skill_name);
    if let Some(b) = super::taixu_skills_data::lookup_l11_b1_host_behavior(clean) {
        return Some(b);
    }
    match clean {
        "anlianying" => Some(HostBehavior::SwdAnlianying),
        "anwugu" => Some(HostBehavior::XianjianAnwugu),
        "anwugu2" => Some(HostBehavior::XianjianAnwugu2),
        "aojian" => Some(HostBehavior::SwdAojian),
        "baoxue" => Some(HostBehavior::WudianBaoxue),
        "beige" => Some(HostBehavior::YunchouBeige),
        "bingfeng" => Some(HostBehavior::Bingfeng),
        "bingfeng2" => Some(HostBehavior::Bingfeng2),
        "bingjian" => Some(HostBehavior::SwdBingjian),
        "binxin" => Some(HostBehavior::XianjianBinxin),
        "boss_chuanyun" => Some(HostBehavior::BossChuanyun),
        "boss_tianyun" => Some(HostBehavior::BossTianyun),
        "bshaowei" => Some(HostBehavior::WudianBaoleiSwitch),
        "busi" => Some(HostBehavior::SwdBusi),
        "changnian" => Some(HostBehavior::XianjianChangnian),
        "changnian2" => Some(HostBehavior::XianjianChangnian2),
        "chengjian" => Some(HostBehavior::SwdChengjian),
        "chengxin" => Some(HostBehavior::XianjianChengxin),
        "chezhen" => Some(HostBehavior::WudianChezhen),
        "choudu" => Some(HostBehavior::WudianChoudu),
        "ciqiu" | "ciqiu_dying" | "ciqiu_destroy" => Some(HostBehavior::WudianCiqiu),
        "ctianfu" => Some(HostBehavior::XianjianCtianfu),
        "ctianfu2" => Some(HostBehavior::XianjianCtianfu2),
        "ctianfu3" => Some(HostBehavior::XianjianCtianfu3),
        "cyqiaoxie" => Some(HostBehavior::Cyqiaoxie),
        "cyxianjiang" => Some(HostBehavior::SwdCyxianjiang),
        "daixing" => Some(HostBehavior::SwdDaixing),
        "daixing2" => Some(HostBehavior::SwdDaixing2),
        "daixing3" => Some(HostBehavior::SwdDaixing3),
        "dangping" => Some(HostBehavior::SwdDangping),
        "daofa" => Some(HostBehavior::SwdDaofa),
        "datong" => Some(HostBehavior::SwdDatong),
        "dddjinggou" => Some(HostBehavior::MaxRangeWeaponDamage),
        "dddmoyan" => Some(HostBehavior::D3Moyan),
        "dddxiaxing" => Some(HostBehavior::DddXiaxing),
        "diaobingqianjiang" | "youdishenru" | "g_youdishenru" => {
            Some(HostBehavior::WudianDiaobingCard)
        }
        "diewu" => Some(HostBehavior::XianjianDiewu),
        "diychanyuan" => Some(HostBehavior::YunchouDiychanyuan),
        "diyguhuo" => Some(HostBehavior::YunchouDiyguhuo),
        "diyjuntun" | "diyjuntun2" | "juntun" | "juntun_global" => {
            Some(HostBehavior::WudianDiyjuntun)
        }
        "diykuanggu" => Some(HostBehavior::YunchouDiykuanggu),
        "duanxing" => Some(HostBehavior::SwdDuanxing),
        "duanyi" => Some(HostBehavior::SwdDuanyi),
        "duanyue" => Some(HostBehavior::SwdDuanyue),
        "duci" => Some(HostBehavior::XianjianDuci),
        "duishi" => Some(HostBehavior::SwdDuishi),
        "dunxing" => Some(HostBehavior::SwdDunxing),
        "duoqi" => Some(HostBehavior::WudianDuoqi),
        "duwen" => Some(HostBehavior::WudianDuwen),
        "fanling" => Some(HostBehavior::XianjianFanling),
        "feidan" => Some(HostBehavior::WudianFeidan),
        "feiren" => Some(HostBehavior::WudianFeiren),
        "feitiao" => Some(HostBehavior::WudianFeitiao),
        "feixu" => Some(HostBehavior::XianjianFeixu),
        "feng" => Some(HostBehavior::XianjianFeng),
        "fenglue" => Some(HostBehavior::XianjianFenglue),
        "fengming" => Some(HostBehavior::SwdFengming),
        "fengze" => Some(HostBehavior::SwdFengze),
        "fenshi" => Some(HostBehavior::XianjianFenshi),
        "fuchou" | "fuchou3" | "liduan2" => Some(HostBehavior::WudianFuchou),
        "fuchou2" => Some(HostBehavior::WudianFuchou2),
        "funiao" => Some(HostBehavior::SwdFuniao),
        "fuyan" => Some(HostBehavior::SwdFuyan),
        "fzhenwei" => Some(HostBehavior::SwdFzhenwei),
        "gaizao" => Some(HostBehavior::Gaizao),
        "geju" => Some(HostBehavior::WudianGeju),
        "gongshen" => Some(HostBehavior::SwdGongshen),
        "gongxin" => Some(HostBehavior::Gongxin),
        "gtiandao" => Some(HostBehavior::SwdGTiandao),
        "guaili" => Some(HostBehavior::SwdGuaili),
        "guangshu" => Some(HostBehavior::WudianGuangshu),
        "guangshu_club" => Some(HostBehavior::WudianGuangshuClub),
        "guangshu_diamond" => Some(HostBehavior::WudianGuangshuDiamond),
        "guangshu_heart" => Some(HostBehavior::WudianGuangshuHeart),
        "guangshu_spade" => Some(HostBehavior::WudianGuangshuSpade),
        "guanhu" => Some(HostBehavior::SwdGuanhu),
        "guanri" => Some(HostBehavior::XianjianGuanri),
        "guihan" => Some(HostBehavior::WudianGuihan),
        "guijin" => Some(HostBehavior::XianjianGuijin),
        "guiyan" => Some(HostBehavior::SwdGuiyan),
        "guiying" => Some(HostBehavior::SwdGuiying),
        "guiyuan" => Some(HostBehavior::XianjianGuiyuan),
        "guozao" => Some(HostBehavior::SwdGuozao),
        "guxing" => Some(HostBehavior::SwdGuxing),
        "guxing3" => Some(HostBehavior::SwdGuxing3),
        "gxianyin" => Some(HostBehavior::Gxianyin),
        "heihuo" => Some(HostBehavior::SwdHeihuo),
        "hlingbo" => Some(HostBehavior::SwdHlingbo),
        "hlss_fengqi" => Some(HostBehavior::HlssFengqi),
        "hlss_fengqi2" => Some(HostBehavior::HlssFengqi2),
        "hlss_gaiming" => Some(HostBehavior::HlssGaiming),
        "hlss_xiangxing" => Some(HostBehavior::HlssXiangxing),
        "hlss_yueyin" => Some(HostBehavior::HlssYueyin),
        "hlss_yueyin2" => Some(HostBehavior::HlssYueyin2),
        "honglian" => Some(HostBehavior::YunchouHonglian),
        "huahu" => Some(HostBehavior::XianjianHuahu),
        "huajian" => Some(HostBehavior::SwdHuajian),
        "huajin" => Some(HostBehavior::SwdHuajin),
        "huajing" => Some(HostBehavior::SwdHuajing),
        "huangyu" => Some(HostBehavior::SwdHuangyu),
        "huanhun" => Some(HostBehavior::SwdHuanhun),
        "huanjian" => Some(HostBehavior::SwdHuanjian),
        "monkey" | "huanle_monkey" => Some(HostBehavior::HuanleMonkeyCard),
        "mianju" | "huanle_mianju" => Some(HostBehavior::HuanleMianjuCard),
        "kuwu" | "huanle_kuwu" => Some(HostBehavior::ShaDamageForceDiscard),
        "xuelunyang" | "huanle_xuelunyang" => Some(HostBehavior::HuanleXuelunyangCard),
        "jiuwei" | "huanle_jiuwei" => Some(HostBehavior::PhaseEndRecoverOrDraw),
        "huanling" => Some(HostBehavior::SwdHuanling),
        "huanxia" => Some(HostBehavior::SwdHuanxia),
        "huanxing" => Some(HostBehavior::SwdHuanxing),
        "huanyin" => Some(HostBehavior::SwdHuanyin),
        "hudun" => Some(HostBehavior::SwdHudun),
        "huhuan" => Some(HostBehavior::WudianHuhuan),
        "huimeng" => Some(HostBehavior::XianjianHuimeng),
        "huiqi" => Some(HostBehavior::SwdHuiqi),
        "hujing" => Some(HostBehavior::SwdHujing),
        "huodan" => Some(HostBehavior::SwdHuodan),
        "huopu" => Some(HostBehavior::SwdHuopu),
        "husha" => Some(HostBehavior::XianjianHusha),
        "husha_count" => Some(HostBehavior::XianjianHushaCount),
        "hutian" => Some(HostBehavior::Hutian),
        "hwendao" => Some(HostBehavior::SwdHwendao),
        "hxunzhi" => Some(HostBehavior::Hxunzhi),
        "hxunzhi2" => Some(HostBehavior::Hxunzhi2),
        "hyunshen" => Some(HostBehavior::SwdHyunshen),
        "hyunshen2" => Some(HostBehavior::SwdHyunshen2),
        "jianbi" => Some(HostBehavior::WudianJianbi),
        "jiang" => Some(HostBehavior::SwdJiang),
        "jianji" => Some(HostBehavior::SwdJianji),
        "jiaoxia" => Some(HostBehavior::YunchouJiaoxia),
        "jidong" => Some(HostBehavior::WudianJidong),
        "jiefen" => Some(HostBehavior::SwdJiefen),
        "jieyan" => Some(HostBehavior::YunchouJieyan),
        "jifeng" => Some(HostBehavior::Jifeng),
        "jijia" => Some(HostBehavior::WudianJijia),
        "jikong" => Some(HostBehavior::SwdJikong),
        "jilve" => Some(HostBehavior::Jilve),
        "jinlin" => Some(HostBehavior::SwdJinlin),
        "jinyan" => Some(HostBehavior::WudianJinyan),
        "jishe" => Some(HostBehavior::Jishe),
        "jiying" => Some(HostBehavior::SwdJiyingHeld),
        "jqimou" => Some(HostBehavior::SwdJqimou),
        "jtjubao" => Some(HostBehavior::XianjianJtjubao),
        "juedao" => Some(HostBehavior::WudianJuedao),
        "jufu" => Some(HostBehavior::SwdJufu),
        "juxi" => Some(HostBehavior::SwdJuxi),
        "juxi_count" => Some(HostBehavior::SwdJuxiCount),
        "kangyin" | "kangyin2" => Some(HostBehavior::WudianKangyin),
        "k_danggu" => Some(HostBehavior::KDanggu),
        "k_mowang" => Some(HostBehavior::KMowang),
        "kbolan" => Some(HostBehavior::SwdKbolan),
        "kbolan2" => Some(HostBehavior::SwdKbolan2),
        "kunlunjing" => Some(HostBehavior::Kunlunjing),
        "kuoyin" => Some(HostBehavior::WudianKuoyin),
        "kurou" => Some(HostBehavior::Kurou),
        "lanzhi" => Some(HostBehavior::SwdLanzhi),
        "lexue" => Some(HostBehavior::SwdLexue),
        "lianda" => Some(HostBehavior::SwdLianda),
        "liangce" => Some(HostBehavior::WudianLiangceRemain),
        "liangou" => Some(HostBehavior::WudianLiangou),
        "lianwu" => Some(HostBehavior::SwdLianwu),
        "liaoyuan" => Some(HostBehavior::Liaoyuan),
        "liduan" => Some(HostBehavior::WudianLiduan),
        "liefeng" => Some(HostBehavior::XianjianLiefeng),
        "lieren" => Some(HostBehavior::SwdLieren),
        "liguang" => Some(HostBehavior::XianjianLiguang),
        "lingfeng" => Some(HostBehavior::SwdLingfeng),
        "linghuo" => Some(HostBehavior::XianjianLinghuo),
        "lingquan" => Some(HostBehavior::XianjianLingquan),
        "lingshi" => Some(HostBehavior::SwdLingshi),
        "lingwu" => Some(HostBehavior::SwdLingwu),
        "lingxian" => Some(HostBehavior::SwdLingxian),
        "lingxin" => Some(HostBehavior::SwdLingxin),
        "lingxue" => Some(HostBehavior::XianjianLingxue),
        "lingyu" => Some(HostBehavior::XianjianLingyu),
        "lingyue" => Some(HostBehavior::SwdLingyue),
        "linyun" => Some(HostBehavior::SwdLinyun),
        "ljifeng" => Some(HostBehavior::SwdLjifeng),
        "lmazui" => Some(HostBehavior::SwdLmazui),
        "lmazui3" => Some(HostBehavior::SwdLmazui3),
        "longhuo" => Some(HostBehavior::XianjianLonghuo),
        "longxiang" => Some(HostBehavior::WudianLongxiang),
        "lqingcheng" => Some(HostBehavior::SwdLqingcheng),
        "luanji" => Some(HostBehavior::SwdLuanji),
        "luanjian" => Some(HostBehavior::XianjianLuanjian),
        "luanjian2" => Some(HostBehavior::XianjianLuanjian2),
        "lueying" => Some(HostBehavior::WudianLueying),
        "luomei" => Some(HostBehavior::SwdLuomei),
        "luweiyan" => Some(HostBehavior::YunchouLuweiyan),
        "luweiyan2" => Some(HostBehavior::YunchouLuweiyan2),
        "lxianglong" => Some(HostBehavior::Lxianglong),
        "mailun" => Some(HostBehavior::Mailun),
        "mbcmfangzhu" => Some(HostBehavior::MbcmFangzhu),
        "mbcmjiushi" => Some(HostBehavior::MbcmJiushi),
        "mbcmqingzheng" => Some(HostBehavior::MbcmQingzheng),
        "mbjuejin" => Some(HostBehavior::MbJuejin),
        "mbqianlong" => Some(HostBehavior::MbQianlong),
        "mbweitong" => Some(HostBehavior::MbWeitong),
        "meihuo" => Some(HostBehavior::SwdMeihuo),
        "mengji" => Some(HostBehavior::WudianMengji),
        "miedao" => Some(HostBehavior::Miedao),
        "miles_xueyi" => Some(HostBehavior::SwdMilesXueyi),
        "mingfu" => Some(HostBehavior::SwdMingfu),
        "minjing" => Some(HostBehavior::SwdMinjing),
        "minsha" => Some(HostBehavior::XianjianMinsha),
        "mohua2" => Some(HostBehavior::Mohua2),
        "mojian" => Some(HostBehavior::SwdMojian),
        "moshou" => Some(HostBehavior::MoshouImmunity),
        "moyan" => Some(HostBehavior::Moyan),
        "mufeng" => Some(HostBehavior::SwdMufengHeld),
        "mujing" => Some(HostBehavior::WudianMujing),
        "new_meibu" => Some(HostBehavior::NewMeibu),
        "new_meibu_range" => Some(HostBehavior::NewMeibuRange),
        "new_zhixi" => Some(HostBehavior::NewZhixi),
        "new_zhixi_mark2" => Some(HostBehavior::NewZhixi),
        "niepan" => Some(HostBehavior::Niepan),
        "ningbin" => Some(HostBehavior::XianjianNingbin),
        "ningshuang" => Some(HostBehavior::SwdNingshuang),
        "ningxian" => Some(HostBehavior::SwdNingxian),
        "nlianji" => Some(HostBehavior::SwdNlianji),
        "nsshuaiyan" => Some(HostBehavior::WudianShuaiyan),
        "owliyong" => Some(HostBehavior::WudianOwliyong),
        "paotai" => Some(HostBehavior::WudianPaotai),
        "paotai2" => Some(HostBehavior::WudianPaotaiFire),
        "pingxu" => Some(HostBehavior::SwdPingxu),
        "pingzhang" => Some(HostBehavior::WudianPingzhang),
        "pojian" => Some(HostBehavior::SwdPojian),
        "polang" => Some(HostBehavior::SwdPolang),
        "poxiao" => Some(HostBehavior::SwdPoxiao),
        "poxing" => Some(HostBehavior::SwdPoxing),
        "poyue" => Some(HostBehavior::SwdPoyue),
        "poyun" => Some(HostBehavior::XianjianPoyun),
        "pozhen" => Some(HostBehavior::SwdPozhen),
        "pozhou" => Some(HostBehavior::SwdPozhou),
        "pozhou2" => Some(HostBehavior::SwdPozhou2),
        "pozhou_sealed" => Some(HostBehavior::SwdPozhouSealed),
        "qianfang" => Some(HostBehavior::XianjianQianfang),
        "qianggu" => Some(HostBehavior::WudianQianggu),
        "qianggu2" => Some(HostBehavior::WudianQiangguGuard),
        "rangjie" => Some(HostBehavior::Rangjie),
        "rechanyuan" => Some(HostBehavior::Rechanyuan),
        "reguhuo" => Some(HostBehavior::Reguhuo),
        "rexue" => Some(HostBehavior::SwdRexue),
        "reyingzi" => Some(HostBehavior::SwdReyingzi),
        "rumeng" => Some(HostBehavior::SwdRumeng),
        "runxin" => Some(HostBehavior::XianjianRunxin),
        "sajin" => Some(HostBehavior::XianjianSajin),
        "sanjian" => Some(HostBehavior::SanjianWeaponDamage),
        "sbzhaxiang" => Some(HostBehavior::SbZhaXiang),
        "scsanruo"
        | "scschihe"
        | "scschiyan"
        | "scskuiji"
        | "scsniqu"
        | "scspicai"
        | "scstaoluan"
        | "scsxiaolu"
        | "scsyaozhuo"
        | "scszimou" => Some(HostBehavior::L10ScsCompanion),
        "senluo" => Some(HostBehavior::SwdSenluo),
        "shangxi" => Some(HostBehavior::SwdShangxi),
        "shaoying" => Some(HostBehavior::YunchouShaoying),
        "shehun" => Some(HostBehavior::SwdShehun),
        "sheling" => Some(HostBehavior::XianjianSheling),
        "shending" => Some(HostBehavior::SwdShending),
        "shengong" => Some(HostBehavior::Shengong),
        "wangmeizhike" => Some(HostBehavior::WangmeiZhikeCard),
        "youzhan" => Some(HostBehavior::WudianYouzhan),
        "shengshou" => Some(HostBehavior::SwdShengshou),
        "shenmu" => Some(HostBehavior::XianjianShenmu),
        "shenqiang" => Some(HostBehavior::WudianShenqiang),
        "shenwu" => Some(HostBehavior::XianjianShenwu),
        "shenyan" => Some(HostBehavior::SwdShenyan),
        "sheying" => Some(HostBehavior::XianjianSheying),
        "shihuo" => Some(HostBehavior::WudianShihuo),
        "shishi" => Some(HostBehavior::Shishi),
        "shiying" => Some(HostBehavior::XianjianShiying),
        "shouhu" => Some(HostBehavior::WudianShouhu),
        "shoulie" => Some(HostBehavior::SwdShoulie),
        "shouyin" => Some(HostBehavior::SwdShouyin),
        "shuiyun" => Some(HostBehavior::XianjianShuiyun),
        "shuiyun5" => Some(HostBehavior::XianjianShuiyun5),
        "siji" => Some(HostBehavior::SijiDiscardDraw),
        "sliufeng" => Some(HostBehavior::SwdSliufeng),
        "song" => Some(HostBehavior::XianjianSong),
        "songci" => Some(HostBehavior::Songci),
        "suiyan" => Some(HostBehavior::SwdSuiyan),
        "susheng" => Some(HostBehavior::SwdSusheng),
        "swd_guiyan2" => Some(HostBehavior::SwdGuiyan2),
        "swd_huajin2" => Some(HostBehavior::SwdHuajin2),
        "swd_huanxing_state" => Some(HostBehavior::SwdHuanxing2),
        "swd_jinlin2" => Some(HostBehavior::SwdJinlin2),
        "swd_lmazui2" => Some(HostBehavior::SwdLmazui2),
        "swd_wuxie" => Some(HostBehavior::SwdWuxie),
        "swd_xiuluo" => Some(HostBehavior::Xiuluo),
        "swd_zhenjiu2" => Some(HostBehavior::SwdZhenjiu2),
        "swdliuhong" => Some(HostBehavior::SwdLiuhong),
        "swdtianshu" => Some(HostBehavior::SwdTianshu),
        "sxianjing" => Some(HostBehavior::SwdSxianjing),
        "sxianjing_damage" => Some(HostBehavior::SwdSxianjingDamage),
        "sxianjing_gain" => Some(HostBehavior::SwdSxianjingGain),
        "tannang" => Some(HostBehavior::XianjianTannang),
        "tianhuo" => Some(HostBehavior::SwdTianhuo),
        "tianjian" => Some(HostBehavior::XianjianTianjian),
        "tianlun" => Some(HostBehavior::Tianlun),
        "tianshe" => Some(HostBehavior::XianjianTianshe),
        "tianshe2" => Some(HostBehavior::XianjianTianshe2),
        "tianxian" => Some(HostBehavior::XianjianTianxian),
        "tianxiang" => Some(HostBehavior::Tianxiang),
        "tianxiang2" => Some(HostBehavior::Tianxiang2),
        "tiebi" => Some(HostBehavior::SwdTiebi),
        "toudan" => Some(HostBehavior::SwdToudan),
        "touxi" => Some(HostBehavior::SwdTouxi),
        "touxi2" => Some(HostBehavior::SwdTouxi2),
        "tuji" => Some(HostBehavior::WudianTuji),
        "tuzhen" => Some(HostBehavior::SwdTuzhen),
        "txianqu" => Some(HostBehavior::XianjianTxianqu),
        "wangchen" => Some(HostBehavior::SwdWangchenHeld),
        "wangyou" => Some(HostBehavior::XianjianWangyou),
        "wangyou2" => Some(HostBehavior::XianjianWangyou2),
        "wanjun" => Some(HostBehavior::SwdWanjun),
        "wd_sanjian" => Some(HostBehavior::SanjianWeaponDamage),
        "wd_wangmeizhike" => Some(HostBehavior::WangmeiZhikeCard),
        "_shengke" | "wuxing_shengke" => Some(HostBehavior::ShengkeCheck),
        "wuxingpan_skill" => Some(HostBehavior::ChangeCardNature),
        "xiandan" => Some(HostBehavior::WudianXiandan),
        "xianghui" => Some(HostBehavior::SwdXianghui),
        "xianyin" => Some(HostBehavior::SwdXianyin),
        "xiaomoyu" => Some(HostBehavior::SwdXiaomoyu),
        "xicai" => Some(HostBehavior::YunchouXicai),
        "xiefang" => Some(HostBehavior::Xiefang),
        "xielei" => Some(HostBehavior::SwdXielei),
        "xielv" => Some(HostBehavior::SwdXielv),
        "xiepan" => Some(HostBehavior::XianjianXiepan),
        "xigua_mopaifanbei" | "_xigua_mopaifanbei_" => Some(HostBehavior::XiguaMopaiFanbei),
        "xigua_xueliangfanbei" | "_xigua_xueliangfanbei_" => {
            Some(HostBehavior::XiguaXueliangFanbei)
        }
        "xingdian" => Some(HostBehavior::SwdXingdian),
        "xingzhui" => Some(HostBehavior::SwdXingzhui),
        "xiongzi" => Some(HostBehavior::YunchouXiongzi),
        "xiyang" => Some(HostBehavior::WudianXiyang),
        "xjyingfeng" => Some(HostBehavior::XianjianXjyingfeng),
        "xjyufeng" => Some(HostBehavior::XianjianXjyufeng),
        "xjzhimeng" => Some(HostBehavior::XianjianXjzhimeng),
        "xjzhimeng2" => Some(HostBehavior::XianjianXjzhimeng2),
        "xshuangren" => Some(HostBehavior::XianjianXshuangren),
        "xshuangren2" => Some(HostBehavior::XianjianXshuangren2),
        "xtanxi" => Some(HostBehavior::XianjianXtanxi),
        "xuanmo" => Some(HostBehavior::XianjianXuanmo),
        "xuanyan" => Some(HostBehavior::XianjianXuanyan),
        "xuanyan2" => Some(HostBehavior::XianjianXuanyan2),
        "xuanyan3" => Some(HostBehavior::XianjianXuanyan3),
        "xuanying" => Some(HostBehavior::SwdXuanying),
        "xuanying_use" => Some(HostBehavior::SwdXuanyingUse),
        "xuanyuan" => Some(HostBehavior::Xuanyuan),
        "xuanyuanjian2" => Some(HostBehavior::XuanyuanjianDamage),
        "xuanyuanjian3" => Some(HostBehavior::XuanyuanjianBacklash),
        "xuanzhou" => Some(HostBehavior::SwdXuanzhou),
        "xuehuang" => Some(HostBehavior::SwdXuehuang),
        "xunmeng" => Some(HostBehavior::Xunmeng),
        "xunying" => Some(HostBehavior::XianjianXunying),
        "xuying" => Some(HostBehavior::SwdXuying),
        "ya" => Some(HostBehavior::XianjianYa),
        "yaliang" => Some(HostBehavior::YunchouYaliang),
        "yaotong" => Some(HostBehavior::SwdYaotong),
        "ycndiychanyuan" => Some(HostBehavior::YunchouDiychanyuan),
        "ycndiyguhuo" => Some(HostBehavior::YunchouDiyguhuo),
        "ycndiykuanggu" => Some(HostBehavior::YunchouDiykuanggu),
        "ycnguihan" => Some(HostBehavior::WudianGuihan),
        "ycnhonglian" => Some(HostBehavior::YunchouHonglian),
        "ycnjiaoxia" => Some(HostBehavior::YunchouJiaoxia),
        "ycnjieyan" => Some(HostBehavior::YunchouJieyan),
        "ycnluweiyan" => Some(HostBehavior::YunchouLuweiyan),
        "ycnluweiyan2" => Some(HostBehavior::YunchouLuweiyan2),
        "ycnshaoying" => Some(HostBehavior::YunchouShaoying),
        "ycnxicai" => Some(HostBehavior::YunchouXicai),
        "ycnxiongzi" => Some(HostBehavior::YunchouXiongzi),
        "ycnyaliang" => Some(HostBehavior::YunchouYaliang),
        "ycnyiesheng" => Some(HostBehavior::YunchouYiesheng),
        "ycnzonghuo" => Some(HostBehavior::YunchouZonghuo),
        "yeying" => Some(HostBehavior::SwdYeying),
        "yiesheng" => Some(HostBehavior::YunchouYiesheng),
        "yihua" => Some(HostBehavior::Yihua),
        "yijue" => Some(HostBehavior::SwdYijue),
        "yinbo" => Some(HostBehavior::WudianYinbo),
        "yinguo" => Some(HostBehavior::SwdYinguo),
        "yinshen" => Some(HostBehavior::WudianYinshen),
        "yinyue" => Some(HostBehavior::SwdYinyue),
        "yishan" => Some(HostBehavior::SwdYishan),
        "youyin" => Some(HostBehavior::SwdYouyin),
        "yuchen" => Some(HostBehavior::SwdYuchen),
        "yudun" => Some(HostBehavior::SwdYudun),
        "yuedong" => Some(HostBehavior::WudianYuedong),
        "yueren" => Some(HostBehavior::SwdYueren),
        "yuexing" => Some(HostBehavior::XianjianYuexing),
        "yulin" => Some(HostBehavior::SwdYulin),
        "yunchou" => Some(HostBehavior::SwdYunchou),
        "yxs_fanji" => Some(HostBehavior::YxsFanji),
        "yxs_fanji2" => Some(HostBehavior::YxsFanji2),
        "yxs_menshen" => Some(HostBehavior::YxsMenshen),
        "yxs_menshen2" => Some(HostBehavior::YxsMenshen2),
        "yxs_menshen3" => Some(HostBehavior::YxsMenshen3),
        "zaibian" => Some(HostBehavior::Zaibian),
        "zhangmu" => Some(HostBehavior::XianjianZhangmu),
        "zhanlu" => Some(HostBehavior::SwdZhanlu),
        "zhanxing" => Some(HostBehavior::SwdZhanxing),
        "zhaoyao" => Some(HostBehavior::XianjianZhaoyao),
        "zhencha" => Some(HostBehavior::WudianBaoleiSwitch),
        "zhencha_draw" => Some(HostBehavior::WudianZhenchaDraw),
        "zhenjiu" => Some(HostBehavior::SwdZhenjiu),
        "zhenlie" => Some(HostBehavior::Zhenlie),
        "zhiyuan" => Some(HostBehavior::WudianZhiyuan),
        "zhucheng" => Some(HostBehavior::WudianZhucheng),
        "zhucheng2" => Some(HostBehavior::WudianZhuchengGuard),
        "zhuhai" => Some(HostBehavior::SwdZhuhai),
        "zhuwanzhongji" | "_xigua_zhuwanzhongji_" => Some(HostBehavior::ZhuWanzhongJi),
        "zhuyu" => Some(HostBehavior::SwdZhuyu),
        "zhuyue" => Some(HostBehavior::XianjianZhuyue),
        "ziyu" => Some(HostBehavior::WudianZiyu),
        "zonghuo" => Some(HostBehavior::YunchouZonghuo),
        "donghuangzhong" => Some(HostBehavior::SwdDonghuangzhongCard),
        "g_chenhuodajie" => Some(HostBehavior::ChenhuodajieCard),
        "g_jinlianzhu" => Some(HostBehavior::JinlianzhuCancel),
        "g_shencaojie" => Some(HostBehavior::SwdShencaojieGuard),
        "g_shenmiguo" => Some(HostBehavior::SwdShenmiguoWindow),
        "g_shihuifen" => Some(HostBehavior::U05GujianCardBatch),
        "g_yuchan_swap" => Some(HostBehavior::SwdYuchanSwap),
        "gouhunluo2" => Some(HostBehavior::SwdGouhunluoSub),
        "gw_ciguhanshuang" => Some(HostBehavior::GwentCiguState),
        "gw_kunenfayin" => Some(HostBehavior::GwentKunenState),
        "gw_qinpendayu" => Some(HostBehavior::GwentQinpenState),
        "gw_birinongwu" => Some(HostBehavior::GwentBirinState),
        "gw_aerdeyin"
        | "gw_baobaoshu"
        | "gw_nuhaifengbao"
        | "gw_youer"
        | "gw_dieyi"
        | "gw_leizhoushu"
        | "_gainspell"
        | "gw_xianzumaijiu"
        | "gwmaoxian_hengsaite_sha" => Some(HostBehavior::U05GwentCardSkill),
        "sqlongyin"
        | "sqlongnu"
        | "sqlonghuo"
        | "sqlongwu"
        | "jielue"
        | "gwhuanshuang"
        | "gwhuanshuang_disable"
        | "gwjinli"
        | "gwjinli_jiu"
        | "gwliaotian"
        | "gwhuanbi"
        | "gwminxiang"
        | "gwlangshi"
        | "gwjingtian"
        | "gwjingtian2"
        | "gwjingshi"
        | "gwweitu"
        | "gwweitu_gain"
        | "gwgouhun"
        | "gwfutian"
        | "gwzhongmo"
        | "gwyewu"
        | "g_gw_yewu" => Some(HostBehavior::U05GwentCharBatchA),
        "shuangxi"
        | "yangfan"
        | "gwfengshi"
        | "gwchenshui"
        | "gwliedi"
        | "julian"
        | "gwfusheng"
        | "gwqinwu"
        | "gwqinwu2"
        | "huanshu"
        | "huanshu2"
        | "huanshu3"
        | "gwjieyin"
        | "zhengjun"
        | "gwxuezhan"
        | "bolang"
        | "bolang_gain"
        | "gwjushi"
        | "gwjushi2"
        | "gwfengchi" => Some(HostBehavior::U05GwentCharBatchB),
        "lingji"
        | "gwjinyan"
        | "gwshenyu"
        | "junchi"
        | "hupeng"
        | "hupeng2"
        | "hupeng3"
        | "hupeng4"
        | "hunmo"
        | "hunmo2"
        | "shuijian"
        | "yunhuo"
        | "yinzhang"
        | "gwtianbian"
        | "gwxiaoshou"
        | "kuanglie"
        | "kuanglie2"
        | "gwjiquan" => Some(HostBehavior::U05GwentCharBatchC),
        "nuhou"
        | "shewu"
        | "shewu_dist"
        | "gwzhanjiang"
        | "gwzhanjiang2"
        | "gwzhanjiang3"
        | "gwchuanxin"
        | "fengjian"
        | "huandie"
        | "huandie_discard"
        | "xuezhou"
        | "xuezhou_hp"
        | "xuezhou_hp2"
        | "fayin"
        | "gwbaquan"
        | "huihun"
        | "lanquan" => Some(HostBehavior::U05GwentCharBatchD),
        "huajin2" => Some(HostBehavior::SwdHuajin2),
        "huanxing2" => Some(HostBehavior::SwdHuanxing2),
        "jiguanyaoshu_skill" => Some(HostBehavior::SwdJiguanyaoState),
        "jiguanyuan" => Some(HostBehavior::SwdJiguanyuanCard),
        "lianyaohu_skill" => Some(HostBehavior::SwdLianyaohuShouna),
        "lmazui2" => Some(HostBehavior::SwdLmazui2),
        "nigong" => Some(HostBehavior::SwdNigongAccumulate),
        "shuchui2" => Some(HostBehavior::SwdShuchuiUse),
        "xiajiao" => Some(HostBehavior::GujianFoodXiajiao),
        "xuanyuanjian" => Some(HostBehavior::XuanyuanjianDamage),
        "dddyuanzhen"
        | "dddzhishu"
        | "dddjuxian"
        | "dddjungui"
        | "dddshilie"
        | "dddyeshen"
        | "dddqiaoduan"
        | "ddddiedang"
        | "dddanliu"
        | "dddguiying"
        | "dddbingjian"
        | "dddtongyu" => Some(HostBehavior::U01DddBatchA),
        "dddzhilian"
        | "dddjijian"
        | "dddjijian_old"
        | "dddzhengjun"
        | "dddxianxi"
        | "dddlingyong"
        | "dddxuxiao"
        | "dddxuyu"
        | "dddshijian"
        | "dddxuyu_old"
        | "dddshijian_old"
        | "dddtaisi"
        | "dddquche"
        | "dddqianlong"
        | "ddddongcha"
        | "dddzhijie" => Some(HostBehavior::U01DddBatchB),
        "dddqiahua"
        | "dddxunxun"
        | "dddfusi"
        | "dddfusi_global"
        | "dddtuoji"
        | "dddchashi"
        | "dddqice"
        | "dddmiaoxing"
        | "dddfushi"
        | "dddfenji"
        | "dddxiaheng"
        | "dddfengzheng"
        | "dddyulv"
        | "dddyulv_old"
        | "dddfenye"
        | "dddshichao" => Some(HostBehavior::U01DddBatchC),
        "dddyouxue"
        | "dddchengjing"
        | "dddduanbing"
        | "dddxiaoxing"
        | "dddlangzhi"
        | "dddfuyi"
        | "dddzhuanshe"
        | "dddweiqiu"
        | "dddlianer"
        | "dddanzhi"
        | "dddshixing"
        | "ddddanggu"
        | "dddfuzong"
        | "dddlanghuai"
        | "dddxuanlun"
        | "dddjiexing"
        | "dddbailei" => Some(HostBehavior::U01DddBatchD),
        // U05-2 古剑奇谭 CardBatch & Food/Equip (card/gujian/skill.js)
        "ziyangdan" => Some(HostBehavior::GujianFoodZiyangdan),
        "luyugeng" => Some(HostBehavior::GujianFoodLuyugeng),
        "mizhilianou" => Some(HostBehavior::GujianFoodMizhilianou),
        "chunbing" => Some(HostBehavior::GujianFoodChunbing),
        "gudonggeng" => Some(HostBehavior::GujianFoodGudonggeng),
        "qingtuan" => Some(HostBehavior::GujianFoodQingtuan),
        "liyutang" => Some(HostBehavior::GujianFoodLiyutang),
        "yougeng" => Some(HostBehavior::GujianFoodYougeng),
        "molicha" => Some(HostBehavior::GujianFoodMolicha),
        "yuanbaorou" => Some(HostBehavior::GujianFoodYuanbaorou),
        "tanhuadong" => Some(HostBehavior::GujianFoodTanhuadong),
        "mapodoufu" => Some(HostBehavior::GujianFoodMapodoufu),
        "yunvyuanshen_skill" => Some(HostBehavior::GujianYunvyuanshen),
        "mutoumianju_skill" => Some(HostBehavior::GujianMutouEquipSkill),
        "gjyuheng_skill" => Some(HostBehavior::GujianYuhengEquipSkill),
        "xiajiao2"
        | "xiajiao3"
        | "heilonglinpian"
        | "gjyuheng_plus_temp"
        | "gjyuheng_plus_skill"
        | "gjyuheng_pro_skill"
        | "shihuifen" => Some(HostBehavior::U05GujianCardBatch),
        // U05-2 古剑奇谭 CharBatchA..E (character/gujian/skill.js)
        "xuelu"
        | "fanshi"
        | "qinglan"
        | "yuehua"
        | "gjqtwuxie"
        | "xlqianhuan"
        | "meihu"
        | "xidie"
        | "xidie2"
        | "fanyin"
        | "mingkong"
        | "mingkong2"
        | "fumo"
        | "shahun"
        | "shahun2" => Some(HostBehavior::U05GujianCharBatchA),
        "zuiji"
        | "manwu"
        | "xfanghua"
        | "jianwu"
        | "zuizhan"
        | "meiying"
        | "yanjia"
        | "xiuhua"
        | "liuying" => Some(HostBehavior::U05GujianCharBatchB),
        "shishui"
        | "chizhen"
        | "shahai-gujian__ref_dangping"
        | "dangping2"
        | "yunyin"
        | "duhun"
        | "shahai-gujian__ref_xuanning"
        | "xuanning1"
        | "xuanning2"
        | "liuguang"
        | "yangming"
        | "yangming2" => Some(HostBehavior::U05GujianCharBatchC),
        "zhaolu"
        | "jiehuo"
        | "jiehuo2"
        | "yuling"
        | "yuling1"
        | "yuling2"
        | "yuling3"
        | "yuling4"
        | "yuling5"
        | "yuling6" => Some(HostBehavior::U05GujianCharBatchD),
        "lingyan"
        | "xunjian"
        | "woxue"
        | "yange"
        | "lianjing"
        | "cihong"
        | "zhenying"
        | "lingnu"
        | "humeng"
        | "xianju"
        | "xuanci"
        | "qingshu"
        | "yunyou"
        | "xuanzhen" => Some(HostBehavior::U05GujianCharBatchE),
        // L28 雷霆万钧 Batch A（武将 1–9）
        "th_dunshi" | "th_bushi" | "th_zhongzhuang" | "th_caiyi" | "th_guili" | "th_chongxu"
        | "miaojian" | "shhlianhua" | "th_shengong" | "olqisi" | "th_pingcai"
        | "th_pcaudio_wolong_card" | "th_pcaudio_fengchu_card" | "th_pcaudio_shuijing_card"
        | "th_pcaudio_xuanjian_card" | "xinfu_pdgyingshi" | "th_zhengjing" | "th_zhengjing2"
        | "th_chengye" | "th_buxu" | "th_yufeng" | "th_tianshu" | "th_g_taipingyaoshu_ai"
        | "th_taipingyaoshu"
        | "wushuangfangtianji_skill" | "guilongzhanyuedao" | "chixueqingfeng"
        | "bintieshuangji_skill" | "wutiesuolian_skill" | "wuxinghelingshan_skill"
        | "linglongshimandai_skill" | "hongmianbaihuapao_skill" | "qimenbagua"
        | "guofengyupao" | "huxinjing" | "heiguangkai_skill"
        | "shufazijinguan_skill" | "xuwangzhimian" | "tianjitu_skill"
        | "taigongyinfu_skill" | "taigongyinfu_link" | "sanlve_skill"
        | "zhaogujing_skill" => Some(HostBehavior::L28ThunderBatchA),
        // L28 雷霆万钧 Batch B（武将 10–14）
        "th_jingxie" | "th_qiaosi" | "th_tiansuan" | "th_tiansuan2" | "th_tiansuan2_0"
        | "th_tiansuan2_1" | "th_tiansuan2_damage" | "th_tiansuan2_2" | "th_tiansuan2_ai"
        | "th_tiansuan2_fire" | "th_tiansuan2_3" | "th_tiansuan2_4" | "th_gushe" | "th_gushe2"
        | "rejici" | "th_geyuan" | "th_jieshu" | "th_gusuan" | "th_yeyan" | "th_qinyin"
        | "pro_qinyin" | "th_tianming" => Some(HostBehavior::L28ThunderBatchB),
        // L27 选将美化 Batch A（16 核心技能 + 3 标记/死代码桩）
        "regushe" | "kuangcai" | "sbtongye" | "baonvezhi" | "baonvezhi_mark" | "clandianzhan"
        | "rehuashen" | "rehuashen_mark" | "rexinsheng" | "taoluan" | "shouxi" | "dinghan"
        | "quanji" | "huamu" | "potzhuangshi" | "qimou" | "reqimou" | "sbjianxiong"
        | "jinghe" => Some(HostBehavior::L27BeautifyBatchA),
        // L27 选将美化 Batch B（其余 16 核心技能 + 2 辅助子技能 + 生命周期钩子）
        "bolan" | "bolan_g" | "drlt_duorui" | "pro_drlt_duorui" | "xinfu_xionghuo" | "mbwuling"
        | "sbqingzheng" | "spyanjiao" | "twxiafeng" | "twxingzhui" | "xingchong" | "dcxiaoxi"
        | "twfengji" | "twharvestinori" | "twzhouhu" | "twzuhuo" | "twzhouzu" | "jishe2"
        | "new_zhixi_recover" | "dcyingshi" | "drlt_congjian" | "new_rehenduan"
        | "new_requanji" | "sbliegong" | "sbmingjian" | "splveying" | "spshidi" | "spyingjian"
        | "twfuman" | "twguowu" | "twjieli" | "zhuiting" | "l27_init" | "l27_precontent" => {
            Some(HostBehavior::L27BeautifyBatchB)
        }
        // L19 末日乱斗 Batch A（模式 1–3 核心技能 + 20 注释/VFX 桩）
        "baozou" | "ganran" | "ganran2" | "_jisuangailv" | "_tuizhi" | "_tuizhi2" | "_tuizhi3"
        | "_jiangshi" | "_jiangshi2" | "_jiangshi3" | "_jiangshi4" | "_huzhu" | "_gainSkill"
        | "_wujinmoshiRE" | "wujinmoshiRE1" | "wujinmoshiRE2" | "_wujinmoshiDamage"
        | "_wujinmoshiDamageE" | "_wujinmoshiMaxHandCard" | "_wujinmoshiMaxHandCardE"
        | "_wujinmoshiDraw" | "_wujinmoshiDrawE" | "_wujinmoshiRecover"
        | "_wujinmoshiRecoverE" | "chanceIdentity" | "chanceIdentity0" | "chanceIdentity2"
        | "chanceIdentity3" | "chanceIdentity4" | "chanceIdentity5" | "chanceIdentity6"
        | "chanceIdentity7" | "chanceIdentity8" | "chanceIdentity9" | "chanceIdentity10"
        | "chanceIdentity11" | "chanceIdentity12" | "chanceIdentity13" | "chanceIdentity14"
        | "chanceIdentity15" | "_jiangshiTx" | "_jiangshiTx2" | "_jiangshiTx3"
        | "_keepideskill" => Some(HostBehavior::L19DoomsdayBatchA),
        // L19 末日乱斗 Batch B（模式 4–10 RPG/祖安桥接技能）
        "_levelskill" | "_expskill" | "_exp2skill" | "_useskill" | "_killskill"
        | "_relifeskill" | "_yuanqiskill" | "_zhongzhubuff" | "_jingchengadddie"
        | "_zhongfandieskill" | "_tamodeskill" | "_zhudieskill" | "_qunxiongzhengbaskill"
        | "_guanmodeskill" | "_guandieskill" | "_guixinmodeskill" | "_huanhaiskill"
        | "_seadreammodeskill" | "_dietime" | "_anshaskill" => {
            Some(HostBehavior::L19DoomsdayBatchB)
        }
        // L20 标记补充 Batch A（8 个国战/修改技能 + 3 个空标记桩 + 34 个纯 UI/Spine/音效钩子）
        "wengua" | "wengua5" | "diaohulishan" | "_guozhan_marks" | "xianqu_mark"
        | "_zhulianbihe_mark_tao" | "_yinyang_mark_add" | "_aozhan_judge" | "zhulianbihe_mark"
        | "yinyang_mark" | "yexinjia_mark" | "_xigua_feiyanbahu_" | "_shiqihujia_"
        | "_guozhanChooseControlCHG" | "_guozhan_diebiaoji_" | "_guozhan_duiweibiaoji_gua_"
        | "_guozhan_duiweibiaoji_gua_gua_" | "_guozhanhalfchghp" | "_shoushajisha_"
        | "_shoushajisha_delete_" | "_diankuangtulu_gua" | "_wanjunqushou_gua"
        | "_o_o_recovertrigger" | "_o_o_miaoshouhuichun" | "_o_o_yishugaochao" | "_losehp_"
        | "_mianshang_" | "_player_baojiA" | "_player_baojiB" | "_player_baojiC"
        | "_player_baojiD" | "_player_baojiE" | "_player_baojiF" | "_player_baojiG"
        | "_player_baojiH" | "_jiubuff_" | "_jiubuff_xiaoshi_" | "_player_game_guagua"
        | "_guozhan_quanjubiaoji_guagua" | "_shiyongkapaitexiao_"
        | "_shiyongkapaitexiao_xi_gua_" | "_shanghaipaishiyongtexiao_"
        | "_player_daojianfu_xxgg_fuzi_" | "_binglintu" | "_dysp" => {
            Some(HostBehavior::L20MarkBatchA)
        }
        // L20 标记补充 Batch B（23 个核心规则技能；`zuoxing` 在上游 extension.js:6155-6204
        // 整段注释，L20 不覆盖该技能）
        "zhengsu" | "zhenyi_spade" | "xinhongyan" | "xinfu_langxi" | "tspowei" | "qingyu"
        | "mibei" | "fz_new_rewusheng" | "fz_gzpaoxiao" | "fz_new_longdan" | "fz_new_tieji"
        | "fz_liegong" | "fz_xinkuanggu" | "rekuanshi" | "kuanshi" | "kuanshi2" | "rechuhai"
        | "xinfu_lingren" | "sbtieji" | "sbduanliang" | "shouye" | "mbmowang" | "_chongzhu" => {
            Some(HostBehavior::L20MarkBatchB)
        }
        // U08 英雄杀 Batch A（23 技能）
        "guimian" | "lyuxue" | "lyuxue2" | "yaoji" | "liebo" | "zhuxin" | "wlianhuan"
        | "huli" | "yixin" | "xianqu" | "zbudao" | "zbudao2" | "taiji" | "fengliu"
        | "luobi" | "yaoyi" | "shiqin" | "zyhufu" | "hanbei" | "yxsdili" | "kuangchan"
        | "zhijie" | "chujia" => Some(HostBehavior::U08YxsBatchA),
        // U08 英雄杀 Batch B（26 技能 + kanpo）
        "baihe" | "yinyang" | "xiushen" | "jiean" | "yanyi" | "batu" | "wumu" | "ysheshen"
        | "yxschangnian" | "yxschangnian2" | "yxsciqiu" | "sanbanfu" | "sanbanfu2"
        | "sanbanfu3" | "bingsheng" | "taolue" | "shentan" | "kanpo" | "hanqiang" | "biaoqi"
        | "wluoyan" | "heqin" | "heqin2" | "heqin3" | "chajue" | "chajue2" | "tiewan" => {
            Some(HostBehavior::U08YxsBatchB)
        }
        // U08 英雄杀 Batch C（27 技能）
        "qianglue" | "sheshu" | "lguiyin" | "tongyu" | "jimin" | "xiadao" | "lzhangyi"
        | "yizhuang" | "yizhuang2" | "kongju" | "tuqiang" | "xumou" | "zhensha" | "ducai"
        | "ducai2" | "ducai3" | "tongling" | "fanpu" | "fenghuo" | "nichang" | "nichang2"
        | "fengyan" | "fengyi" | "wange" | "wange2" | "seyou" | "sheshi" => {
            Some(HostBehavior::U08YxsBatchC)
        }
        // U08 英雄杀 Batch D（26 技能 + qiangxi）
        "bolehuiyan" | "bolehuiyan2" | "bolehuiyan3" | "oldbolehuiyan" | "oldbolehuiyan2"
        | "xiangma" | "yxsweiyi" | "qiandu" | "nvquan" | "nvquan1" | "nvquan2" | "nvquan3"
        | "feigong" | "jianai" | "jieyong" | "jieyong2" | "jieyong3" | "zhulu" | "xieling"
        | "qiangyun" | "cike" | "qiangxi" | "miaobix" | "miaobi" | "zhexian" | "guifu"
        | "lshengong" => Some(HostBehavior::U08YxsBatchD),
        // L01 AI优化（68 项 L01 专属技能）
        "_aiyh_firstKs" | "_findZhong" | "_aiyh_fake_prohibited" | "_aiyh_nhFriends"
        | "_mjAiSkill" | "_fenghuyunlong" | "_tianxiayitong" | "jlnj" | "_wanjunpiyi" => {
            Some(HostBehavior::L01FirstKs)
        }
        "_aiyh_neiKey" => Some(HostBehavior::L01NeiKey),
        "_attzhi" | "attcard" | "attdamage" | "attlosehp" | "attrecover" | "attgain"
        | "attlose" | "attturnover" | "attbo" | "attmingbian" | "attmingbian1"
        | "attfanzhuang" | "attbaolu" | "atttiaofan" | "tfattzhu" | "tfattzhong" | "atttf"
        | "_yingshilanggu" => Some(HostBehavior::L01AttitudeTracker),
        "_aiyh_meks" | "_aiyh_fixEQz" | "_aiyh_fixECf" | "_aiyh_fixWj" | "aiMangju"
        | "_shyg_qjAi" | "_aiyh_reserved_shan" | "aiyh_disabled_shan" | "_shyg_wuxie"
        | "_shyg_wuxie_jiedao" | "_shyg_wuxie_lei" | "shyg_wuxie_fy" | "_shyg_wugu"
        | "shyg_wugu_ss_pai" | "shyg_wugu_wss_pai" | "_shyg_Discard" | "shyg_Discard_key"
        | "_shyg_Discard_du" | "shyg_Discard_du_useful" | "shyg_mz" | "shyg_smz" | "shyg_tz"
        | "shyg_tf" | "chaofeng_common" | "chaofeng_rare" | "chaofeng_epic"
        | "chaofeng_legend" | "_shyg" | "gjcx_zhuAi" | "gjcx_neiAi" | "gjcx_neiZhong"
        | "gjcx_neiFan" | "gjcx_neiJiang" | "_aiyh_lianhe" | "_aiyh_wqjz" | "zhenlie_lose"
        | "twzhian_ai" | "dcwumei_all" | "aiyh_gjcx_qj" | "_gjcx_correct" => {
            Some(HostBehavior::L01AiRuleHooks)
        }
        _ => super::zuan_data::lookup_l25_zuan_host_behavior(clean),
    }
}

/// Push a `HostSkillDecl` (plus any `group_children`) if not yet present in `seen_skills`.
pub fn push_host_skill_if_unseen(
    skill_name: &str,
    behavior: HostBehavior,
    host_skills: &mut Vec<HostSkillDecl>,
    seen_skills: &mut BTreeSet<String>,
) {
    if seen_skills.insert(skill_name.to_string()) {
        host_skills.push(HostSkillDecl {
            name: names::intern(skill_name.to_string()),
            behavior,
            presentation_binding: None,
        });
    }
    for &(child_name, child_behavior) in behavior.group_children() {
        if seen_skills.insert(child_name.to_string()) {
            host_skills.push(HostSkillDecl {
                name: child_name,
                behavior: child_behavior,
                presentation_binding: None,
            });
        }
    }
}

/// Dedicated U05-7d (`swd/skill.js` 4132..8107) host behavior lookup used only by
/// `shahai_attach` so frozen `decision34_pack9` coexistence locks remain untouched.
pub(crate) fn lookup_u05_swd_7d_host_behavior(skill_name: &str) -> Option<HostBehavior> {
    match skill_name {
        "duijue" => Some(HostBehavior::SwdDuijue),
        "duijue2" => Some(HostBehavior::SwdDuijueOut),
        "duijue3" => Some(HostBehavior::SwdDuijueTrigger),
        "tanlin" => Some(HostBehavior::SwdTanlin),
        "tanlin2" => Some(HostBehavior::SwdTanlinTarget),
        "tanlin3" => Some(HostBehavior::SwdTanlinOwner),
        "tanlin4" => Some(HostBehavior::SwdTanlinRange),
        "tanlin_defence" => Some(HostBehavior::SwdTanlinDefence),
        "oldliaoyuan" => Some(HostBehavior::SwdOldLiaoyuan),
        "oldliaoyuan2" => Some(HostBehavior::SwdOldLiaoyuan2),
        "huanling2" => Some(HostBehavior::SwdHuanling2),
        "lexue2" => Some(HostBehavior::SwdLexue2),
        "swdtianshu_remove" => Some(HostBehavior::SwdTianshuRemove),
        "yinguo2" => Some(HostBehavior::SwdYinguo2),
        "susheng2" => Some(HostBehavior::SwdSusheng2),
        "moyu" => Some(HostBehavior::SwdMoyu),
        "guiyan2" => Some(HostBehavior::SwdGuiyan2),
        "zhenjiu2" => Some(HostBehavior::SwdZhenjiu2),
        "wangchen2"
        | "wangchen3"
        | "kunlunjing1"
        | "kunlunjing2"
        | "liaoyuan2"
        | "liaoyuan3"
        | "yishan2"
        | "lexue1"
        | "swdtianshu2"
        | "swdtianshu3"
        | "xiaomoyu2"
        | "guxing1"
        | "yueren2"
        | "yinguo3"
        | "mailun31"
        | "mailun32"
        | "mailun41"
        | "mailun42"
        | "mailun5"
        | "mailun6"
        | "mailun7"
        | "mailun8" => Some(HostBehavior::SwdU05_7dSub),
        _ => None,
    }
}

pub(crate) fn push_companion_if_absent(
    host_skills: &mut Vec<HostSkillDecl>,
    sk_name: &str,
    behavior: HostBehavior,
) {
    let interned = names::intern(sk_name.to_string());
    if !host_skills.iter().any(|d| d.name == interned) {
        host_skills.push(HostSkillDecl {
            name: interned,
            behavior,
            presentation_binding: None,
        });
    }
}

pub(crate) fn inject_pre_general_companions(
    host_skills: &mut Vec<HostSkillDecl>,
    deferred: &[String],
) {
    if host_skills.iter().any(|d| d.name == "reguhuo") {
        push_companion_if_absent(host_skills, "rechanyuan", HostBehavior::Rechanyuan);
    }
    if deferred.iter().any(|name| name == "shichangshi") {
        for &(sk_name, behavior) in &[
            ("k_danggu", HostBehavior::KDanggu),
            ("k_mowang", HostBehavior::KMowang),
        ] {
            push_companion_if_absent(host_skills, sk_name, behavior);
        }
        for &sk_name in &crate::decision34_closure::L10_SCS_COMPANION_SKILLS {
            push_companion_if_absent(host_skills, sk_name, HostBehavior::L10ScsCompanion);
        }
    }
}

pub(crate) fn inject_post_general_companions(host_skills: &mut Vec<HostSkillDecl>) {
    for (trigger_beh, sub_name, sub_beh) in [
        (HostBehavior::SwdYinguo, "yinguo2", HostBehavior::SwdYinguo2),
        (HostBehavior::SwdSusheng, "susheng2", HostBehavior::SwdSusheng2),
    ] {
        if host_skills.iter().any(|d| d.behavior == trigger_beh) {
            push_companion_if_absent(host_skills, sub_name, sub_beh);
        }
    }
}
