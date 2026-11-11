//! Fixed source ordering; source paths and fingerprints are recorded with C331 evidence.
use super::*;

impl HostBehavior {
    pub fn source_order(&self) -> Option<(Timing, i32, bool)> {
        Some(match self {
            // Source flags determine these tuples; event/optional semantics stay in metadata.
            Self::BossBaimang2 => (Timing::Normal, 1, false),
            Self::BossBaimang3 => (Timing::Normal, 1, false),
            Self::BossBianshen3 => (Timing::Normal, 1, false),
            Self::BossBianshen4 => (Timing::Normal, 1, false),
            Self::BossChiyan2 => (Timing::Normal, 1, false),
            Self::BossChiyan3 => (Timing::Normal, 1, false),
            Self::BossHuanrenDie => (Timing::Normal, -999, false),
            Self::BossQingmu2 => (Timing::Normal, 1, false),
            Self::BossQingmu3 => (Timing::Normal, 1, false),
            Self::BossXuanlin2 => (Timing::Normal, 1, false),
            Self::BossXuanlin3 => (Timing::Normal, 1, false),
            Self::GujianYunvyuanshen => (Timing::Normal, 610, true),
            Self::GwentBirinState => (Timing::Normal, 1, false),
            Self::GwentQinpenState => (Timing::Normal, 1, false),
            Self::SwdYuchanSwap => (Timing::Normal, -99, false),
            Self::Xiefang => (Timing::Normal, -99, false),
            Self::XiguaXueliangFanbei => (Timing::Normal, 1, false),
            // Phase 5 default-matching source-order bindings
            Self::BossBaiyi
            | Self::BossBaiyiThunder
            | Self::BossChuanyun
            | Self::BossChiyi
            | Self::BossChiyiMark
            | Self::BossHuanren
            | Self::BossQingyi
            | Self::BossTianyun
            | Self::DddXiaxing
            | Self::GujianFoodGudonggeng
            | Self::GujianFoodLiyutang
            | Self::GujianFoodMapodoufu
            | Self::GujianFoodQingtuan
            | Self::GujianFoodTanhuadong
            | Self::GujianFoodXiajiao
            | Self::GujianFoodYougeng
            | Self::GwentCiguState
            | Self::GwentKunenState
            | Self::Miedao
            | Self::Shishi
            | Self::SwdBaishouzhihuHeld
            | Self::SwdCangchizhibiHeld
            | Self::SwdGouhunluoSub
            | Self::SwdHuanglinzhicongHeld
            | Self::SwdJiguanyaoGain
            | Self::SwdJiguanyuanGain
            | Self::SwdJuxiCount
            | Self::SwdQinglongzhiguiHeld
            | Self::SwdSxianjingDamage
            | Self::SwdSxianjingGain
            | Self::SwdXuanwuzhihuangHeld
            | Self::SwdZhuquezhizhangHeld
            | Self::WudianPaotaiFire
            | Self::WudianQiangguGuard
            | Self::WudianZhenchaDraw
            | Self::XianjianHushaCount
            | Self::Xunmeng
            | Self::Zaibian => (Timing::Normal, 0, true),
            // C331 16 translation-context source-order bindings (empty lib.translate context -> order_selectable = false)
            Self::BossBaimang2x
            | Self::BossBaimang3x
            | Self::BossBianshen3x
            | Self::BossBianshen4x
            | Self::BossChiyan2x
            | Self::BossChiyan3x
            | Self::BossQingmu2x
            | Self::BossQingmu3x
            | Self::BossXuanlin2x
            | Self::BossXuanlin3x => (Timing::Normal, -1000, false),
            Self::JinlianzhuCancel
            | Self::ShengkeCheck
            | Self::SwdLexue
            | Self::SwdShencaojieGuard
            | Self::SwdShenmiguoWindow
            | Self::ZhuWanzhongJi => (Timing::Normal, 0, false),
            // C331 12 mechanism-coupled source-order bindings
            Self::SanjianWeaponDamage => (Timing::Normal, -25, true),
            Self::SwdNigongAccumulate => (Timing::Normal, -25, true),
            Self::SwdXuanyingDamage => (Timing::Normal, 0, true),
            Self::MbWeitong => (Timing::Normal, 1000, true),
            Self::MbQianlong => (Timing::Normal, 600, true),
            Self::Kunlunjing => (Timing::Normal, 1000, true),
            Self::GwentFengyinState
            | Self::SwdFengyinState
            | Self::Niepan
            | Self::SwdShouyin
            | Self::XianjianChengxin
            | Self::XianjianShuiyun5 => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/extra.js:456
            Self::HlssFengqi => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/extra.js:296
            Self::HlssXiangxing => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/extra.js:181
            Self::HlssYueyin => (Timing::First, 1, false),
            // noname-liuli/noname/character/extra.js:263
            Self::HlssYueyin2 => (Timing::Normal, 2000, true),
            // noname-liuli/noname/character/mobile.js:17117
            Self::MbcmQingzheng => (Timing::Normal, 0, true),
            // noname/apps/core/character/sp/skill.js:35046
            Self::NewMeibu => (Timing::Normal, 0, true),
            // noname/apps/core/character/sp/skill.js:35205
            Self::NewZhixi => (Timing::First, 0, true),
            // noname-liuli/noname/character/mobile.js:13342
            Self::Rangjie => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/shenhua.js:3841
            Self::SwdJiang => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/shenhua.js:4710
            Self::SwdLieren => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/refresh.js:12865
            Self::SwdReyingzi => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/refresh.js:13643
            Self::SwdZhuhai => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/shenhua.js:6501
            Self::Tianxiang => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/shenhua.js:6577
            Self::Tianxiang2 => (Timing::Normal, 0, false),
            // noname-liuli/noname/character/shenhua.js:4163
            Self::YunchouBeige => (Timing::Normal, 0, true),
            // noname-liuli/noname/mode/boss.js:7147
            Self::YunchouJiaoxia => (Timing::Normal, 0, true),
            // noname-liuli/noname/character/yijiang.js:11678
            Self::Zhenlie => (Timing::Normal, 0, true),
            // noname-liuli/noname/extension/标记补充/extension.js:5199
            Self::FzXinKuanggu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1806 (bingfeng2)
            Self::Bingfeng2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4 (cyqiaoxie)
            Self::Cyqiaoxie => (Timing::Normal, 0, true),
            // extension/3D精选/character/skill.js:471 (dddmoyan)
            Self::D3Moyan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:741 (gaizao)
            Self::Gaizao => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2913 (hutian)
            Self::Hutian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1264 (hxunzhi2)
            Self::Hxunzhi2 => (Timing::Normal, 0, false),
            // extension/杀海拾遗/character/swd/skill.js:7031 (jifeng)
            Self::Jifeng => (Timing::Normal, 600, false),
            // extension/杀海拾遗/character/swd/skill.js:4855 (liaoyuan)
            Self::Liaoyuan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7880 (lxianglong)
            Self::Lxianglong => (Timing::Normal, 500, true),
            // extension/杀海拾遗/character/swd/skill.js:7555 (mailun)
            Self::Mailun => (Timing::Normal, 0, true),
            // extension/3D精选/character/skill.js:441 (dddjinggou)
            Self::MaxRangeWeaponDamage => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7118 (mohua2)
            Self::Mohua2 => (Timing::Normal, 1000, true),
            // extension/杀海拾遗/character/swd/skill.js:5455 (shengong)
            Self::Shengong => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:51 (siji)
            Self::SijiDiscardDraw => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4319 (anlianying)
            Self::SwdAnlianying => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7343 (busi)
            Self::SwdBusi => (Timing::Normal, 700, true),
            // extension/杀海拾遗/character/swd/skill.js:3021 (chengjian)
            Self::SwdChengjian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:60 (cyxianjiang)
            Self::SwdCyxianjiang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5277 (daixing)
            Self::SwdDaixing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5328 (daixing2)
            Self::SwdDaixing2 => (Timing::Normal, 1, false),
            // extension/杀海拾遗/character/swd/skill.js:5338 (daixing3)
            Self::SwdDaixing3 => (Timing::Normal, 1, false),
            // extension/杀海拾遗/character/gujian/skill.js:1277 (dangping)
            Self::SwdDangping => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5260 (daofa)
            Self::SwdDaofa => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:397 (datong)
            Self::SwdDatong => (Timing::Normal, 0, true),
            // extension/杀海拾遗/card/swd/skill.js:949 (donghuangzhong)
            Self::SwdDonghuangzhongCard => (Timing::Normal, -25, true),
            // extension/杀海拾遗/character/swd/skill.js:4981 (duanxing)
            Self::SwdDuanxing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4935 (dunxing)
            Self::SwdDunxing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2541 (fuyan)
            Self::SwdFuyan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2450 (fzhenwei)
            Self::SwdFzhenwei => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1432 (gtiandao)
            Self::SwdGTiandao => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3674 (gongshen)
            Self::SwdGongshen => (Timing::Normal, 1500, true),
            // extension/杀海拾遗/character/swd/skill.js:2599 (guaili)
            Self::SwdGuaili => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5158 (guanhu)
            Self::SwdGuanhu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7489 (guiyan2)
            Self::SwdGuiyan2 => (Timing::Normal, 600, true),
            // extension/杀海拾遗/character/swd/skill.js:1891 (guozao)
            Self::SwdGuozao => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7007 (guxing3)
            Self::SwdGuxing3 => (Timing::Normal, 0, false),
            // extension/杀海拾遗/character/swd/skill.js:1392 (hlingbo)
            Self::SwdHlingbo => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5568 (huajian)
            Self::SwdHuajian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2191 (huajin2)
            Self::SwdHuajin2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1664 (huajing)
            Self::SwdHuajing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5195 (huanhun)
            Self::SwdHuanhun => (Timing::Normal, 600, true),
            // extension/杀海拾遗/character/swd/skill.js:7811 (huanling)
            Self::SwdHuanling => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3038 (huanxing)
            Self::SwdHuanxing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3118 (huanxing2)
            Self::SwdHuanxing2 => (Timing::Normal, -1500, true),
            // extension/杀海拾遗/character/swd/skill.js:6700 (huanyin)
            Self::SwdHuanyin => (Timing::Normal, 590, true),
            // extension/杀海拾遗/character/swd/skill.js:7987 (hudun)
            Self::SwdHudun => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2341 (huiqi)
            Self::SwdHuiqi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:719 (hujing)
            Self::SwdHujing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1112 (hwendao)
            Self::SwdHwendao => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1328 (hyunshen)
            Self::SwdHyunshen => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1373 (hyunshen2)
            Self::SwdHyunshen2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5617 (jikong)
            Self::SwdJikong => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1542 (jinlin2)
            Self::SwdJinlin2 => (Timing::Normal, 1, false),
            // extension/杀海拾遗/character/swd/skill.js:5924 (jqimou)
            Self::SwdJqimou => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1689 (jufu)
            Self::SwdJufu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:671 (kbolan)
            Self::SwdKbolan => (Timing::Normal, 500, true),
            // extension/杀海拾遗/card/swd/skill.js:1171 (kongdongyin)
            Self::SwdKongdongyinGuard => (Timing::Normal, -25, true),
            // extension/杀海拾遗/character/swd/skill.js:6875 (lanzhi)
            Self::SwdLanzhi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2317 (lianda)
            Self::SwdLianda => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3826 (lianwu)
            Self::SwdLianwu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1148 (lingfeng)
            Self::SwdLingfeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5438 (lingwu)
            Self::SwdLingwu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2670 (lingxian)
            Self::SwdLingxian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5409 (lingxin)
            Self::SwdLingxin => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1559 (lingyue)
            Self::SwdLingyue => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3551 (swdliuhong)
            Self::SwdLiuhong => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:1302 (lmazui2)
            Self::SwdLmazui2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5360 (lqingcheng)
            Self::SwdLqingcheng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6253 (luomei)
            Self::SwdLuomei => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5004 (meihuo)
            Self::SwdMeihuo => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7022 (miles_xueyi)
            Self::SwdMilesXueyi => (Timing::Normal, 1000, true),
            // extension/杀海拾遗/character/swd/skill.js:3907 (minjing)
            Self::SwdMinjing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3535 (mojian)
            Self::SwdMojian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3874 (mufeng)
            Self::SwdMufengHeld => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6503 (ningshuang)
            Self::SwdNingshuang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3174 (ningxian)
            Self::SwdNingxian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/card/swd/skill.js:1186 (nvwashi)
            Self::SwdNvwashiGuard => (Timing::Normal, 575, true),
            // extension/杀海拾遗/character/swd/skill.js:2122 (pojian)
            Self::SwdPojian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5600 (polang)
            Self::SwdPolang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5185 (poxing)
            Self::SwdPoxing => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3561 (poyue)
            Self::SwdPoyue => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5704 (pozhen)
            Self::SwdPozhen => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3400 (pozhou)
            Self::SwdPozhou => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4411 (rexue)
            Self::SwdRexue => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2276 (rumeng)
            Self::SwdRumeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2503 (shangxi)
            Self::SwdShangxi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:877 (shenyan)
            Self::SwdShenyan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7955 (shoulie)
            Self::SwdShoulie => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3145 (suiyan)
            Self::SwdSuiyan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4474 (susheng)
            Self::SwdSusheng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:846 (tiebi)
            Self::SwdTiebi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3513 (tuzhen)
            Self::SwdTuzhen => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4192 (wangchen)
            Self::SwdWangchenHeld => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6618 (xiaomoyu)
            Self::SwdXiaomoyu => (Timing::Normal, 100, true),
            // extension/杀海拾遗/character/swd/skill.js:5659 (xielei)
            Self::SwdXielei => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6562 (xielv)
            Self::SwdXielv => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7366 (xuying)
            Self::SwdXuying => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7386 (yinguo)
            Self::SwdYinguo => (Timing::Normal, 600, true),
            // extension/杀海拾遗/character/swd/skill.js:5239 (yinyue)
            Self::SwdYinyue => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:5058 (yishan)
            Self::SwdYishan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:4293 (youyin)
            Self::SwdYouyin => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:2207 (yuchen)
            Self::SwdYuchen => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7315 (yueren)
            Self::SwdYueren => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6326 (yulin)
            Self::SwdYulin => (Timing::Normal, -1000, true),
            // extension/杀海拾遗/character/swd/skill.js:7941 (zhenjiu2)
            Self::SwdZhenjiu2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6453 (zhuyu)
            Self::SwdZhuyu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:6814 (tianlun)
            Self::Tianlun => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:74 (ciqiu)
            Self::WudianCiqiu => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:109 (duoqi)
            Self::WudianDuoqi => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:422 (youzhan)
            Self::WudianYouzhan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:2779 (feidan)
            Self::WudianFeidan => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:507 (fuchou)
            Self::WudianFuchou => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:547 (fuchou2)
            Self::WudianFuchou2 => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:301 (geju)
            Self::WudianGeju => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:3065 (guangshu_diamond)
            Self::WudianGuangshuDiamond => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:3077 (guangshu_heart)
            Self::WudianGuangshuHeart => (Timing::Normal, 600, true),
            // extension/杀海拾遗/character/ow/skill.js:3109 (guangshu_spade)
            Self::WudianGuangshuSpade => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:725 (jianbi)
            Self::WudianJianbi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:1856 (jidong)
            Self::WudianJidong => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:615 (liduan)
            Self::WudianLiduan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:763 (mengji)
            Self::WudianMengji => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:511 (owliyong)
            Self::WudianOwliyong => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:423 (pingzhang)
            Self::WudianPingzhang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:1218 (shenqiang)
            Self::WudianShenqiang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:1087 (shihuo)
            Self::WudianShihuo => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:2 (nsshuaiyan)
            Self::WudianShuaiyan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:578 (xiyang)
            Self::WudianXiyang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:702 (yinshen)
            Self::WudianYinshen => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:2838 (yuedong)
            Self::WudianYuedong => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:1436 (zhiyuan)
            Self::WudianZhiyuan => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:159 (zhucheng)
            Self::WudianZhucheng => (Timing::Normal, 0, true),
            // extension/玩点论杀/character/skill.js:217 (zhucheng2)
            Self::WudianZhuchengGuard => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/ow/skill.js:3121 (ziyu)
            Self::WudianZiyu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:463 (anwugu)
            Self::XianjianAnwugu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1518 (binxin)
            Self::XianjianBinxin => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3162 (changnian)
            Self::XianjianChangnian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3199 (changnian2)
            Self::XianjianChangnian2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2918 (ctianfu2)
            Self::XianjianCtianfu2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2950 (ctianfu3)
            Self::XianjianCtianfu3 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3287 (duci)
            Self::XianjianDuci => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1408 (fanling)
            Self::XianjianFanling => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2721 (feixu)
            Self::XianjianFeixu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1984 (feng)
            Self::XianjianFeng => (Timing::Normal, 500, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1248 (fenglue)
            Self::XianjianFenglue => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1754 (fenshi)
            Self::XianjianFenshi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:4001 (huimeng)
            Self::XianjianHuimeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1619 (husha)
            Self::XianjianHusha => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3243 (jtjubao)
            Self::XianjianJtjubao => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:171 (liefeng)
            Self::XianjianLiefeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:832 (liguang)
            Self::XianjianLiguang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:573 (linghuo)
            Self::XianjianLinghuo => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:4 (lingquan)
            Self::XianjianLingquan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2501 (lingxue)
            Self::XianjianLingxue => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2650 (lingyu)
            Self::XianjianLingyu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1768 (longhuo)
            Self::XianjianLonghuo => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2867 (luanjian2)
            Self::XianjianLuanjian2 => (Timing::Normal, 0, false),
            // extension/杀海拾遗/character/xianjian/skill.js:2764 (ningbin)
            Self::XianjianNingbin => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3556 (poyun)
            Self::XianjianPoyun => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3471 (qianfang)
            Self::XianjianQianfang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3813 (runxin)
            Self::XianjianRunxin => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2195 (sheling)
            Self::XianjianSheling => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3435 (shenmu)
            Self::XianjianShenmu => (Timing::Normal, 600, true),
            // extension/杀海拾遗/character/xianjian/skill.js:25 (shenwu)
            Self::XianjianShenwu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1446 (sheying)
            Self::XianjianSheying => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:784 (shiying)
            Self::XianjianShiying => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2961 (shuiyun)
            Self::XianjianShuiyun => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2037 (song)
            Self::XianjianSong => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:4011 (tianshe)
            Self::XianjianTianshe => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:4042 (tianshe2)
            Self::XianjianTianshe2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:92 (txianqu)
            Self::XianjianTxianqu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3089 (wangyou)
            Self::XianjianWangyou => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3151 (wangyou2)
            Self::XianjianWangyou2 => (Timing::Normal, 1, false),
            // extension/杀海拾遗/character/xianjian/skill.js:872 (xiepan)
            Self::XianjianXiepan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1050 (xjyingfeng)
            Self::XianjianXjyingfeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:1858 (xjyufeng)
            Self::XianjianXjyufeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3855 (xjzhimeng)
            Self::XianjianXjzhimeng => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3896 (xjzhimeng2)
            Self::XianjianXjzhimeng2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3328 (xshuangren)
            Self::XianjianXshuangren => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:3384 (xshuangren2)
            Self::XianjianXshuangren2 => (Timing::Normal, 1, false),
            // extension/杀海拾遗/character/xianjian/skill.js:2743 (xuanyan2)
            Self::XianjianXuanyan2 => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2753 (xuanyan3)
            Self::XianjianXuanyan3 => (Timing::Normal, 0, false),
            // extension/杀海拾遗/character/xianjian/skill.js:152 (xunying)
            Self::XianjianXunying => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2008 (ya)
            Self::XianjianYa => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2297 (zhangmu)
            Self::XianjianZhangmu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/xianjian/skill.js:2230 (zhaoyao)
            Self::XianjianZhaoyao => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:7071 (swd_xiuluo)
            Self::Xiuluo => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/swd/skill.js:3253 (xuanyuan)
            Self::Xuanyuan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/card/swd/skill.js:1032 (xuanyuanjian3)
            Self::XuanyuanjianBacklash => (Timing::Normal, -25, false),
            // extension/杀海拾遗/card/swd/skill.js:1017 (xuanyuanjian2)
            Self::XuanyuanjianDamage => (Timing::Normal, -25, true),
            // extension/杀海拾遗/character/swd/skill.js:4252 (yihua)
            Self::Yihua => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:401 (diychanyuan)
            Self::YunchouDiychanyuan => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:370 (diyguhuo)
            Self::YunchouDiyguhuo => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:107 (diykuanggu)
            Self::YunchouDiykuanggu => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:347 (honglian)
            Self::YunchouHonglian => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:296 (luweiyan2)
            Self::YunchouLuweiyan2 => (Timing::Normal, 0, false),
            // extension/杀海拾遗/character/yunchou/skill.js:457 (shaoying)
            Self::YunchouShaoying => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:4 (xicai)
            Self::YunchouXicai => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:142 (xiongzi)
            Self::YunchouXiongzi => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:149 (yaliang)
            Self::YunchouYaliang => (Timing::Normal, 0, true),
            // extension/杀海拾遗/character/yunchou/skill.js:428 (zonghuo)
            Self::YunchouZonghuo => (Timing::Normal, 1000, true),
            // extension/英雄杀/character/skill.js:4 (yxs_fanji)
            Self::YxsFanji => (Timing::Normal, 1200, true),
            // extension/英雄杀/character/skill.js:22 (yxs_fanji2)
            Self::YxsFanji2 => (Timing::Normal, 0, true),
            // extension/英雄杀/character/skill.js:52 (yxs_menshen)
            Self::YxsMenshen => (Timing::Normal, 1500, true),
            // extension/英雄杀/character/skill.js:86 (yxs_menshen2)
            Self::YxsMenshen2 => (Timing::Normal, 1500, true),
            // extension/英雄杀/character/skill.js:34 (yxs_menshen3)
            Self::YxsMenshen3 => (Timing::Normal, 1, false),
            // extension/欢乐卡牌/card/skill.js:129 (jiuwei)
            Self::PhaseEndRecoverOrDraw => (Timing::Normal, -25, true),
            // extension/欢乐卡牌/card/skill.js:53 (kuwu)
            Self::ShaDamageForceDiscard => (Timing::Normal, -25, true),
            // extension/欢乐卡牌/card/skill.js:32 (mianju)
            Self::HuanleMianjuCard => (Timing::Normal, -25, true),
            // extension/欢乐卡牌/card/skill.js:5 (monkey)
            Self::HuanleMonkeyCard => (Timing::Normal, -25, true),
            // extension/欢乐卡牌/card/skill.js:70 (xuelunyang)
            Self::HuanleXuelunyangCard => (Timing::Normal, -25, true),
            // noname-liuli/noname/card/gujian.js:1088; Liuli game.js:32348-32365 (silent).
            Self::GujianFoodZiyangdan => (Timing::Normal, 1, false),
            _ => return None,
        })
    }
}
