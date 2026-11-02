use super::names::*;
use crate::catalog::Faction;

/// 完整武将授予（N08 内容扩面）：武将名 → 技能集。
/// 授予武将即同时授予其全部技能。「张角」为官方技能迁移；
/// 「样例」前缀条目为 ADR 0712 机制样例组合（不计官方技能迁移）。
/// 武将性别（离间/结姻等性别门控用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

pub const STANDARD_GENERALS: &[(&str, Faction, &[&str], u32, Gender)] = &[
    // 四元组：名称 / 势力 / 技能集 / 体力上限（N11 将领装配，ADR 0769）
    (
        "样例·张角组合",
        Faction::Qun,
        &[GUI_DAO, TIAN_DU],
        3,
        Gender::Male,
    ),
    ("司马懿", Faction::Wei, &[GUI_CAI, FAN_KUI], 3, Gender::Male),
    ("曹操", Faction::Wei, &[JIAN_XIONG, HU_JIA], 4, Gender::Male),
    ("貂蝉", Faction::Qun, &[BI_YUE, LI_JIAN], 3, Gender::Female),
    (
        "甄姬",
        Faction::Wei,
        &[LUO_SHEN, QING_GUO],
        3,
        Gender::Female,
    ),
    ("关羽", Faction::Shu, &[WU_SHENG], 4, Gender::Male),
    ("赵云", Faction::Shu, &[LONG_DAN], 4, Gender::Male),
    ("吕布", Faction::Qun, &[WU_SHUANG], 4, Gender::Male),
    ("黄月英", Faction::Shu, &[JI_ZHI, QI_CAI], 3, Gender::Female),
    ("大乔", Faction::Wu, &[GUO_SE, LIU_LI], 3, Gender::Female),
    ("吕蒙", Faction::Wu, &[KE_JI], 4, Gender::Male),
    ("甘宁", Faction::Wu, &[QI_XI], 4, Gender::Male),
    ("张飞", Faction::Shu, &[PAO_XIAO], 4, Gender::Male),
    ("周瑜", Faction::Wu, &[YING_ZI, FAN_JIAN], 3, Gender::Male),
    (
        "孙尚香",
        Faction::Wu,
        &[XIAO_JI, JIE_YIN],
        3,
        Gender::Female,
    ),
    ("华佗", Faction::Qun, &[QING_NANG, JI_JIU], 3, Gender::Male),
    ("黄盖", Faction::Wu, &[KU_ROU], 4, Gender::Male),
    ("孙权", Faction::Wu, &[ZHI_HENG, JIU_YUAN], 4, Gender::Male),
    ("郭嘉", Faction::Wei, &[TIAN_DU, JI_JI], 3, Gender::Male),
    ("夏侯惇", Faction::Wei, &[GANG_LIE], 4, Gender::Male),
    (
        "诸葛亮",
        Faction::Shu,
        &[GUAN_XING, KONG_CHENG],
        3,
        Gender::Male,
    ),
    ("陆逊", Faction::Wu, &[QIAN_XUN, LIAN_YING], 3, Gender::Male),
    ("刘备", Faction::Shu, &[REN_DE, JI_JIANG], 4, Gender::Male),
    ("马超", Faction::Shu, &[MA_SHU, TIE_JI], 4, Gender::Male),
    ("张辽", Faction::Wei, &[TU_XI], 4, Gender::Male),
    ("许褚", Faction::Wei, &[LUO_YI], 4, Gender::Male),
    ("曹仁", Faction::Wei, &[JU_SHOU], 4, Gender::Male),
    (
        "样例·策援",
        Faction::Qun,
        &[YI_SHANG, SHOU_YI],
        4,
        Gender::Male,
    ),
    (
        "样例·守财",
        Faction::Qun,
        &[SUI_DUN, SAN_CAI, SU_QING],
        4,
        Gender::Female,
    ),
    // P2/N08 标准缺失将补齐（2026-10-07，上游 standard/character.js 对齐）：
    ("乐进", Faction::Wei, &[XIAO_GUO], 4, Gender::Male),
    ("潘凤", Faction::Qun, &[KUANG_FU], 4, Gender::Male),
    ("华雄", Faction::Qun, &[YAO_WU], 6, Gender::Male),
    (
        "甘夫人",
        Faction::Shu,
        &[SHU_SHEN, SHEN_ZHI],
        3,
        Gender::Female,
    ),
    ("李典", Faction::Wei, &[XUN_XUN, WANG_XI], 3, Gender::Male),
    ("公孙瓒", Faction::Qun, &[YI_CONG], 4, Gender::Male),
    ("伊籍", Faction::Shu, &[JI_JIE, JI_YUAN], 3, Gender::Male),
    ("袁术", Faction::Qun, &[WANG_ZUN, TONG_JI], 4, Gender::Male),
];

/// Boss 模式内置武将（上游 mode/boss.js characterPack.mode_boss 首批：
/// 规则 v208 烛阴；追加于 STANDARD 索引空间尾——既有将索引不受扰动）。
/// Actual Liuli mode/boss.js boss_mushengoumang declaration in the append-only boss roster.
pub const BOSS_MUSHENGOUMANG: (usize, (&str, Faction, &[&str], u32, Gender)) = (
    16,
    (
        "勾芒",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_BUCHUN, BOSS_QINGMU3, BOSS_QINGMU3X],
        5,
        Gender::Male,
    ),
);

/// Immutable Liuli boss_jinshenrushou declaration.
pub const BOSS_JINSHENRUSHOU: (usize, (&str, Faction, &[&str], u32, Gender)) = (
    15,
    (
        "蓐收",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_XINGQIU, BOSS_BAIMANG3, BOSS_BAIMANG3X],
        5,
        Gender::Male,
    ),
);
pub const BOSS_GENERALS: &[(&str, Faction, &[&str], u32, Gender)] = &[
    ("烛阴", Faction::Qun, &[BOSS_XIONGSHOU], 4, Gender::Male),
    // 混沌（上游 boss_hundun hp25）：隐贼以「体力≤一半条件内生效」近似
    // 上游 switch 激活门控（首次激活永久语义登记差异）
    (
        "混沌",
        Faction::Qun,
        &[BOSS_XIONGSHOU, BOSS_WUZANG, BOSS_XIANGDE, BOSS_YINZEI],
        25,
        Gender::Male,
    ),
    // 穷奇（上游 boss_qiongqi hp20/maxHp25）：逐厄单技本批；覆泰(+全场
    // 禁桃)/延妒(+switch) 未落随下批（maxHp 差异登记：max_hp 暂=hp=20）
    // 延妒已落（v213，激活门控以 SkillInstance.enabled 同隐贼先例）；
    // 覆泰2 全场禁桃未落随下批
    (
        "穷奇",
        Faction::Qun,
        &[BOSS_XIONGSHOU, BOSS_ZHUE, BOSS_FUTAI, BOSS_YANDU],
        20,
        Gender::Male,
    ),
    // 饕餮四技全齐（v215 贪欲；v216 藏目+集财——集财激活以
    // SkillInstance.enabled 同隐贼/延妒先例）
    (
        "饕餮",
        Faction::Qun,
        &[BOSS_XIONGSHOU, BOSS_TANYU, BOSS_CANGMU, BOSS_JICAI],
        20,
        Gender::Male,
    ),
    // 梼杌四技全齐（v217 拟态；v218 乱常；v219 冥顽——目标锁定集以
    // TurnState.minwan_targets 回合级承载）
    (
        "梼杌",
        Faction::Qun,
        &[BOSS_XIONGSHOU, BOSS_NITAI, BOSS_LUANCHANG, BOSS_MINWAN],
        25,
        Gender::Male,
    ),
    // 毕方四技全齐（v227 招祸；v228 红莲；v229 炎狱——激活门控以
    // SkillInstance.enabled 同隐贼先例；每回合至多 3 以 TurnState 计数器）
    (
        "毕方",
        Faction::Qun,
        &[BOSS_YAOSHOU, BOSS_ZHAOHUO, BOSS_HONGLIANX, BOSS_YANYU],
        25,
        Gender::Male,
    ),
    // 书精（上游 boss_shujing hp2 女神）：催毒+中毒全技（中毒为运行时
    // 授技面，表列仅催毒——授技入实例同 catalog skill 空间）
    ("书精", Faction::Shu, &[BOSS_CUIDUI], 2, Gender::Female),
    // 相柳四技全齐（v221 妖兽；v222 兽九；v223 毒妒+邪酬——蛇毒标记
    // 以 PlayerState.shedu 承载）
    (
        "相柳",
        Faction::Qun,
        &[BOSS_YAOSHOU, BOSS_JIUSHOU, BOSS_DUQU, BOSS_ECHOU],
        25,
        Gender::Male,
    ),
    // 应昭四技全齐（v228 封冻；v231 巡游；v232 司圃使用面——响应四面
    // 登记差异；激活门控同隐贼先例）
    (
        "应昭",
        Faction::Qun,
        &[BOSS_YAOSHOU, BOSS_FENGDONG, BOSS_XUNYOU, BOSS_SIPU],
        25,
        Gender::Male,
    ),
    // 朱厌四技全齐（v224 兵先；v225 虚失；v226 巨源——体力上限+5 由
    // maxHp30 表意[登记同穷奇]；记录=PlayerState.juyuan_storage）
    (
        "朱厌",
        Faction::Qun,
        &[BOSS_YAOSHOU, BOSS_BINGXIAN, BOSS_XUSHI, BOSS_JUYUAN],
        25,
        Gender::Male,
    ),
    // 青龙（v233 神裔、v234 雷击；v292 青木/青木·转阶隐藏链）
    (
        "青龙",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_RELEIJI, BOSS_QINGMU2, BOSS_QINGMU2X],
        4,
        Gender::Male,
    ),
    // 白虎（v235 狂笑；神裔共用 v233；v293 白芒/白芒·转阶隐藏链；
    // 上游势力 shen→群目录近似[ Faction 枚举无神势力，烛阴空势力→群先例]）
    (
        "白虎",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_KUANGXIAO, BOSS_BAIMANG2, BOSS_BAIMANG2X],
        4,
        Gender::Male,
    ),
    // 朱雀（v236 焚天；神裔共用 v233；v293 赤炎/赤炎·转阶隐藏链；
    // 势力 shen→群目录近似；上游 female）
    (
        "朱雀",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_FENTIAN, BOSS_CHIYAN2, BOSS_CHIYAN2X],
        4,
        Gender::Female,
    ),
    // 玄武（v237 灵渠；神裔共用 v233；v293 玄麟/玄麟·转阶隐藏链；
    // 势力 shen→群目录近似）
    (
        "玄武",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_LINGQU, BOSS_XUANLIN2, BOSS_XUANLIN2X],
        4,
        Gender::Male,
    ),
    // 太昊（v238 神恩光环；神裔共用 v233；boss_qingyi 青翼未落随下批；
    // 上游 names 风|伏羲、shen→群目录近似）
    (
        "太昊",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_SHENEN, BOSS_QINGYI],
        6,
        Gender::Male,
    ),
    // 蓐收（v239 星囚；神裔共用 v233；v294 白芒3/白芒3·转阶隐藏链；
    // 上游 names 嬴|该、shen→群目录近似）
    BOSS_JINSHENRUSHOU.1,
    // 勾芒（v240 不臣；神裔共用 v233；v292 青木3/青木3·转阶隐藏链；
    // 上游 names 姬|重、shen→群目录近似）
    BOSS_MUSHENGOUMANG.1,
    // 焰灵（v241 回火+复燃；上游技能表无神裔——陪将不在神裔名单；
    // shen→群目录近似）
    (
        "焰灵",
        Faction::Qun,
        &[BOSS_HUIHUO, BOSS_FURAN],
        4,
        Gender::Male,
    ),
    // 祝融（v242 星夏；神裔共用 v233；v294 赤炎3/赤炎3·转阶隐藏链；
    // 上游 names 姜|重黎、shen→群目录近似）
    (
        "祝融",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_XINGXIA, BOSS_CHIYAN3, BOSS_CHIYAN3X],
        5,
        Gender::Male,
    ),
    // 炎帝（v243 纯入库：神裔共用 v233+神恩光环共用 v238；boss_chiyi 赤
    // 翼=青翼同族已处置；上游 names 姜|轨、shen→群目录近似）
    (
        "炎帝",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_SHENEN, BOSS_CHIYI],
        6,
        Gender::Male,
    ),
    // 少昊（v243 纯入库：神裔共用 v233+神恩光环共用 v238；boss_baiyi 白
    // 翼=青翼同族已处置；上游 names 嬴|挚、shen→群目录近似）
    (
        "少昊",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_SHENEN, BOSS_BAIYI, BOSS_BAIYI_THUNDER],
        6,
        Gender::Male,
    ),
    // 共工（v244 绝红；神裔共用 v233；v294 玄麟3/玄麟3·转阶隐藏链；
    // 上游 names 姜|共工、shen→群目录近似）
    (
        "共工",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_JUEHONG, BOSS_XUANLIN3, BOSS_XUANLIN3X],
        5,
        Gender::Male,
    ),
    // 明星（v245 庆祝+夹击随条目；急叮=星囚授予技不随条目装配[上游
    // boss_jiding 由蓐收 boss_xingquiaaddSkills 授予——庆祝禁杀窗口依赖
    // 此初始缺席]；上游 female、shen→群目录近似）
    (
        "明星",
        Faction::Qun,
        &[BOSS_QINGZHU, BOSS_JIAZU],
        3,
        Gender::Female,
    ),
    // 颛顼（v246 造异；神裔共用 v233+神恩光环共用 v238；造异异变=运行
    // 时授予技不随条目；上游 names 姬|颛顼、shen→群目录近似）
    (
        "颛顼",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_SHENEN, BOSS_ZAOYI],
        6,
        Gender::Male,
    ),
    // 玄冥（v247 自润；神裔共用 v233；上游亦挂 boss_xuanlin3——原生仅共工
    // 条目承载 3/3x（青木3 同款只挂阶段 2 Boss）；female、names 嬴|禺强、
    // shen→群目录近似——入库激活造异玄冥半门）
    (
        "玄冥",
        Faction::Qun,
        &[BOSS_SHENYI, BOSS_ZIRUN],
        5,
        Gender::Female,
    ),
    // 年兽合体（v248 年瑞+猛态；v249 形态轮换随条目[轮换载体]；
    // shen→群目录近似）
    (
        "年兽",
        Faction::Qun,
        &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_NBIANSHEN],
        12,
        Gender::Male,
    ),
    // 忍性年兽（v249 忍性+形态轮换；年瑞/猛态共用 v248；shen→群目录近似）
    (
        "忍性年兽",
        Faction::Qun,
        &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_RENXING, BOSS_NBIANSHEN],
        12,
        Gender::Male,
    ),
    // 睿智年兽（v250 睿智+形态轮换；年瑞/猛态共用 v248；shen→群目录近似）
    (
        "睿智年兽",
        Faction::Qun,
        &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_RUIZHI, BOSS_NBIANSHEN],
        12,
        Gender::Male,
    ),
    // 守翼年兽（v251 守翼+形态轮换；年瑞/猛态共用 v248；上游形态表含
    // boss_nbaonu[暴怒,未落随批]——条目只列已转换技；shen→群目录近似）
    (
        "守翼年兽",
        Faction::Qun,
        &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_SHOUYI, BOSS_NBIANSHEN],
        12,
        Gender::Male,
    ),
    // 精觉年兽（v252 精觉[继承胆术]+形态轮换；年瑞/猛态共用 v248；shen→群目录近似）
    (
        "精觉年兽",
        Faction::Qun,
        &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_JINGJUE, BOSS_NBIANSHEN],
        12,
        Gender::Male,
    ),
    // 夜叉（v253 魔道+魔剑+胆术复用 v252；bazhen=标准装备八卦阵由 Armor
    // 判定机承载不入条目；shen→群目录近似）
    (
        "夜叉",
        Faction::Qun,
        &[BOSS_MODAO, BOSS_MOJIAN, BOSS_DANSHU],
        11,
        Gender::Male,
    ),
    // 罗刹（v254 魔道共用+御兽+魔炎；yizhong=一将包防御技未落登记；
    // 上游 female、shen→群目录近似）
    (
        "罗刹",
        Faction::Qun,
        &[BOSS_MODAO, BOSS_YUSHOU, BOSS_MOYANY],
        12,
        Gender::Female,
    ),
    // 白无常（v255 爆敛+强征+醉酒；juece 界绝策[refresh 包]混入未落登记；
    // shen→群目录近似）
    (
        "白无常",
        Faction::Qun,
        &[
            BOSS_BAOLIAN,
            BOSS_QIANGZHENG,
            BOSS_ZUIJIU,
            BOSS_BIANSHEN4,
            BOSS_BIANSHEN4X,
        ],
        8,
        Gender::Male,
    ),
    // 黑无常（v256 鬼计+太平+索命+吸行；bianshen4 死亡换将=已处置关卡
    // 面；names 范|无救、shen→群目录近似）
    (
        "黑无常",
        Faction::Qun,
        &[
            BOSS_GUIJI,
            BOSS_TAIPING,
            BOSS_SUOMING,
            BOSS_XIXING,
            BOSS_BIANSHEN4,
            BOSS_BIANSHEN4X,
        ],
        9,
        Gender::Male,
    ),
    // 马面（v257 炼狱+鬼计共用 v256+反馈[标准 FAN_KUI 已承载]+绝策 v258；
    // shen→群目录近似）
    (
        "马面",
        Faction::Qun,
        &[
            BOSS_GUIJI,
            FAN_KUI,
            BOSS_LIANYU,
            BOSS_JUECE,
            BOSS_BIANSHEN3,
            BOSS_BIANSHEN3X,
        ],
        6,
        Gender::Male,
    ),
    // 董卓（v258 酒池+强征/爆敛混入已落[v255]；shen→群目录近似）
    (
        "董卓",
        Faction::Qun,
        &[BOSS_JIUCHI, BOSS_QIANGZHENG, BOSS_BAOLIAN],
        20,
        Gender::Male,
    ),
    // 牛头（v255 爆敛共用+枭首；蛮甲[extra-equip 藤甲挂点分散]+niepan
    // [boss 本地死亡复活面]混入未落登记；shen→群目录近似）
    (
        "牛头",
        Faction::Qun,
        &[BOSS_BAOLIAN, BOSS_XIAOSHOU, BOSS_BIANSHEN3, BOSS_BIANSHEN3X],
        7,
        Gender::Male,
    ),
    // 郭嘉（v259 鬼谋+远策+奇佐；天妒=标准技共用；上游 group shen、
    // names 郭|嘉、shen→群目录近似）
    (
        "郭嘉",
        Faction::Qun,
        &[TIAN_DU, BOSS_GUIMOU, BOSS_YUANCE, BOSS_QIZUO],
        4,
        Gender::Male,
    ),
    // 张春华（v260 绝情+无心+伤逝；上游 group shen→群目录近似）
    (
        "张春华",
        Faction::Qun,
        &[JUE_QING, BOSS_WUXIN, BOSS_SHANGSHIX],
        4,
        Gender::Female,
    ),
    // 刘备（v261 枭雄+章武；上游 group shen→群目录近似）
    (
        "刘备",
        Faction::Qun,
        &[BOSS_XIAOXIONG, BOSS_ZHANGWU],
        8,
        Gender::Male,
    ),
    // 甄姬（v262 踏水/凌波/娇孝/芳华；上游 group shen→群目录近似）
    (
        "甄姬",
        Faction::Qun,
        &[BOSS_TASHUI, BOSS_LINGBO, BOSS_JIAOXIA, BOSS_FANGHUA],
        4,
        Gender::Female,
    ),
    // 周瑜（v263 火神+红莲·周瑜+仙音；上游 group shen→群目录近似）
    (
        "周瑜",
        Faction::Qun,
        &[BOSS_HUOSHEN, BOSS_ZYHONGLIAN, BOSS_XIANYIN],
        6,
        Gender::Male,
    ),
    // 张角（v263 雷击·张角+天道+极点；上游 group shen→群目录近似）
    (
        "张角",
        Faction::Qun,
        &[BOSS_ZJLEIJI, BOSS_TIANDAO, BOSS_JIDIAN],
        8,
        Gender::Male,
    ),
    // 五官王（v265 枳枉+公正+血池；十殿阎罗第二殿；上游 group qun→群
    // 目录直取）
    (
        "五官王",
        Faction::Qun,
        &[BOSS_ZHIWANG, BOSS_GONGZHENG, BOSS_XUECHI],
        4,
        Gender::Male,
    ),
    // 宋帝王（v266 黑绳+胜负；恩怨=外部包组技未落登记随外部包批；上游
    // group qun→群目录直取）
    (
        "宋帝王",
        Faction::Qun,
        &[BOSS_HEISHENG, BOSS_SHENGFU],
        4,
        Gender::Male,
    ),
    // 卞城王（v267 雷舟+雷斧+雷诛；上游 group qun→群目录直取）
    (
        "卞城王",
        Faction::Qun,
        &[BOSS_LEIZHOU, BOSS_LEIFU, BOSS_LEIZHU],
        4,
        Gender::Male,
    ),
    // 泰山王（v268 服毒+苦酒+热恼；上游 group qun→群目录直取）
    (
        "泰山王",
        Faction::Qun,
        &[BOSS_FUDU, BOSS_KUJIU, BOSS_RENAO],
        4,
        Gender::Male,
    ),
    // 转轮王（v269 魔道[共用]+轮回+往生+反噬；上游 group qun→群目录直取）
    (
        "转轮王",
        Faction::Qun,
        &[BOSS_MODAO, BOSS_LUNHUI, BOSS_WANGSHENG, BOSS_ZLFANSHI],
        6,
        Gender::Male,
    ),
    // 孟婆（v270 拾忧+忘魂+往事；上游 group qun→群目录直取；female）
    (
        "孟婆",
        Faction::Qun,
        &[BOSS_SHIYOU, BOSS_WANGHUN, BOSS_WANGSHI],
        3,
        Gender::Female,
    ),
    // 阎罗王（v271 铁面+铡刀+诛心；上游 group qun→群目录直取）
    (
        "阎罗王",
        Faction::Qun,
        &[BOSS_TIEMIAN, BOSS_ZHADAO, BOSS_ZHUXIN],
        4,
        Gender::Male,
    ),
    // 平等王（v272 锁足+阿鼻+平等；上游 group qun→群目录直取）
    (
        "平等王",
        Faction::Qun,
        &[BOSS_SUOZU, BOSS_ABI, BOSS_PINGDENG],
        4,
        Gender::Male,
    ),
    // 都市王（v273 热闷+炙焚+火刑；上游 group qun→群目录直取）
    (
        "都市王",
        Faction::Qun,
        &[BOSS_REMEN, BOSS_ZHIFEN, BOSS_HUOXING],
        4,
        Gender::Male,
    ),
    // 地藏王（v274 不佛+无量+谛听；大愿=判定改写机 AI 态度面登记不落；
    // 上游 group qun→群目录直取）
    (
        "地藏王",
        Faction::Qun,
        &[BOSS_BUFO, BOSS_WULIANG, BOSS_DITING],
        8,
        Gender::Male,
    ),
    // 楚江王（v275 帷幕+反馈[标准 FAN_KUI 复用——原生反馈语义=任意伤害
    // +手牌/装备，与界反馈一致]+冰封；上游 group qun→群目录直取）
    (
        "楚江王",
        Faction::Qun,
        &[BOSS_WEIMU, FAN_KUI, BOSS_BINGFENG],
        4,
        Gender::Male,
    ),
    // 秦广王（v276 判官+拘魂+望乡；还人=关卡流程面登记不落；上游 group
    // qun→群目录直取）
    (
        "秦广王",
        Faction::Qun,
        &[
            BOSS_PANGUAN,
            BOSS_JUHUN,
            BOSS_WANGXIANG,
            BOSS_HUANREN,
            BOSS_HUANREN_DIE,
        ],
        3,
        Gender::Male,
    ),
    // 庞统·涅槃凤雏（v277 天狱+栖梧；v306 批 118 补涅槃+浴火——浴火授予
    // 看破/神威/朱羽，神威已落、看破/朱羽外部包依赖登记；上游 group
    // shen→群近似登记）
    (
        "庞统",
        Faction::Qun,
        &[BOSS_TIANYU, BOSS_QIWU, BOSS_NIEPAN, BOSS_YUHOU],
        4,
        Gender::Male,
    ),
];

/// 武将目录下标 → 性别（越界返回 None）。
pub fn general_gender(g: u16) -> Option<Gender> {
    STANDARD_GENERALS
        .get(g as usize)
        .map(|(_, _, _, _, gen)| *gen)
}

/// 武将目录下标 → 体力上限（越界返回 None）。
pub fn general_hp(g: u16) -> Option<u32> {
    STANDARD_GENERALS.get(g as usize).map(|(_, _, _, h, _)| *h)
}

/// 武将目录下标 → 技能集（越界返回空）。
pub fn general_skills(g: u16) -> &'static [&'static str] {
    STANDARD_GENERALS
        .get(g as usize)
        .map(|(_, _, s, _, _)| *s)
        .unwrap_or(&[])
}

/// 武将目录下标 → 势力（越界返回 None）。
/// 国战明置门控：势力判定只认「已明置」的武将槽（ADR 0756/0757）。
pub fn faction_of_general(g: u16) -> Option<crate::catalog::Faction> {
    STANDARD_GENERALS.get(g as usize).map(|(_, f, _, _, _)| *f)
}

/// 扩展提供的额外武将条目。
#[derive(Debug, Clone)]
pub struct ExtensionGeneral {
    pub name: &'static str,
    pub faction: Faction,
    pub skills: &'static [&'static str],
    pub max_hp: u32,
    pub gender: Gender,
}
