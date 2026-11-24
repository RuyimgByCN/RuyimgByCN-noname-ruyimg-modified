import { CARD_KINDS } from "../presentationMetadata";

export type TabletopCardCategory = "basic" | "trick" | "equip";

export interface CardKindCatalogEntry {
  readonly label: string;
  readonly category: TabletopCardCategory;
  readonly categoryTag: string;
}

type TupleEntry = readonly [label: string, category: TabletopCardCategory, categoryTag?: string];

const DEFAULT_TAG: Readonly<Record<TabletopCardCategory, string>> = {
  basic: "基本",
  trick: "锦囊",
  equip: "装备",
};

/**
 * Exact `CardKind` discriminant order (0..=174) matching
 * `crates/engine-native-kernel/src/catalog.rs:38-552` (`session.rs:322`: `def.kind as u16`).
 */
const CARD_KIND_BY_DEF: ReadonlyArray<TupleEntry> = [
  ["杀", "basic"], // 0: Sha
  ["闪", "basic"], // 1: Shan
  ["桃", "basic"], // 2: Tao
  ["+1马", "equip"], // 3: HorsePlus
  ["-1马", "equip"], // 4: HorseMinus
  ["武器", "equip"], // 5: Weapon
  ["防具", "equip"], // 6: Armor
  ["火杀", "basic"], // 7: FireSha
  ["铁索连环", "trick"], // 8: Chain
  ["南蛮入侵", "trick"], // 9: SavageAttack
  ["万箭齐发", "trick"], // 10: ArcheryAttack
  ["决斗", "trick"], // 11: Duel
  ["无懈可击", "trick"], // 12: Nullify
  ["乐不思蜀", "trick"], // 13: Le
  ["兵粮寸断", "trick"], // 14: Bing
  ["闪电", "trick"], // 15: ShanDian
  ["过河拆桥", "trick"], // 16: Dismantle
  ["顺手牵羊", "trick"], // 17: Snatch
  ["酒", "basic"], // 18: Jiu
  ["雷杀", "basic"], // 19: LeiSha
  ["手里剑", "basic"], // 20: ShouLiJian
  ["五行盘", "equip"], // 21: WuxingPan
  ["望梅止渴", "trick"], // 22: WangmeiZhike
  ["三尖", "equip"], // 23: Sanjian
  ["水攻", "trick"], // 24: Shuigong
  ["釜底抽薪", "trick"], // 25: Fudichouxin
  ["趁火打劫", "trick"], // 26: Chenhuodajie
  ["索链甲", "equip"], // 27: Suolianjia
  ["洪水", "trick"], // 28: HongShui
  ["偷梁换柱", "trick"], // 29: Toulianghuanzhu
  ["灵魂之火", "trick"], // 30: Linghunzhihuo
  ["照明弹", "trick"], // 31: Zhaomingdan
  ["急耗卡", "trick"], // 32: Jihuocard
  ["诱敌深入", "trick"], // 33: Youdishenru
  ["隔岸观火", "trick"], // 34: Geanguanhuo
  ["草船借箭", "trick"], // 35: Caochuanjiejian
  ["舌战群儒", "trick"], // 36: Shezhanqunru
  ["舒筋散", "trick"], // 37: Shujinsan
  ["面里金针", "trick", "机关"], // 38: Mianlijinzhen
  ["飞镖", "trick", "机关"], // 39: Feibiao
  ["金莲烛", "trick"], // 40: Jinlianzhu
  ["摊黄冻", "trick", "食物"], // 41: Tanhuadong
  ["有更", "trick", "食物"], // 42: Yougeng
  ["青团", "trick", "食物"], // 43: Qingtuan
  ["元宝肉", "trick", "食物"], // 44: Yuanbaorou
  ["咕咚羹", "trick", "食物"], // 45: Gudonggeng
  ["麻婆豆腐", "trick", "食物"], // 46: Mapodoufu
  ["春饼", "trick", "食物"], // 47: Chunbing
  ["鲤鱼汤", "trick", "食物"], // 48: Liyutang
  ["麻茶", "trick", "食物"], // 49: Molicha
  ["菱角", "trick", "食物"], // 50: Xiajiao
  ["蜜链藕", "trick", "食物"], // 51: Mizhilianou
  ["卤玉羹", "trick", "食物"], // 52: Luyugeng
  ["冰魄通", "trick", "机关"], // 53: Bingpotong
  ["乾坤镖", "trick", "机关"], // 54: Qiankunbiao
  ["龙须钩", "trick", "机关"], // 55: Longxugou
  ["神火飞鸦", "trick", "机关"], // 56: Shenhuofeiya
  ["紫阳丹", "basic"], // 57: Ziyangdan
  ["玉女元神", "basic"], // 58: Yunvyuanshen
  ["石会粉", "trick"], // 59: Shihuifen
  ["六风散", "trick"], // 60: Liufengsan
  ["砂糖", "trick"], // 61: Shatang
  ["黑龙鳞片", "trick"], // 62: Heilonglinpian
  ["硫头阁", "trick", "机关"], // 63: Liutouge
  ["木头面具", "equip"], // 64: Mutoumianju
  ["帝女玄霜", "basic"], // 65: Dinvxuanshuang
  ["玉衡", "equip"], // 66: Gjyuheng
  ["盾牌格挡", "trick"], // 67: Dunpaigedang
  ["治疗波", "trick"], // 68: Zhiliaobo
  ["元素毁灭", "trick"], // 69: Yuansuhuimie
  ["星界沟通", "trick"], // 70: Xingjiegoutong
  ["传送门", "trick"], // 71: Chuansongmen
  ["坦蛇至人", "trick"], // 72: Tanshezhiren
  ["世界树", "trick"], // 73: Shijieshu
  ["燕子药水", "trick"], // 74: Yanziyaoshui
  ["奥祖之雷", "trick"], // 75: Aozuzhilei
  ["瘟疫", "trick"], // 76: Wenyi
  ["置换军", "trick"], // 77: Zhihuanjun
  ["暴雪药水", "trick"], // 78: Baoxueyaoshui
  ["倾盆大雨", "trick"], // 79: Qinpendayu
  ["碧日浓雾", "trick"], // 80: Birinongwu
  ["刺骨寒霜", "trick"], // 81: Ciguhanshuang
  ["昆恩法印", "trick"], // 82: Kunenfayin
  ["山崩术", "trick"], // 83: Shanbengshu
  ["破晓", "trick"], // 84: Poxiao
  ["祖墨手铠", "trick"], // 85: Zumoshoukao
  ["猴子", "equip"], // 86: Monkey
  ["写轮眼", "equip"], // 87: Xuelunyang
  ["漩涡面具", "equip"], // 88: Mianju
  ["酒尾", "equip"], // 89: Jiuwei
  ["无中生有", "trick"], // 90: Wuzhong
  ["桃园结义", "trick"], // 91: Taoyuan
  ["五谷丰登", "trick"], // 92: Wugu
  ["借刀杀人", "trick"], // 93: Jiedao
  ["仁王盾", "equip"], // 94: Renwang
  ["青釭剑", "equip"], // 95: Qinggang
  ["轩辕剑", "equip"], // 96: Xuanyuanjian
  ["麒麟弓", "equip"], // 97: Qilin
  ["寒冰剑", "equip"], // 98: Hanbing
  ["诸葛连弩", "equip"], // 99: Zhuge
  ["贯石斧", "equip"], // 100: Guanshi
  ["青龙偃月刀", "equip"], // 101: Qinglong
  ["丈八蛇矛", "equip"], // 102: Zhangba
  ["雌雄双股剑", "equip"], // 103: Cixiong
  ["方天画戟", "equip"], // 104: Fangtian
  ["护符", "basic"], // 105: Hufu
  ["流星火雨", "trick"], // 106: Liuxinghuoyu
  ["草药", "basic"], // 107: Caoyao
  ["赤龙牙", "equip"], // 108: Chilongya
  ["风雪刃", "equip"], // 109: Fengxueren
  ["天仙酒", "basic"], // 110: Tianxianjiu
  ["吸血鬼指环", "equip"], // 111: Xixueguizhihuan
  ["归有界", "trick"], // 112: Guiyoujie
  ["潜偷面具", "equip"], // 113: Shentoumianju
  ["星君眼", "equip"], // 114: Xingjunyan
  ["青龙令珠", "equip"], // 115: Qinglonglingzhu
  ["贴房绅视", "trick"], // 116: Zhufangshenshi
  ["御夫录", "equip"], // 117: Yufulu
  ["羊皮卷", "trick"], // 118: Yangpijuan
  ["带火风轮", "equip"], // 119: Daihuofenglun
  ["霞云彩绫", "equip"], // 120: Xiayuncailing
  ["蓝格怪衣", "equip"], // 121: Langeguaiyi
  ["龙颅山峰", "trick", "地"], // 122: Longlushanfeng
  ["海岛", "trick", "地"], // 123: Haidao
  ["水没墓穴", "trick", "地"], // 124: Shuimomuxue
  ["废墟", "trick", "地"], // 125: Feixu
  ["依夏兰", "trick", "地"], // 126: Yixialan
  ["沙漠绿洲", "trick", "地"], // 127: Shamolvzhou
  ["林中尖塔", "trick", "地"], // 128: Linzhongjianta
  ["冰河要塞", "trick", "地"], // 129: Bingheyaosai
  ["幽林", "trick", "地"], // 130: Youlin
  ["邻地溪流", "trick", "地"], // 131: Lindixiliu
  ["地载阵", "trick"], // 132: Dizaizhen
  ["机关牌", "trick", "机关"], // 133: Jiguan
  ["灵剑", "trick", "法器"], // 134: Hslingjian
  ["雪霁冰雹", "basic"], // 135: Xuejibingbao
  ["神秘果", "basic"], // 136: Shenmiguo
  ["封印之丹", "basic"], // 137: Fengyinzhidan
  ["炼妖壶", "equip"], // 138: Lianyaohu
  ["乾坤袋", "equip"], // 139: Qiankundai
  ["神农鼎", "equip"], // 140: Shennongding
  ["女娲石", "equip"], // 141: Nvwashi
  ["昆仑镜", "equip"], // 142: Kunlunjingc
  ["东皇钟", "equip"], // 143: Donghuangzhong
  ["空洞印", "equip"], // 144: Kongdongyin
  ["惊雷山", "trick"], // 145: Jingleishan
  ["炽羽袭", "trick"], // 146: Chiyuxi
  ["鬼蛇旗", "trick"], // 147: Guisheqi
  ["仙露回", "trick"], // 148: Xianluhui
  ["龟灵之桃", "equip"], // 149: Guilingzhitao
  ["四非珍面", "equip"], // 150: Sifeizhenmian
  ["书锤", "equip"], // 151: Shuchui
  ["散灯金环", "equip"], // 152: Sadengjinhuan
  ["昊天塔", "equip"], // 153: Haotianta
  ["玉如意", "equip"], // 154: Yuruyi
  ["玉蝉·乾", "trick", "宝器"], // 155: Yuchanqian
  ["玉蝉·坤", "trick", "宝器"], // 156: Yuchankun
  ["玉蝉·震", "trick", "宝器"], // 157: Yuchanzhen
  ["玉蝉·巽", "trick", "宝器"], // 158: Yuchanxun
  ["玉蝉·坎", "trick", "宝器"], // 159: Yuchankan
  ["玉蝉·离", "trick", "宝器"], // 160: Yuchanli
  ["玉蝉·艮", "trick", "宝器"], // 161: Yuchangen
  ["玉蝉·兑", "trick", "宝器"], // 162: Yuchandui
  ["神草结", "trick"], // 163: Shencaojie
  ["青龙之圭", "trick", "宝器"], // 164: Qinglongzhigui
  ["朱雀之璋", "trick", "宝器"], // 165: Zhuquezhizhang
  ["白兽之琥", "trick", "宝器"], // 166: Baishouzhihu
  ["玄武之璜", "trick", "宝器"], // 167: Xuanwuzhihuang
  ["黄麟之琮", "trick", "宝器"], // 168: Huanglinzhicong
  ["苍螭之璧", "trick", "宝器"], // 169: Cangchizhibi
  ["天覆阵", "trick"], // 170: Tianfuzhen
  ["七星阵", "trick"], // 171: Qixingzhen
  ["鸟翔阵", "trick"], // 172: Niaoxiangzhen
  ["长蛇阵", "trick"], // 173: Changshezhen
  ["破阵诀", "trick"], // 174: Pozhenjue
];

export interface CardKeyCatalogEntry extends CardKindCatalogEntry {
  readonly key: string;
}

export function lookupCardKindByDef(def: number | null | undefined): CardKeyCatalogEntry | null {
  if (def == null || !Number.isInteger(def) || def < 0 || def >= CARD_KIND_BY_DEF.length) {
    return null;
  }
  const [label, category, tag] = CARD_KIND_BY_DEF[def]!;
  const key = (CARD_KINDS[def] ?? label).toLowerCase();
  return { key, label, category, categoryTag: tag ?? DEFAULT_TAG[category] };
}

const CARD_ENTRY_BY_KEY: ReadonlyMap<string, CardKeyCatalogEntry> = new Map(
  CARD_KINDS.map((rawKey, idx) => {
    const key = rawKey.toLowerCase();
    const [label, category, tag] = CARD_KIND_BY_DEF[idx] ?? [rawKey, "basic", undefined];
    return [key, { key, label, category, categoryTag: tag ?? DEFAULT_TAG[category] }] as const;
  }),
);

const CARD_VIEW_KIND_ALIASES: Readonly<Record<string, string>> = {
  shafire: "huosha",
  shathunder: "leisha",
  horseoff: "horse_minus",
  horsedef: "horse_plus",
  trickle: "lebu",
};

/** Resolves lowercase presentation keys or PascalCase `CardView.kind` tags to catalog category. */
export function lookupCardKindByKey(rawKey: string | null | undefined): CardKeyCatalogEntry | null {
  if (typeof rawKey !== "string") return null;
  const trimmed = rawKey.trim();
  if (!trimmed) return null;
  const lower = trimmed.toLowerCase();
  const direct = CARD_ENTRY_BY_KEY.get(lower);
  if (direct) return direct;
  const alias = CARD_VIEW_KIND_ALIASES[lower];
  if (alias) return CARD_ENTRY_BY_KEY.get(alias) ?? null;
  if (trimmed.startsWith("Trick")) {
    const stripped = trimmed.slice(5).toLowerCase();
    return (
      CARD_ENTRY_BY_KEY.get(stripped) ?? {
        key: stripped,
        label: trimmed,
        category: "trick",
        categoryTag: DEFAULT_TAG.trick,
      }
    );
  }
  return null;
}
