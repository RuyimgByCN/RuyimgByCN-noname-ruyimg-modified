//! IR → bytecode + package indexes + source maps.

mod effect_ban_active_until_turn_end;
mod effect_convert_current_sha_to_fire_lose_hp_if_damage;
mod effect_gain_whole_skill_pile;
mod effect_resolve_wdb_yangyi;
mod effect_reveal_target_hand_sha_or_discard_bind;
mod effect_set_draw_phase_from_bottom;
mod effect_tag_event_cards;

use crate::{
    CompileError, ADDITIVE_RULE_VM_ABI_VERSION_10, ADDITIVE_RULE_VM_ABI_VERSION_100,
    ADDITIVE_RULE_VM_ABI_VERSION_101, ADDITIVE_RULE_VM_ABI_VERSION_102,
    ADDITIVE_RULE_VM_ABI_VERSION_103, ADDITIVE_RULE_VM_ABI_VERSION_104,
    ADDITIVE_RULE_VM_ABI_VERSION_108, ADDITIVE_RULE_VM_ABI_VERSION_109,
    ADDITIVE_RULE_VM_ABI_VERSION_11, ADDITIVE_RULE_VM_ABI_VERSION_110,
    ADDITIVE_RULE_VM_ABI_VERSION_113, ADDITIVE_RULE_VM_ABI_VERSION_114,
    ADDITIVE_RULE_VM_ABI_VERSION_116, ADDITIVE_RULE_VM_ABI_VERSION_117,
    ADDITIVE_RULE_VM_ABI_VERSION_118, ADDITIVE_RULE_VM_ABI_VERSION_119,
    ADDITIVE_RULE_VM_ABI_VERSION_12, ADDITIVE_RULE_VM_ABI_VERSION_120,
    ADDITIVE_RULE_VM_ABI_VERSION_121, ADDITIVE_RULE_VM_ABI_VERSION_122,
    ADDITIVE_RULE_VM_ABI_VERSION_123, ADDITIVE_RULE_VM_ABI_VERSION_124,
    ADDITIVE_RULE_VM_ABI_VERSION_125, ADDITIVE_RULE_VM_ABI_VERSION_127,
    ADDITIVE_RULE_VM_ABI_VERSION_129, ADDITIVE_RULE_VM_ABI_VERSION_13,
    ADDITIVE_RULE_VM_ABI_VERSION_130, ADDITIVE_RULE_VM_ABI_VERSION_132,
    ADDITIVE_RULE_VM_ABI_VERSION_133, ADDITIVE_RULE_VM_ABI_VERSION_134,
    ADDITIVE_RULE_VM_ABI_VERSION_135, ADDITIVE_RULE_VM_ABI_VERSION_136,
    ADDITIVE_RULE_VM_ABI_VERSION_137, ADDITIVE_RULE_VM_ABI_VERSION_138,
    ADDITIVE_RULE_VM_ABI_VERSION_140, ADDITIVE_RULE_VM_ABI_VERSION_142,
    ADDITIVE_RULE_VM_ABI_VERSION_143, ADDITIVE_RULE_VM_ABI_VERSION_144,
    ADDITIVE_RULE_VM_ABI_VERSION_146, ADDITIVE_RULE_VM_ABI_VERSION_147,
    ADDITIVE_RULE_VM_ABI_VERSION_148, ADDITIVE_RULE_VM_ABI_VERSION_149,
    ADDITIVE_RULE_VM_ABI_VERSION_15, ADDITIVE_RULE_VM_ABI_VERSION_150,
    ADDITIVE_RULE_VM_ABI_VERSION_151, ADDITIVE_RULE_VM_ABI_VERSION_152,
    ADDITIVE_RULE_VM_ABI_VERSION_153, ADDITIVE_RULE_VM_ABI_VERSION_154,
    ADDITIVE_RULE_VM_ABI_VERSION_155, ADDITIVE_RULE_VM_ABI_VERSION_156,
    ADDITIVE_RULE_VM_ABI_VERSION_159, ADDITIVE_RULE_VM_ABI_VERSION_16,
    ADDITIVE_RULE_VM_ABI_VERSION_160, ADDITIVE_RULE_VM_ABI_VERSION_161,
    ADDITIVE_RULE_VM_ABI_VERSION_162, ADDITIVE_RULE_VM_ABI_VERSION_168,
    ADDITIVE_RULE_VM_ABI_VERSION_171, ADDITIVE_RULE_VM_ABI_VERSION_172,
    ADDITIVE_RULE_VM_ABI_VERSION_173, ADDITIVE_RULE_VM_ABI_VERSION_174,
    ADDITIVE_RULE_VM_ABI_VERSION_177, ADDITIVE_RULE_VM_ABI_VERSION_178,
    ADDITIVE_RULE_VM_ABI_VERSION_18, ADDITIVE_RULE_VM_ABI_VERSION_180,
    ADDITIVE_RULE_VM_ABI_VERSION_181, ADDITIVE_RULE_VM_ABI_VERSION_182,
    ADDITIVE_RULE_VM_ABI_VERSION_183, ADDITIVE_RULE_VM_ABI_VERSION_185,
    ADDITIVE_RULE_VM_ABI_VERSION_186, ADDITIVE_RULE_VM_ABI_VERSION_188,
    ADDITIVE_RULE_VM_ABI_VERSION_189, ADDITIVE_RULE_VM_ABI_VERSION_190,
    ADDITIVE_RULE_VM_ABI_VERSION_191, ADDITIVE_RULE_VM_ABI_VERSION_192,
    ADDITIVE_RULE_VM_ABI_VERSION_194, ADDITIVE_RULE_VM_ABI_VERSION_195,
    ADDITIVE_RULE_VM_ABI_VERSION_196, ADDITIVE_RULE_VM_ABI_VERSION_197,
    ADDITIVE_RULE_VM_ABI_VERSION_198, ADDITIVE_RULE_VM_ABI_VERSION_200,
    ADDITIVE_RULE_VM_ABI_VERSION_201, ADDITIVE_RULE_VM_ABI_VERSION_202,
    ADDITIVE_RULE_VM_ABI_VERSION_203, ADDITIVE_RULE_VM_ABI_VERSION_204,
    ADDITIVE_RULE_VM_ABI_VERSION_205, ADDITIVE_RULE_VM_ABI_VERSION_206,
    ADDITIVE_RULE_VM_ABI_VERSION_207, ADDITIVE_RULE_VM_ABI_VERSION_208,
    ADDITIVE_RULE_VM_ABI_VERSION_209, ADDITIVE_RULE_VM_ABI_VERSION_210,
    ADDITIVE_RULE_VM_ABI_VERSION_212, ADDITIVE_RULE_VM_ABI_VERSION_213,
    ADDITIVE_RULE_VM_ABI_VERSION_214, ADDITIVE_RULE_VM_ABI_VERSION_215,
    ADDITIVE_RULE_VM_ABI_VERSION_216, ADDITIVE_RULE_VM_ABI_VERSION_217,
    ADDITIVE_RULE_VM_ABI_VERSION_218, ADDITIVE_RULE_VM_ABI_VERSION_219,
    ADDITIVE_RULE_VM_ABI_VERSION_220, ADDITIVE_RULE_VM_ABI_VERSION_221,
    ADDITIVE_RULE_VM_ABI_VERSION_222, ADDITIVE_RULE_VM_ABI_VERSION_223,
    ADDITIVE_RULE_VM_ABI_VERSION_224, ADDITIVE_RULE_VM_ABI_VERSION_225,
    ADDITIVE_RULE_VM_ABI_VERSION_226, ADDITIVE_RULE_VM_ABI_VERSION_227,
    ADDITIVE_RULE_VM_ABI_VERSION_228, ADDITIVE_RULE_VM_ABI_VERSION_229,
    ADDITIVE_RULE_VM_ABI_VERSION_230, ADDITIVE_RULE_VM_ABI_VERSION_231,
    ADDITIVE_RULE_VM_ABI_VERSION_232, ADDITIVE_RULE_VM_ABI_VERSION_233,
    ADDITIVE_RULE_VM_ABI_VERSION_234, ADDITIVE_RULE_VM_ABI_VERSION_235,
    ADDITIVE_RULE_VM_ABI_VERSION_236, ADDITIVE_RULE_VM_ABI_VERSION_237,
    ADDITIVE_RULE_VM_ABI_VERSION_238, ADDITIVE_RULE_VM_ABI_VERSION_239,
    ADDITIVE_RULE_VM_ABI_VERSION_240, ADDITIVE_RULE_VM_ABI_VERSION_241,
    ADDITIVE_RULE_VM_ABI_VERSION_242, ADDITIVE_RULE_VM_ABI_VERSION_243,
    ADDITIVE_RULE_VM_ABI_VERSION_244, ADDITIVE_RULE_VM_ABI_VERSION_245,
    ADDITIVE_RULE_VM_ABI_VERSION_246, ADDITIVE_RULE_VM_ABI_VERSION_247,
    ADDITIVE_RULE_VM_ABI_VERSION_248, ADDITIVE_RULE_VM_ABI_VERSION_249,
    ADDITIVE_RULE_VM_ABI_VERSION_250, ADDITIVE_RULE_VM_ABI_VERSION_251,
    ADDITIVE_RULE_VM_ABI_VERSION_252, ADDITIVE_RULE_VM_ABI_VERSION_253,
    ADDITIVE_RULE_VM_ABI_VERSION_254, ADDITIVE_RULE_VM_ABI_VERSION_255,
    ADDITIVE_RULE_VM_ABI_VERSION_256, ADDITIVE_RULE_VM_ABI_VERSION_257,
    ADDITIVE_RULE_VM_ABI_VERSION_258, ADDITIVE_RULE_VM_ABI_VERSION_259,
    ADDITIVE_RULE_VM_ABI_VERSION_260, ADDITIVE_RULE_VM_ABI_VERSION_261,
    ADDITIVE_RULE_VM_ABI_VERSION_262, ADDITIVE_RULE_VM_ABI_VERSION_263,
    ADDITIVE_RULE_VM_ABI_VERSION_264, ADDITIVE_RULE_VM_ABI_VERSION_265,
    ADDITIVE_RULE_VM_ABI_VERSION_266, ADDITIVE_RULE_VM_ABI_VERSION_267,
    ADDITIVE_RULE_VM_ABI_VERSION_268, ADDITIVE_RULE_VM_ABI_VERSION_269,
    ADDITIVE_RULE_VM_ABI_VERSION_270, ADDITIVE_RULE_VM_ABI_VERSION_271,
    ADDITIVE_RULE_VM_ABI_VERSION_272, ADDITIVE_RULE_VM_ABI_VERSION_275,
    ADDITIVE_RULE_VM_ABI_VERSION_277, ADDITIVE_RULE_VM_ABI_VERSION_278,
    ADDITIVE_RULE_VM_ABI_VERSION_279, ADDITIVE_RULE_VM_ABI_VERSION_280,
    ADDITIVE_RULE_VM_ABI_VERSION_282, ADDITIVE_RULE_VM_ABI_VERSION_283,
    ADDITIVE_RULE_VM_ABI_VERSION_284, ADDITIVE_RULE_VM_ABI_VERSION_285,
    ADDITIVE_RULE_VM_ABI_VERSION_286, ADDITIVE_RULE_VM_ABI_VERSION_287,
    ADDITIVE_RULE_VM_ABI_VERSION_288, ADDITIVE_RULE_VM_ABI_VERSION_291,
    ADDITIVE_RULE_VM_ABI_VERSION_294, ADDITIVE_RULE_VM_ABI_VERSION_295,
    ADDITIVE_RULE_VM_ABI_VERSION_296, ADDITIVE_RULE_VM_ABI_VERSION_297,
    ADDITIVE_RULE_VM_ABI_VERSION_298, ADDITIVE_RULE_VM_ABI_VERSION_299,
    ADDITIVE_RULE_VM_ABI_VERSION_30, ADDITIVE_RULE_VM_ABI_VERSION_300,
    ADDITIVE_RULE_VM_ABI_VERSION_301, ADDITIVE_RULE_VM_ABI_VERSION_302,
    ADDITIVE_RULE_VM_ABI_VERSION_303, ADDITIVE_RULE_VM_ABI_VERSION_307,
    ADDITIVE_RULE_VM_ABI_VERSION_308, ADDITIVE_RULE_VM_ABI_VERSION_309,
    ADDITIVE_RULE_VM_ABI_VERSION_31, ADDITIVE_RULE_VM_ABI_VERSION_310,
    ADDITIVE_RULE_VM_ABI_VERSION_311, ADDITIVE_RULE_VM_ABI_VERSION_312,
    ADDITIVE_RULE_VM_ABI_VERSION_313, ADDITIVE_RULE_VM_ABI_VERSION_314,
    ADDITIVE_RULE_VM_ABI_VERSION_315, ADDITIVE_RULE_VM_ABI_VERSION_316,
    ADDITIVE_RULE_VM_ABI_VERSION_317, ADDITIVE_RULE_VM_ABI_VERSION_318,
    ADDITIVE_RULE_VM_ABI_VERSION_319, ADDITIVE_RULE_VM_ABI_VERSION_32,
    ADDITIVE_RULE_VM_ABI_VERSION_320, ADDITIVE_RULE_VM_ABI_VERSION_321,
    ADDITIVE_RULE_VM_ABI_VERSION_322, ADDITIVE_RULE_VM_ABI_VERSION_323,
    ADDITIVE_RULE_VM_ABI_VERSION_324, ADDITIVE_RULE_VM_ABI_VERSION_325,
    ADDITIVE_RULE_VM_ABI_VERSION_326, ADDITIVE_RULE_VM_ABI_VERSION_327,
    ADDITIVE_RULE_VM_ABI_VERSION_328, ADDITIVE_RULE_VM_ABI_VERSION_329,
    ADDITIVE_RULE_VM_ABI_VERSION_33, ADDITIVE_RULE_VM_ABI_VERSION_330,
    ADDITIVE_RULE_VM_ABI_VERSION_331, ADDITIVE_RULE_VM_ABI_VERSION_332,
    ADDITIVE_RULE_VM_ABI_VERSION_333, ADDITIVE_RULE_VM_ABI_VERSION_334,
    ADDITIVE_RULE_VM_ABI_VERSION_336, ADDITIVE_RULE_VM_ABI_VERSION_338,
    ADDITIVE_RULE_VM_ABI_VERSION_339, ADDITIVE_RULE_VM_ABI_VERSION_34,
    ADDITIVE_RULE_VM_ABI_VERSION_340, ADDITIVE_RULE_VM_ABI_VERSION_341,
    ADDITIVE_RULE_VM_ABI_VERSION_342, ADDITIVE_RULE_VM_ABI_VERSION_343,
    ADDITIVE_RULE_VM_ABI_VERSION_344, ADDITIVE_RULE_VM_ABI_VERSION_348,
    ADDITIVE_RULE_VM_ABI_VERSION_349, ADDITIVE_RULE_VM_ABI_VERSION_35,
    ADDITIVE_RULE_VM_ABI_VERSION_350, ADDITIVE_RULE_VM_ABI_VERSION_352,
    ADDITIVE_RULE_VM_ABI_VERSION_355, ADDITIVE_RULE_VM_ABI_VERSION_356,
    ADDITIVE_RULE_VM_ABI_VERSION_359, ADDITIVE_RULE_VM_ABI_VERSION_36,
    ADDITIVE_RULE_VM_ABI_VERSION_360, ADDITIVE_RULE_VM_ABI_VERSION_361,
    ADDITIVE_RULE_VM_ABI_VERSION_362, ADDITIVE_RULE_VM_ABI_VERSION_363,
    ADDITIVE_RULE_VM_ABI_VERSION_364, ADDITIVE_RULE_VM_ABI_VERSION_365,
    ADDITIVE_RULE_VM_ABI_VERSION_366, ADDITIVE_RULE_VM_ABI_VERSION_367,
    ADDITIVE_RULE_VM_ABI_VERSION_368, ADDITIVE_RULE_VM_ABI_VERSION_369,
    ADDITIVE_RULE_VM_ABI_VERSION_37, ADDITIVE_RULE_VM_ABI_VERSION_370,
    ADDITIVE_RULE_VM_ABI_VERSION_371, ADDITIVE_RULE_VM_ABI_VERSION_372,
    ADDITIVE_RULE_VM_ABI_VERSION_373, ADDITIVE_RULE_VM_ABI_VERSION_374,
    ADDITIVE_RULE_VM_ABI_VERSION_375, ADDITIVE_RULE_VM_ABI_VERSION_376,
    ADDITIVE_RULE_VM_ABI_VERSION_377, ADDITIVE_RULE_VM_ABI_VERSION_378,
    ADDITIVE_RULE_VM_ABI_VERSION_379, ADDITIVE_RULE_VM_ABI_VERSION_38,
    ADDITIVE_RULE_VM_ABI_VERSION_380, ADDITIVE_RULE_VM_ABI_VERSION_381,
    ADDITIVE_RULE_VM_ABI_VERSION_382, ADDITIVE_RULE_VM_ABI_VERSION_384,
    ADDITIVE_RULE_VM_ABI_VERSION_385, ADDITIVE_RULE_VM_ABI_VERSION_386,
    ADDITIVE_RULE_VM_ABI_VERSION_387, ADDITIVE_RULE_VM_ABI_VERSION_388,
    ADDITIVE_RULE_VM_ABI_VERSION_389, ADDITIVE_RULE_VM_ABI_VERSION_39,
    ADDITIVE_RULE_VM_ABI_VERSION_390, ADDITIVE_RULE_VM_ABI_VERSION_391,
    ADDITIVE_RULE_VM_ABI_VERSION_392, ADDITIVE_RULE_VM_ABI_VERSION_393,
    ADDITIVE_RULE_VM_ABI_VERSION_394, ADDITIVE_RULE_VM_ABI_VERSION_395,
    ADDITIVE_RULE_VM_ABI_VERSION_396, ADDITIVE_RULE_VM_ABI_VERSION_397,
    ADDITIVE_RULE_VM_ABI_VERSION_398, ADDITIVE_RULE_VM_ABI_VERSION_399,
    ADDITIVE_RULE_VM_ABI_VERSION_40, ADDITIVE_RULE_VM_ABI_VERSION_400,
    ADDITIVE_RULE_VM_ABI_VERSION_401, ADDITIVE_RULE_VM_ABI_VERSION_403,
    ADDITIVE_RULE_VM_ABI_VERSION_404, ADDITIVE_RULE_VM_ABI_VERSION_406,
    ADDITIVE_RULE_VM_ABI_VERSION_409, ADDITIVE_RULE_VM_ABI_VERSION_41,
    ADDITIVE_RULE_VM_ABI_VERSION_410, ADDITIVE_RULE_VM_ABI_VERSION_411,
    ADDITIVE_RULE_VM_ABI_VERSION_412, ADDITIVE_RULE_VM_ABI_VERSION_413,
    ADDITIVE_RULE_VM_ABI_VERSION_414, ADDITIVE_RULE_VM_ABI_VERSION_415,
    ADDITIVE_RULE_VM_ABI_VERSION_416, ADDITIVE_RULE_VM_ABI_VERSION_417,
    ADDITIVE_RULE_VM_ABI_VERSION_418, ADDITIVE_RULE_VM_ABI_VERSION_419,
    ADDITIVE_RULE_VM_ABI_VERSION_42, ADDITIVE_RULE_VM_ABI_VERSION_420,
    ADDITIVE_RULE_VM_ABI_VERSION_421, ADDITIVE_RULE_VM_ABI_VERSION_422,
    ADDITIVE_RULE_VM_ABI_VERSION_423, ADDITIVE_RULE_VM_ABI_VERSION_424,
    ADDITIVE_RULE_VM_ABI_VERSION_425, ADDITIVE_RULE_VM_ABI_VERSION_426,
    ADDITIVE_RULE_VM_ABI_VERSION_427, ADDITIVE_RULE_VM_ABI_VERSION_428,
    ADDITIVE_RULE_VM_ABI_VERSION_429, ADDITIVE_RULE_VM_ABI_VERSION_43,
    ADDITIVE_RULE_VM_ABI_VERSION_430, ADDITIVE_RULE_VM_ABI_VERSION_431,
    ADDITIVE_RULE_VM_ABI_VERSION_432, ADDITIVE_RULE_VM_ABI_VERSION_433,
    ADDITIVE_RULE_VM_ABI_VERSION_434, ADDITIVE_RULE_VM_ABI_VERSION_435,
    ADDITIVE_RULE_VM_ABI_VERSION_436, ADDITIVE_RULE_VM_ABI_VERSION_437,
    ADDITIVE_RULE_VM_ABI_VERSION_439, ADDITIVE_RULE_VM_ABI_VERSION_44,
    ADDITIVE_RULE_VM_ABI_VERSION_440, ADDITIVE_RULE_VM_ABI_VERSION_441,
    ADDITIVE_RULE_VM_ABI_VERSION_442, ADDITIVE_RULE_VM_ABI_VERSION_443,
    ADDITIVE_RULE_VM_ABI_VERSION_444, ADDITIVE_RULE_VM_ABI_VERSION_445,
    ADDITIVE_RULE_VM_ABI_VERSION_446, ADDITIVE_RULE_VM_ABI_VERSION_447,
    ADDITIVE_RULE_VM_ABI_VERSION_45, ADDITIVE_RULE_VM_ABI_VERSION_450,
    ADDITIVE_RULE_VM_ABI_VERSION_451, ADDITIVE_RULE_VM_ABI_VERSION_453,
    ADDITIVE_RULE_VM_ABI_VERSION_455, ADDITIVE_RULE_VM_ABI_VERSION_456,
    ADDITIVE_RULE_VM_ABI_VERSION_457, ADDITIVE_RULE_VM_ABI_VERSION_458,
    ADDITIVE_RULE_VM_ABI_VERSION_459, ADDITIVE_RULE_VM_ABI_VERSION_46,
    ADDITIVE_RULE_VM_ABI_VERSION_460, ADDITIVE_RULE_VM_ABI_VERSION_461,
    ADDITIVE_RULE_VM_ABI_VERSION_462, ADDITIVE_RULE_VM_ABI_VERSION_463,
    ADDITIVE_RULE_VM_ABI_VERSION_464, ADDITIVE_RULE_VM_ABI_VERSION_465,
    ADDITIVE_RULE_VM_ABI_VERSION_466, ADDITIVE_RULE_VM_ABI_VERSION_467,
    ADDITIVE_RULE_VM_ABI_VERSION_468, ADDITIVE_RULE_VM_ABI_VERSION_469,
    ADDITIVE_RULE_VM_ABI_VERSION_47, ADDITIVE_RULE_VM_ABI_VERSION_470,
    ADDITIVE_RULE_VM_ABI_VERSION_471, ADDITIVE_RULE_VM_ABI_VERSION_472,
    ADDITIVE_RULE_VM_ABI_VERSION_473, ADDITIVE_RULE_VM_ABI_VERSION_474,
    ADDITIVE_RULE_VM_ABI_VERSION_475, ADDITIVE_RULE_VM_ABI_VERSION_476,
    ADDITIVE_RULE_VM_ABI_VERSION_477, ADDITIVE_RULE_VM_ABI_VERSION_478,
    ADDITIVE_RULE_VM_ABI_VERSION_479, ADDITIVE_RULE_VM_ABI_VERSION_48,
    ADDITIVE_RULE_VM_ABI_VERSION_480, ADDITIVE_RULE_VM_ABI_VERSION_481,
    ADDITIVE_RULE_VM_ABI_VERSION_482, ADDITIVE_RULE_VM_ABI_VERSION_483,
    ADDITIVE_RULE_VM_ABI_VERSION_484, ADDITIVE_RULE_VM_ABI_VERSION_485,
    ADDITIVE_RULE_VM_ABI_VERSION_486, ADDITIVE_RULE_VM_ABI_VERSION_487,
    ADDITIVE_RULE_VM_ABI_VERSION_49, ADDITIVE_RULE_VM_ABI_VERSION_490,
    ADDITIVE_RULE_VM_ABI_VERSION_491, ADDITIVE_RULE_VM_ABI_VERSION_492,
    ADDITIVE_RULE_VM_ABI_VERSION_493, ADDITIVE_RULE_VM_ABI_VERSION_494,
    ADDITIVE_RULE_VM_ABI_VERSION_495, ADDITIVE_RULE_VM_ABI_VERSION_496,
    ADDITIVE_RULE_VM_ABI_VERSION_497, ADDITIVE_RULE_VM_ABI_VERSION_498,
    ADDITIVE_RULE_VM_ABI_VERSION_499, ADDITIVE_RULE_VM_ABI_VERSION_50,
    ADDITIVE_RULE_VM_ABI_VERSION_500, ADDITIVE_RULE_VM_ABI_VERSION_501,
    ADDITIVE_RULE_VM_ABI_VERSION_502, ADDITIVE_RULE_VM_ABI_VERSION_503,
    ADDITIVE_RULE_VM_ABI_VERSION_504, ADDITIVE_RULE_VM_ABI_VERSION_505,
    ADDITIVE_RULE_VM_ABI_VERSION_506, ADDITIVE_RULE_VM_ABI_VERSION_507,
    ADDITIVE_RULE_VM_ABI_VERSION_508, ADDITIVE_RULE_VM_ABI_VERSION_509,
    ADDITIVE_RULE_VM_ABI_VERSION_51, ADDITIVE_RULE_VM_ABI_VERSION_510,
    ADDITIVE_RULE_VM_ABI_VERSION_511, ADDITIVE_RULE_VM_ABI_VERSION_512,
    ADDITIVE_RULE_VM_ABI_VERSION_513, ADDITIVE_RULE_VM_ABI_VERSION_514,
    ADDITIVE_RULE_VM_ABI_VERSION_515, ADDITIVE_RULE_VM_ABI_VERSION_516,
    ADDITIVE_RULE_VM_ABI_VERSION_517, ADDITIVE_RULE_VM_ABI_VERSION_518,
    ADDITIVE_RULE_VM_ABI_VERSION_519, ADDITIVE_RULE_VM_ABI_VERSION_52,
    ADDITIVE_RULE_VM_ABI_VERSION_520, ADDITIVE_RULE_VM_ABI_VERSION_521,
    ADDITIVE_RULE_VM_ABI_VERSION_522, ADDITIVE_RULE_VM_ABI_VERSION_523,
    ADDITIVE_RULE_VM_ABI_VERSION_524, ADDITIVE_RULE_VM_ABI_VERSION_525,
    ADDITIVE_RULE_VM_ABI_VERSION_526, ADDITIVE_RULE_VM_ABI_VERSION_527,
    ADDITIVE_RULE_VM_ABI_VERSION_528, ADDITIVE_RULE_VM_ABI_VERSION_529,
    ADDITIVE_RULE_VM_ABI_VERSION_53, ADDITIVE_RULE_VM_ABI_VERSION_531,
    ADDITIVE_RULE_VM_ABI_VERSION_532, ADDITIVE_RULE_VM_ABI_VERSION_533,
    ADDITIVE_RULE_VM_ABI_VERSION_534, ADDITIVE_RULE_VM_ABI_VERSION_535,
    ADDITIVE_RULE_VM_ABI_VERSION_536, ADDITIVE_RULE_VM_ABI_VERSION_537,
    ADDITIVE_RULE_VM_ABI_VERSION_538, ADDITIVE_RULE_VM_ABI_VERSION_539,
    ADDITIVE_RULE_VM_ABI_VERSION_54, ADDITIVE_RULE_VM_ABI_VERSION_540,
    ADDITIVE_RULE_VM_ABI_VERSION_542, ADDITIVE_RULE_VM_ABI_VERSION_543,
    ADDITIVE_RULE_VM_ABI_VERSION_544, ADDITIVE_RULE_VM_ABI_VERSION_545,
    ADDITIVE_RULE_VM_ABI_VERSION_546, ADDITIVE_RULE_VM_ABI_VERSION_547,
    ADDITIVE_RULE_VM_ABI_VERSION_548, ADDITIVE_RULE_VM_ABI_VERSION_549,
    ADDITIVE_RULE_VM_ABI_VERSION_55, ADDITIVE_RULE_VM_ABI_VERSION_551,
    ADDITIVE_RULE_VM_ABI_VERSION_553, ADDITIVE_RULE_VM_ABI_VERSION_554,
    ADDITIVE_RULE_VM_ABI_VERSION_555, ADDITIVE_RULE_VM_ABI_VERSION_556,
    ADDITIVE_RULE_VM_ABI_VERSION_558, ADDITIVE_RULE_VM_ABI_VERSION_559,
    ADDITIVE_RULE_VM_ABI_VERSION_56, ADDITIVE_RULE_VM_ABI_VERSION_560,
    ADDITIVE_RULE_VM_ABI_VERSION_561, ADDITIVE_RULE_VM_ABI_VERSION_562,
    ADDITIVE_RULE_VM_ABI_VERSION_563, ADDITIVE_RULE_VM_ABI_VERSION_564,
    ADDITIVE_RULE_VM_ABI_VERSION_565, ADDITIVE_RULE_VM_ABI_VERSION_566,
    ADDITIVE_RULE_VM_ABI_VERSION_567, ADDITIVE_RULE_VM_ABI_VERSION_568,
    ADDITIVE_RULE_VM_ABI_VERSION_569, ADDITIVE_RULE_VM_ABI_VERSION_57,
    ADDITIVE_RULE_VM_ABI_VERSION_570, ADDITIVE_RULE_VM_ABI_VERSION_571,
    ADDITIVE_RULE_VM_ABI_VERSION_572, ADDITIVE_RULE_VM_ABI_VERSION_573,
    ADDITIVE_RULE_VM_ABI_VERSION_574, ADDITIVE_RULE_VM_ABI_VERSION_575,
    ADDITIVE_RULE_VM_ABI_VERSION_576, ADDITIVE_RULE_VM_ABI_VERSION_577,
    ADDITIVE_RULE_VM_ABI_VERSION_578, ADDITIVE_RULE_VM_ABI_VERSION_579,
    ADDITIVE_RULE_VM_ABI_VERSION_58, ADDITIVE_RULE_VM_ABI_VERSION_580,
    ADDITIVE_RULE_VM_ABI_VERSION_581, ADDITIVE_RULE_VM_ABI_VERSION_582,
    ADDITIVE_RULE_VM_ABI_VERSION_583, ADDITIVE_RULE_VM_ABI_VERSION_584,
    ADDITIVE_RULE_VM_ABI_VERSION_585, ADDITIVE_RULE_VM_ABI_VERSION_586,
    ADDITIVE_RULE_VM_ABI_VERSION_587, ADDITIVE_RULE_VM_ABI_VERSION_588,
    ADDITIVE_RULE_VM_ABI_VERSION_589, ADDITIVE_RULE_VM_ABI_VERSION_59,
    ADDITIVE_RULE_VM_ABI_VERSION_590, ADDITIVE_RULE_VM_ABI_VERSION_591,
    ADDITIVE_RULE_VM_ABI_VERSION_593, ADDITIVE_RULE_VM_ABI_VERSION_594,
    ADDITIVE_RULE_VM_ABI_VERSION_595, ADDITIVE_RULE_VM_ABI_VERSION_596,
    ADDITIVE_RULE_VM_ABI_VERSION_597, ADDITIVE_RULE_VM_ABI_VERSION_598,
    ADDITIVE_RULE_VM_ABI_VERSION_599, ADDITIVE_RULE_VM_ABI_VERSION_6,
    ADDITIVE_RULE_VM_ABI_VERSION_60, ADDITIVE_RULE_VM_ABI_VERSION_600,
    ADDITIVE_RULE_VM_ABI_VERSION_601, ADDITIVE_RULE_VM_ABI_VERSION_602,
    ADDITIVE_RULE_VM_ABI_VERSION_603, ADDITIVE_RULE_VM_ABI_VERSION_604,
    ADDITIVE_RULE_VM_ABI_VERSION_605, ADDITIVE_RULE_VM_ABI_VERSION_606,
    ADDITIVE_RULE_VM_ABI_VERSION_607, ADDITIVE_RULE_VM_ABI_VERSION_608,
    ADDITIVE_RULE_VM_ABI_VERSION_609, ADDITIVE_RULE_VM_ABI_VERSION_61,
    ADDITIVE_RULE_VM_ABI_VERSION_610, ADDITIVE_RULE_VM_ABI_VERSION_611,
    ADDITIVE_RULE_VM_ABI_VERSION_612, ADDITIVE_RULE_VM_ABI_VERSION_613,
    ADDITIVE_RULE_VM_ABI_VERSION_614, ADDITIVE_RULE_VM_ABI_VERSION_615,
    ADDITIVE_RULE_VM_ABI_VERSION_616, ADDITIVE_RULE_VM_ABI_VERSION_617,
    ADDITIVE_RULE_VM_ABI_VERSION_618, ADDITIVE_RULE_VM_ABI_VERSION_619,
    ADDITIVE_RULE_VM_ABI_VERSION_62, ADDITIVE_RULE_VM_ABI_VERSION_621,
    ADDITIVE_RULE_VM_ABI_VERSION_622, ADDITIVE_RULE_VM_ABI_VERSION_623,
    ADDITIVE_RULE_VM_ABI_VERSION_624, ADDITIVE_RULE_VM_ABI_VERSION_625,
    ADDITIVE_RULE_VM_ABI_VERSION_626, ADDITIVE_RULE_VM_ABI_VERSION_627,
    ADDITIVE_RULE_VM_ABI_VERSION_628, ADDITIVE_RULE_VM_ABI_VERSION_629,
    ADDITIVE_RULE_VM_ABI_VERSION_63, ADDITIVE_RULE_VM_ABI_VERSION_630,
    ADDITIVE_RULE_VM_ABI_VERSION_631, ADDITIVE_RULE_VM_ABI_VERSION_632,
    ADDITIVE_RULE_VM_ABI_VERSION_633, ADDITIVE_RULE_VM_ABI_VERSION_634,
    ADDITIVE_RULE_VM_ABI_VERSION_635, ADDITIVE_RULE_VM_ABI_VERSION_636,
    ADDITIVE_RULE_VM_ABI_VERSION_637, ADDITIVE_RULE_VM_ABI_VERSION_638,
    ADDITIVE_RULE_VM_ABI_VERSION_639, ADDITIVE_RULE_VM_ABI_VERSION_64,
    ADDITIVE_RULE_VM_ABI_VERSION_640, ADDITIVE_RULE_VM_ABI_VERSION_641,
    ADDITIVE_RULE_VM_ABI_VERSION_642, ADDITIVE_RULE_VM_ABI_VERSION_643,
    ADDITIVE_RULE_VM_ABI_VERSION_644, ADDITIVE_RULE_VM_ABI_VERSION_645,
    ADDITIVE_RULE_VM_ABI_VERSION_646, ADDITIVE_RULE_VM_ABI_VERSION_647,
    ADDITIVE_RULE_VM_ABI_VERSION_648, ADDITIVE_RULE_VM_ABI_VERSION_649,
    ADDITIVE_RULE_VM_ABI_VERSION_65, ADDITIVE_RULE_VM_ABI_VERSION_650,
    ADDITIVE_RULE_VM_ABI_VERSION_651, ADDITIVE_RULE_VM_ABI_VERSION_652,
    ADDITIVE_RULE_VM_ABI_VERSION_653, ADDITIVE_RULE_VM_ABI_VERSION_654,
    ADDITIVE_RULE_VM_ABI_VERSION_655, ADDITIVE_RULE_VM_ABI_VERSION_656,
    ADDITIVE_RULE_VM_ABI_VERSION_657, ADDITIVE_RULE_VM_ABI_VERSION_658,
    ADDITIVE_RULE_VM_ABI_VERSION_659, ADDITIVE_RULE_VM_ABI_VERSION_66,
    ADDITIVE_RULE_VM_ABI_VERSION_660, ADDITIVE_RULE_VM_ABI_VERSION_661,
    ADDITIVE_RULE_VM_ABI_VERSION_662, ADDITIVE_RULE_VM_ABI_VERSION_663,
    ADDITIVE_RULE_VM_ABI_VERSION_664, ADDITIVE_RULE_VM_ABI_VERSION_665,
    ADDITIVE_RULE_VM_ABI_VERSION_666, ADDITIVE_RULE_VM_ABI_VERSION_667,
    ADDITIVE_RULE_VM_ABI_VERSION_668, ADDITIVE_RULE_VM_ABI_VERSION_669,
    ADDITIVE_RULE_VM_ABI_VERSION_67, ADDITIVE_RULE_VM_ABI_VERSION_670,
    ADDITIVE_RULE_VM_ABI_VERSION_671, ADDITIVE_RULE_VM_ABI_VERSION_672,
    ADDITIVE_RULE_VM_ABI_VERSION_673, ADDITIVE_RULE_VM_ABI_VERSION_674,
    ADDITIVE_RULE_VM_ABI_VERSION_675, ADDITIVE_RULE_VM_ABI_VERSION_676,
    ADDITIVE_RULE_VM_ABI_VERSION_677, ADDITIVE_RULE_VM_ABI_VERSION_678,
    ADDITIVE_RULE_VM_ABI_VERSION_679, ADDITIVE_RULE_VM_ABI_VERSION_68,
    ADDITIVE_RULE_VM_ABI_VERSION_680, ADDITIVE_RULE_VM_ABI_VERSION_681,
    ADDITIVE_RULE_VM_ABI_VERSION_682, ADDITIVE_RULE_VM_ABI_VERSION_683,
    ADDITIVE_RULE_VM_ABI_VERSION_684, ADDITIVE_RULE_VM_ABI_VERSION_685,
    ADDITIVE_RULE_VM_ABI_VERSION_686, ADDITIVE_RULE_VM_ABI_VERSION_687,
    ADDITIVE_RULE_VM_ABI_VERSION_688, ADDITIVE_RULE_VM_ABI_VERSION_689,
    ADDITIVE_RULE_VM_ABI_VERSION_69, ADDITIVE_RULE_VM_ABI_VERSION_690,
    ADDITIVE_RULE_VM_ABI_VERSION_691, ADDITIVE_RULE_VM_ABI_VERSION_692,
    ADDITIVE_RULE_VM_ABI_VERSION_693, ADDITIVE_RULE_VM_ABI_VERSION_694,
    ADDITIVE_RULE_VM_ABI_VERSION_695, ADDITIVE_RULE_VM_ABI_VERSION_696,
    ADDITIVE_RULE_VM_ABI_VERSION_697, ADDITIVE_RULE_VM_ABI_VERSION_698,
    ADDITIVE_RULE_VM_ABI_VERSION_699, ADDITIVE_RULE_VM_ABI_VERSION_7,
    ADDITIVE_RULE_VM_ABI_VERSION_70, ADDITIVE_RULE_VM_ABI_VERSION_700,
    ADDITIVE_RULE_VM_ABI_VERSION_701, ADDITIVE_RULE_VM_ABI_VERSION_702,
    ADDITIVE_RULE_VM_ABI_VERSION_703, ADDITIVE_RULE_VM_ABI_VERSION_704,
    ADDITIVE_RULE_VM_ABI_VERSION_705, ADDITIVE_RULE_VM_ABI_VERSION_706,
    ADDITIVE_RULE_VM_ABI_VERSION_707, ADDITIVE_RULE_VM_ABI_VERSION_708,
    ADDITIVE_RULE_VM_ABI_VERSION_709, ADDITIVE_RULE_VM_ABI_VERSION_710,
    ADDITIVE_RULE_VM_ABI_VERSION_711, ADDITIVE_RULE_VM_ABI_VERSION_712,
    ADDITIVE_RULE_VM_ABI_VERSION_713, ADDITIVE_RULE_VM_ABI_VERSION_714,
    ADDITIVE_RULE_VM_ABI_VERSION_715, ADDITIVE_RULE_VM_ABI_VERSION_716,
    ADDITIVE_RULE_VM_ABI_VERSION_717, ADDITIVE_RULE_VM_ABI_VERSION_718,
    ADDITIVE_RULE_VM_ABI_VERSION_719, ADDITIVE_RULE_VM_ABI_VERSION_720,
    ADDITIVE_RULE_VM_ABI_VERSION_721, ADDITIVE_RULE_VM_ABI_VERSION_722,
    ADDITIVE_RULE_VM_ABI_VERSION_723, ADDITIVE_RULE_VM_ABI_VERSION_724,
    ADDITIVE_RULE_VM_ABI_VERSION_725, ADDITIVE_RULE_VM_ABI_VERSION_726,
    ADDITIVE_RULE_VM_ABI_VERSION_727, ADDITIVE_RULE_VM_ABI_VERSION_730,
    ADDITIVE_RULE_VM_ABI_VERSION_732, ADDITIVE_RULE_VM_ABI_VERSION_733,
    ADDITIVE_RULE_VM_ABI_VERSION_734, ADDITIVE_RULE_VM_ABI_VERSION_735,
    ADDITIVE_RULE_VM_ABI_VERSION_736, ADDITIVE_RULE_VM_ABI_VERSION_737,
    ADDITIVE_RULE_VM_ABI_VERSION_738, ADDITIVE_RULE_VM_ABI_VERSION_739,
    ADDITIVE_RULE_VM_ABI_VERSION_74, ADDITIVE_RULE_VM_ABI_VERSION_740,
    ADDITIVE_RULE_VM_ABI_VERSION_741, ADDITIVE_RULE_VM_ABI_VERSION_742,
    ADDITIVE_RULE_VM_ABI_VERSION_744, ADDITIVE_RULE_VM_ABI_VERSION_745,
    ADDITIVE_RULE_VM_ABI_VERSION_746, ADDITIVE_RULE_VM_ABI_VERSION_747,
    ADDITIVE_RULE_VM_ABI_VERSION_748, ADDITIVE_RULE_VM_ABI_VERSION_749,
    ADDITIVE_RULE_VM_ABI_VERSION_75, ADDITIVE_RULE_VM_ABI_VERSION_750,
    ADDITIVE_RULE_VM_ABI_VERSION_751, ADDITIVE_RULE_VM_ABI_VERSION_752,
    ADDITIVE_RULE_VM_ABI_VERSION_753, ADDITIVE_RULE_VM_ABI_VERSION_76,
    ADDITIVE_RULE_VM_ABI_VERSION_77, ADDITIVE_RULE_VM_ABI_VERSION_78,
    ADDITIVE_RULE_VM_ABI_VERSION_79, ADDITIVE_RULE_VM_ABI_VERSION_8,
    ADDITIVE_RULE_VM_ABI_VERSION_80, ADDITIVE_RULE_VM_ABI_VERSION_82,
    ADDITIVE_RULE_VM_ABI_VERSION_83, ADDITIVE_RULE_VM_ABI_VERSION_84,
    ADDITIVE_RULE_VM_ABI_VERSION_85, ADDITIVE_RULE_VM_ABI_VERSION_88,
    ADDITIVE_RULE_VM_ABI_VERSION_9, ADDITIVE_RULE_VM_ABI_VERSION_90,
    ADDITIVE_RULE_VM_ABI_VERSION_92, ADDITIVE_RULE_VM_ABI_VERSION_93,
    ADDITIVE_RULE_VM_ABI_VERSION_94, ADDITIVE_RULE_VM_ABI_VERSION_95,
    ADDITIVE_RULE_VM_ABI_VERSION_96, ADDITIVE_RULE_VM_ABI_VERSION_97,
    ADDITIVE_RULE_VM_ABI_VERSION_98, ADDITIVE_RULE_VM_ABI_VERSION_99,
};
use content_schema::{
    ActiveIr, ActiveTargetsIr, ExprIr, ModifierIr, RuleModuleIr, SourceMap, SourceMapEntry, Span,
    StmtIr, TriggerChoiceActor, TriggerIr,
};
use rule_vm::bytecode::{assemble, caps, effect_tag, query_tag, BytecodeModule, Op};

/// Codegen output for one trigger module entry.
#[derive(Debug, Clone)]
pub struct TriggerEmit {
    /// Trigger def id.
    pub trigger_def_id: u32,
    /// Skill def.
    pub skill_def_id: u32,
    /// Event.
    pub event: String,
    /// Filter.
    pub filter: String,
    /// Player confirmation flag.
    pub optional: bool,
    /// Player who confirms an optional trigger.
    pub choice_actor: TriggerChoiceActor,
    /// Module name.
    pub module: String,
    /// Entry PC.
    pub entry_pc: u32,
}

/// Modifier emit.
#[derive(Debug, Clone)]
pub struct ModifierEmit {
    /// Skill def.
    pub skill_def_id: u32,
    /// Target.
    pub target: String,
    /// Optional card selector.
    pub selector: Option<String>,
    /// Mode.
    pub mode: String,
    /// Optional query-time value source.
    pub value_source: Option<String>,
    /// Value.
    pub value: i32,
}

/// Active skill emit.
#[derive(Debug, Clone)]
pub struct ActiveEmit {
    /// Skill def.
    pub skill_def_id: u32,
    /// Window.
    pub window: String,
    /// Module name.
    pub module: String,
    /// Entry PC.
    pub entry_pc: u32,
    /// Optional player-target count contract.
    pub targets: Option<ActiveTargetsIr>,
    /// Optional authoritative per-turn activation limit.
    pub uses_per_turn: Option<u8>,
    /// Optional authoritative activation limit scoped to one Play phase.
    pub uses_per_play_phase: Option<u8>,
    /// Optional dynamic per-turn activation limit source.
    pub uses_per_turn_source: Option<String>,
    /// Optional owner-mark threshold required before activation.
    pub mark_at_least: Option<(u32, u32)>,
    /// Require the owner not to have this skill definition.
    pub owner_missing_skill: Option<u32>,
    /// Require the owner's hand count to exceed current HP.
    pub hand_gt_hp: bool,
    /// Require hand count above maximum HP.
    pub hand_gt_max_hp: bool,
    /// Require a positive current hand limit.
    pub hand_limit_positive: bool,
    /// Require at least one equipped card.
    pub equip_nonempty: bool,
    /// Require at least one hand card.
    pub hand_nonempty: bool,
    /// Require the owner to be face up.
    pub face_up: bool,
    /// Require the owner's hand to hold no card carrying this gaintag.
    pub hand_missing_gaintag: Option<u32>,
    /// Require the owner not to be linked.
    pub owner_unlinked: bool,
    /// Allow this active skill to resolve only once in the match.
    pub once_per_game: bool,
    /// Optional card selection zone/min/max.
    pub selection: Option<content_schema::ActiveSelectionIr>,
}

/// View-as emit.
#[derive(Debug, Clone)]
pub struct ViewAsEmit {
    /// Skill def.
    pub skill_def_id: u32,
    /// As kind.
    pub as_kind: String,
    /// Window.
    pub window: String,
    /// Filter.
    pub filter: String,
    /// Optional source zone; omitted means hand.
    pub source_zone: Option<String>,
    /// Optional authoritative per-turn activation limit.
    pub uses_per_turn: Option<u8>,
    /// Whether ordinary card distance checks are ignored.
    pub ignore_distance: bool,
    /// Whether a red source makes the converted Sha unrespondable.
    pub red_unrespondable: bool,
    /// Whether a black source avoids consuming the ordinary Sha use count.
    pub black_no_use_count: bool,
    pub no_use_count: bool,
    pub target_lacks_source_suit_unrespondable: bool,
    /// Refund the ordinary Sha use count if this exact converted Sha deals no damage.
    pub refund_use_count_if_no_damage: bool,
    /// Minimum number of physical source cards.
    pub min_cards: u8,
    /// Maximum number of physical source cards.
    pub max_cards: u8,
    /// Require exactly max(1, current HP) physical source cards.
    pub owner_hp_min_one_cards: bool,
    pub owner_basic_uses_plus_one_cards: bool,
    pub once_per_turn_per_branch: bool,
    /// Require equal effective suits for all selected source cards.
    pub same_suit: bool,
    /// Optional logical card nature.
    pub nature: Option<String>,
    /// Optional multi-card conversion bonus.
    pub multi_card_bonus: Option<String>,
    /// Conversion branch required for a source-free family rule.
    pub branch: Option<u8>,
    /// Whether this branch may be used only once in each round.
    pub once_per_round: bool,
    /// Branch selected after a successful conversion.
    pub next_branch: Option<u8>,
    /// Declarative source-free conversion cost.
    pub cost: Option<String>,
    /// Skill-definition mark consumed by the conversion cost.
    pub mark_cost: Option<u32>,
    /// Cards drawn after the converted card is used or responded with.
    pub post_draw: u8,
}

mod package;
pub use package::compile_module_ir;
pub(crate) use package::compile_module_ir_with_required_abi;

/// Deferred jump fixup record. Offsets are computed from actual assembled bytes.
enum BackpatchEntry {
    If {
        jif_op_idx: usize,
        jmp_op_idx: usize,
        else_op_idx: usize,
        end_op_idx: usize,
    },
    Loop {
        jif_op_idx: usize,
        jmp_back_op_idx: usize,
        loop_start_op_idx: usize,
        end_op_idx: usize,
    },
}

struct Assembler {
    ops: Vec<Op>,
    locals: Vec<String>,
    local_count: u8,
    max_stack: u8,
    smap: Vec<SourceMapEntry>,
    required_abi: u32,
    backpatches: Vec<BackpatchEntry>,
}

impl Assembler {
    fn new() -> Self {
        Self {
            ops: Vec::new(),
            locals: Vec::new(),
            local_count: 0,
            max_stack: 8,
            smap: Vec::new(),
            required_abi: 3,
            backpatches: Vec::new(),
        }
    }

    fn mark(&mut self, span: Span, label: &str) {
        let pc = estimate_pc(&self.ops);
        self.smap.push(SourceMapEntry {
            pc,
            span,
            label: label.into(),
        });
    }

    fn local_slot(&mut self, name: &str) -> Result<u8, CompileError> {
        if let Some(i) = self.locals.iter().position(|l| l == name) {
            return Ok(i as u8);
        }
        if self.locals.len() >= 32 {
            return Err(CompileError::Codegen("too many locals".into()));
        }
        self.locals.push(name.into());
        self.local_count = self.locals.len() as u8;
        Ok((self.locals.len() - 1) as u8)
    }

    fn emit_stmts(&mut self, stmts: &[StmtIr]) -> Result<(), CompileError> {
        for s in stmts {
            self.emit_stmt(s)?;
        }
        Ok(())
    }

    fn emit_stmt(&mut self, s: &StmtIr) -> Result<(), CompileError> {
        match s {
            StmtIr::EffectSetDrawPhaseFromBottom { .. }
            | StmtIr::EffectDrawFromBottom { .. }
            | StmtIr::EffectResolveChenglve { .. }
            | StmtIr::EffectResolveShicai { .. }
            | StmtIr::EffectResolveSbbiyue { .. }
            | StmtIr::EffectResolveSxrmqiewang { .. }
            | StmtIr::EffectResolveSxrmqianliu { .. }
            | StmtIr::EffectResolveGzbuqu { .. }
            | StmtIr::EffectResolveGzbuquRecover { .. }
            | StmtIr::EffectResolveXiongluan { .. }
            | StmtIr::EffectResolveTwjuxiang { .. }
            | StmtIr::EffectDraw { .. }
            | StmtIr::EffectRecover { .. }
            | StmtIr::EffectNote { .. }
            | StmtIr::EffectDamage { .. }
            | StmtIr::EffectChangeMaxHp { .. }
            | StmtIr::EffectAddMark { .. }
            | StmtIr::EffectAddMarkExpr { .. }
            | StmtIr::EffectRemoveMark { .. }
            | StmtIr::EffectRemoveMarkExpr { .. }
            | StmtIr::EffectDiscardHand { .. }
            | StmtIr::EffectDiscardHandExpr { .. }
            | StmtIr::EffectDiscardHandEquipExpr { .. }
            | StmtIr::EffectDiscardHandEquip { .. }
            | StmtIr::EffectAdjustHandTo { .. }
            | StmtIr::EffectDiscardSelected { .. }
            | StmtIr::EffectRecastSelected { .. }
            | StmtIr::EffectDisableAllEquipSlots { .. }
            | StmtIr::EffectMoveEquipmentThenDraw { .. }
            | StmtIr::EffectChooseUseVirtualShaOrDraw { .. }
            | StmtIr::EffectDiscardEquipmentPairWithArmor { .. }
            | StmtIr::EffectDiscardTargetEquipmentGainMount { .. }
            | StmtIr::EffectDiscardTargetEquipmentDrawWeaponRange { .. }
            | StmtIr::EffectChooseTargetRevealHandDamageIfSha { .. }
            | StmtIr::EffectChooseOtherGiveOneHandEquip { .. }
            | StmtIr::EffectChooseGiveOrRecastOneHand { .. }
            | StmtIr::EffectResolveOlJieDanshou { .. }
            | StmtIr::EffectChooseRevealHandCardBindCategoryTarget { .. }
            | StmtIr::EffectRecoverToMaxThenOptionalExchangeSkill { .. }
            | StmtIr::EffectRevealHandDiscardKind { .. }
            | StmtIr::EffectChooseMoveTargetEquipmentAndOrDraw { .. }
            | StmtIr::EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                ..
            }
            | StmtIr::EffectChooseDrawOrRecastAddSkill { .. }
            | StmtIr::EffectChooseGainShaForOpeningExtraPlay { .. }
            | StmtIr::EffectFinishOpeningExtraPlayShaGain { .. }
            | StmtIr::EffectRevealTargetHandGainShanArmorOrDrawTwo { .. }
            | StmtIr::EffectGiveAllHand { .. }
            | StmtIr::EffectGainCardIfDiscard { .. }
            | StmtIr::EffectChooseDrawOrDiscardTargetCardRecordSha { .. }
            | StmtIr::EffectRevealHandToPlayer { .. }
            | StmtIr::EffectChooseOptionalDiscardTargetCard { .. }
            | StmtIr::EffectRevealHandChooseDiscardColor { .. }
            | StmtIr::EffectChooseDiscardTargetCardsRecoverSameCategory { .. }
            | StmtIr::EffectChooseScskuijiDiscard { .. }
            | StmtIr::EffectChoosePskuijiDiscard { .. }
            | StmtIr::EffectChooseCompareSelectedHandCardSecondFailureSeatDamage { .. }
            | StmtIr::EffectDiscardTargetCardGainMount { .. }
            | StmtIr::EffectMakeCurrentShaTargetUnrespondable { .. }
            | StmtIr::EffectJudgeMakeCurrentShaTargetUnrespondableOnRed { .. }
            | StmtIr::EffectResolveRetieji { .. }
            | StmtIr::EffectResolveNewReluoyi { .. }
            | StmtIr::EffectResolveRemoshi { .. }
            | StmtIr::EffectResolveXinyaoming { .. }
            | StmtIr::EffectResolveRehuaiyi { .. }
            | StmtIr::EffectResolveReanguo { .. }
            | StmtIr::EffectChooseTargetDiscardDistanceHandOrIncreaseDamage { .. }
            | StmtIr::EffectChooseDrawViewTwoGainOrMoveEquipment { .. }
            | StmtIr::EffectResolveBeizhu { .. }
            | StmtIr::EffectResolveMbshanjia { .. }
            | StmtIr::EffectResolveDaoji { .. }
            | StmtIr::EffectResolveTwDaoji { .. }
            | StmtIr::EffectGiveAllHandThenChooseUseOneOrDamage { .. }
            | StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable { .. }
            | StmtIr::EffectChooseLoseHpUseVirtualShaByBasicVariants { .. }
            | StmtIr::EffectResolveTwfupan { .. }
            | StmtIr::EffectSuppressNonLockedSkillsUntilTurnEnd { .. }
            | StmtIr::EffectChooseRecoverTarget { .. }
            | StmtIr::EffectResolveJiaohua { .. }
            | StmtIr::EffectResolveLuanqun { .. }
            | StmtIr::EffectChooseGiveTwoDifferentColorsOrRecover { .. }
            | StmtIr::EffectResolveMbzhuguo { .. }
            | StmtIr::EffectResolveMbanxian { .. }
            | StmtIr::EffectResolveMbjinzu { .. }
            | StmtIr::EffectApplyMbjinzuNextSha { .. }
            | StmtIr::EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage { .. }
            | StmtIr::EffectChooseCurrentShaTargetDiscardRedirectToOwner { .. }
            | StmtIr::EffectChooseTargetGainBottomCard { .. }
            | StmtIr::EffectJudgeBlackGainRepeat { .. }
            | StmtIr::EffectUseVirtualJuedouSecondOnFirst { .. }
            | StmtIr::EffectRecoverAs { .. }
            | StmtIr::EffectResolveDuoduan { .. }
            | StmtIr::EffectResolveQiangyong { .. }
            | StmtIr::EffectResolveDcqiaolve { .. }
            | StmtIr::EffectResolveTwxiongjun { .. }
            | StmtIr::EffectResolveYixiang { .. }
            | StmtIr::EffectResolveXinfuQiai { .. }
            | StmtIr::EffectResolveOlwenren { .. }
            | StmtIr::EffectResolveSpolaige { .. }
            | StmtIr::EffectResolveOlzhijie { .. }
            | StmtIr::EffectResolveOlsuyi { .. }
            | StmtIr::EffectResolveOlqiaozhi { .. }
            | StmtIr::EffectResolveDczhee { .. }
            | StmtIr::EffectResolveDchaoxian { .. }
            | StmtIr::EffectResolveZhongjie { .. }
            | StmtIr::EffectResolveStarcuxiaWeiming { .. }
            | StmtIr::EffectResolveSanshou { .. }
            | StmtIr::EffectResolveNhlundao { .. }
            | StmtIr::EffectResolveXiangshu { .. }
            | StmtIr::EffectResolveDingce { .. }
            | StmtIr::EffectResolveMeiyong { .. }
            | StmtIr::EffectResolveMbzhenann { .. }
            | StmtIr::EffectResolveFriendbiwei { .. }
            | StmtIr::EffectResolveChulao { .. }
            | StmtIr::EffectResolveDcanxu { .. }
            | StmtIr::EffectResolveGaoyuan { .. }
            | StmtIr::EffectChooseDiscardRedirectCurrentSha { .. }
            | StmtIr::EffectJudgeSetSkillMarkToRank { .. }
            | StmtIr::EffectChooseDiscardOneFromEachLinked { .. }
            | StmtIr::EffectDisableSkillByDefUntilTurnEnd { .. }
            | StmtIr::EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner { .. }
            | StmtIr::EffectChooseTargetLoseHpDiscardHandShaIfBasic { .. }
            | StmtIr::EffectChooseDiscardTargetCard { .. }
            | StmtIr::EffectJudgePreventDamageLoseMaxHpOnNonHeart { .. }
            | StmtIr::EffectChooseGainTopCardsBySuit { .. }
            | StmtIr::EffectChooseOtherDrawByHandCount { .. }
            | StmtIr::EffectChooseDiscardOneThenGainTargetEquipment { .. }
            | StmtIr::EffectChooseTargetDraw { .. }
            | StmtIr::EffectDrawByPlayPhaseCardCategoryCount { .. }
            | StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRed { .. }
            | StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank { .. }
            | StmtIr::EffectQueueOpeningExtraPlayPhase { .. } => {
                self.emit_effect_set_draw_phase_from_bottom(s)
            }
            StmtIr::EffectConvertCurrentShaToFireLoseHpIfDamage { .. }
            | StmtIr::EffectChooseAddCurrentShaTarget { .. }
            | StmtIr::EffectChooseAddCurrentShaTargetWithinDistance { .. }
            | StmtIr::EffectRefundCurrentShaUse { .. }
            | StmtIr::EffectDamageThenDrawSelectedTargets { .. }
            | StmtIr::EffectResolveYoudi { .. }
            | StmtIr::EffectResolveKuangfu { .. }
            | StmtIr::EffectResolveShanjia { .. }
            | StmtIr::EffectResolveNewYajiao { .. }
            | StmtIr::EffectResolveDingpin { .. }
            | StmtIr::EffectChooseRepoluTargetsDraw { .. }
            | StmtIr::EffectResolveXinfuGuolun { .. }
            | StmtIr::EffectFinishCurrentPhaseAbortResolution { .. }
            | StmtIr::EffectGainTopCardToSkillPile { .. }
            | StmtIr::EffectChooseDiscardSkillPileCardAddMarkedSkill { .. }
            | StmtIr::EffectGainOneHandFromSelectedTargets { .. }
            | StmtIr::EffectChooseStoreHandShaInSkillPile { .. }
            | StmtIr::EffectChooseSpendSkillPileForDyingJiu { .. }
            | StmtIr::EffectChooseCompareSelectedHandCard { .. }
            | StmtIr::EffectChooseAddSkillBoundCardTarget { .. }
            | StmtIr::EffectChooseDiscardMatchingEquipColorAddSkill { .. }
            | StmtIr::EffectUseVirtualShaFromCardColorLockSourceIfNoSha { .. }
            | StmtIr::EffectChooseCurrentCardUseEnhancements { .. }
            | StmtIr::EffectMakeCurrentCardUnrespondable { .. }
            | StmtIr::EffectChooseDrawOrDiscardAddSkill { .. }
            | StmtIr::EffectChooseDiscardedBlackBasicAsBingliang { .. }
            | StmtIr::EffectChooseDiscardJudgeUseVirtualShaDrawIfNoDamage { .. }
            | StmtIr::EffectDiscardSelectedRecordUniqueCategoryTarget { .. }
            | StmtIr::EffectChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded { .. }
            | StmtIr::EffectChooseRevealOtherHandCardRememberCategory { .. }
            | StmtIr::EffectDrawThenGiveExactRecordPhaseTarget { .. }
            | StmtIr::EffectClearPhaseGiftTarget { .. }
            | StmtIr::EffectChooseGainResolvedShaOrShanThenOptionalUseSha { .. }
            | StmtIr::EffectSkipCurrentPhase { .. }
            | StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamage { .. }
            | StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamageDraw { .. }
            | StmtIr::EffectRevealUniformHandDrawSelfChooseOthers { .. }
            | StmtIr::EffectChooseCardUserOrAddTarget { .. }
            | StmtIr::EffectChooseDiscardEquipmentReduceCurrentDamage { .. }
            | StmtIr::EffectChooseDiscardEquipmentPreventCurrentDamage { .. }
            | StmtIr::EffectChooseMoveTargetEquipmentOrDraw { .. }
            | StmtIr::EffectGiveAllEquipmentChooseRecoverOrDamage { .. }
            | StmtIr::EffectChooseTargetDiscardWeaponRangeHandOrIncreaseDamage { .. }
            | StmtIr::EffectChooseHandCountDifferentTargetDrawOrRandomDiscard { .. }
            | StmtIr::EffectIncreaseCurrentDrawCount { .. }
            | StmtIr::EffectChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { .. }
            | StmtIr::EffectDrawThenChooseGainOneFromDamageTargetAndNeighbors { .. }
            | StmtIr::EffectChoosePreventDamageOrLoseMaxHpIncreaseDamage { .. }
            | StmtIr::EffectRandomizeCurrentHandCompareOpponent { .. }
            | StmtIr::EffectGainHighestRankShaFromEventCards { .. }
            | StmtIr::EffectGainHandCompareResultCard { .. }
            | StmtIr::EffectChooseAddOrRemoveCurrentCardTarget { .. }
            | StmtIr::EffectChooseHandCompare { .. }
            | StmtIr::EffectChooseHandCompareGrantTargetOnWin { .. }
            | StmtIr::EffectResolveLieren { .. }
            | StmtIr::EffectResolveTuntian { .. }
            | StmtIr::EffectResolvePotdimeng { .. }
            | StmtIr::EffectResolvePothaoshi { .. }
            | StmtIr::EffectResolveSpmiaolve { .. }
            | StmtIr::EffectResolveSpyingjia { .. }
            | StmtIr::EffectResolveZaoxian { .. }
            | StmtIr::EffectResolveTiaoxin { .. }
            | StmtIr::EffectResolveZhiji { .. }
            | StmtIr::EffectResolveZaiqi { .. }
            | StmtIr::EffectResolveTwqiushou { .. }
            | StmtIr::EffectResolveZhengrong { .. }
            | StmtIr::EffectResolveHongju { .. }
            | StmtIr::EffectResolveQingjian { .. }
            | StmtIr::EffectResolveReleiji { .. }
            | StmtIr::EffectResolveHuangtian { .. }
            | StmtIr::EffectResolveHaoshi { .. }
            | StmtIr::EffectResolveDimeng { .. }
            | StmtIr::EffectResolveQiaobian { .. }
            | StmtIr::EffectExcludeSavedShaTarget { .. }
            | StmtIr::EffectResolveXiangle { .. }
            | StmtIr::EffectResolveFangquanSkip { .. }
            | StmtIr::EffectResolveFangquanGrant { .. }
            | StmtIr::EffectResolveRuoyu { .. }
            | StmtIr::EffectResolveZuilun { .. }
            | StmtIr::EffectResolveFuyin { .. }
            | StmtIr::EffectBanUseAgainst { .. }
            | StmtIr::EffectResolveJuzhan { .. }
            | StmtIr::EffectResolveQizhi { .. }
            | StmtIr::EffectResolveJinqu { .. }
            | StmtIr::EffectResolveJianxiang { .. }
            | StmtIr::EffectResolveShenshi { .. }
            | StmtIr::EffectResolveMingren { .. }
            | StmtIr::EffectResolveZhenliang { .. }
            | StmtIr::EffectResolveKuizhu { .. }
            | StmtIr::EffectResolveZhizheng { .. }
            | StmtIr::EffectResolveLijun { .. }
            | StmtIr::EffectResolveLiangyin { .. }
            | StmtIr::EffectResolveKongsheng { .. }
            | StmtIr::EffectResolvePotzhouyu { .. }
            | StmtIr::EffectResolvePotyuji { .. }
            | StmtIr::EffectResolveRehengjiang { .. }
            | StmtIr::EffectAddFirstMissingSkill { .. }
            | StmtIr::EffectResolveDccunsi { .. }
            | StmtIr::EffectRecordCurrentShaAllTargets { .. }
            | StmtIr::EffectChooseGiveOrDiscardHand { .. }
            | StmtIr::EffectChooseDiscardHandEquipThenRecoverTarget { .. }
            | StmtIr::EffectResolvePotlvyi { .. }
            | StmtIr::EffectResolvePotcaoshuang { .. }
            | StmtIr::EffectResolvePottuntian { .. }
            | StmtIr::EffectResolvePotjixi { .. }
            | StmtIr::EffectResolvePotceduan { .. }
            | StmtIr::EffectResolvePottaishici { .. }
            | StmtIr::EffectResolveMbzhonghui { .. }
            | StmtIr::EffectResolvePotjiejie { .. }
            | StmtIr::EffectResolvePotqingshi { .. }
            | StmtIr::EffectResolvePotweiyan { .. }
            | StmtIr::EffectChooseOtherDamage { .. }
            | StmtIr::EffectChooseOtherDiscardTargetCard { .. }
            | StmtIr::EffectGainDiscardCardByKind { .. }
            | StmtIr::EffectRevealUniformHandChooseDamage { .. }
            | StmtIr::EffectChooseGainDiscardCardByKind { .. }
            | StmtIr::EffectDrawForLifetimeCardActionDivisors { .. }
            | StmtIr::EffectChooseAddMissingSkill { .. }
            | StmtIr::EffectChooseTargetsDraw { .. }
            | StmtIr::EffectChooseTargetsDamage { .. }
            | StmtIr::EffectChooseOptionalTargetsDraw { .. } => {
                self.emit_effect_convert_current_sha_to_fire_lose_hp_if_damage(s)
            }
            StmtIr::EffectRevealTargetHandShaOrDiscardBind { .. }
            | StmtIr::EffectDrawRememberedHandCards { .. }
            | StmtIr::EffectDiscardRememberedHandCards { .. }
            | StmtIr::EffectGainDrawCardsForLifetimeCardActionDivisors { .. }
            | StmtIr::EffectLoseHpGainDiscardCardForOpeningExtraPlay { .. }
            | StmtIr::EffectChooseGainOwnLostDiscardCardThisTurn { .. }
            | StmtIr::EffectChooseMoveFieldCardTo { .. }
            | StmtIr::EffectDrawThenOptionallyDistributeHandCards { .. }
            | StmtIr::EffectGiveSelectedThenContinueDistinctRecoverAtTwo { .. }
            | StmtIr::EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { .. }
            | StmtIr::EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver { .. }
            | StmtIr::EffectChooseGainCardFromEachOtherThenTurnOver { .. }
            | StmtIr::EffectResolveHiddenSkillPile { .. }
            | StmtIr::EffectSetNonLockedSkillSuppression { .. }
            | StmtIr::EffectResolveDiliYuheng { .. }
            | StmtIr::EffectDiscardSelectedDrawAllHandBonus { .. }
            | StmtIr::EffectChooseOtherYinghun { .. }
            | StmtIr::EffectChooseOtherBizheng { .. }
            | StmtIr::EffectChooseOtherXiashu { .. }
            | StmtIr::EffectResolveGuanchaoRank { .. }
            | StmtIr::EffectResolveGuanwei { .. }
            | StmtIr::EffectResolveLanjiang { .. }
            | StmtIr::EffectResolveRebingyi { .. }
            | StmtIr::EffectResolveRebotu { .. }
            | StmtIr::EffectResolveOldimeng { .. }
            | StmtIr::EffectResolveRejieyin { .. }
            | StmtIr::EffectResolveReanxu { .. }
            | StmtIr::EffectResolveDrltJueyan { .. }
            | StmtIr::EffectResolveRejizhi { .. }
            | StmtIr::EffectResolveJiexun { .. }
            | StmtIr::EffectResolveWenguaSetup { .. }
            | StmtIr::EffectResolveWengua { .. }
            | StmtIr::EffectResolveXinanguo { .. }
            | StmtIr::EffectResolveGanglieThree { .. }
            | StmtIr::EffectResolveZhongyi { .. }
            | StmtIr::EffectDiscardEquipmentSlot { .. }
            | StmtIr::EffectResolveOldAnxu { .. }
            | StmtIr::EffectStoreSelectedInSkillPile { .. }
            | StmtIr::EffectEndMatchForCampIfSkillPileAtLeast { .. }
            | StmtIr::EffectResolveOldQinqing { .. }
            | StmtIr::EffectResolveOldMingjian { .. }
            | StmtIr::EffectResolveOldXuanfeng { .. }
            | StmtIr::EffectResolveJianrou { .. }
            | StmtIr::EffectResolveOldHuisheng { .. }
            | StmtIr::EffectResolveOldJiefan { .. }
            | StmtIr::EffectResolveChegu { .. }
            | StmtIr::EffectResolveJunkguixin { .. }
            | StmtIr::EffectResolveYaoChenwei { .. }
            | StmtIr::EffectAddSkillUntilTurnEnd { .. }
            | StmtIr::EffectResolveMassDamageDiscardTurnOver { .. }
            | StmtIr::EffectResolveMassDamageDiscardHandEquip { .. }
            | StmtIr::EffectRecordActiveTarget { .. }
            | StmtIr::EffectChooseDiscardWeaponOrLoseHp { .. }
            | StmtIr::EffectResolveMarkParityControl { .. }
            | StmtIr::EffectResolveSelectedEquipmentDiscardThenChooseDamage { .. }
            | StmtIr::EffectReplaceDrawWithDistinctSuitSelection { .. }
            | StmtIr::EffectInspectTargetHandChooseHeartDiscardOrTop { .. }
            | StmtIr::EffectResolveYeyan { .. }
            | StmtIr::EffectChooseAllLoseHpOrRecover { .. }
            | StmtIr::EffectResolveQixing { .. }
            | StmtIr::EffectResolveQixingWeather { .. }
            | StmtIr::EffectResolveNzryLongnu { .. }
            | StmtIr::EffectChooseLinkOther { .. }
            | StmtIr::EffectResolveDrltPoxi { .. }
            | StmtIr::EffectResolveDrltJieying { .. }
            | StmtIr::EffectResolveDrltDuorui { .. }
            | StmtIr::EffectResolveDrltZhiti { .. }
            | StmtIr::EffectResolveShenSimayi { .. }
            | StmtIr::EffectResolveBeige { .. }
            | StmtIr::EffectResolveBeigeConfigured { .. }
            | StmtIr::EffectResolveZhuiyi { .. }
            | StmtIr::EffectResolveDecadezhenjun { .. }
            | StmtIr::EffectResolveSpXiaoguo { .. }
            | StmtIr::EffectChooseGainCardFromEachOtherReplaceDraw { .. }
            | StmtIr::EffectGiveSelectedThenRecoverSelf { .. }
            | StmtIr::EffectChooseHandCompareBetween { .. }
            | StmtIr::EffectGuanxingTop { .. }
            | StmtIr::EffectGuanxingTopByFaluMarks { .. }
            | StmtIr::EffectChooseOtherGiveThenRecover { .. }
            | StmtIr::EffectGiveSelectedThenRecoverTarget { .. }
            | StmtIr::EffectResolveDingpan { .. }
            | StmtIr::EffectResolveTanlu { .. }
            | StmtIr::EffectJudgeSameColorGainRepeat { .. }
            | StmtIr::EffectChooseCurrentTrickTargetsToNullify { .. }
            | StmtIr::EffectJudgeBlackGainRepeatProtectHandLimit { .. }
            | StmtIr::EffectChooseGainAllResolvedShaShanThenColorRewards { .. }
            | StmtIr::EffectGainEventCards { .. }
            | StmtIr::EffectChooseTargetDrawToMaxHpCap { .. }
            | StmtIr::EffectChooseOtherDrawOwnerLostHpTurnOver { .. }
            | StmtIr::EffectDiscardOneHandOrEquip { .. }
            | StmtIr::EffectDiscardAllHandAndEquip { .. }
            | StmtIr::EffectRemoveAllSkills { .. }
            | StmtIr::EffectJudgeGainEquipmentByColor { .. }
            | StmtIr::EffectJudgeRecoverOnSuit { .. }
            | StmtIr::EffectJudgeRecoverOtherOnSuit { .. }
            | StmtIr::EffectForceOthersUseShaOnNearestOrLoseHp { .. }
            | StmtIr::EffectJudgeDamageOtherOnSuit { .. }
            | StmtIr::EffectRevealDiscardSuitGainOrUse { .. }
            | StmtIr::EffectChooseDiscardOrFireDamageSkipPhases { .. }
            | StmtIr::EffectDrawSelected { .. }
            | StmtIr::EffectDrawEachBySeat { .. }
            | StmtIr::EffectDrawThenGiveOne { .. }
            | StmtIr::EffectDrawThenTakeOne { .. }
            | StmtIr::EffectTakeOneCard { .. }
            | StmtIr::EffectTakeOneScopedCard { .. }
            | StmtIr::EffectRevealHand { .. }
            | StmtIr::EffectDrawLinkedPlayers { .. }
            | StmtIr::EffectTakeOneEventTargetHand { .. }
            | StmtIr::EffectTakeOneHandCard { .. }
            | StmtIr::EffectChooseOtherHandStoreSkillPile { .. }
            | StmtIr::EffectGainAllSkillPileCards { .. }
            | StmtIr::EffectGainOtherSkillPileCards { .. }
            | StmtIr::EffectChooseDiscardOneHeOrLoseHp { .. }
            | StmtIr::EffectChooseEndExtraDrawOrPlayByUsedSuits { .. }
            | StmtIr::EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice { .. }
            | StmtIr::EffectChooseSourceConfirmOwnerLoseHp { .. }
            | StmtIr::EffectLoseHpDoubleCurrentDamageThenReplace { .. }
            | StmtIr::EffectTurnOver { .. }
            | StmtIr::EffectRestoreCharacterState { .. }
            | StmtIr::EffectDiscardAllHandEquipJudge { .. }
            | StmtIr::EffectDamageThenDiscardAllJudge { .. }
            | StmtIr::EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors { .. }
            | StmtIr::EffectChooseGainTargetCardAlternateZoneThenTargetDraw { .. }
            | StmtIr::EffectChooseDiscardTargetCardResolveCurrentSha { .. }
            | StmtIr::EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus { .. }
            | StmtIr::EffectResolveDcqiaomeng { .. }
            | StmtIr::EffectUseVirtualCardOnSelectedTargets { .. }
            | StmtIr::EffectDealDamageBanActiveIfDying { .. }
            | StmtIr::EffectSpjungongDiscardOrLoseThenSha { .. } => {
                self.emit_effect_reveal_target_hand_sha_or_discard_bind(s)
            }
            StmtIr::EffectBanActiveUntilTurnEnd { .. }
            | StmtIr::EffectSpwanweiActive { .. }
            | StmtIr::EffectSpwanweiDying { .. }
            | StmtIr::EffectResolveSpxizhan { .. }
            | StmtIr::EffectResolveSpsongshu { .. }
            | StmtIr::EffectResolveSpcunsi { .. }
            | StmtIr::EffectResolveYuli { .. }
            | StmtIr::EffectResolveTingwei { .. }
            | StmtIr::EffectResolveXinmingshi { .. }
            | StmtIr::EffectResolveXunyi { .. }
            | StmtIr::EffectResolveBoming { .. }
            | StmtIr::EffectResolveEjian { .. }
            | StmtIr::EffectResolveYiyong { .. }
            | StmtIr::EffectResolveShanxie { .. }
            | StmtIr::EffectResolveSheyi { .. }
            | StmtIr::EffectResolveTianyin { .. }
            | StmtIr::EffectResolveDbzhuifeng { .. }
            | StmtIr::EffectResolveDbquedi { .. }
            | StmtIr::EffectResolveSpyinju { .. }
            | StmtIr::EffectResolveSpchijie { .. }
            | StmtIr::EffectResolveSpshangyi { .. }
            | StmtIr::EffectResolveSpjianyi { .. }
            | StmtIr::EffectResolveSpyajun { .. }
            | StmtIr::EffectResolveSpzundi { .. }
            | StmtIr::EffectResolveXinyangjie { .. }
            | StmtIr::EffectResolveQingyu { .. }
            | StmtIr::EffectResolveZhengsu { .. }
            | StmtIr::EffectResolveHoufeng { .. }
            | StmtIr::EffectResolveMbdiancai { .. }
            | StmtIr::EffectResolveRechuhai { .. }
            | StmtIr::EffectResolveXiongshu { .. }
            | StmtIr::EffectResolveJianhui { .. }
            | StmtIr::EffectResolveSpqiai { .. }
            | StmtIr::EffectResolveSpshanxi { .. }
            | StmtIr::EffectResolveSpshanxiSuoming { .. }
            | StmtIr::EffectResolveTongduo { .. }
            | StmtIr::EffectResolveYuanqing { .. }
            | StmtIr::EffectResolveShuchen { .. }
            | StmtIr::EffectResolveDuanbi { .. }
            | StmtIr::EffectResolveZaoli { .. }
            | StmtIr::EffectResolveGuying { .. }
            | StmtIr::EffectResolveFyjianyu { .. }
            | StmtIr::EffectResolveLiubing { .. }
            | StmtIr::EffectResolveHeji { .. }
            | StmtIr::EffectResolveTwjiaohua { .. }
            | StmtIr::EffectResolveTwkujian { .. }
            | StmtIr::EffectResolveTwruilian { .. }
            | StmtIr::EffectEquipWeaponFromDrawPile { .. }
            | StmtIr::EffectResolveTwjianwei { .. }
            | StmtIr::EffectResolveTwtanfeng { .. }
            | StmtIr::EffectResolveTwzhibian { .. }
            | StmtIr::EffectResolveTwyuyan { .. }
            | StmtIr::EffectResolveTwguoyi { .. }
            | StmtIr::EffectResolveTwchuhai { .. }
            | StmtIr::EffectResolveTwyizhu { .. }
            | StmtIr::EffectResolveTwluanchou { .. }
            | StmtIr::EffectResolveTwxiawei { .. }
            | StmtIr::EffectResolveTwbianfuren { .. }
            | StmtIr::EffectResolveTwfeiyi { .. }
            | StmtIr::EffectResolveTwchenzhen { .. }
            | StmtIr::EffectResolveTwqianju { .. }
            | StmtIr::EffectResolveTwqingxi { .. }
            | StmtIr::EffectResolveTwzaoli { .. }
            | StmtIr::EffectResolveTwzhengjian { .. }
            | StmtIr::EffectResolveTwtianyu { .. }
            | StmtIr::EffectResolveTwwujing { .. }
            | StmtIr::EffectResolveTwshanghe { .. }
            | StmtIr::EffectResolveTwwangchang { .. }
            | StmtIr::EffectResolveTwcaozhao { .. }
            | StmtIr::EffectResolveTwguohuai { .. }
            | StmtIr::EffectResolveTwchengpu { .. }
            | StmtIr::EffectResolveTwcaoang { .. }
            | StmtIr::EffectResolveTwniujin { .. }
            | StmtIr::EffectResolveTwmoukui { .. }
            | StmtIr::EffectResolveTwfuman { .. }
            | StmtIr::EffectResolveCuijin { .. }
            | StmtIr::EffectResolveTwqingkou { .. }
            | StmtIr::EffectResolveTwchenxun { .. }
            | StmtIr::EffectResolveTwfenxian { .. }
            | StmtIr::EffectSetSkillContentFlag { .. }
            | StmtIr::EffectChooseTargetReclaimEquipmentDrawIfRangeReduced { .. }
            | StmtIr::EffectChooseUseVirtualCard { .. }
            | StmtIr::EffectChooseUseVirtualNanman { .. }
            | StmtIr::EffectResolveQiluan { .. }
            | StmtIr::EffectDiscardAllOwnJudge { .. }
            | StmtIr::EffectStoreEventCardInSkillPile { .. }
            | StmtIr::EffectResolveOllinjie { .. }
            | StmtIr::EffectChooseTargetGainMaxHp { .. }
            | StmtIr::EffectDiscardBasicDrawEqual { .. }
            | StmtIr::EffectStartDebateAllOthers { .. }
            | StmtIr::EffectStartDebateTargets { .. }
            | StmtIr::EffectPinDebateOpinions { .. }
            | StmtIr::EffectAlignDebateOpinions { .. }
            | StmtIr::EffectGainShadowCards { .. }
            | StmtIr::EffectChangeGroup { .. }
            | StmtIr::EffectDistributeShadowCards { .. }
            | StmtIr::EffectRegisterTurnEndDebate { .. }
            | StmtIr::EffectResolveTurnEndDebates { .. }
            | StmtIr::EffectChooseGroup { .. }
            | StmtIr::EffectChooseGainOtherSkill { .. }
            | StmtIr::EffectLoseGainedSkill { .. }
            | StmtIr::EffectResolvePotheyun { .. }
            | StmtIr::EffectPlaceXumou { .. }
            | StmtIr::EffectPlaceDeckTopXumou { .. }
            | StmtIr::EffectChoosePlaceHandXumou { .. }
            | StmtIr::EffectDiscardOneOwnXumou { .. }
            | StmtIr::EffectPlaceDeckTopExpansion { .. }
            | StmtIr::EffectCityConsume { .. }
            | StmtIr::EffectQueueExtraTurn { .. }
            | StmtIr::EffectChooseOthersInsertExtraTurn { .. }
            | StmtIr::EffectQueueExtraPhases { .. }
            | StmtIr::EffectResolveTwjieqiu { .. }
            | StmtIr::EffectSkipRemainingPhases { .. }
            | StmtIr::EffectChooseOthersAddMark { .. }
            | StmtIr::EffectResolveJsrgchaozheng { .. }
            | StmtIr::EffectResolveJsrgsheju { .. }
            | StmtIr::EffectChooseOtherAddMark { .. } => {
                self.emit_effect_ban_active_until_turn_end(s)
            }
            StmtIr::EffectGainWholeSkillPile { .. }
            | StmtIr::EffectPlaceHandCardsToDeckTop { .. }
            | StmtIr::EffectRearrangePileToDeckTop { .. }
            | StmtIr::EffectResolveQianlong { .. }
            | StmtIr::EffectResolveJsrgzuozhan { .. }
            | StmtIr::EffectResolveJsrgzuozhanGain { .. }
            | StmtIr::EffectResolveTwmouzhi { .. }
            | StmtIr::EffectResolveTwdaigui { .. }
            | StmtIr::EffectResolveTwmitu { .. }
            | StmtIr::EffectResolveQiantun { .. }
            | StmtIr::EffectResolveYangming { .. }
            | StmtIr::EffectResolveCanyun { .. }
            | StmtIr::EffectResolveHengwu { .. }
            | StmtIr::EffectResolveBiaozhao3 { .. }
            | StmtIr::EffectResolveChuanyun { .. }
            | StmtIr::EffectResolveBiaozhao2 { .. }
            | StmtIr::EffectResolveDcniji { .. }
            | StmtIr::EffectResolveDcmoukui { .. }
            | StmtIr::EffectResolveZhongyan { .. }
            | StmtIr::EffectResolveChumei { .. }
            | StmtIr::EffectResolveShuaiyan { .. }
            | StmtIr::EffectResolveMbbeiming { .. }
            | StmtIr::EffectResolveReliangyin { .. }
            | StmtIr::EffectResolveZhengjian { .. }
            | StmtIr::EffectResolveQinmian { .. }
            | StmtIr::EffectResolveJsrgdaoren { .. }
            | StmtIr::EffectResolveDuwang { .. }
            | StmtIr::EffectResolveQuanjia { .. }
            | StmtIr::EffectResolvePxqixian { .. }
            | StmtIr::EffectResolveDcbingji { .. }
            | StmtIr::EffectChooseHandCardsToSkillPile { .. }
            | StmtIr::EffectChooseTargetHandCardToPile { .. }
            | StmtIr::EffectChooseOneFromSkillPileDiscard { .. }
            | StmtIr::EffectPlaceTopDeckToExpansion { .. }
            | StmtIr::EffectDiscardSkillPile { .. }
            | StmtIr::EffectGainFromSkillPile { .. }
            | StmtIr::EffectPlaceHandCardToExpansion { .. }
            | StmtIr::EffectExtraResolveCurrentTrick { .. }
            | StmtIr::EffectChooseClanNotMaxHandLimitGain { .. }
            | StmtIr::EffectGainTargetHandCard { .. }
            | StmtIr::EffectIncreaseCurrentShaBaseDamage { .. }
            | StmtIr::EffectSkipNextPlayPhase { .. }
            | StmtIr::EffectSkipNextJudgePhase { .. }
            | StmtIr::EffectSkipJudgeAndDrawPhases { .. }
            | StmtIr::EffectChooseDiscardDamageEndpointCardIncreaseIfSuit { .. }
            | StmtIr::EffectChooseOtherDrawOrRecover { .. }
            | StmtIr::EffectChooseOtherRecover { .. }
            | StmtIr::EffectEnableEquipSlot { .. }
            | StmtIr::EffectChooseDiscardHandEquipThenRemoveSelfSkill { .. }
            | StmtIr::EffectResetLinkedSelectedTargets { .. }
            | StmtIr::EffectDrawSelectedTargets { .. }
            | StmtIr::EffectRecoverIfMinCurrentHp { .. }
            | StmtIr::EffectIncreaseCurrentCardBaseDamage { .. }
            | StmtIr::EffectModifyCurrentDamage { .. }
            | StmtIr::EffectLoseHpThenDamageOtherLoseMaxHpIfDying { .. }
            | StmtIr::EffectRecoverSelectedTargets { .. }
            | StmtIr::EffectRecordSelectedPlayPhaseCategory { .. }
            | StmtIr::EffectTakeTargetHandThenRepeatUntilMax { .. }
            | StmtIr::EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark { .. }
            | StmtIr::EffectDrawThenChooseGiveOrDiscardHand { .. }
            | StmtIr::EffectGainTargetEquipment { .. }
            | StmtIr::EffectDiscardAllTargetEquipment { .. }
            | StmtIr::EffectIncreaseCurrentShaTargetDamage { .. }
            | StmtIr::EffectChooseTargetDrawThenOwnerIfBelowMaxHp { .. }
            | StmtIr::EffectChooseBoundSourceAttackRangeDamage { .. }
            | StmtIr::EffectChooseTargetDrawMaxHpCapThenDiscardExcess { .. }
            | StmtIr::EffectChooseTargetDrawMaxHpCapThenAddSkill { .. }
            | StmtIr::EffectResolveTianxiang { .. }
            | StmtIr::EffectResolveRetianxiang { .. }
            | StmtIr::EffectResolveLianzhou { .. }
            | StmtIr::EffectResolveStddechong { .. }
            | StmtIr::EffectDiscardHandEquipCount { .. }
            | StmtIr::EffectResolveJieliangGain { .. }
            | StmtIr::EffectResolveStdjujian { .. }
            | StmtIr::EffectResolveLulve { .. }
            | StmtIr::EffectResolveDctingxian { .. }
            | StmtIr::EffectResolveHfjieying { .. }
            | StmtIr::EffectResolveDcnuchen { .. }
            | StmtIr::EffectResolveDcdanji { .. }
            | StmtIr::EffectResolveDczhiheng { .. }
            | StmtIr::EffectResolveStdgaobian { .. }
            | StmtIr::EffectResolveSpjiedao { .. }
            | StmtIr::EffectResolveSibian { .. }
            | StmtIr::EffectRevealChooseCategoryDistributeSelfThresholdHpLoss { .. }
            | StmtIr::EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp { .. }
            | StmtIr::EffectResolveRefuhai { .. }
            | StmtIr::EffectChooseGainTargetHandCeilHalfDifference { .. }
            | StmtIr::EffectResolveGzjinfa { .. }
            | StmtIr::EffectResolveWnQifeng { .. }
            | StmtIr::EffectResolveDcjue { .. }
            | StmtIr::EffectResolveRuanji { .. }
            | StmtIr::EffectResolveZhengbi { .. }
            | StmtIr::EffectResolveFengying { .. }
            | StmtIr::EffectResolveQiaojian { .. }
            | StmtIr::EffectResolveXicha { .. }
            | StmtIr::EffectResolveHuzun { .. }
            | StmtIr::EffectResolveZhugemengxue { .. }
            | StmtIr::EffectResolveBailingyun { .. }
            | StmtIr::EffectResolveDcshaowei { .. }
            | StmtIr::EffectResolveDcdichou { .. }
            | StmtIr::EffectResolveDcdufeng { .. }
            | StmtIr::EffectChooseDiscardHandRange { .. }
            | StmtIr::EffectResolveDcctjiuxian { .. }
            | StmtIr::EffectResolveBianyue { .. }
            | StmtIr::EffectResolveZhupeilan { .. }
            | StmtIr::EffectResolveDcSbZhangxiu { .. }
            | StmtIr::EffectResolveDcSbZhouyu { .. }
            | StmtIr::EffectResolveDcSbLusu { .. }
            | StmtIr::EffectResolveZhangjian { .. }
            | StmtIr::EffectResolveZhugeruoxue { .. }
            | StmtIr::EffectResolveCaoyi { .. }
            | StmtIr::EffectResolveMalingli { .. }
            | StmtIr::EffectResolveNsshuaiyan { .. }
            | StmtIr::EffectResolveWdbHanlong { .. }
            | StmtIr::EffectResolveWdbLiufu { .. }
            | StmtIr::EffectResolveWdbLiuyan { .. }
            | StmtIr::EffectResolveWdbLiuzan { .. }
            | StmtIr::EffectResolveWdbTianyu { .. }
            | StmtIr::EffectResolveWdbXizhenxihong { .. } => {
                self.emit_effect_gain_whole_skill_pile(s)
            }
            StmtIr::EffectResolveWdbYangyi { .. }
            | StmtIr::EffectResolveWdbZaozhiRenjun { .. }
            | StmtIr::EffectResolveDddXianxi { .. }
            | StmtIr::EffectResolveDddXiaheng { .. }
            | StmtIr::EffectResolveDddLiuchong { .. }
            | StmtIr::EffectResolveDddDuanbing { .. }
            | StmtIr::EffectResolveDddWuzhi { .. }
            | StmtIr::EffectResolveDddXujing { .. }
            | StmtIr::EffectResolveDddCaomao { .. }
            | StmtIr::EffectResolveDddXinxianying { .. }
            | StmtIr::EffectResolveDddXianglang { .. }
            | StmtIr::EffectResolveDddYujin { .. }
            | StmtIr::EffectResolveDddLiuye { .. }
            | StmtIr::EffectResolveDddBaosanniang { .. }
            | StmtIr::EffectResolveDddZhenji { .. }
            | StmtIr::EffectResolveDddZhaoang { .. }
            | StmtIr::EffectResolveDddLiuba { .. }
            | StmtIr::EffectResolveDddJianshuo { .. }
            | StmtIr::EffectResolveDddGuanning { .. }
            | StmtIr::EffectResolveDddCaoshuang { .. }
            | StmtIr::EffectResolveDddXuelingyun { .. }
            | StmtIr::EffectResolveDddLiuhong { .. }
            | StmtIr::EffectResolveDddXiahouxuan { .. }
            | StmtIr::EffectResolveDddZhangkai { .. }
            | StmtIr::EffectResolveDddLiangxi { .. }
            | StmtIr::EffectGuozhanUseZhulianBihe { .. }
            | StmtIr::EffectResolveShoulijian { .. }
            | StmtIr::EffectCancelIncomingCardUse { .. }
            | StmtIr::EffectChooseOtherGainOneOfTwoSkills { .. }
            | StmtIr::EffectGwyewuRandomDiscardChain { .. }
            | StmtIr::EffectGiveSelectedThenTargetMayUseDrawOwner { .. }
            | StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne { .. }
            | StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                ..
            }
            | StmtIr::EffectResolveMobiledanshou { .. }
            | StmtIr::EffectResolveJsrgchiying { .. }
            | StmtIr::EffectResolveMbqingdao { .. }
            | StmtIr::EffectResolveMbxiugeng { .. }
            | StmtIr::EffectResolveMbchenshe { .. }
            | StmtIr::EffectResolveMbquanchong { .. }
            | StmtIr::EffectResolveMbrenxing { .. }
            | StmtIr::EffectResolveMbchizhang { .. }
            | StmtIr::EffectResolveMbduanyang { .. }
            | StmtIr::EffectResolveMbrunwei { .. }
            | StmtIr::EffectResolveMbshuanghuai { .. }
            | StmtIr::EffectResolveMbxiongtu { .. }
            | StmtIr::EffectResolveMbganggeng { .. }
            | StmtIr::EffectResolveMbsijian { .. }
            | StmtIr::EffectResolveMbjianji { .. }
            | StmtIr::EffectResolveMbyuanmo { .. }
            | StmtIr::EffectResolveMbfeijing { .. }
            | StmtIr::EffectResolveMbxiaoge { .. }
            | StmtIr::EffectResolveMbxuye { .. }
            | StmtIr::EffectResolveMbkuangxiang { .. }
            | StmtIr::EffectResolvePotchengpu { .. }
            | StmtIr::EffectResolvePotjugu { .. }
            | StmtIr::EffectResolvePotsifeng { .. }
            | StmtIr::EffectResolvePotchenqun { .. }
            | StmtIr::EffectResolvePotwanglie { .. }
            | StmtIr::EffectResolvePothongyi { .. }
            | StmtIr::EffectResolvePotguansha { .. }
            | StmtIr::EffectResolvePotjiyu { .. }
            | StmtIr::EffectResolveMbzhuji { .. }
            | StmtIr::EffectResolvePotlibing { .. }
            | StmtIr::EffectResolvePotpoxi { .. }
            | StmtIr::EffectResolvePotfuan { .. }
            | StmtIr::EffectResolvePotyinxian { .. }
            | StmtIr::EffectResolvePothuanjie { .. }
            | StmtIr::EffectResolvePotnigu { .. }
            | StmtIr::EffectResolvePotlulian { .. }
            | StmtIr::EffectSkipNextDiscardPhase { .. }
            | StmtIr::EffectResolveSpfengyin { .. }
            | StmtIr::EffectResolveXinfuLianpian { .. }
            | StmtIr::EffectResolveGongjian { .. }
            | StmtIr::EffectResolveXinfuQinguo { .. }
            | StmtIr::EffectResolveDcmanzhi { .. }
            | StmtIr::EffectResolveXinxingluan { .. }
            | StmtIr::EffectResolveDckuizhen { .. }
            | StmtIr::EffectResolveDckuizhenResolve { .. }
            | StmtIr::EffectResolveNsdingzhou { .. }
            | StmtIr::EffectResolveSxrmlanjiao { .. }
            | StmtIr::EffectResolveRekuanshi { .. }
            | StmtIr::EffectResolveRekuanshiResolve { .. }
            | StmtIr::EffectResolveDcmoyu { .. }
            | StmtIr::EffectResolveXizhen { .. }
            | StmtIr::EffectResolveXuxie { .. }
            | StmtIr::EffectResolveXuxieAdd { .. }
            | StmtIr::EffectResolveDcchiying { .. }
            | StmtIr::EffectResolveXinkuangfu { .. }
            | StmtIr::EffectResolveDcxiangshu { .. }
            | StmtIr::EffectResolveDcxiangshuLottery { .. }
            | StmtIr::EffectResolveShuangxiong { .. }
            | StmtIr::EffectResolveZhijian { .. }
            | StmtIr::EffectResolveGuzheng { .. }
            | StmtIr::EffectResolveZhiba { .. }
            | StmtIr::EffectResolveDrltCongjian { .. }
            | StmtIr::EffectResolveNzryFeijun { .. }
            | StmtIr::EffectResolveDrltYongsi { .. }
            | StmtIr::EffectResolveDrltWeidi { .. }
            | StmtIr::EffectResolveDcyingtu { .. }
            | StmtIr::EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp { .. }
            | StmtIr::EffectGainEquipmentFromDrawPile { .. }
            | StmtIr::EffectTargetChooseSuitGainOwnerHandDamageIfMismatch { .. }
            | StmtIr::EffectJudgeSourceDiscardTwoHandOrDamage { .. }
            | StmtIr::EffectGiveSelectedRecoverAfterPhaseTotalTwo { .. }
            | StmtIr::EffectChooseOtherHandTargetsGainOneReplaceDraw { .. }
            | StmtIr::EffectChooseOtherHandTargetsGainOneReduceDraw { .. }
            | StmtIr::EffectDistributeTopTwoCards { .. }
            | StmtIr::EffectGiveAllHandRecoverToOne { .. }
            | StmtIr::EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { .. }
            | StmtIr::EffectChooseGiveOneHandEquipOrRecover { .. }
            | StmtIr::EffectRemoveSkillByDef { .. }
            | StmtIr::EffectChooseDiscardHandAddSkillByColor { .. }
            | StmtIr::EffectRevealTopGainNonBasicOrSwapBasic { .. }
            | StmtIr::EffectRevealTopGainIfColor { .. }
            | StmtIr::EffectPreventCurrentDamage { .. }
            | StmtIr::EffectJudgeCancelCurrentShaTargetOnRed { .. }
            | StmtIr::EffectChooseDrawOrRecover { .. }
            | StmtIr::EffectChooseUseVirtualSha { .. }
            | StmtIr::EffectTakeHand { .. }
            | StmtIr::EffectLoseHp { .. }
            | StmtIr::EffectAddSkill { .. }
            | StmtIr::EffectAddBoundSkill { .. }
            | StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive { .. }
            | StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { .. }
            | StmtIr::EffectChooseDiscardHandEquipThenRecover { .. }
            | StmtIr::EffectRemoveSkill { .. }
            | StmtIr::EffectRefreshSkill { .. } => self.emit_effect_resolve_wdb_yangyi(s),
            StmtIr::EffectTagEventCards { .. }
            | StmtIr::EffectUntagEventCards { .. }
            | StmtIr::EffectStorageSet { .. }
            | StmtIr::EffectDrawAllEventTargets { .. }
            | StmtIr::EffectClanshenjunReveal { .. }
            | StmtIr::EffectClanshenjunUse { .. }
            | StmtIr::EffectClanyuzhiReveal { .. }
            | StmtIr::EffectClanyuzhiRoundEnd { .. }
            | StmtIr::EffectQieyiPhaseStart { .. }
            | StmtIr::EffectQieyiUseAfter { .. }
            | StmtIr::EffectJianzhiPhaseEnd { .. }
            | StmtIr::EffectMoveCard { .. }
            | StmtIr::EffectSetTurnState { .. }
            | StmtIr::EffectUseCardOn { .. }
            | StmtIr::EffectUseVirtualCardOn { .. }
            | StmtIr::EffectEnterDying { .. }
            | StmtIr::EffectRevealCard { .. }
            | StmtIr::EffectUseCard { .. }
            | StmtIr::EffectRemoveCardsByKind { .. }
            | StmtIr::EffectGainCardByKind { .. }
            | StmtIr::EffectKillPlayer { .. }
            | StmtIr::Let { .. }
            | StmtIr::If { .. }
            | StmtIr::Loop { .. } => self.emit_effect_tag_event_cards(s),
        }?;
        Ok(())
    }

    /// Patch jump offsets directly in assembled bytes using actual byte positions.
    /// Walks the code to build an op-index → byte-offset map, then fixes all backpatches.
    fn patch_jumps_in_bytes(&self, code: &mut [u8]) {
        if self.backpatches.is_empty() {
            return;
        }
        use rule_vm::bytecode::op;
        // Build op-index → byte-offset by walking the actual assembled code.
        let mut op_byte: Vec<usize> = Vec::with_capacity(self.ops.len() + 1);
        let mut i = 0usize;
        while i < code.len() {
            op_byte.push(i);
            i += match code[i] {
                c if c == op::CONST_I32 || c == op::CONST_U32 => 5,
                c if c == op::JUMP || c == op::JUMP_IF_FALSE => 3,
                c if c == op::HOST_EFFECT_EXT => 3, // opcode(1) + u16 tag(2)
                c if c == op::HOST_EFFECT
                    || c == op::HOST_QUERY
                    || c == op::HOST_CHOICE
                    || c == op::LOAD
                    || c == op::STORE =>
                {
                    2
                }
                _ => 1,
            };
        }
        op_byte.push(i); // one past end

        for bp in &self.backpatches {
            match bp {
                BackpatchEntry::If {
                    jif_op_idx,
                    jmp_op_idx,
                    else_op_idx,
                    end_op_idx,
                } => {
                    // JIF: offset = else_byte - (jif_byte + 3)
                    let jif_b = op_byte[*jif_op_idx];
                    let else_b = op_byte[*else_op_idx];
                    let off = (else_b as i32 - (jif_b as i32 + 3)) as i16;
                    code[jif_b + 1] = (off & 0xFF) as u8;
                    code[jif_b + 2] = ((off >> 8) & 0xFF) as u8;
                    // JMP: offset = end_byte - (jmp_byte + 3)
                    let jmp_b = op_byte[*jmp_op_idx];
                    let end_b = op_byte[*end_op_idx];
                    let off2 = (end_b as i32 - (jmp_b as i32 + 3)) as i16;
                    code[jmp_b + 1] = (off2 & 0xFF) as u8;
                    code[jmp_b + 2] = ((off2 >> 8) & 0xFF) as u8;
                }
                BackpatchEntry::Loop {
                    jif_op_idx,
                    jmp_back_op_idx,
                    loop_start_op_idx,
                    end_op_idx,
                } => {
                    let jif_b = op_byte[*jif_op_idx];
                    let end_b = op_byte[*end_op_idx];
                    let off = (end_b as i32 - (jif_b as i32 + 3)) as i16;
                    code[jif_b + 1] = (off & 0xFF) as u8;
                    code[jif_b + 2] = ((off >> 8) & 0xFF) as u8;
                    let jmp_b = op_byte[*jmp_back_op_idx];
                    let start_b = op_byte[*loop_start_op_idx];
                    let off2 = (start_b as i32 - (jmp_b as i32 + 3)) as i16;
                    code[jmp_b + 1] = (off2 & 0xFF) as u8;
                    code[jmp_b + 2] = ((off2 >> 8) & 0xFF) as u8;
                }
            }
        }
    }

    fn emit_expr(&mut self, e: &ExprIr) -> Result<(), CompileError> {
        match e {
            ExprIr::Int(n) => self.ops.push(Op::ConstI32(*n)),
            ExprIr::U32(n) => self.ops.push(Op::ConstU32(*n)),
            ExprIr::SelfPlayer => self.ops.push(Op::HostSelf),
            ExprIr::SelfSkill => self.ops.push(Op::HostSkill),
            ExprIr::EventPlayer => self.ops.push(Op::HostEventPlayer),
            ExprIr::EventSource => self.ops.push(Op::HostEventSource),
            ExprIr::EventCard => self.ops.push(Op::HostEventCard),
            ExprIr::EventAmount => self.ops.push(Op::HostEventAmount),
            // Card-target count: Phase 3 — dedicated opcode reads host.event_targets().len().
            ExprIr::EventTargetCount => self.ops.push(Op::HostEventTargetCount),
            // Event-card count: Phase 3 — reads host.event_cards().len().
            ExprIr::EventCardCount => self.ops.push(Op::HostEventCardCount),
            // Phase-end turn history is event-disjoint from damage amount.
            ExprIr::TurnDamagedPlayerCount => self.ops.push(Op::HostEventAmount),
            ExprIr::Local(name) => {
                let slot = self
                    .locals
                    .iter()
                    .position(|l| l == name)
                    .ok_or_else(|| CompileError::Codegen(format!("local {name}")))?;
                self.ops.push(Op::Load(slot as u8));
            }
            ExprIr::BinOp { op, left, right } => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                match op.as_str() {
                    "+" => self.ops.push(Op::Add),
                    "-" => self.ops.push(Op::Sub),
                    "*" => self.ops.push(Op::Mul),
                    "==" => self.ops.push(Op::Eq),
                    "<" => self.ops.push(Op::Lt),
                    ">" => self.ops.push(Op::Gt),
                    "&&" => self.ops.push(Op::And),
                    _ => {
                        return Err(CompileError::Codegen(format!("bad op {op}")));
                    }
                }
            }
            ExprIr::Query { name, args } => {
                for a in args {
                    self.emit_expr(a)?;
                }
                let tag = match name.as_str() {
                    "distance" => query_tag::DISTANCE,
                    "attack_range" => query_tag::ATTACK_RANGE,
                    "hand_limit" => query_tag::HAND_LIMIT,
                    "hp" => query_tag::HP,
                    "hand_count" => query_tag::HAND_COUNT,
                    "empty_equip_slots" => query_tag::EMPTY_EQUIP_SLOTS,
                    "max_hp" => query_tag::MAX_HP,
                    "relation" => query_tag::RELATION,
                    "card_usable" => query_tag::CARD_USABLE,
                    "target_legal" => query_tag::TARGET_LEGAL,
                    "card_attribute" => query_tag::CARD_ATTRIBUTE,
                    "skill_uses_remaining" => query_tag::SKILL_USES_REMAINING,
                    "zone" => query_tag::ZONE,
                    "equipped_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_681);
                        query_tag::EQUIPPED_COUNT
                    }
                    "card_zone" => query_tag::CARD_ZONE,
                    "mark_count" => query_tag::MARK_COUNT,
                    "skill_pile_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_565);
                        query_tag::SKILL_PILE_COUNT
                    }
                    "alive_count" => query_tag::ALIVE_COUNT,
                    "card_action_count" => query_tag::CARD_ACTION_COUNT,
                    "cards_drawn_this_turn" => query_tag::CARDS_DRAWN_THIS_TURN,
                    "damage_dealt_this_turn" => query_tag::DAMAGE_DEALT_THIS_TURN,
                    "has_skill" => query_tag::HAS_SKILL,
                    "card_name_length" => query_tag::CARD_NAME_LENGTH,
                    "in_play_phase" => query_tag::IN_PLAY_PHASE,
                    "card_has_tag" => query_tag::CARD_HAS_TAG,
                    "storage_get" => query_tag::STORAGE_GET,
                    "storage_has" => query_tag::STORAGE_HAS,
                    "cards_used_this_round" => query_tag::CARDS_USED_THIS_ROUND,
                    "alive_group_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_6);
                        query_tag::ALIVE_GROUP_COUNT
                    }
                    "unique_max_current_hp" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_7);
                        query_tag::UNIQUE_MAX_CURRENT_HP
                    }
                    "unique_max_current_hp_at" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_7);
                        query_tag::UNIQUE_MAX_CURRENT_HP_AT
                    }
                    "lifetime_damage_dealt" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_8);
                        query_tag::LIFETIME_DAMAGE_DEALT
                    }
                    "public_card_kind_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_11);
                        query_tag::PUBLIC_CARD_KIND_COUNT
                    }
                    "field_equipment_kind_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_102);
                        query_tag::FIELD_EQUIPMENT_KIND_COUNT
                    }
                    "alive_wounded_or_guiming_wu_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_190);
                        query_tag::ALIVE_WOUNDED_OR_GUIMING_WU_COUNT
                    }
                    "hand_equip_suit_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_229);
                        query_tag::HAND_EQUIP_SUIT_COUNT
                    }
                    "hand_suit_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
                        query_tag::HAND_SUIT_COUNT
                    }
                    "hand_color_count" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
                        query_tag::HAND_COLOR_COUNT
                    }
                    "skill_content_flag" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
                        query_tag::SKILL_CONTENT_FLAG
                    }
                    "current_phase_player" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_453);
                        query_tag::CURRENT_PHASE_PLAYER
                    }
                    "round_number_capped" => {
                        self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_75);
                        query_tag::ROUND_NUMBER_CAPPED
                    }
                    _ => return Err(CompileError::Codegen(format!("query {name}"))),
                };
                self.ops.push(Op::HostQuery(tag));
            }
            ExprIr::Choose { options } => {
                for o in options {
                    self.emit_expr(o)?;
                }
                self.ops.push(Op::HostChoice(options.len() as u8));
            }
            ExprIr::ChooseRange { min, max } => {
                self.emit_expr(min)?;
                self.emit_expr(max)?;
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_92);
                self.ops.push(Op::HostChoiceRange);
            }
            ExprIr::Min { left, right } => {
                self.emit_expr(left)?;
                self.emit_expr(right)?;
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_451);
                self.ops.push(Op::Min);
            }
        }
        Ok(())
    }
}

fn required_caps(ops: &[Op]) -> u32 {
    let mut mask = 0;
    for op in ops {
        match op {
            Op::HostEffect(tag) => {
                mask |= caps::EMIT_EFFECTS;
                if matches!(
                    *tag,
                    effect_tag::CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE
                        | effect_tag::CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK
                        | effect_tag::DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND
                        | effect_tag::DRAW_THEN_OPTIONALLY_DISTRIBUTE_HAND_CARDS
                        | effect_tag::GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO
                        | effect_tag::CHOOSE_MAX_MARK_OTHER_JUDGE_KILL_UNLESS_TAO_OR_TAOYUAN
                        | effect_tag::CHOOSE_ZONE_PRIORITY_GAIN_RANDOM_CARD_FROM_EACH_OTHER_THEN_TURN_OVER
                        | effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER
                        | effect_tag::RESOLVE_DINGPAN
                        | effect_tag::RESOLVE_TANLU
                        | effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT
                        | effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY
                        | effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL
                        | effect_tag::RESOLVE_TIANXIANG
                        | effect_tag::RESOLVE_RETIANXIANG
                        | effect_tag::RESOLVE_LIANZHOU
                        | effect_tag::RESOLVE_STDDECHONG
                        | effect_tag::DISCARD_HAND_EQUIP_COUNT
                        | effect_tag::RESOLVE_JIELIANG_GAIN
                        | effect_tag::RESOLVE_STDJUJIAN
                        | effect_tag::RESOLVE_LULVE
                        | effect_tag::RESOLVE_DCTINGXIAN
                        | effect_tag::RESOLVE_HFJIEYING
                        | effect_tag::RESOLVE_DCNUCHEN
                        | effect_tag::RESOLVE_DCZHIHENG
                        | effect_tag::RESOLVE_STDGAOBIAN
                        | effect_tag::RESOLVE_SPJIEDAO
                        | effect_tag::RESOLVE_SIBIAN
                        | effect_tag::RESOLVE_SPFENGYIN
                        | effect_tag::RESOLVE_XINFU_LIANPIAN
                        | effect_tag::RESOLVE_GONGJIAN
                        | effect_tag::RESOLVE_XINFU_QINGUO
                        | effect_tag::RESOLVE_DCMANZHI
                        | effect_tag::RESOLVE_XINXINGLUAN
                        | effect_tag::RESOLVE_DCKUIZHEN
                        | effect_tag::RESOLVE_NSDINGZHOU
                        | effect_tag::RESOLVE_SXRMLANJIAO
                        | effect_tag::RESOLVE_REKUANSHI
                        | effect_tag::RESOLVE_DCMOYU
                        | effect_tag::RESOLVE_XIZHEN
                        | effect_tag::RESOLVE_XUXIE
                        | effect_tag::RESOLVE_DCCHIYING
                        | effect_tag::RESOLVE_JSRGCHIYING
                        | effect_tag::RESOLVE_MBQINGDAO
                        | effect_tag::RESOLVE_MBXIUGENG
                        | effect_tag::RESOLVE_MBCHENSHE
                        | effect_tag::RESOLVE_MBQUNCHONG
                        | effect_tag::RESOLVE_MBRENXING
                        | effect_tag::RESOLVE_MBCHIZHANG
                        | effect_tag::RESOLVE_MBDUANYANG
                        | effect_tag::RESOLVE_MBRUNWEI
                        | effect_tag::RESOLVE_MBSHUANGHUAI
                        | effect_tag::RESOLVE_MBXIONGTU
                        | effect_tag::RESOLVE_MBGANGGENG
                        | effect_tag::RESOLVE_MBSIJIAN
                        | effect_tag::RESOLVE_XINKUANGFU
                        | effect_tag::RESOLVE_DCXIANG_SHU
                        | effect_tag::RESOLVE_DCYINGTU
                ) {
                    mask |= caps::REQUEST_CHOICE;
                }
            }
            Op::HostQuery(_) => mask |= caps::RUN_QUERIES,
            Op::HostChoice(_) | Op::HostChoiceRange => mask |= caps::REQUEST_CHOICE,
            _ => {}
        }
    }
    mask
}

fn estimate_pc(ops: &[Op]) -> u32 {
    let mut n = 0u32;
    for op in ops {
        n += match op {
            Op::Nop
            | Op::Halt
            | Op::Dup
            | Op::Pop
            | Op::Add
            | Op::Sub
            | Op::Min
            | Op::Mul
            | Op::Eq
            | Op::Lt
            | Op::Gt
            | Op::And
            | Op::Not
            | Op::HostSelf
            | Op::HostEventPlayer
            | Op::HostEventSource
            | Op::HostEventCard
            | Op::HostSkill
            | Op::HostEventAmount
            | Op::HostEventTargetCount
            | Op::HostEventCardCount
            | Op::HostChoiceRange
            | Op::Trap => 1,
            Op::Load(_) | Op::Store(_) | Op::HostQuery(_) | Op::HostChoice(_) => 2,
            Op::HostEffect(tag) => {
                if u8::try_from(*tag).is_ok() {
                    2 // HOST_EFFECT(1) + tag(1)
                } else {
                    3 // HOST_EFFECT_EXT(1) + tag_le_u16(2)
                }
            }
            Op::Jump(_) | Op::JumpIfFalse(_) => 3,
            Op::ConstI32(_) | Op::ConstU32(_) => 5,
        };
    }
    n
}
