//! IR → bytecode + package indexes + source maps.

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
    ADDITIVE_RULE_VM_ABI_VERSION_646, ADDITIVE_RULE_VM_ABI_VERSION_647, ADDITIVE_RULE_VM_ABI_VERSION_648,
    ADDITIVE_RULE_VM_ABI_VERSION_649, ADDITIVE_RULE_VM_ABI_VERSION_650, ADDITIVE_RULE_VM_ABI_VERSION_651,
    ADDITIVE_RULE_VM_ABI_VERSION_652, ADDITIVE_RULE_VM_ABI_VERSION_653, ADDITIVE_RULE_VM_ABI_VERSION_654,
    ADDITIVE_RULE_VM_ABI_VERSION_655, ADDITIVE_RULE_VM_ABI_VERSION_656, ADDITIVE_RULE_VM_ABI_VERSION_657,
    ADDITIVE_RULE_VM_ABI_VERSION_658, ADDITIVE_RULE_VM_ABI_VERSION_659, ADDITIVE_RULE_VM_ABI_VERSION_660, ADDITIVE_RULE_VM_ABI_VERSION_661, ADDITIVE_RULE_VM_ABI_VERSION_662, ADDITIVE_RULE_VM_ABI_VERSION_663, ADDITIVE_RULE_VM_ABI_VERSION_664, ADDITIVE_RULE_VM_ABI_VERSION_665,
    ADDITIVE_RULE_VM_ABI_VERSION_666,
    ADDITIVE_RULE_VM_ABI_VERSION_667,
    ADDITIVE_RULE_VM_ABI_VERSION_668,
    ADDITIVE_RULE_VM_ABI_VERSION_669,
    ADDITIVE_RULE_VM_ABI_VERSION_670,
    ADDITIVE_RULE_VM_ABI_VERSION_671,
    ADDITIVE_RULE_VM_ABI_VERSION_672,
    ADDITIVE_RULE_VM_ABI_VERSION_673,
    ADDITIVE_RULE_VM_ABI_VERSION_674, ADDITIVE_RULE_VM_ABI_VERSION_675, ADDITIVE_RULE_VM_ABI_VERSION_676, ADDITIVE_RULE_VM_ABI_VERSION_677,
    ADDITIVE_RULE_VM_ABI_VERSION_678,
    ADDITIVE_RULE_VM_ABI_VERSION_679,
    ADDITIVE_RULE_VM_ABI_VERSION_680,
    ADDITIVE_RULE_VM_ABI_VERSION_681,
    ADDITIVE_RULE_VM_ABI_VERSION_682,
    ADDITIVE_RULE_VM_ABI_VERSION_683,
    ADDITIVE_RULE_VM_ABI_VERSION_684,
    ADDITIVE_RULE_VM_ABI_VERSION_685,
    ADDITIVE_RULE_VM_ABI_VERSION_686,
    ADDITIVE_RULE_VM_ABI_VERSION_688,
    ADDITIVE_RULE_VM_ABI_VERSION_689,
    ADDITIVE_RULE_VM_ABI_VERSION_690,
    ADDITIVE_RULE_VM_ABI_VERSION_691,
    ADDITIVE_RULE_VM_ABI_VERSION_692,
    ADDITIVE_RULE_VM_ABI_VERSION_693,
    ADDITIVE_RULE_VM_ABI_VERSION_694,
    ADDITIVE_RULE_VM_ABI_VERSION_695,
    ADDITIVE_RULE_VM_ABI_VERSION_696,
    ADDITIVE_RULE_VM_ABI_VERSION_697,
    ADDITIVE_RULE_VM_ABI_VERSION_698,
    ADDITIVE_RULE_VM_ABI_VERSION_699,
    ADDITIVE_RULE_VM_ABI_VERSION_700,
    ADDITIVE_RULE_VM_ABI_VERSION_701,
    ADDITIVE_RULE_VM_ABI_VERSION_702,
    ADDITIVE_RULE_VM_ABI_VERSION_703,
    ADDITIVE_RULE_VM_ABI_VERSION_704,
    ADDITIVE_RULE_VM_ABI_VERSION_705,
    ADDITIVE_RULE_VM_ABI_VERSION_706,
    ADDITIVE_RULE_VM_ABI_VERSION_707,
    ADDITIVE_RULE_VM_ABI_VERSION_708,
    ADDITIVE_RULE_VM_ABI_VERSION_709,
    ADDITIVE_RULE_VM_ABI_VERSION_710,
    ADDITIVE_RULE_VM_ABI_VERSION_711,
    ADDITIVE_RULE_VM_ABI_VERSION_712,
    ADDITIVE_RULE_VM_ABI_VERSION_713,
    ADDITIVE_RULE_VM_ABI_VERSION_714,
    ADDITIVE_RULE_VM_ABI_VERSION_715,
    ADDITIVE_RULE_VM_ABI_VERSION_716,
    ADDITIVE_RULE_VM_ABI_VERSION_717,
    ADDITIVE_RULE_VM_ABI_VERSION_718,
    ADDITIVE_RULE_VM_ABI_VERSION_719,
    ADDITIVE_RULE_VM_ABI_VERSION_720,
    ADDITIVE_RULE_VM_ABI_VERSION_721,
    ADDITIVE_RULE_VM_ABI_VERSION_722,
    ADDITIVE_RULE_VM_ABI_VERSION_723,
    ADDITIVE_RULE_VM_ABI_VERSION_724,
    ADDITIVE_RULE_VM_ABI_VERSION_725,
    ADDITIVE_RULE_VM_ABI_VERSION_726,
    ADDITIVE_RULE_VM_ABI_VERSION_727,
    ADDITIVE_RULE_VM_ABI_VERSION_730,
    ADDITIVE_RULE_VM_ABI_VERSION_732,
    ADDITIVE_RULE_VM_ABI_VERSION_733,
    ADDITIVE_RULE_VM_ABI_VERSION_734,
    ADDITIVE_RULE_VM_ABI_VERSION_735,
    ADDITIVE_RULE_VM_ABI_VERSION_736,
    ADDITIVE_RULE_VM_ABI_VERSION_737,
    ADDITIVE_RULE_VM_ABI_VERSION_738,
    ADDITIVE_RULE_VM_ABI_VERSION_739,
    ADDITIVE_RULE_VM_ABI_VERSION_740,
    ADDITIVE_RULE_VM_ABI_VERSION_741,
    ADDITIVE_RULE_VM_ABI_VERSION_742,
    ADDITIVE_RULE_VM_ABI_VERSION_744,
    ADDITIVE_RULE_VM_ABI_VERSION_745,
    ADDITIVE_RULE_VM_ABI_VERSION_746,
    ADDITIVE_RULE_VM_ABI_VERSION_747,
    ADDITIVE_RULE_VM_ABI_VERSION_748,
    ADDITIVE_RULE_VM_ABI_VERSION_749,
    ADDITIVE_RULE_VM_ABI_VERSION_750,
    ADDITIVE_RULE_VM_ABI_VERSION_751,
    ADDITIVE_RULE_VM_ABI_VERSION_752,
    ADDITIVE_RULE_VM_ABI_VERSION_753,
    ADDITIVE_RULE_VM_ABI_VERSION_687,
    ADDITIVE_RULE_VM_ABI_VERSION_65,
    ADDITIVE_RULE_VM_ABI_VERSION_66, ADDITIVE_RULE_VM_ABI_VERSION_67,
    ADDITIVE_RULE_VM_ABI_VERSION_68, ADDITIVE_RULE_VM_ABI_VERSION_69,
    ADDITIVE_RULE_VM_ABI_VERSION_7, ADDITIVE_RULE_VM_ABI_VERSION_70,
    ADDITIVE_RULE_VM_ABI_VERSION_74, ADDITIVE_RULE_VM_ABI_VERSION_75,
    ADDITIVE_RULE_VM_ABI_VERSION_76, ADDITIVE_RULE_VM_ABI_VERSION_77,
    ADDITIVE_RULE_VM_ABI_VERSION_78, ADDITIVE_RULE_VM_ABI_VERSION_79,
    ADDITIVE_RULE_VM_ABI_VERSION_8, ADDITIVE_RULE_VM_ABI_VERSION_80,
    ADDITIVE_RULE_VM_ABI_VERSION_82, ADDITIVE_RULE_VM_ABI_VERSION_83,
    ADDITIVE_RULE_VM_ABI_VERSION_84, ADDITIVE_RULE_VM_ABI_VERSION_85,
    ADDITIVE_RULE_VM_ABI_VERSION_88, ADDITIVE_RULE_VM_ABI_VERSION_9,
    ADDITIVE_RULE_VM_ABI_VERSION_90, ADDITIVE_RULE_VM_ABI_VERSION_92,
    ADDITIVE_RULE_VM_ABI_VERSION_93, ADDITIVE_RULE_VM_ABI_VERSION_94,
    ADDITIVE_RULE_VM_ABI_VERSION_95, ADDITIVE_RULE_VM_ABI_VERSION_96,
    ADDITIVE_RULE_VM_ABI_VERSION_97, ADDITIVE_RULE_VM_ABI_VERSION_98,
    ADDITIVE_RULE_VM_ABI_VERSION_99,
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

fn compile_module_ir_inner(
    ir: &RuleModuleIr,
) -> Result<
    (
        Vec<BytecodeModule>,
        Vec<TriggerEmit>,
        Vec<ModifierEmit>,
        Vec<ActiveEmit>,
        Vec<ViewAsEmit>,
        Vec<SourceMap>,
        u32,
    ),
    CompileError,
> {
    let mut modules = Vec::new();
    let mut triggers = Vec::new();
    let mut modifiers = Vec::new();
    let mut actives = Vec::new();
    let mut view_as = Vec::new();
    let mut maps = Vec::new();
    let mut required_abi = 3;
    let mut trigger_def_seq = 1u32;

    for skill in &ir.skills {
        for m in &skill.modifiers {
            if m.value_source.as_deref() == Some("owner_skill_state_u32") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            if m.value_source.as_deref() == Some("owner_skill_state_u32_negative") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_612);
            }
            if m.value_source.as_deref() == Some("owner_skill_pile_count") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_566);
            }
            if m.value_source.as_deref() == Some("queried_player_qun_owner_lord_alive") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_553);
            }
            if m.target == "judgment_rewrite_red_hand_equip_expansion" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_342);
            }
            if m.target == "card_number" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_301);
            }
            if m.target == "card_gain_protected" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_300);
            }
            if m.target == "sha_distance_unlimited_if_target_distance_lte_rank" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_223);
            }
            if m.value_source.as_deref().is_some_and(|source| {
                source.starts_with("owner_skill_play_phase_fire_count_below_")
            }) {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_220);
            }
            if m.value_source.as_deref() == Some("owner_skill_use_count") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_143);
            }
            if m.target == "link_loss_cancel"
                || m.value_source.as_deref()
                    == Some("queried_player_linked_first_living_skill_owner")
                || m.selector.as_deref() == Some("sha_thunder")
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
            }
            if matches!(
                m.target.as_str(),
                "global_draw_count" | "global_sha_use_limit"
            ) || m
                .value_source
                .as_deref()
                .is_some_and(|source| source.starts_with("queried_player_mark_positive_"))
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_215);
            }
            if m.target
                .starts_with("judgment_rewrite_hand_equip_cost_mark_")
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_218);
            }
            modifiers.push(ModifierEmit {
                skill_def_id: skill.def_id,
                target: m.target.clone(),
                selector: m.selector.clone(),
                mode: m.mode.clone(),
                value_source: m.value_source.clone(),
                value: m.value,
            });
            let _ = m as &ModifierIr;
        }
        for (trigger_index, tr) in skill.triggers.iter().enumerate() {
            if matches!(
                tr.filter.as_str(),
                "owner_alive_other_turn_discarded_recorded_potfuan_suit"
                    | "owner_source_sha_resolved_potfuan_dealt_damage"
            ) {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_525);
            }
            if tr.filter == "owner_discarded_sha_to_discard" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            if tr.filter == "owner_source_or_targeted_club_linked_exists_once_per_turn" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_344);
            }
            if tr.filter
                == "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn"
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_340);
            }
            if tr.filter == "owner_is_target_event_card_sha" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_339);
            }
            if tr.filter == "owner_alive_gained_two_this_turn_can_compare_other" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_325);
            }
            if tr.filter == "owner_is_target_draw_not_fixed_hp_one_or_hand_empty" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_300);
            }
            if tr.filter == "owner_is_target_draw_not_replaced" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_225);
            }
            if tr.filter == "owner_is_target_judge_nonempty" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_224);
            }
            if tr.filter == "owner_source_in_turn_target_unharmed_card_actions_ge_target_hp" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_749);
            }
            if tr.filter == "owner_is_target_source_hp_gt_owner_or_source_damaged_owner"
                || tr.filter == "owner_is_target_owner_hp_gt_source_or_source_damaged_owner"
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_750);
            }
            if tr.filter == "owner_is_target_sanshou_type_tracked" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_751);
            }
            if tr.filter == "owner_source_sha_target_hand_lte_owner_or_hp_gte_owner" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_223);
            }
            let duplicate_event = skill
                .triggers
                .iter()
                .filter(|candidate| candidate.event == tr.event)
                .count()
                > 1;
            let mod_name = if duplicate_event {
                format!("{}_{}_{}", skill.name, tr.event, trigger_index)
            } else {
                format!("{}_{}", skill.name, tr.event)
            };
            let (module, smap, entry, module_abi) = compile_body(&mod_name, &tr.body)?;
            required_abi = required_abi.max(module_abi);
            triggers.push(TriggerEmit {
                trigger_def_id: trigger_def_seq,
                skill_def_id: skill.def_id,
                event: tr.event.clone(),
                filter: tr.filter.clone(),
                optional: tr.optional,
                choice_actor: tr.choice_actor,
                module: mod_name,
                entry_pc: entry,
            });
            trigger_def_seq += 1;
            modules.push(module);
            maps.push(smap);
            let _ = tr as &TriggerIr;
        }
        for (i, ac) in skill.actives.iter().enumerate() {
            let mod_name = format!("{}_active_{}_{}", skill.name, ac.window, i);
            let (module, smap, entry, module_abi) = compile_body(&mod_name, &ac.body)?;
            required_abi = required_abi.max(module_abi);
            if ac.uses_per_play_phase.is_some()
                || ac
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.sum_effective_number_exact.is_some())
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_301);
            }
            if ac.uses_per_turn_source.as_deref() == Some("living_rebel_count") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_302);
            }
            if ac.hand_missing_gaintag.is_some() {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_745);
            }
            if ac.hand_limit_positive {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_143);
            }
            if ac.mark_at_least.is_some() {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
            }
            if ac.owner_missing_skill.is_some() {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_234);
            }
            if ac.owner_unlinked {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_611);
            }
            if ac
                .targets
                .as_ref()
                .is_some_and(|targets| targets.linked || targets.max_by_owner_mark.is_some())
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
            }
            if ac
                .targets
                .as_ref()
                .is_some_and(|targets| targets.has_gainable_equip)
                || ac
                    .selection
                    .as_ref()
                    .is_some_and(|selection| selection.filter.as_deref() == Some("card_nonbasic"))
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_327);
            }
            if ac
                .selection
                .as_ref()
                .is_some_and(|selection| selection.filter.as_deref() == Some("card_non_equip"))
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_497);
            }
            if ac
                .targets
                .as_ref()
                .is_some_and(|targets| targets.has_gainable_hand_equip)
            {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_355);
            }
            actives.push(ActiveEmit {
                skill_def_id: skill.def_id,
                window: ac.window.clone(),
                module: mod_name,
                entry_pc: entry,
                targets: ac.targets.clone(),
                uses_per_turn: ac.uses_per_turn,
                uses_per_turn_source: ac.uses_per_turn_source.clone(),
                uses_per_play_phase: ac.uses_per_play_phase,
                mark_at_least: ac.mark_at_least,
                owner_missing_skill: ac.owner_missing_skill,
                hand_gt_hp: ac.hand_gt_hp,
                hand_gt_max_hp: ac.hand_gt_max_hp,
                hand_limit_positive: ac.hand_limit_positive,
                equip_nonempty: ac.equip_nonempty,
                hand_nonempty: ac.hand_nonempty,
                face_up: ac.face_up,
                hand_missing_gaintag: ac.hand_missing_gaintag,
                owner_unlinked: ac.owner_unlinked,
                once_per_game: ac.once_per_game,
                selection: ac.selection.clone(),
            });
            modules.push(module);
            maps.push(smap);
            let _ = ac as &ActiveIr;
        }
        for v in &skill.view_as {
            if v.no_use_count || v.target_lacks_source_suit_unrespondable {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_521);
            }
            if v.cost.as_deref() == Some("hand_equip_mark") || v.as_kind == "trick" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_502);
            }
            if v.filter == "owner_single_hand" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_341);
            }
            if v.owner_hp_min_one_cards {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_236);
            }
            if v.owner_basic_uses_plus_one_cards {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_741);
            }
            if v.max_cards == u8::MAX {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_742);
            }
            if v.once_per_turn_per_branch {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_744);
            }
            if v.min_cards > 1 || v.as_kind == "wanjian" {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_221);
            }
            if v.filter == "card_trick" || v.nature.as_deref() == Some("thunder") {
                required_abi = required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
            }
            view_as.push(ViewAsEmit {
                skill_def_id: skill.def_id,
                as_kind: v.as_kind.clone(),
                window: v.window.clone(),
                filter: v.filter.clone(),
                source_zone: v.source_zone.clone(),
                uses_per_turn: v.uses_per_turn,
                ignore_distance: v.ignore_distance,
                red_unrespondable: v.red_unrespondable,
                black_no_use_count: v.black_no_use_count,
                no_use_count: v.no_use_count,
                target_lacks_source_suit_unrespondable: v.target_lacks_source_suit_unrespondable,
                refund_use_count_if_no_damage: v.refund_use_count_if_no_damage,
                min_cards: v.min_cards,
                max_cards: v.max_cards,
                owner_hp_min_one_cards: v.owner_hp_min_one_cards,
                same_suit: v.same_suit,
                owner_basic_uses_plus_one_cards: v.owner_basic_uses_plus_one_cards,
                once_per_turn_per_branch: v.once_per_turn_per_branch,
                nature: v.nature.clone(),
                multi_card_bonus: v.multi_card_bonus.clone(),
                branch: v.branch,
                once_per_round: v.once_per_round,
                next_branch: v.next_branch,
                cost: v.cost.clone(),
                mark_cost: v.mark_cost,
                post_draw: v.post_draw,
            });
        }
    }

    Ok((
        modules,
        triggers,
        modifiers,
        actives,
        view_as,
        maps,
        required_abi,
    ))
}

/// Compile IR to modules + indexes.
pub fn compile_module_ir(
    ir: &RuleModuleIr,
) -> Result<
    (
        Vec<BytecodeModule>,
        Vec<TriggerEmit>,
        Vec<ModifierEmit>,
        Vec<ActiveEmit>,
        Vec<ViewAsEmit>,
        Vec<SourceMap>,
    ),
    CompileError,
> {
    let (modules, triggers, modifiers, actives, view_as, maps, _) = compile_module_ir_inner(ir)?;
    Ok((modules, triggers, modifiers, actives, view_as, maps))
}

pub(crate) fn compile_module_ir_with_required_abi(
    ir: &RuleModuleIr,
) -> Result<
    (
        Vec<BytecodeModule>,
        Vec<TriggerEmit>,
        Vec<ModifierEmit>,
        Vec<ActiveEmit>,
        Vec<ViewAsEmit>,
        Vec<SourceMap>,
        u32,
    ),
    CompileError,
> {
    compile_module_ir_inner(ir)
}

fn compile_body(
    name: &str,
    body: &[StmtIr],
) -> Result<(BytecodeModule, SourceMap, u32, u32), CompileError> {
    let mut asm = Assembler::new();
    asm.emit_stmts(body)?;
    asm.ops.push(Op::Halt);

    let caps_mask = required_caps(&asm.ops);
    let mut code = assemble(&asm.ops);
    asm.patch_jumps_in_bytes(&mut code);
    let logic_hash = BytecodeModule::compute_logic_hash(name, &code, asm.local_count, caps_mask, 0);
    let module = BytecodeModule {
        name: name.into(),
        code,
        local_count: asm.local_count,
        max_stack: asm.max_stack.max(8),
        entry_pc: 0,
        logic_hash,
        caps_mask,
    };
    let required_abi = asm.required_abi;
    let smap = SourceMap {
        module: name.into(),
        entries: asm.smap,
    };
    Ok((module, smap, 0, required_abi))
}

/// Deferred jump fixup record. Offsets are computed from actual assembled bytes.
enum BackpatchEntry {
    If { jif_op_idx: usize, jmp_op_idx: usize, else_op_idx: usize, end_op_idx: usize },
    Loop { jif_op_idx: usize, jmp_back_op_idx: usize, loop_start_op_idx: usize, end_op_idx: usize },
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
            StmtIr::EffectSetDrawPhaseFromBottom { player, span } => {
                self.mark(*span, "set_draw_phase_bottom");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_DRAW_PHASE_FROM_BOTTOM));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectDrawFromBottom {
                player,
                count,
                span,
            } => {
                self.mark(*span, "draw_cards_from_bottom");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_CARDS_FROM_BOTTOM));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectResolveChenglve {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_chenglve");
                self.emit_expr(player)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHENGLVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectResolveShicai {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_shicai");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHICAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_589);
            }
            StmtIr::EffectResolveSbbiyue { player, span } => {
                self.mark(*span, "resolve_sbbiyue");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SBBIYUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_599);
            }
            StmtIr::EffectResolveSxrmqiewang {
                player,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_sxrmqiewang");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SXRMQIEWANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_596);
            }
            StmtIr::EffectResolveSxrmqianliu {
                player,
                source,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_sxrmqianliu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(stage)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SXRMQIANLIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_596);
            }
            StmtIr::EffectResolveGzbuqu {
                player,
                delta,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_gzbuqu");
                self.emit_expr(player)?;
                self.emit_expr(delta)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GZBUQU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_591);
            }
            StmtIr::EffectResolveGzbuquRecover {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_gzbuqu_recover");
                self.emit_expr(player)?;
                self.emit_expr(stage)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GZBUQU_RECOVER));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_591);
            }
            StmtIr::EffectResolveXiongluan {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_xiongluan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIONGLUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_590);
            }
            StmtIr::EffectResolveTwjuxiang {
                player,
                event_player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_twjuxiang");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJUXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_590);
            }
            StmtIr::EffectDraw {
                player,
                count,
                span,
            } => {
                self.mark(*span, "draw_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_CARDS));
            }
            StmtIr::EffectRecover {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "recover_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RECOVER_HP));
            }
            StmtIr::EffectNote { code, span } => {
                self.mark(*span, "emit_note");
                self.ops.push(Op::ConstI32(*code));
                self.ops.push(Op::HostEffect(effect_tag::EMIT_NOTE));
            }
            StmtIr::EffectDamage {
                source,
                target,
                amount,
                nature,
                span,
            } => {
                self.mark(*span, "deal_damage");
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                if *nature == 0 {
                    self.ops.push(Op::HostEffect(effect_tag::DEAL_DAMAGE));
                } else {
                    self.ops.push(Op::ConstI32(i32::from(*nature)));
                    self.ops
                        .push(Op::HostEffect(effect_tag::DEAL_NATURE_DAMAGE));
                }
            }
            StmtIr::EffectChangeMaxHp {
                player,
                delta,
                span,
            } => {
                self.mark(*span, "change_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(delta)?;
                self.ops.push(Op::HostEffect(effect_tag::CHANGE_MAX_HP));
            }
            StmtIr::EffectAddMark {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "add_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.ops.push(Op::ConstI32(*delta));
                self.ops.push(Op::HostEffect(effect_tag::ADD_MARK));
            }
            StmtIr::EffectAddMarkExpr {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "add_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.emit_expr(delta)?;
                self.ops.push(Op::HostEffect(effect_tag::ADD_MARK));
            }
            StmtIr::EffectRemoveMark {
                player,
                mark,
                amount,
                span,
            } => {
                self.mark(*span, "remove_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.ops.push(Op::ConstI32(*amount));
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_MARK));
            }
            StmtIr::EffectRemoveMarkExpr {
                player,
                mark,
                amount,
                span,
            } => {
                self.mark(*span, "remove_mark");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_MARK));
            }
            StmtIr::EffectDiscardHand {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*count));
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_HAND));
            }
            StmtIr::EffectDiscardHandExpr {
                player,
                count,
                span,
            } => {
                // Same HostEffect op as literal-count discard_hand; the count is
                // pushed by an ordinary expression, so no new VM capability is
                // required and the ABI stays unchanged.
                self.mark(*span, "discard_hand_expr");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_HAND));
            }
            StmtIr::EffectDiscardHandEquipExpr {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand_equip_expr");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_HAND_EQUIP));
            }
            StmtIr::EffectDiscardHandEquip {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_350);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*count));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_HAND_EQUIP));
            }
            StmtIr::EffectAdjustHandTo {
                player,
                count,
                span,
            } => {
                self.mark(*span, "adjust_hand_to");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_9);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::ADJUST_HAND_TO));
            }
            StmtIr::EffectDiscardSelected { player, span } => {
                self.mark(*span, "discard_selected");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_SELECTED));
            }
            StmtIr::EffectRecastSelected { player, span } => {
                self.mark(*span, "recast_selected");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_16);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RECAST_SELECTED));
            }
            StmtIr::EffectDisableAllEquipSlots { player, span } => {
                self.mark(*span, "disable_all_equip_slots");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_18);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISABLE_ALL_EQUIP_SLOTS));
            }
            StmtIr::EffectMoveEquipmentThenDraw { player, span } => {
                self.mark(*span, "move_equipment_then_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_30);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::MOVE_EQUIPMENT_THEN_DRAW));
            }
            StmtIr::EffectChooseUseVirtualShaOrDraw { player, span } => {
                self.mark(*span, "choose_use_virtual_sha_or_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_31);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_SHA_OR_DRAW));
            }
            StmtIr::EffectDiscardEquipmentPairWithArmor { player, span } => {
                self.mark(*span, "discard_equipment_pair_with_armor");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_32);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_EQUIPMENT_PAIR_WITH_ARMOR,
                ));
            }
            StmtIr::EffectDiscardTargetEquipmentGainMount {
                player,
                target,
                span,
            } => {
                self.mark(*span, "discard_target_equipment_gain_mount");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_33);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_TARGET_EQUIPMENT_GAIN_MOUNT,
                ));
            }
            StmtIr::EffectDiscardTargetEquipmentDrawWeaponRange {
                player,
                target,
                span,
            } => {
                self.mark(*span, "discard_target_equipment_draw_weapon_range");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_136);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_TARGET_EQUIPMENT_DRAW_WEAPON_RANGE,
                ));
            }
            StmtIr::EffectChooseTargetRevealHandDamageIfSha {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_target_reveal_hand_damage_if_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_137);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_REVEAL_HAND_DAMAGE_IF_SHA,
                ));
            }
            StmtIr::EffectChooseOtherGiveOneHandEquip { player, span } => {
                self.mark(*span, "choose_other_give_one_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_138);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_GIVE_ONE_HAND_EQUIP));
            }
            StmtIr::EffectChooseRevealHandCardBindCategoryTarget {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_reveal_hand_card_bind_category_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_140);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_REVEAL_HAND_CARD_BIND_CATEGORY_TARGET,
                ));
            }
            StmtIr::EffectRecoverToMaxThenOptionalExchangeSkill {
                player,
                removable_skill_def,
                span,
            } => {
                self.mark(*span, "recover_to_max_then_optional_exchange_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_142);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*removable_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::RECOVER_TO_MAX_THEN_OPTIONAL_EXCHANGE_SKILL,
                ));
            }
            StmtIr::EffectRevealHandDiscardKind { player, kind, span } => {
                self.mark(*span, "reveal_hand_discard_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_144);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_HAND_DISCARD_KIND));
            }
            StmtIr::EffectChooseMoveTargetEquipmentAndOrDraw {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_move_target_equipment_and_or_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_146);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_AND_OR_DRAW,
                ));
            }
            StmtIr::EffectChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_147);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_ATTACK_RANGE_CARDS_OR_INCREASE_DAMAGE_WEAPON_FORCES_BOTH,
                ));
            }
            StmtIr::EffectChooseDrawOrRecastAddSkill {
                player,
                draw_skill_def,
                recast_skill_def,
                span,
            } => {
                self.mark(*span, "choose_draw_or_recast_add_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_148);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*draw_skill_def as i32));
                self.ops.push(Op::ConstI32(*recast_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DRAW_OR_RECAST_ADD_SKILL));
            }
            StmtIr::EffectChooseGainShaForOpeningExtraPlay { player, span } => {
                self.mark(*span, "choose_gain_sha_for_opening_extra_play");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_149);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_SHA_FOR_OPENING_EXTRA_PLAY,
                ));
            }
            StmtIr::EffectFinishOpeningExtraPlayShaGain { player, span } => {
                self.mark(*span, "finish_opening_extra_play_sha_gain");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_149);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::FINISH_OPENING_EXTRA_PLAY_SHA_GAIN,
                ));
            }
            StmtIr::EffectRevealTargetHandGainShanArmorOrDrawTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "reveal_target_hand_gain_shan_armor_or_draw_two");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_150);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TARGET_HAND_GAIN_SHAN_ARMOR_OR_DRAW_TWO,
                ));
            }
            StmtIr::EffectGiveAllHand {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_151);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::GIVE_ALL_HAND));
            }
            StmtIr::EffectGainCardIfDiscard { player, card, span } => {
                self.mark(*span, "gain_card_if_discard");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_151);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_CARD_IF_DISCARD));
            }
            StmtIr::EffectChooseDrawOrDiscardTargetCardRecordSha {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "choose_draw_or_discard_target_card_record_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_152);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DRAW_OR_DISCARD_TARGET_CARD_RECORD_SHA,
                ));
            }
            StmtIr::EffectRevealHandToPlayer {
                player,
                observer,
                span,
            } => {
                self.mark(*span, "reveal_hand_to_player");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_153);
                self.emit_expr(player)?;
                self.emit_expr(observer)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_HAND_TO_PLAYER));
            }
            StmtIr::EffectChooseOptionalDiscardTargetCard {
                player,
                target,
                scope,
                span,
            } => {
                self.mark(*span, "choose_optional_discard_target_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_153);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OPTIONAL_DISCARD_TARGET_CARD,
                ));
            }
            StmtIr::EffectRevealHandChooseDiscardColor { player, span } => {
                self.mark(*span, "reveal_hand_choose_discard_color");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_154);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_HAND_CHOOSE_DISCARD_COLOR));
            }
            StmtIr::EffectChooseDiscardTargetCardsRecoverSameCategory {
                player,
                target,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_target_cards_recover_same_category");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_155);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_TARGET_CARDS_RECOVER_SAME_CATEGORY,
                ));
            }
            StmtIr::EffectChooseScskuijiDiscard {
                player,
                target,
                count,
                span,
            } => {
                self.mark(*span, "choose_scskuiji_discard");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_348);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_SCSKUIJI_DISCARD));
            }
            StmtIr::EffectChoosePskuijiDiscard {
                player,
                target,
                count,
                span,
            } => {
                self.mark(*span, "choose_pskuiji_discard");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_349);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_PSKKUIJI_DISCARD));
            }
            StmtIr::EffectChooseCompareSelectedHandCardSecondFailureSeatDamage {
                player,
                target,
                draw_count,
                failure_count,
                lose_hp,
                damage,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_compare_selected_hand_card_second_failure_seat_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_156);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(failure_count)?;
                self.emit_expr(lose_hp)?;
                self.emit_expr(damage)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD_SECOND_FAILURE_SEAT_DAMAGE,
                ));
            }
            StmtIr::EffectDiscardTargetCardGainMount {
                player,
                target,
                scope,
                span,
            } => {
                self.mark(*span, "discard_target_card_gain_mount");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_34);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_TARGET_CARD_GAIN_MOUNT));
            }
            StmtIr::EffectMakeCurrentShaTargetUnrespondable {
                source,
                card,
                target,
                span,
            } => {
                self.mark(*span, "make_current_sha_target_unrespondable");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_35);
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE,
                ));
            }
            StmtIr::EffectJudgeMakeCurrentShaTargetUnrespondableOnRed {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "judge_make_current_sha_target_unrespondable_on_red");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_177);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_MAKE_CURRENT_SHA_TARGET_UNRESPONDABLE_ON_RED,
                ));
            }
            StmtIr::EffectResolveRetieji {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "resolve_retieji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_316);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RETIEJI));
            }
            StmtIr::EffectResolveNewReluoyi {
                player,
                buff_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_new_reluoyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_317);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*buff_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NEW_RELUOYI));
            }
            StmtIr::EffectResolveRemoshi {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_remoshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_318);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REMOSHI));
            }
            StmtIr::EffectResolveXinyaoming { player, span } => {
                self.mark(*span, "resolve_xinyaoming");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_319);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINYAOMING));
            }
            StmtIr::EffectResolveRehuaiyi { player, span } => {
                self.mark(*span, "resolve_rehuaiyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_320);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REHUAIYI));
            }
            StmtIr::EffectResolveReanguo { player, span } => {
                self.mark(*span, "resolve_reanguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_321);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REANGUO));
            }
            StmtIr::EffectChooseTargetDiscardDistanceHandOrIncreaseDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_discard_distance_hand_or_increase_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_322);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_DISTANCE_HAND_OR_INCREASE_DAMAGE,
                ));
            }
            StmtIr::EffectChooseDrawViewTwoGainOrMoveEquipment {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_draw_view_two_gain_or_move_equipment");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_323);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DRAW_VIEW_TWO_GAIN_OR_MOVE_EQUIPMENT,
                ));
            }
            StmtIr::EffectResolveBeizhu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_beizhu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_324);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BEIZHU));
            }
            StmtIr::EffectResolveMbshanjia {
                player,
                lost_equipment_count,
                distance_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_mbshanjia");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_326);
                self.emit_expr(player)?;
                self.emit_expr(lost_equipment_count)?;
                self.ops.push(Op::ConstU32(*distance_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBSHANJIA));
            }
            StmtIr::EffectResolveDaoji {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_daoji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_327);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DAOJI));
            }
            StmtIr::EffectResolveTwDaoji {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_tw_daoji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_355);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TW_DAOJI));
            }
            StmtIr::EffectGiveAllHandThenChooseUseOneOrDamage {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand_then_choose_use_one_or_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_359);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_ALL_HAND_THEN_CHOOSE_USE_ONE_OR_DAMAGE,
                ));
            }
            StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrShaUnrespondable {
                player,
                card,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_category_then_target_give_matching_or_sha_unrespondable",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_360);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_SHA_UNRESPONDABLE,
                ));
            }
            StmtIr::EffectChooseLoseHpUseVirtualShaByBasicVariants { player, span } => {
                self.mark(*span, "choose_lose_hp_use_virtual_sha_by_basic_variants");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_361);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_LOSE_HP_USE_VIRTUAL_SHA_BY_BASIC_VARIANTS,
                ));
            }
            StmtIr::EffectResolveTwfupan {
                player,
                damage_amount,
                span,
            } => {
                self.mark(*span, "resolve_twfupan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_362);
                self.emit_expr(player)?;
                self.emit_expr(damage_amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFUPAN));
            }
            StmtIr::EffectSuppressNonLockedSkillsUntilTurnEnd { player, span } => {
                self.mark(*span, "suppress_non_locked_skills_until_turn_end");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_356);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::SUPPRESS_NON_LOCKED_SKILLS_UNTIL_TURN_END,
                ));
            }
            StmtIr::EffectChooseRecoverTarget {
                chooser,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "choose_recover_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_356);
                self.emit_expr(chooser)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_RECOVER_TARGET));
            }
            StmtIr::EffectResolveJiaohua { player, span } => {
                self.mark(*span, "resolve_jiaohua");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_328);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIAOHUA));
            }
            StmtIr::EffectResolveLuanqun {
                player,
                effect_skill_def,
                direct_hit_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_luanqun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_329);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*effect_skill_def));
                self.ops.push(Op::ConstU32(*direct_hit_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LUANQUN));
            }
            StmtIr::EffectChooseGiveTwoDifferentColorsOrRecover {
                owner,
                giver,
                target,
                span,
            } => {
                self.mark(*span, "choose_give_two_different_colors_or_recover");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_330);
                self.emit_expr(owner)?;
                self.emit_expr(giver)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GIVE_TWO_DIFFERENT_COLORS_OR_RECOVER,
                ));
            }
            StmtIr::EffectResolveMbzhuguo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mbzhuguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_331);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBZHUGUO));
            }
            StmtIr::EffectResolveMbanxian { player, span } => {
                self.mark(*span, "resolve_mbanxian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_332);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBANXIAN));
            }
            StmtIr::EffectResolveMbjinzu {
                player,
                target,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_mbjinzu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_333);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(0));
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstU32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBJINZU));
            }
            StmtIr::EffectApplyMbjinzuNextSha { player, card, span } => {
                self.mark(*span, "apply_mbjinzu_next_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_333);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(0));
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(0));
                self.ops.push(Op::ConstU32(2));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBJINZU));
            }
            StmtIr::EffectChooseDiscardBasicThenTargetDiscardEquipOrDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_basic_then_target_discard_equip_or_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_178);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_BASIC_THEN_TARGET_DISCARD_EQUIP_OR_DAMAGE,
                ));
            }
            StmtIr::EffectChooseCurrentShaTargetDiscardRedirectToOwner {
                owner,
                source,
                card,
                target,
                span,
            } => {
                self.mark(*span, "choose_current_sha_target_discard_redirect_to_owner");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_180);
                self.emit_expr(owner)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CURRENT_SHA_TARGET_DISCARD_REDIRECT_TO_OWNER,
                ));
            }
            StmtIr::EffectChooseTargetGainBottomCard { player, span } => {
                self.mark(*span, "choose_target_gain_bottom_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_181);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_GAIN_BOTTOM_CARD));
            }
            StmtIr::EffectJudgeBlackGainRepeat { player, span } => {
                self.mark(*span, "judge_black_gain_repeat");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_182);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_BLACK_GAIN_REPEAT));
            }
            StmtIr::EffectUseVirtualJuedouSecondOnFirst { span } => {
                self.mark(*span, "use_virtual_juedou_second_on_first");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_183);
                self.ops.push(Op::HostEffect(
                    effect_tag::USE_VIRTUAL_JUEDOU_SECOND_ON_FIRST,
                ));
            }
            StmtIr::EffectRecoverAs {
                source,
                player,
                amount,
                span,
            } => {
                self.mark(*span, "recover_hp_as");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_185);
                self.emit_expr(source)?;
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RECOVER_HP_AS));
            }
            StmtIr::EffectResolveDuoduan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_duoduan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_721);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUODUAN));
            }
            StmtIr::EffectResolveQiangyong {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_qiangyong");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_727);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_QIANGYONG));
            }
            StmtIr::EffectResolveDcqiaolve { player, target, span } => {
                self.mark(*span, "resolve_dcqiaolve");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_732);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCQIAOLVE));
            }
            StmtIr::EffectResolveTwxiongjun { player, span } => {
                self.mark(*span, "resolve_twxiongjun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_733);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TW_XIONGJUN));
            }
            StmtIr::EffectResolveYixiang { player, span } => {
                self.mark(*span, "resolve_yixiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_734);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YIXIANG));
            }
            StmtIr::EffectResolveXinfuQiai { player, span } => {
                self.mark(*span, "resolve_xinfu_qiai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_735);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XINFU_QIAI));
            }
            StmtIr::EffectResolveOlwenren { player, span } => {
                self.mark(*span, "resolve_olwenren");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_736);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLWENREN));
            }
            StmtIr::EffectResolveSpolaige { player, span } => {
                self.mark(*span, "resolve_spolaige");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_737);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPOLAIGE));
            }
            StmtIr::EffectResolveOlzhijie { player, span } => {
                self.mark(*span, "resolve_olzhijie");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_738);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLZHIJIE));
            }
            StmtIr::EffectResolveOlsuyi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_olsuyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_739);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLSUYI));
            }
            StmtIr::EffectResolveOlqiaozhi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_olqiaozhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_740);
                self.emit_expr(player)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLQIAOZHI));
            }
            StmtIr::EffectResolveDczhee {
                player,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_dczhee");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_746);
                self.emit_expr(player)?;
                self.emit_expr(mode)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCZHEE));
            }
            StmtIr::EffectResolveDchaoxian { player, span } => {
                self.mark(*span, "resolve_dchaoxian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_747);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCHAOXIAN));
            }
            StmtIr::EffectResolveZhongjie { player, span } => {
                self.mark(*span, "resolve_zhongjie");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_748);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGJIE));
            }
            StmtIr::EffectResolveStarcuxiaWeiming {
                player,
                source,
                variant,
                span,
            } => {
                self.mark(*span, "resolve_starcuxia_weiming");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_750);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(variant)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_STARCUXIA_WEIMING,
                ));
            }
            StmtIr::EffectResolveSanshou { player, span } => {
                self.mark(*span, "resolve_sanshou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_751);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SANSHOU));
            }
            StmtIr::EffectResolveNhlundao { player, source, span } => {
                self.mark(*span, "resolve_nhlundao");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_752);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_NHLUNDAO));
            }
            StmtIr::EffectResolveXiangshu { player, span } => {
                self.mark(*span, "resolve_xiangshu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_753);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIANGSHU));
            }
            StmtIr::EffectResolveDingce {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_dingce");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_730);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGCE));
            }
            StmtIr::EffectResolveMeiyong { player, span } => {
                self.mark(*span, "resolve_meiyong");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_726);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MEIYONG));
            }
            StmtIr::EffectResolveMbzhenann { player, span } => {
                self.mark(*span, "resolve_mbzhenann");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_725);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBZHENANN));
            }
            StmtIr::EffectResolveFriendbiwei {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_friendbiwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_724);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_FRENDBIWEI));
            }
            StmtIr::EffectResolveChulao { player, span } => {
                self.mark(*span, "resolve_chulao");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_723);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHULAO));
            }
            StmtIr::EffectResolveDcanxu { player, span } => {
                self.mark(*span, "resolve_dcanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_722);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCANXU));
            }
            StmtIr::EffectResolveGaoyuan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_gaoyuan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_720);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GAOYUAN));
            }
            StmtIr::EffectChooseDiscardRedirectCurrentSha {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "choose_discard_redirect_current_sha");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_186);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_REDIRECT_CURRENT_SHA,
                ));
            }
            StmtIr::EffectJudgeSetSkillMarkToRank {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "judge_set_skill_mark_to_rank");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_188);
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_SET_SKILL_MARK_TO_RANK));
            }
            StmtIr::EffectChooseDiscardOneFromEachLinked {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_discard_one_from_each_linked");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_189);
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_FROM_EACH_LINKED,
                ));
            }
            StmtIr::EffectDisableSkillByDefUntilTurnEnd {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "disable_skill_by_def_until_turn_end");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_191);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::DISABLE_SKILL_BY_DEF_UNTIL_TURN_END,
                ));
            }
            StmtIr::EffectTargetDiscardThenDuelIfNonShaAndHpGteOwner {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "target_discard_then_duel_if_non_sha_and_hp_gte_owner",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_192);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TARGET_DISCARD_THEN_DUEL_IF_NON_SHA_AND_HP_GTE_OWNER,
                ));
            }
            StmtIr::EffectChooseTargetLoseHpDiscardHandShaIfBasic { player, span } => {
                self.mark(*span, "choose_target_lose_hp_discard_hand_sha_if_basic");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_194);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_LOSE_HP_DISCARD_HAND_SHA_IF_BASIC,
                ));
            }
            StmtIr::EffectChooseDiscardTargetCard {
                player,
                target,
                scope,
                span,
            } => {
                self.mark(*span, "choose_discard_target_card");
                self.required_abi = self.required_abi.max(
                    if matches!(scope, content_schema::TargetCardScopeIr::Hand) {
                        ADDITIVE_RULE_VM_ABI_VERSION_153
                    } else {
                        ADDITIVE_RULE_VM_ABI_VERSION_36
                    },
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_TARGET_CARD));
            }
            StmtIr::EffectJudgePreventDamageLoseMaxHpOnNonHeart {
                player,
                target,
                span,
            } => {
                self.mark(*span, "judge_prevent_damage_lose_max_hp_on_nonheart");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_37);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_PREVENT_DAMAGE_LOSE_MAX_HP_ON_NONHEART,
                ));
            }
            StmtIr::EffectChooseGainTopCardsBySuit {
                player,
                count,
                suit,
                span,
            } => {
                self.mark(*span, "choose_gain_top_cards_by_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_38);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*count)));
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_TOP_CARDS_BY_SUIT));
            }
            StmtIr::EffectChooseOtherDrawByHandCount {
                player,
                normal_count,
                empty_count,
                times,
                span,
            } => {
                self.mark(*span, "choose_other_draw_by_hand_count");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_39);
                self.emit_expr(player)?;
                self.emit_expr(normal_count)?;
                self.emit_expr(empty_count)?;
                self.emit_expr(times)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DRAW_BY_HAND_COUNT));
            }
            StmtIr::EffectChooseDiscardOneThenGainTargetEquipment {
                player,
                target,
                slot,
                span,
            } => {
                self.mark(*span, "choose_discard_one_then_gain_target_equipment");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_40);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*slot)));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_THEN_GAIN_TARGET_EQUIPMENT,
                ));
            }
            StmtIr::EffectChooseTargetDraw {
                player,
                count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_target_draw");
                self.required_abi = self.required_abi.max(match scope {
                    content_schema::DrawTargetScopeIr::HandLteChooser => {
                        ADDITIVE_RULE_VM_ABI_VERSION_338
                    }
                    content_schema::DrawTargetScopeIr::HandEmpty => {
                        ADDITIVE_RULE_VM_ABI_VERSION_162
                    }
                    content_schema::DrawTargetScopeIr::Wounded => ADDITIVE_RULE_VM_ABI_VERSION_195,
                    content_schema::DrawTargetScopeIr::SameHp => ADDITIVE_RULE_VM_ABI_VERSION_135,
                    content_schema::DrawTargetScopeIr::MinHand => ADDITIVE_RULE_VM_ABI_VERSION_83,
                    _ => ADDITIVE_RULE_VM_ABI_VERSION_41,
                });
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_DRAW));
            }
            StmtIr::EffectDrawByPlayPhaseCardCategoryCount { player, span } => {
                self.mark(*span, "draw_by_play_phase_card_category_count");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_42);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_BY_PLAY_PHASE_CARD_CATEGORY_COUNT,
                ));
            }
            StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRed {
                player,
                sha_user,
                card,
                span,
            } => {
                self.mark(*span, "choose_discard_one_for_sha_refund_and_draw_if_red");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_43);
                self.emit_expr(player)?;
                self.emit_expr(sha_user)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED,
                ));
            }
            StmtIr::EffectChooseDiscardOneForShaRefundAndDrawIfRedRestoreIfEqualRank {
                player,
                sha_user,
                card,
                restore_skill_def,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_295);
                self.emit_expr(player)?;
                self.emit_expr(sha_user)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*restore_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ONE_FOR_SHA_REFUND_AND_DRAW_IF_RED_RESTORE_IF_EQUAL_RANK,
                ));
            }
            StmtIr::EffectQueueOpeningExtraPlayPhase { player, span } => {
                self.mark(*span, "queue_opening_extra_play_phase");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_44);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::QUEUE_OPENING_EXTRA_PLAY_PHASE));
            }
            StmtIr::EffectConvertCurrentShaToFireLoseHpIfDamage { player, card, span } => {
                self.mark(*span, "convert_current_sha_to_fire_lose_hp_if_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_45);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CONVERT_CURRENT_SHA_TO_FIRE_LOSE_HP_IF_DAMAGE,
                ));
            }
            StmtIr::EffectChooseAddCurrentShaTarget { player, card, span } => {
                self.mark(*span, "choose_add_current_sha_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_45);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET));
            }
            StmtIr::EffectChooseAddCurrentShaTargetWithinDistance {
                player,
                card,
                max_distance,
                span,
            } => {
                self.mark(*span, "choose_add_current_sha_target_within_distance");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_277);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(max_distance)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ADD_CURRENT_SHA_TARGET_WITHIN_DISTANCE,
                ));
            }
            StmtIr::EffectRefundCurrentShaUse { player, card, span } => {
                self.mark(*span, "refund_current_sha_use");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_278);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REFUND_CURRENT_SHA_USE));
            }
            StmtIr::EffectDamageThenDrawSelectedTargets {
                source,
                damage,
                draw,
                span,
            } => {
                self.mark(*span, "damage_then_draw_selected_targets");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_279);
                self.emit_expr(source)?;
                self.emit_expr(damage)?;
                self.emit_expr(draw)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DAMAGE_THEN_DRAW_SELECTED_TARGETS,
                ));
            }
            StmtIr::EffectResolveYoudi { player, span } => {
                self.mark(*span, "resolve_youdi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_280);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YOUDI));
            }
            StmtIr::EffectResolveKuangfu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_kuangfu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_282);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_KUANGFU));
            }
            StmtIr::EffectResolveShanjia {
                player,
                count,
                span,
            } => {
                self.mark(*span, "resolve_shanjia");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_283);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHANJIA));
            }
            StmtIr::EffectResolveNewYajiao { player, card, span } => {
                self.mark(*span, "resolve_new_yajiao");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_284);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NEW_YAJIAO));
            }
            StmtIr::EffectResolveDingpin {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dingpin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_285);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGPIN));
            }
            StmtIr::EffectChooseRepoluTargetsDraw { player, span } => {
                self.mark(*span, "choose_repolu_targets_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_291);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_REPOLU_TARGETS_DRAW));
            }
            StmtIr::EffectResolveXinfuGuolun {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_xinfu_guolun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_294);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_GUOLUN));
            }
            StmtIr::EffectFinishCurrentPhaseAbortResolution { player, span } => {
                self.mark(*span, "finish_current_phase_abort_resolution");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_286);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::FINISH_CURRENT_PHASE_ABORT_RESOLUTION,
                ));
            }
            StmtIr::EffectGainTopCardToSkillPile { player, span } => {
                self.mark(*span, "gain_top_card_to_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_287);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_TOP_CARD_TO_SKILL_PILE));
            }
            StmtIr::EffectChooseDiscardSkillPileCardAddMarkedSkill {
                player,
                target,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_skill_pile_card_add_marked_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_287);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_SKILL_PILE_CARD_ADD_MARKED_SKILL,
                ));
            }
            StmtIr::EffectGainOneHandFromSelectedTargets { player, span } => {
                self.mark(*span, "gain_one_hand_from_selected_targets");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_288);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GAIN_ONE_HAND_FROM_SELECTED_TARGETS,
                ));
            }
            StmtIr::EffectChooseStoreHandShaInSkillPile { player, span } => {
                self.mark(*span, "choose_store_hand_sha_in_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_46);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_STORE_HAND_SHA_IN_SKILL_PILE,
                ));
            }
            StmtIr::EffectChooseSpendSkillPileForDyingJiu {
                player,
                dying,
                span,
            } => {
                self.mark(*span, "choose_spend_skill_pile_for_dying_jiu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_46);
                self.emit_expr(player)?;
                self.emit_expr(dying)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_SPEND_SKILL_PILE_FOR_DYING_JIU,
                ));
            }
            StmtIr::EffectChooseCompareSelectedHandCard {
                player,
                target,
                win_skill_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "choose_compare_selected_hand_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_47);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*win_skill_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_COMPARE_SELECTED_HAND_CARD,
                ));
            }
            StmtIr::EffectChooseAddSkillBoundCardTarget { player, card, span } => {
                self.mark(*span, "choose_add_skill_bound_card_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_47);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ADD_SKILL_BOUND_CARD_TARGET,
                ));
            }
            StmtIr::EffectChooseDiscardMatchingEquipColorAddSkill {
                player,
                target,
                skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_matching_equip_color_add_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_48);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_MATCHING_EQUIP_COLOR_ADD_SKILL,
                ));
            }
            StmtIr::EffectUseVirtualShaFromCardColorLockSourceIfNoSha { player, span } => {
                self.mark(
                    *span,
                    "use_virtual_sha_from_card_color_lock_source_if_no_sha",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_48);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::USE_VIRTUAL_SHA_FROM_CARD_COLOR_LOCK_SOURCE_IF_NO_SHA,
                ));
            }
            StmtIr::EffectChooseCurrentCardUseEnhancements {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "choose_current_card_use_enhancements");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_49);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CURRENT_CARD_USE_ENHANCEMENTS,
                ));
            }
            StmtIr::EffectMakeCurrentCardUnrespondable {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "make_current_card_unrespondable");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_220);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::MAKE_CURRENT_CARD_UNRESPONDABLE));
            }
            StmtIr::EffectChooseDrawOrDiscardAddSkill {
                player,
                draw_skill_def,
                discard_skill_def,
                three_way,
                span,
            } => {
                self.mark(
                    *span,
                    if *three_way {
                        "choose_three_way_draw_or_discard_add_skill"
                    } else {
                        "choose_draw_or_discard_add_skill"
                    },
                );
                self.required_abi = self.required_abi.max(if *three_way {
                    ADDITIVE_RULE_VM_ABI_VERSION_315
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_50
                });
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*draw_skill_def as i32));
                self.ops.push(Op::ConstI32(*discard_skill_def as i32));
                self.ops.push(Op::HostEffect(if *three_way {
                    effect_tag::CHOOSE_THREE_WAY_DRAW_OR_DISCARD_ADD_SKILL
                } else {
                    effect_tag::CHOOSE_DRAW_OR_DISCARD_ADD_SKILL
                }));
            }
            StmtIr::EffectChooseDiscardedBlackBasicAsBingliang { player, span } => {
                self.mark(*span, "choose_discarded_black_basic_as_bingliang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_51);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARDED_BLACK_BASIC_AS_BINGLIANG,
                ));
            }
            StmtIr::EffectChooseDiscardJudgeUseVirtualShaDrawIfNoDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_judge_use_virtual_sha_draw_if_no_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_51);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_JUDGE_USE_VIRTUAL_SHA_DRAW_IF_NO_DAMAGE,
                ));
            }
            StmtIr::EffectDiscardSelectedRecordUniqueCategoryTarget {
                player,
                target,
                span,
            } => {
                self.mark(*span, "discard_selected_record_unique_category_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_52);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_SELECTED_RECORD_UNIQUE_CATEGORY_TARGET,
                ));
            }
            StmtIr::EffectChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded {
                player,
                target,
                toggle_link,
                span,
            } => {
                self.mark(
                    *span,
                    if *toggle_link {
                        "choose_target_draw_or_discard_by_skill_use_count_toggle_link_if_wounded"
                    } else {
                        "choose_target_draw_or_discard_by_skill_use_count_link_if_wounded"
                    },
                );
                self.required_abi = self.required_abi.max(if *toggle_link {
                    ADDITIVE_RULE_VM_ABI_VERSION_314
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_52
                });
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(if *toggle_link {
                    effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_TOGGLE_LINK_IF_WOUNDED
                } else {
                    effect_tag::CHOOSE_TARGET_DRAW_OR_DISCARD_BY_SKILL_USE_COUNT_LINK_IF_WOUNDED
                }));
            }
            StmtIr::EffectChooseRevealOtherHandCardRememberCategory { player, span } => {
                self.mark(*span, "choose_reveal_other_hand_card_remember_category");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_53);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_REVEAL_OTHER_HAND_CARD_REMEMBER_CATEGORY,
                ));
            }
            StmtIr::EffectDrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                span,
            } => {
                self.mark(*span, "draw_then_give_exact_record_phase_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_53);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_GIVE_EXACT_RECORD_PHASE_TARGET,
                ));
            }
            StmtIr::EffectClearPhaseGiftTarget { player, span } => {
                self.mark(*span, "clear_phase_gift_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_53);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CLEAR_PHASE_GIFT_TARGET));
            }
            StmtIr::EffectChooseGainResolvedShaOrShanThenOptionalUseSha {
                player,
                sha_card,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_gain_resolved_sha_or_shan_then_optional_use_sha",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_54);
                self.emit_expr(player)?;
                self.emit_expr(sha_card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_RESOLVED_SHA_OR_SHAN_THEN_OPTIONAL_USE_SHA,
                ));
            }
            StmtIr::EffectSkipCurrentPhase { player, span } => {
                self.mark(*span, "skip_current_phase");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_55);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_CURRENT_PHASE));
            }
            StmtIr::EffectChooseOtherTargetsBySkippedPhaseCountDamage {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_other_targets_by_skipped_phase_count_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_55);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE,
                ));
            }
            StmtIr::EffectRevealUniformHandDrawSelfChooseOthers { player, span } => {
                self.mark(*span, "reveal_uniform_hand_draw_self_choose_others");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_56);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_UNIFORM_HAND_DRAW_SELF_CHOOSE_OTHERS,
                ));
            }
            StmtIr::EffectChooseCardUserOrAddTarget { player, card, span } => {
                self.mark(*span, "choose_card_user_or_add_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_57);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_CARD_USER_OR_ADD_TARGET));
            }
            StmtIr::EffectChooseDiscardEquipmentReduceCurrentDamage { player, span } => {
                self.mark(*span, "choose_discard_equipment_reduce_current_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_57);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_EQUIPMENT_REDUCE_CURRENT_DAMAGE,
                ));
            }
            StmtIr::EffectChooseDiscardEquipmentPreventCurrentDamage { player, span } => {
                self.mark(*span, "choose_discard_equipment_prevent_current_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_398);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_EQUIPMENT_PREVENT_CURRENT_DAMAGE,
                ));
            }
            StmtIr::EffectChooseMoveTargetEquipmentOrDraw {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_move_target_equipment_or_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_58);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_MOVE_TARGET_EQUIPMENT_OR_DRAW,
                ));
            }
            StmtIr::EffectGiveAllEquipmentChooseRecoverOrDamage {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_equipment_choose_recover_or_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_58);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_ALL_EQUIPMENT_CHOOSE_RECOVER_OR_DAMAGE,
                ));
            }
            StmtIr::EffectChooseTargetDiscardWeaponRangeHandOrIncreaseDamage {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_discard_weapon_range_hand_or_increase_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_59);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_WEAPON_RANGE_HAND_OR_INCREASE_DAMAGE,
                ));
            }
            StmtIr::EffectChooseHandCountDifferentTargetDrawOrRandomDiscard { player, span } => {
                self.mark(
                    *span,
                    "choose_hand_count_different_target_draw_or_random_discard",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_60);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_HAND_COUNT_DIFFERENT_TARGET_DRAW_OR_RANDOM_DISCARD,
                ));
            }
            StmtIr::EffectIncreaseCurrentDrawCount {
                player,
                count,
                span,
            } => {
                self.mark(*span, "increase_current_draw_count");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_61);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::INCREASE_CURRENT_DRAW_COUNT));
            }
            StmtIr::EffectChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw { player, span } => {
                self.mark(
                    *span,
                    "choose_target_discard_excess_then_self_discard_or_target_draw",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_62);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DISCARD_EXCESS_THEN_SELF_DISCARD_OR_TARGET_DRAW,
                ));
            }
            StmtIr::EffectDrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "draw_then_choose_gain_one_from_damage_target_and_neighbors",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_63);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_CHOOSE_GAIN_ONE_FROM_DAMAGE_TARGET_AND_NEIGHBORS,
                ));
            }
            StmtIr::EffectChoosePreventDamageOrLoseMaxHpIncreaseDamage {
                player,
                amount,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_prevent_damage_or_lose_max_hp_increase_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_64);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_PREVENT_DAMAGE_OR_LOSE_MAX_HP_INCREASE_DAMAGE,
                ));
            }
            StmtIr::EffectRandomizeCurrentHandCompareOpponent { player, span } => {
                self.mark(*span, "randomize_current_hand_compare_opponent");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_65);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::RANDOMIZE_CURRENT_HAND_COMPARE_OPPONENT,
                ));
            }
            StmtIr::EffectGainHighestRankShaFromEventCards { player, span } => {
                self.mark(*span, "gain_highest_rank_sha_from_event_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_65);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GAIN_HIGHEST_RANK_SHA_FROM_EVENT_CARDS,
                ));
            }
            StmtIr::EffectGainHandCompareResultCard { player, span } => {
                self.mark(*span, "gain_hand_compare_result_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_66);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_HAND_COMPARE_RESULT_CARD));
            }
            StmtIr::EffectChooseAddOrRemoveCurrentCardTarget { player, card, span } => {
                self.mark(*span, "choose_add_or_remove_current_card_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_67);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ADD_OR_REMOVE_CURRENT_CARD_TARGET,
                ));
            }
            StmtIr::EffectChooseHandCompare {
                player,
                win_skill_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "choose_hand_compare");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_68);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*win_skill_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_HAND_COMPARE));
            }
            StmtIr::EffectChooseHandCompareGrantTargetOnWin {
                player,
                win_grant_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "choose_hand_compare_grant_target_on_win");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_678);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*win_grant_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_HAND_COMPARE_GRANT_TARGET_ON_WIN,
                ));
            }
            StmtIr::EffectResolveLieren {
                player,
                target,
                win_skill_def,
                lose_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_lieren");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_531);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*win_skill_def as i32));
                self.ops.push(Op::ConstI32(*lose_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIEREN));
            }
            StmtIr::EffectResolveTuntian { player, span } => {
                self.mark(*span, "resolve_tuntian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_533);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TUNTIAN));
            }
            StmtIr::EffectResolvePotdimeng { player, span } => {
                self.mark(*span, "resolve_potdimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_535);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTDIMENG));
            }
            StmtIr::EffectResolvePothaoshi {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_pothaoshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_536);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTHAOSHI));
            }
            StmtIr::EffectResolveSpmiaolve {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_spmiaolve");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_539);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPMIAOLVE));
            }
            StmtIr::EffectResolveSpyingjia {
                player,
                action,
                helper_def,
                span,
            } => {
                self.mark(*span, "resolve_spyingjia");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_539);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*helper_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPYINGJIA));
            }
            StmtIr::EffectResolveZaoxian {
                player,
                jixi_def,
                tuntian_def,
                span,
            } => {
                self.mark(*span, "resolve_zaoxian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_534);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*jixi_def as i32));
                self.ops.push(Op::ConstI32(*tuntian_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZAOXIAN));
            }
            StmtIr::EffectResolveTiaoxin { player, span } => {
                self.mark(*span, "resolve_tiaoxin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_537);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIAOXIN));
            }
            StmtIr::EffectResolveZhiji {
                player,
                guanxing_def,
                span,
            } => {
                self.mark(*span, "resolve_zhiji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_538);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*guanxing_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIJI));
            }
            StmtIr::EffectResolveZaiqi {
                player,
                extra,
                span,
            } => {
                self.mark(*span, "resolve_zaiqi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_540);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*extra as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZAIQI));
            }
            StmtIr::EffectResolveTwqiushou { player, span } => {
                self.mark(*span, "resolve_twqiushou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_542);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQIUSHOU));
            }
            StmtIr::EffectResolveZhengrong { player, span } => {
                self.mark(*span, "resolve_zhengrong");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_545);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGRONG));
            }
            StmtIr::EffectResolveHongju {
                player,
                qingjian_def,
                rong_def,
                span,
            } => {
                self.mark(*span, "resolve_hongju");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_546);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qingjian_def as i32));
                self.ops.push(Op::ConstI32(*rong_def as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HONGJU));
            }
            StmtIr::EffectResolveQingjian {
                player,
                rong_def,
                span,
            } => {
                self.mark(*span, "resolve_qingjian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_547);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*rong_def as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINGJIAN));
            }
            StmtIr::EffectResolveReleiji { player, span } => {
                self.mark(*span, "resolve_releiji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_549);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RELEIJI));
            }
            StmtIr::EffectResolveHuangtian {
                player,
                actor,
                span,
            } => {
                self.mark(*span, "resolve_huangtian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_551);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HUANGTIAN));
            }
            StmtIr::EffectResolveHaoshi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_haoshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_555);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HAOSHI));
            }
            StmtIr::EffectResolveDimeng { player, span } => {
                self.mark(*span, "resolve_dimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_556);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DIMENG));
            }
            StmtIr::EffectResolveQiaobian {
                player,
                phase,
                span,
            } => {
                self.mark(*span, "resolve_qiaobian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_559);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*phase as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIAOBIAN));
            }
            StmtIr::EffectExcludeSavedShaTarget { card, target, span } => {
                self.mark(*span, "exclude_current_sha_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EXCLUDE_SAVED_SHA_TARGET));
            }
            StmtIr::EffectResolveXiangle {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_xiangle");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIANGLE));
            }
            StmtIr::EffectResolveFangquanSkip { player, span } => {
                self.mark(*span, "resolve_fangquan_skip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_FANGQUAN_SKIP));
            }
            StmtIr::EffectResolveFangquanGrant { player, span } => {
                self.mark(*span, "resolve_fangquan_grant");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_FANGQUAN_GRANT));
            }
            StmtIr::EffectResolveRuoyu {
                player,
                jijiang_def,
                span,
            } => {
                self.mark(*span, "resolve_ruoyu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_560);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*jijiang_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RUOYU));
            }
            StmtIr::EffectResolveZuilun { player, span } => {
                self.mark(*span, "resolve_zuilun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_563);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZUILUN));
            }
            StmtIr::EffectResolveFuyin {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_fuyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_563);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FUYIN));
            }
            StmtIr::EffectBanUseAgainst { user, target, span } => {
                self.mark(*span, "ban_use_against");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_571);
                self.emit_expr(user)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::BAN_USE_AGAINST));
            }
            StmtIr::EffectResolveJuzhan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_juzhan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_571);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JUZHAN));
            }
            StmtIr::EffectResolveQizhi { player, span } => {
                self.mark(*span, "resolve_qizhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_573);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIZHI));
            }
            StmtIr::EffectResolveJinqu {
                player,
                qizhi_def,
                span,
            } => {
                self.mark(*span, "resolve_jinqu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_573);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qizhi_def as i32));
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JINQU));
            }
            StmtIr::EffectResolveJianxiang { player, span } => {
                self.mark(*span, "resolve_jianxiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_574);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANXIANG));
            }
            StmtIr::EffectResolveShenshi {
                player,
                source,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_shenshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_574);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHENSHI));
            }
            StmtIr::EffectResolveMingren {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mingren");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_576);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MINGREN));
            }
            StmtIr::EffectResolveZhenliang {
                player,
                source,
                card,
                mingren_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_zhenliang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_576);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*mingren_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENLIANG));
            }
            StmtIr::EffectResolveKuizhu { player, span } => {
                self.mark(*span, "resolve_kuizhu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_577);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_KUIZHU));
            }
            StmtIr::EffectResolveZhizheng { player, span } => {
                self.mark(*span, "resolve_zhizheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_577);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIZHENG));
            }
            StmtIr::EffectResolveLijun {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_lijun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_577);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIJUN));
            }
            StmtIr::EffectResolveLiangyin {
                player,
                kongsheng_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_liangyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_579);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kongsheng_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIANGYIN));
            }
            StmtIr::EffectResolveKongsheng {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_kongsheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_579);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_KONGSHENG));
            }
            StmtIr::EffectResolvePotzhouyu {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potzhouyu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_561);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTZHOUYU));
            }
            StmtIr::EffectResolvePotyuji {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potyuji");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_570);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTYUJI));
            }
            StmtIr::EffectResolveRehengjiang {
                player,
                actor,
                action,
                hand_limit_def,
                draw_def,
                span,
            } => {
                self.mark(*span, "resolve_rehengjiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_572);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::ConstI32(*hand_limit_def as i32));
                self.ops.push(Op::ConstI32(*draw_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_REHENGJIANG));
            }
            StmtIr::EffectAddFirstMissingSkill {
                player,
                skill_defs,
                span,
            } => {
                self.mark(*span, "add_first_missing_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_575);
                self.emit_expr(player)?;
                for skill_def in skill_defs {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::ADD_FIRST_MISSING_SKILL));
            }
            StmtIr::EffectResolveDccunsi {
                player,
                target,
                guixiu_def,
                yongjue_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_dccunsi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_578);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*guixiu_def as i32));
                self.ops.push(Op::ConstI32(*yongjue_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCCUNSI));
            }
            StmtIr::EffectRecordCurrentShaAllTargets { player, card, span } => {
                self.mark(*span, "record_current_sha_all_targets");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_580);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECORD_CURRENT_SHA_ALL_TARGETS));
            }
            StmtIr::EffectChooseGiveOrDiscardHand {
                player,
                payment_count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_give_or_discard_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_581);
                self.emit_expr(player)?;
                self.emit_expr(payment_count)?;
                self.ops.push(Op::ConstI32(scope.tag() as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GIVE_OR_DISCARD_HAND));
            }
            StmtIr::EffectChooseDiscardHandEquipThenRecoverTarget {
                chooser,
                target,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_recover_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_582);
                self.emit_expr(chooser)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER_TARGET,
                ));
            }
            StmtIr::EffectResolvePotlvyi {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potlvyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_564);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTLVYI));
            }
            StmtIr::EffectResolvePotcaoshuang {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potcaoshuang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_568);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTCAOSHUANG));
            }
            StmtIr::EffectResolvePottuntian {
                player,
                action,
                zaoxian_def,
                span,
            } => {
                self.mark(*span, "resolve_pottuntian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_543);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::ConstI32(*zaoxian_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTTUNTIAN));
            }
            StmtIr::EffectResolvePotjixi {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potjixi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_543);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJIXI));
            }
            StmtIr::EffectResolvePotceduan {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potceduan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_544);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTCEDUAN));
            }
            StmtIr::EffectResolvePottaishici {
                player,
                target,
                action,
                hanzhan_def,
                zhanlie_def,
                span,
            } => {
                self.mark(*span, "resolve_pottaishici");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_548);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.ops.push(Op::ConstI32(*hanzhan_def as i32));
                self.ops.push(Op::ConstI32(*zhanlie_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTTAISHICI));
            }
            StmtIr::EffectResolveMbzhonghui {
                player,
                target,
                action,
                amount,
                mbsizi_def,
                mbkechang_def,
                span,
            } => {
                self.mark(*span, "resolve_mbzhonghui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_553);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*action as i32));
                self.emit_expr(amount)?;
                self.ops.push(Op::ConstI32(*mbsizi_def as i32));
                self.ops.push(Op::ConstI32(*mbkechang_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBZHONGHUI));
            }
            StmtIr::EffectResolvePotjiejie {
                player,
                actor,
                used_def,
                suit_defs,
                qingshi_def,
                span,
            } => {
                self.mark(*span, "resolve_potjiejie");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_554);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(*used_def as i32));
                for def in suit_defs {
                    self.ops.push(Op::ConstI32(*def as i32));
                }
                self.ops.push(Op::ConstI32(*qingshi_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJIEJIE));
            }
            StmtIr::EffectResolvePotqingshi { player, span } => {
                self.mark(*span, "resolve_potqingshi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_554);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTQINGSHI));
            }
            StmtIr::EffectResolvePotweiyan {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_potweiyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_558);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTWEIYAN));
            }
            StmtIr::EffectChooseOtherDamage {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_other_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_69);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DAMAGE));
            }
            StmtIr::EffectChooseOtherDiscardTargetCard {
                player,
                scope,
                span,
            } => {
                self.mark(*span, "choose_other_discard_target_card");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_70);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DISCARD_TARGET_CARD));
            }
            StmtIr::EffectGainDiscardCardByKind { player, kind, span } => {
                self.mark(*span, "gain_discard_card_by_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_74);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_DISCARD_CARD_BY_KIND));
            }
            StmtIr::EffectRevealUniformHandChooseDamage {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "reveal_uniform_hand_choose_damage");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_76);
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_UNIFORM_HAND_CHOOSE_DAMAGE,
                ));
            }
            StmtIr::EffectChooseGainDiscardCardByKind { player, kind, span } => {
                self.mark(*span, "choose_gain_discard_card_by_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_77);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_DISCARD_CARD_BY_KIND));
            }
            StmtIr::EffectDrawForLifetimeCardActionDivisors {
                player,
                divisors,
                span,
            } => {
                self.mark(*span, "draw_for_lifetime_card_action_divisors");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_78);
                self.emit_expr(player)?;
                for divisor in divisors {
                    self.ops.push(Op::ConstI32(*divisor as i32));
                }
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_FOR_LIFETIME_CARD_ACTION_DIVISORS,
                ));
            }
            StmtIr::EffectChooseAddMissingSkill {
                player,
                skill_defs,
                span,
            } => {
                self.mark(*span, "choose_add_missing_skill");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_79);
                self.emit_expr(player)?;
                for skill_def in skill_defs {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ADD_MISSING_SKILL));
            }
            StmtIr::EffectChooseTargetsDraw {
                player,
                count,
                target_count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_targets_draw");
                self.required_abi = self.required_abi.max(match scope {
                    content_schema::DrawTargetScopeIr::HandLteChooser => {
                        ADDITIVE_RULE_VM_ABI_VERSION_338
                    }
                    content_schema::DrawTargetScopeIr::HandEmpty => {
                        ADDITIVE_RULE_VM_ABI_VERSION_162
                    }
                    content_schema::DrawTargetScopeIr::Wounded => ADDITIVE_RULE_VM_ABI_VERSION_195,
                    content_schema::DrawTargetScopeIr::SameHp => ADDITIVE_RULE_VM_ABI_VERSION_135,
                    content_schema::DrawTargetScopeIr::MinHand => ADDITIVE_RULE_VM_ABI_VERSION_83,
                    _ => ADDITIVE_RULE_VM_ABI_VERSION_80,
                });
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(i32::from(*target_count)));
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGETS_DRAW));
            }
            StmtIr::EffectChooseTargetsDamage {
                player,
                min_count,
                max_count,
                scope,
                amount,
                span,
            } => {
                self.mark(*span, "choose_targets_damage");
                self.required_abi = self.required_abi.max(match scope {
                    content_schema::DrawTargetScopeIr::HandEmpty => {
                        ADDITIVE_RULE_VM_ABI_VERSION_162
                    }
                    content_schema::DrawTargetScopeIr::Wounded => ADDITIVE_RULE_VM_ABI_VERSION_195,
                    _ => ADDITIVE_RULE_VM_ABI_VERSION_135,
                });
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*min_count)));
                self.ops.push(Op::ConstI32(i32::from(*max_count)));
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGETS_DAMAGE));
            }
            StmtIr::EffectChooseOptionalTargetsDraw {
                player,
                count,
                min_count,
                max_count,
                scope,
                span,
            } => {
                self.mark(*span, "choose_optional_targets_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_195);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(i32::from(*min_count)));
                self.ops.push(Op::ConstI32(i32::from(*max_count)));
                self.ops.push(Op::ConstI32(i32::from(scope.tag())));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OPTIONAL_TARGETS_DRAW));
            }
            StmtIr::EffectRevealTargetHandShaOrDiscardBind {
                player,
                target,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "reveal_target_hand_sha_or_discard_bind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_196);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TARGET_HAND_SHA_OR_DISCARD_BIND,
                ));
            }
            StmtIr::EffectDrawRememberedHandCards {
                player,
                count,
                span,
            } => {
                self.mark(*span, "draw_remembered_hand_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_197);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_REMEMBERED_HAND_CARDS));
            }
            StmtIr::EffectDiscardRememberedHandCards { player, span } => {
                self.mark(*span, "discard_remembered_hand_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_197);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_REMEMBERED_HAND_CARDS));
            }
            StmtIr::EffectGainDrawCardsForLifetimeCardActionDivisors {
                player,
                divisors,
                groups,
                span,
            } => {
                self.mark(*span, "gain_draw_cards_for_lifetime_card_action_divisors");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_198);
                self.emit_expr(player)?;
                for index in 0..3 {
                    self.ops.push(Op::ConstI32(divisors[index] as i32));
                    self.ops.push(Op::ConstI32(i32::from(groups[index])));
                }
                self.ops.push(Op::HostEffect(
                    effect_tag::GAIN_DRAW_CARDS_FOR_LIFETIME_CARD_ACTION_DIVISORS,
                ));
            }
            StmtIr::EffectLoseHpGainDiscardCardForOpeningExtraPlay {
                player,
                kind,
                mandatory_skill_def,
                span,
            } => {
                self.mark(*span, "lose_hp_gain_discard_card_for_opening_extra_play");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_200);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops.push(Op::ConstU32(*mandatory_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::LOSE_HP_GAIN_DISCARD_CARD_FOR_OPENING_EXTRA_PLAY,
                ));
            }
            StmtIr::EffectChooseGainOwnLostDiscardCardThisTurn { player, span } => {
                self.mark(*span, "choose_gain_own_lost_discard_card_this_turn");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_201);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_OWN_LOST_DISCARD_CARD_THIS_TURN,
                ));
            }
            StmtIr::EffectChooseMoveFieldCardTo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_move_field_card_to");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_202);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_MOVE_FIELD_CARD_TO));
            }
            StmtIr::EffectDrawThenOptionallyDistributeHandCards {
                player,
                draw_count,
                give_cap,
                span,
            } => {
                self.mark(*span, "draw_then_optionally_distribute_hand_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_203);
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(give_cap)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_OPTIONALLY_DISTRIBUTE_HAND_CARDS,
                ));
            }
            StmtIr::EffectGiveSelectedThenContinueDistinctRecoverAtTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_then_continue_distinct_recover_at_two");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_204);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_CONTINUE_DISTINCT_RECOVER_AT_TWO,
                ));
            }
            StmtIr::EffectChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
                player,
                mark_def,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_max_mark_other_judge_kill_unless_tao_or_taoyuan",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_205);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_MAX_MARK_OTHER_JUDGE_KILL_UNLESS_TAO_OR_TAOYUAN,
                ));
            }
            StmtIr::EffectChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                player,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_zone_priority_gain_random_card_from_each_other_then_turn_over",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_206);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_ZONE_PRIORITY_GAIN_RANDOM_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
                ));
            }
            StmtIr::EffectChooseGainCardFromEachOtherThenTurnOver { player, span } => {
                self.mark(*span, "choose_gain_card_from_each_other_then_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_237);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
                ));
            }
            StmtIr::EffectResolveHiddenSkillPile {
                player,
                pile_skill_def,
                action,
                card,
                span,
            } => {
                self.mark(*span, "resolve_hidden_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_238);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*pile_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_HIDDEN_SKILL_PILE));
            }
            StmtIr::EffectSetNonLockedSkillSuppression {
                player,
                enabled,
                span,
            } => {
                self.mark(*span, "set_non_locked_skill_suppression");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_239);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*enabled)));
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_NON_LOCKED_SKILL_SUPPRESSION));
            }
            StmtIr::EffectResolveDiliYuheng {
                player,
                action,
                card,
                span,
            } => {
                self.mark(*span, "resolve_dili_yuheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_240);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DILI_YUHENG));
            }
            StmtIr::EffectDiscardSelectedDrawAllHandBonus { player, span } => {
                self.mark(*span, "discard_selected_draw_all_hand_bonus");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_241);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DISCARD_SELECTED_DRAW_ALL_HAND_BONUS,
                ));
            }
            StmtIr::EffectChooseOtherYinghun { player, span } => {
                self.mark(*span, "choose_other_yinghun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_242);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_YINGHUN));
            }
            StmtIr::EffectChooseOtherBizheng { player, span } => {
                self.mark(*span, "choose_other_bizheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_243);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_BIZHENG));
            }
            StmtIr::EffectChooseOtherXiashu { player, span } => {
                self.mark(*span, "choose_other_xiashu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_244);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_XIASHU));
            }
            StmtIr::EffectResolveGuanchaoRank { player, card, span } => {
                self.mark(*span, "resolve_guanchao_rank");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_245);
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GUANCHAO_RANK));
            }
            StmtIr::EffectResolveGuanwei {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_guanwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_246);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUANWEI));
            }
            StmtIr::EffectResolveLanjiang { player, span } => {
                self.mark(*span, "resolve_lanjiang");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_247);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LANJIANG));
            }
            StmtIr::EffectResolveRebingyi { player, span } => {
                self.mark(*span, "resolve_rebingyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_248);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REBINGYI));
            }
            StmtIr::EffectResolveRebotu { player, span } => {
                self.mark(*span, "resolve_rebotu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_249);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REBOTU));
            }
            StmtIr::EffectResolveOldimeng {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_oldimeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_250);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDIMENG));
            }
            StmtIr::EffectResolveRejieyin { player, span } => {
                self.mark(*span, "resolve_rejieyin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_251);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REJIEYIN));
            }
            StmtIr::EffectResolveReanxu { player, span } => {
                self.mark(*span, "resolve_reanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_252);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REANXU));
            }
            StmtIr::EffectResolveDrltJueyan {
                player,
                sha_skill_def,
                weapon_skill_def,
                mount_skill_def,
                armor_skill_def,
                treasure_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_drlt_jueyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_253);
                self.emit_expr(player)?;
                for skill_def in [
                    sha_skill_def,
                    weapon_skill_def,
                    mount_skill_def,
                    armor_skill_def,
                    treasure_skill_def,
                ] {
                    self.ops.push(Op::ConstI32(*skill_def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_JUEYAN));
            }
            StmtIr::EffectResolveRejizhi { player, span } => {
                self.mark(*span, "resolve_rejizhi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_254);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REJIZHI));
            }
            StmtIr::EffectResolveJiexun {
                player,
                funan_helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_jiexun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_255);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*funan_helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIEXUN));
            }
            StmtIr::EffectResolveWenguaSetup {
                player,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_wengua_setup");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_256);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WENGUA_SETUP));
            }
            StmtIr::EffectResolveWengua {
                player,
                main_skill_def,
                marker_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_wengua");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_257);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*main_skill_def as i32));
                self.ops.push(Op::ConstI32(*marker_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WENGUA));
            }
            StmtIr::EffectResolveXinanguo { player, span } => {
                self.mark(*span, "resolve_xinanguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_258);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XINANGUO));
            }
            StmtIr::EffectResolveGanglieThree { player, span } => {
                self.mark(*span, "resolve_ganglie_three");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_259);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_GANGLIE_THREE));
            }
            StmtIr::EffectResolveZhongyi {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_zhongyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_260);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGYI));
            }
            StmtIr::EffectDiscardEquipmentSlot { player, slot, span } => {
                self.mark(*span, "discard_equipment_slot");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_261);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*slot)));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_EQUIPMENT_SLOT));
            }
            StmtIr::EffectResolveOldAnxu { player, span } => {
                self.mark(*span, "resolve_oldanxu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_262);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDANXU));
            }
            StmtIr::EffectStoreSelectedInSkillPile { player, span } => {
                self.mark(*span, "store_selected_in_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_263);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::STORE_SELECTED_IN_SKILL_PILE));
            }
            StmtIr::EffectEndMatchForCampIfSkillPileAtLeast {
                player,
                pile_skill_def,
                count,
                span,
            } => {
                self.mark(*span, "end_match_for_player_camp_if_skill_pile_at_least");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_264);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*pile_skill_def as i32));
                self.ops.push(Op::ConstI32(*count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::END_MATCH_FOR_CAMP_IF_SKILL_PILE_AT_LEAST,
                ));
            }
            StmtIr::EffectResolveOldQinqing { player, span } => {
                self.mark(*span, "resolve_oldqinqing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_265);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDQINQING));
            }
            StmtIr::EffectResolveOldMingjian { player, span } => {
                self.mark(*span, "resolve_oldmingjian");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_266);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDMINGJIAN));
            }
            StmtIr::EffectResolveOldXuanfeng { player, span } => {
                self.mark(*span, "resolve_oldxuanfeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_267);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDXUANFENG));
            }
            StmtIr::EffectResolveJianrou { player, span } => {
                self.mark(*span, "resolve_jianrou");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_268);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANROU));
            }
            StmtIr::EffectResolveOldHuisheng { player, span } => {
                self.mark(*span, "resolve_oldhuisheng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_269);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_OLDHUISHENG));
            }
            StmtIr::EffectResolveOldJiefan {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_oldjiefan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_270);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLDJIEFAN));
            }
            StmtIr::EffectResolveChegu { player, span } => {
                self.mark(*span, "resolve_chegu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_271);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHEGU));
            }
            StmtIr::EffectResolveJunkguixin { player, span } => {
                self.mark(*span, "resolve_junkguixin");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_272);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JUNKGUIXIN));
            }
            StmtIr::EffectResolveYaoChenwei { player, span } => {
                self.mark(*span, "resolve_yao_chenwei");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_275);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_YAO_CHENWEI));
            }
            StmtIr::EffectAddSkillUntilTurnEnd {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "add_skill_until_turn_end");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::ADD_SKILL_UNTIL_TURN_END));
            }
            StmtIr::EffectResolveMassDamageDiscardTurnOver {
                player,
                mark_def,
                mark_count,
                damage_amount,
                hand_count,
                span,
            } => {
                self.mark(*span, "resolve_mass_damage_discard_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mark_count as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(*hand_count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_TURN_OVER,
                ));
            }
            StmtIr::EffectResolveMassDamageDiscardHandEquip {
                player,
                mark_def,
                mark_count,
                damage_amount,
                discard_count,
                span,
            } => {
                self.mark(*span, "resolve_mass_damage_discard_hand_equip");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_235);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mark_count as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(*discard_count as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_MASS_DAMAGE_DISCARD_HAND_EQUIP,
                ));
            }
            StmtIr::EffectRecordActiveTarget {
                player,
                target,
                span,
            } => {
                self.mark(*span, "record_active_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_207);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECORD_ACTIVE_TARGET));
            }
            StmtIr::EffectChooseDiscardWeaponOrLoseHp { player, span } => {
                self.mark(*span, "choose_discard_weapon_or_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_222);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_WEAPON_OR_LOSE_HP));
            }
            StmtIr::EffectResolveMarkParityControl {
                player,
                mark_def,
                mass_threshold,
                damage_amount,
                span,
            } => {
                self.mark(*span, "resolve_mark_parity_control");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*mass_threshold as i32));
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MARK_PARITY_CONTROL));
            }
            StmtIr::EffectResolveSelectedEquipmentDiscardThenChooseDamage {
                player,
                damage_amount,
                nature,
                span,
            } => {
                self.mark(
                    *span,
                    "resolve_selected_equipment_discard_then_choose_damage",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_208);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*damage_amount as i32));
                self.ops.push(Op::ConstI32(i32::from(*nature)));
                self.ops.push(Op::HostEffect(
                    effect_tag::RESOLVE_SELECTED_EQUIPMENT_DISCARD_THEN_CHOOSE_DAMAGE,
                ));
            }
            StmtIr::EffectReplaceDrawWithDistinctSuitSelection {
                player,
                count,
                span,
            } => {
                self.mark(*span, "replace_draw_with_distinct_suit_selection");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_209);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*count)));
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_DRAW_WITH_DISTINCT_SUIT_SELECTION,
                ));
            }
            StmtIr::EffectInspectTargetHandChooseHeartDiscardOrTop {
                player,
                target,
                span,
            } => {
                self.mark(*span, "inspect_target_hand_choose_heart_discard_or_top");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_209);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::INSPECT_TARGET_HAND_CHOOSE_HEART_DISCARD_OR_TOP,
                ));
            }
            StmtIr::EffectResolveYeyan { player, span } => {
                self.mark(*span, "resolve_yeyan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_210);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YEYAN));
            }
            StmtIr::EffectChooseAllLoseHpOrRecover { player, span } => {
                self.mark(*span, "choose_all_lose_hp_or_recover");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_210);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ALL_LOSE_HP_OR_RECOVER));
            }
            StmtIr::EffectResolveQixing {
                player,
                initialize,
                span,
            } => {
                self.mark(*span, "resolve_qixing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_212);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*initialize)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIXING));
            }
            StmtIr::EffectResolveQixingWeather {
                player,
                qixing_skill_def,
                helper_skill_def,
                max_targets,
                span,
            } => {
                self.mark(*span, "resolve_qixing_weather");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_212);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*qixing_skill_def as i32));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*max_targets)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_QIXING_WEATHER));
            }
            StmtIr::EffectResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_nzry_longnu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*fire_skill_def as i32));
                self.ops.push(Op::ConstI32(*thunder_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NZRY_LONGNU));
            }
            StmtIr::EffectChooseLinkOther { player, span } => {
                self.mark(*span, "choose_link_other");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_213);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::CHOOSE_LINK_OTHER));
            }
            StmtIr::EffectResolveDrltPoxi {
                player,
                target,
                hand_limit_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_drlt_poxi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_214);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*hand_limit_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DRLT_POXI));
            }
            StmtIr::EffectResolveDrltJieying {
                player,
                event_player,
                mark,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_drlt_jieying");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_215);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::ConstI32(*mark as i32));
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_JIEYING));
            }
            StmtIr::EffectResolveDrltDuorui {
                player,
                target,
                helper_skill_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_drlt_duorui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_216);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_DUORUI));
            }
            StmtIr::EffectResolveDrltZhiti {
                player,
                event_player,
                event_source,
                event_amount,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_drlt_zhiti");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_217);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_source)?;
                self.emit_expr(event_amount)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_ZHITI));
            }
            StmtIr::EffectResolveShenSimayi {
                player,
                event_player,
                event_amount,
                mark,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_shen_simayi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_218);
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_amount)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHEN_SIMAYI));
            }
            StmtIr::EffectResolveBeige {
                player,
                target,
                source,
                span,
            } => {
                self.mark(*span, "resolve_beige");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_219);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BEIGE));
            }
            StmtIr::EffectResolveBeigeConfigured {
                player,
                target,
                source,
                heart_recover_amount,
                diamond_draw_count,
                span,
            } => {
                self.mark(*span, "resolve_beige_configured");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_296);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(heart_recover_amount)?;
                self.emit_expr(diamond_draw_count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_BEIGE_CONFIGURED));
            }
            StmtIr::EffectResolveZhuiyi {
                player,
                killer,
                span,
            } => {
                self.mark(*span, "resolve_zhuiyi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_297);
                self.emit_expr(player)?;
                self.emit_expr(killer)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHUIYI));
            }
            StmtIr::EffectResolveDecadezhenjun { player, span } => {
                self.mark(*span, "resolve_decadezhenjun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_298);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DECADEZHENJUN));
            }
            StmtIr::EffectResolveSpXiaoguo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_sp_xiaoguo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_299);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SP_XIAOGUO));
            }
            StmtIr::EffectChooseGainCardFromEachOtherReplaceDraw { player, span } => {
                self.mark(*span, "choose_gain_card_from_each_other_replace_draw");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_300);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_REPLACE_DRAW,
                ));
            }
            StmtIr::EffectGiveSelectedThenRecoverSelf {
                player,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "give_selected_then_recover_self");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_679);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_RECOVER_SELF,
                ));
            }
            StmtIr::EffectChooseHandCompareBetween {
                player,
                first,
                span,
            } => {
                self.mark(*span, "choose_hand_compare_between");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_680);
                self.emit_expr(player)?;
                self.emit_expr(first)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_HAND_COMPARE_BETWEEN));
            }
            StmtIr::EffectGuanxingTop {
                player,
                count,
                span,
            } => {
                self.mark(*span, "guanxing_top");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_681);
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::GUANXING_TOP));
            }
            StmtIr::EffectGuanxingTopByFaluMarks {
                player,
                marks,
                span,
            } => {
                self.mark(*span, "guanxing_top_by_falu_marks");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_683);
                self.emit_expr(player)?;
                for def in marks {
                    self.ops.push(Op::ConstI32(*def as i32));
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::GUANXING_TOP_BY_FALU_MARKS));
            }
            StmtIr::EffectChooseOtherGiveThenRecover { player, span } => {
                self.mark(*span, "choose_other_give_then_recover");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_682);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_GIVE_THEN_RECOVER,
                ));
            }
            StmtIr::EffectGiveSelectedThenRecoverTarget {
                player,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "give_selected_then_recover_target");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_301);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_RECOVER_TARGET,
                ));
            }
            StmtIr::EffectResolveDingpan {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dingpan");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_302);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DINGPAN));
            }
            StmtIr::EffectResolveTanlu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_tanlu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_303);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TANLU));
            }
            StmtIr::EffectJudgeSameColorGainRepeat { player, span } => {
                self.mark(*span, "judge_same_color_gain_repeat");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_307);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_SAME_COLOR_GAIN_REPEAT));
            }
            StmtIr::EffectChooseCurrentTrickTargetsToNullify {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "choose_current_trick_targets_to_nullify");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_308);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY,
                ));
            }
            StmtIr::EffectJudgeBlackGainRepeatProtectHandLimit { player, span } => {
                self.mark(*span, "judge_black_gain_repeat_protect_hand_limit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_309);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT,
                ));
            }
            StmtIr::EffectChooseGainAllResolvedShaShanThenColorRewards {
                player,
                sha_card,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_gain_all_resolved_sha_shan_then_color_rewards",
                );
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_310);
                self.emit_expr(player)?;
                self.emit_expr(sha_card)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_ALL_RESOLVED_SHA_SHAN_THEN_COLOR_REWARDS,
                ));
            }
            StmtIr::EffectGainEventCards { player, span } => {
                self.mark(*span, "gain_event_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_82);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::GAIN_EVENT_CARDS));
            }
            StmtIr::EffectChooseTargetDrawToMaxHpCap { player, cap, span } => {
                self.mark(*span, "choose_target_draw_to_max_hp_cap");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_84);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*cap as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_DRAW_TO_MAX_HP_CAP));
            }
            StmtIr::EffectChooseOtherDrawOwnerLostHpTurnOver { player, span } => {
                self.mark(*span, "choose_other_draw_owner_lost_hp_turn_over");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_85);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_DRAW_OWNER_LOST_HP_TURN_OVER,
                ));
            }
            StmtIr::EffectDiscardOneHandOrEquip { player, span } => {
                self.mark(*span, "discard_one_hand_or_equip");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ONE_HAND_OR_EQUIP));
            }
            StmtIr::EffectDiscardAllHandAndEquip { player, span } => {
                self.mark(*span, "discard_all_hand_and_equip");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_HAND_AND_EQUIP));
            }
            StmtIr::EffectRemoveAllSkills { player, span } => {
                self.mark(*span, "remove_all_skills");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_159);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_ALL_SKILLS));
            }
            StmtIr::EffectJudgeGainEquipmentByColor { player, span } => {
                self.mark(*span, "judge_gain_equipment_by_color");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_GAIN_EQUIPMENT_BY_COLOR));
            }
            StmtIr::EffectJudgeRecoverOnSuit {
                player,
                suit,
                amount,
                span,
            } => {
                self.mark(*span, "judge_recover_on_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_12);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_RECOVER_ON_SUIT));
            }
            StmtIr::EffectJudgeRecoverOtherOnSuit {
                judge_player,
                recover_player,
                suit,
                amount,
                span,
            } => {
                self.mark(*span, "judge_recover_other_on_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_88);
                self.emit_expr(judge_player)?;
                self.emit_expr(recover_player)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_RECOVER_OTHER_ON_SUIT));
            }
            StmtIr::EffectForceOthersUseShaOnNearestOrLoseHp { player, span } => {
                self.mark(*span, "force_others_use_sha_on_nearest_or_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_90);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::FORCE_OTHERS_USE_SHA_ON_NEAREST_OR_LOSE_HP,
                ));
            }
            StmtIr::EffectJudgeDamageOtherOnSuit {
                player,
                suit,
                amount,
                nature,
                span,
            } => {
                self.mark(*span, "judge_damage_other_on_suit");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_15);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops.push(Op::ConstI32(i32::from(*nature)));
                self.ops
                    .push(Op::HostEffect(effect_tag::JUDGE_DAMAGE_OTHER_ON_SUIT));
            }
            StmtIr::EffectRevealDiscardSuitGainOrUse {
                player,
                mark,
                base_count,
                max_mark,
                span,
            } => {
                self.mark(*span, "reveal_discard_suit_gain_or_use");
                self.emit_expr(player)?;
                self.ops
                    .push(i32::try_from(*mark).map_or(Op::ConstU32(*mark), Op::ConstI32));
                self.ops.push(Op::ConstI32(i32::from(*base_count)));
                self.ops.push(Op::ConstI32(i32::from(*max_mark)));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_DISCARD_SUIT_GAIN_OR_USE));
            }
            StmtIr::EffectChooseDiscardOrFireDamageSkipPhases {
                player,
                max_discard,
                damage,
                span,
            } => {
                self.mark(*span, "choose_discard_or_fire_damage_skip_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*max_discard)));
                self.ops.push(Op::ConstI32(i32::from(*damage)));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_OR_FIRE_DAMAGE_SKIP_PHASES,
                ));
            }
            StmtIr::EffectDrawSelected { player, span } => {
                self.mark(*span, "draw_selected");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_SELECTED));
            }
            StmtIr::EffectDrawEachBySeat {
                first,
                second,
                count,
                span,
            } => {
                self.mark(*span, "draw_each_by_seat");
                self.emit_expr(first)?;
                self.emit_expr(second)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_EACH_BY_SEAT));
            }
            StmtIr::EffectDrawThenGiveOne {
                player,
                target,
                count,
                drawn_only,
                use_given_equipment,
                span,
            } => {
                self.mark(*span, "draw_then_give_one");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(count)?;
                self.ops.push(Op::ConstI32(
                    i32::from(*drawn_only) | (i32::from(*use_given_equipment) << 1),
                ));
                if *use_given_equipment {
                    self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_227);
                }
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_THEN_GIVE_ONE));
            }
            StmtIr::EffectDrawThenTakeOne {
                player,
                taker,
                count,
                span,
            } => {
                self.mark(*span, "draw_then_take_one");
                self.emit_expr(player)?;
                self.emit_expr(taker)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_THEN_TAKE_ONE));
            }
            StmtIr::EffectTakeOneCard {
                from,
                to,
                equip_only,
                span,
            } => {
                self.mark(*span, "take_one_card");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::ConstI32(i32::from(*equip_only)));
                self.ops.push(Op::HostEffect(effect_tag::TAKE_ONE_CARD));
            }
            StmtIr::EffectTakeOneScopedCard {
                from,
                to,
                scope,
                span,
            } => {
                self.mark(*span, "take_one_card");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::ConstI32(scope.tag()));
                self.ops
                    .push(Op::HostEffect(effect_tag::TAKE_ONE_SCOPED_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_334);
            }
            StmtIr::EffectRevealHand { player, span } => {
                self.mark(*span, "reveal_hand");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::REVEAL_HAND));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_336);
            }
            StmtIr::EffectDrawLinkedPlayers { count, span } => {
                self.mark(*span, "draw_linked_players");
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_LINKED_PLAYERS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_344);
            }
            StmtIr::EffectTakeOneEventTargetHand { player, span } => {
                self.mark(*span, "take_one_event_target_hand");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::TAKE_ONE_EVENT_TARGET_HAND));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_231);
            }
            StmtIr::EffectTakeOneHandCard { player, span } => {
                self.mark(*span, "take_one_event_target_hand");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_231);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::TAKE_ONE_EVENT_TARGET_HAND));
            }
            StmtIr::EffectChooseOtherHandStoreSkillPile {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_other_hand_store_skill_pile");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_567);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_STORE_SKILL_PILE,
                ));
            }
            StmtIr::EffectGainAllSkillPileCards { player, span } => {
                self.mark(*span, "gain_all_skill_pile_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_569);
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_ALL_SKILL_PILE_CARDS));
            }
            StmtIr::EffectGainOtherSkillPileCards {
                player,
                target,
                skill_def,
                span,
            } => {
                self.mark(*span, "gain_other_skill_pile_cards");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_600);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_OTHER_SKILL_PILE_CARDS));
            }
            StmtIr::EffectChooseDiscardOneHeOrLoseHp {
                player,
                lose_amount,
                span,
            } => {
                self.mark(*span, "choose_discard_one_he_or_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_601);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*lose_amount));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_ONE_HE_OR_LOSE_HP));
            }
            StmtIr::EffectChooseEndExtraDrawOrPlayByUsedSuits { player, span } => {
                self.mark(*span, "choose_end_extra_draw_or_play_by_used_suits");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_END_EXTRA_DRAW_OR_PLAY_BY_USED_SUITS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_232);
            }
            StmtIr::EffectJudgeSpadeTwoToNineDrawTwoOrThunderShaTwice {
                source,
                target,
                span,
            } => {
                self.mark(*span, "judge_spade_two_to_nine_draw_two_or_thunder_sha_twice");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_602);
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_SPADE_TWO_TO_NINE_DRAW_TWO_OR_THUNDER_SHA_TWICE,
                ));
            }
            StmtIr::EffectChooseSourceConfirmOwnerLoseHp {
                owner,
                source,
                amount,
                span,
            } => {
                self.mark(*span, "choose_source_confirm_owner_lose_hp");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_603);
                self.emit_expr(owner)?;
                self.emit_expr(source)?;
                self.ops.push(Op::ConstU32(*amount));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_SOURCE_CONFIRM_OWNER_LOSE_HP));
            }
            StmtIr::EffectLoseHpDoubleCurrentDamageThenReplace {
                player,
                replacement_skill_def,
                span,
            } => {
                self.mark(*span, "lose_hp_double_current_damage_then_replace");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*replacement_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::LOSE_HP_DOUBLE_CURRENT_DAMAGE_THEN_REPLACE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_233);
            }
            StmtIr::EffectTurnOver { player, span } => {
                self.mark(*span, "turn_over");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::TURN_OVER));
            }
            StmtIr::EffectRestoreCharacterState { player, span } => {
                self.mark(*span, "restore_character_state");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESTORE_CHARACTER_STATE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_93);
            }
            StmtIr::EffectDiscardAllHandEquipJudge { player, span } => {
                self.mark(*span, "discard_all_hand_equip_judge");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_HAND_EQUIP_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_93);
            }
            StmtIr::EffectDamageThenDiscardAllJudge { player, span } => {
                self.mark(*span, "damage_then_discard_all_judge");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DAMAGE_THEN_DISCARD_ALL_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_224);
            }
            StmtIr::EffectReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
                span,
            } => {
                self.mark(
                    *span,
                    "replace_draw_gain_top_two_add_skills_if_different_colors",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*first_skill_def));
                self.ops.push(Op::ConstU32(*second_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_DRAW_GAIN_TOP_TWO_ADD_SKILLS_IF_DIFFERENT_COLORS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_225);
            }
            StmtIr::EffectChooseGainTargetCardAlternateZoneThenTargetDraw {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_gain_target_card_alternate_zone_then_target_draw",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_TARGET_CARD_ALTERNATE_ZONE_THEN_TARGET_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_226);
            }
            StmtIr::EffectChooseDiscardTargetCardResolveCurrentSha {
                source,
                card,
                target,
                span,
            } => {
                self.mark(*span, "choose_discard_target_card_resolve_current_sha");
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_94);
            }
            StmtIr::EffectChooseDiscardTargetCardResolveCurrentShaNonbasicBonus {
                source,
                card,
                target,
                bonus_skill_def,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_target_card_resolve_current_sha_nonbasic_bonus",
                );
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*bonus_skill_def as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_TARGET_CARD_RESOLVE_CURRENT_SHA_NONBASIC_BONUS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_312);
            }
            StmtIr::EffectResolveDcqiaomeng {
                player,
                card,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcqiaomeng");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCQIAOMENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_313);
            }
            StmtIr::EffectUseVirtualCardOnSelectedTargets { player, kind, span } => {
                self.mark(*span, "use_virtual_card_on_selected_targets");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops.push(Op::HostEffect(
                    effect_tag::USE_VIRTUAL_CARD_ON_SELECTED_TARGETS,
                ));
                self.required_abi = self.required_abi.max(if *kind == 34 {
                    ADDITIVE_RULE_VM_ABI_VERSION_343
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_95
                });
            }
            StmtIr::EffectDealDamageBanActiveIfDying {
                source,
                target,
                amount,
                skill,
                span,
            } => {
                self.mark(*span, "deal_damage_ban_active_if_dying");
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DEAL_DAMAGE_BAN_ACTIVE_IF_DYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_96);
            }
            StmtIr::EffectSpjungongDiscardOrLoseThenSha {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(*span, "spjungong_discard_or_lose_then_sha");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::SPJUNGONG_DISCARD_OR_LOSE_THEN_SHA,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_400);
            }
            StmtIr::EffectBanActiveUntilTurnEnd {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "ban_active_until_turn_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::BAN_ACTIVE_UNTIL_TURN_END));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_400);
            }
            StmtIr::EffectSpwanweiActive {
                player,
                target,
                span,
            } => {
                self.mark(*span, "spwanwei_active");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::SPWANWEI_ACTIVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_401);
            }
            StmtIr::EffectSpwanweiDying {
                player,
                target,
                span,
            } => {
                self.mark(*span, "spwanwei_dying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::SPWANWEI_DYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_401);
            }
            StmtIr::EffectResolveSpxizhan {
                player,
                phase_player,
                fangzong_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_spxizhan");
                self.emit_expr(player)?;
                self.emit_expr(phase_player)?;
                self.ops.push(Op::ConstI32(*fangzong_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPXIZHAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_403);
            }
            StmtIr::EffectResolveSpsongshu {
                player,
                phase_player,
                renku_skill_def,
                block_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_spsongshu");
                self.emit_expr(player)?;
                self.emit_expr(phase_player)?;
                self.ops.push(Op::ConstI32(*renku_skill_def as i32));
                self.ops.push(Op::ConstI32(*block_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPSONGSHU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_404);
            }
            StmtIr::EffectResolveSpcunsi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spcunsi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPCUNSI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_406);
            }
            StmtIr::EffectResolveYuli { player, mode, span } => {
                self.mark(*span, "resolve_yuli");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YULI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_409);
            }
            StmtIr::EffectResolveTingwei { player, mode, span } => {
                self.mark(*span, "resolve_tingwei");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TINGWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_410);
            }
            StmtIr::EffectResolveXinmingshi {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_xinmingshi");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINMINGSHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_411);
            }
            StmtIr::EffectResolveXunyi { player, mode, span } => {
                self.mark(*span, "resolve_xunyi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XUNYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_412);
            }
            StmtIr::EffectResolveBoming {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_boming");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BOMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_413);
            }
            StmtIr::EffectResolveEjian {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_ejian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_EJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_414);
            }
            StmtIr::EffectResolveYiyong { player, span } => {
                self.mark(*span, "resolve_yiyong");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YIYONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_415);
            }
            StmtIr::EffectResolveShanxie { player, span } => {
                self.mark(*span, "resolve_shanxie");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHANXIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_416);
            }
            StmtIr::EffectResolveSheyi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_sheyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHEYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_417);
            }
            StmtIr::EffectResolveTianyin { player, span } => {
                self.mark(*span, "resolve_tianyin");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIANYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_418);
            }
            StmtIr::EffectResolveDbzhuifeng { player, span } => {
                self.mark(*span, "resolve_dbzhuifeng");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DBZHUIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_419);
            }
            StmtIr::EffectResolveDbquedi {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_dbquedi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DBQUEDI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_420);
            }
            StmtIr::EffectResolveSpyinju {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spyinju");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPYINJU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_421);
            }
            StmtIr::EffectResolveSpchijie {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_spchijie");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPCHIJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_422);
            }
            StmtIr::EffectResolveSpshangyi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spshangyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPSHANGYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_423);
            }
            StmtIr::EffectResolveSpjianyi { player, span } => {
                self.mark(*span, "resolve_spjianyi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPJIANYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_424);
            }
            StmtIr::EffectResolveSpyajun {
                player,
                less_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_spyajun");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_428);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*less_def as i32));
                self.ops.push(Op::ConstI32(*stage as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPYAJUN));
            }
            StmtIr::EffectResolveSpzundi {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spzundi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPZUNDI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_429);
            }
            StmtIr::EffectResolveXinyangjie {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_xinyangjie");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINYANGJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_430);
            }
            StmtIr::EffectResolveQingyu {
                player,
                action,
                xuancun_def,
                mbyongjue_def,
                span,
            } => {
                self.mark(*span, "resolve_qingyu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_431);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(*xuancun_def as i32));
                self.ops.push(Op::ConstI32(*mbyongjue_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINGYU));
            }
            StmtIr::EffectResolveZhengsu {
                player,
                mode,
                stage,
                card,
                span,
            } => {
                self.mark(*span, "resolve_zhengsu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_432);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGSU));
            }
            StmtIr::EffectResolveHoufeng {
                player,
                target,
                mode,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_houfeng");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_433);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HOUFENG));
            }
            StmtIr::EffectResolveMbdiancai {
                player,
                actor,
                span,
            } => {
                self.mark(*span, "resolve_mbdiancai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_434);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBDIANCAI));
            }
            StmtIr::EffectResolveRechuhai {
                player,
                target,
                card,
                target_card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_rechuhai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_435);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.emit_expr(target_card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RECHUHAI));
            }
            StmtIr::EffectResolveXiongshu {
                player,
                target,
                card,
                action,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_xiongshu");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_436);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIONGSHU));
            }
            StmtIr::EffectResolveJianhui {
                player,
                source,
                target,
                span,
            } => {
                self.mark(*span, "resolve_jianhui");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_437);
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JIANHUI));
            }
            StmtIr::EffectResolveSpqiai {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spqiai");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_439);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPQIAI));
            }
            StmtIr::EffectResolveSpshanxi {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_spshanxi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_440);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPSHANXI));
            }
            StmtIr::EffectResolveSpshanxiSuoming {
                player,
                actor,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_spshanxi_suoming");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_441);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SPSHANXI_SUOMING));
            }
            StmtIr::EffectResolveTongduo {
                player,
                target,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_tongduo");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_442);
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TONGDUO));
            }
            StmtIr::EffectResolveYuanqing { player, span } => {
                self.mark(*span, "resolve_yuanqing");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_443);
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YUANQING));
            }
            StmtIr::EffectResolveShuchen {
                player,
                dying,
                renku_def,
                span,
            } => {
                self.mark(*span, "resolve_shuchen");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_444);
                self.emit_expr(player)?;
                self.emit_expr(dying)?;
                self.ops.push(Op::ConstI32(*renku_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHUCHEN));
            }
            StmtIr::EffectResolveDuanbi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_duanbi");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_445);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUANBI));
            }
            StmtIr::EffectResolveZaoli { player, mark, span } => {
                self.mark(*span, "resolve_zaoli");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_446);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*mark as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZAOLI));
            }
            StmtIr::EffectResolveGuying {
                player,
                actor,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_guying");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_447);
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUYING));
            }
            StmtIr::EffectResolveFyjianyu { player, a, b, span } => {
                self.mark(*span, "resolve_fyjianyu");
                self.emit_expr(player)?;
                self.emit_expr(a)?;
                self.emit_expr(b)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FYJIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_425);
            }
            StmtIr::EffectResolveLiubing { player, card, span } => {
                self.mark(*span, "resolve_liubing");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIUBING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_426);
            }
            StmtIr::EffectResolveHeji {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_heji");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HEJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_427);
            }
            StmtIr::EffectResolveTwjiaohua {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_twjiaohua");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJIAOHUA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_363);
            }
            StmtIr::EffectResolveTwkujian {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twkujian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWKUJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_364);
            }
            StmtIr::EffectResolveTwruilian {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twruilian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWRUILIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_365);
            }
            StmtIr::EffectEquipWeaponFromDrawPile { player, span } => {
                self.mark(*span, "equip_weapon_from_draw_pile");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EQUIP_WEAPON_FROM_DRAW_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_366);
            }
            StmtIr::EffectResolveTwjianwei {
                player,
                actor,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_twjianwei");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJIANWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_367);
            }
            StmtIr::EffectResolveTwtanfeng { player, span } => {
                self.mark(*span, "resolve_twtanfeng");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWTANFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_368);
            }
            StmtIr::EffectResolveTwzhibian { player, span } => {
                self.mark(*span, "resolve_twzhibian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWZHIBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_369);
            }
            StmtIr::EffectResolveTwyuyan {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "resolve_twyuyan");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWYUYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_370);
            }
            StmtIr::EffectResolveTwguoyi {
                player,
                target,
                card,
                hand_lock_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_twguoyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*hand_lock_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWGUOYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_371);
            }
            StmtIr::EffectResolveTwchuhai {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchuhai");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCHUHAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_372);
            }
            StmtIr::EffectResolveTwyizhu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twyizhu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWYIZHU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_373);
            }
            StmtIr::EffectResolveTwluanchou {
                player,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twluanchou");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWLUANCHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_374);
            }
            StmtIr::EffectResolveTwxiawei {
                player,
                wangxing_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twxiawei");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*wangxing_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWXIAWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_375);
            }
            StmtIr::EffectResolveTwbianfuren {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twbianfuren");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWBIANFUREN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_376);
            }
            StmtIr::EffectResolveTwfeiyi {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twfeiyi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFEIYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_377);
            }
            StmtIr::EffectResolveTwchenzhen {
                player,
                target,
                helper_skill_def,
                record_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchenzhen");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(*record_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWCHENZHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_378);
            }
            StmtIr::EffectResolveTwqianju {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_twqianju");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQIANJU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_379);
            }
            StmtIr::EffectResolveTwqingxi {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_twqingxi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQINGXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_379);
            }
            StmtIr::EffectResolveTwzaoli { player, span } => {
                self.mark(*span, "resolve_twzaoli");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWZAOLI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_380);
            }
            StmtIr::EffectResolveTwzhengjian {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twzhengjian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWZHENGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_381);
            }
            StmtIr::EffectResolveTwtianyu {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twtianyu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWTIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_382);
            }
            StmtIr::EffectResolveTwwujing {
                player,
                target_count,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twwujing");
                self.emit_expr(player)?;
                self.emit_expr(target_count)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWWUJING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_384);
            }
            StmtIr::EffectResolveTwshanghe { player, span } => {
                self.mark(*span, "resolve_twshanghe");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWSHANGHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_385);
            }
            StmtIr::EffectResolveTwwangchang {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twwangchang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TWWANGCHANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_386);
            }
            StmtIr::EffectResolveTwcaozhao {
                player,
                target,
                source,
                card,
                twfuzuan_skill_def,
                twfeifu_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twcaozhao");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*twfuzuan_skill_def as i32));
                self.ops.push(Op::ConstI32(*twfeifu_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCAOZHAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_387);
            }
            StmtIr::EffectResolveTwguohuai {
                player,
                target,
                source,
                card,
                mark_def,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twguohuai");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*mark_def as i32));
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWGUOHUAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_388);
            }
            StmtIr::EffectResolveTwchengpu {
                player,
                target,
                source,
                card,
                chunlao_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchengpu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*chunlao_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCHENGPU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_389);
            }
            StmtIr::EffectResolveTwcaoang {
                player,
                target,
                source,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twcaoang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCAOANG));
                self.required_abi = self.required_abi.max(if *action == 2 {
                    ADDITIVE_RULE_VM_ABI_VERSION_392
                } else {
                    ADDITIVE_RULE_VM_ABI_VERSION_390
                });
            }
            StmtIr::EffectResolveTwniujin {
                player,
                limited_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twniujin");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*limited_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWNIUJIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_391);
            }
            StmtIr::EffectResolveTwmoukui {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_twmoukui");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMOUKUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_393);
            }
            StmtIr::EffectResolveTwfuman {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twfuman");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFUMAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_394);
            }
            StmtIr::EffectResolveCuijin {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_cuijin");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CUIJIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_395);
            }
            StmtIr::EffectResolveTwqingkou {
                player,
                winner,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twqingkou");
                self.emit_expr(player)?;
                self.emit_expr(winner)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWQINGKOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_396);
            }
            StmtIr::EffectResolveTwchenxun {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twchenxun");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWCHENXUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_397);
            }
            StmtIr::EffectResolveTwfenxian {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_twfenxian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWFENXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_399);
            }
            StmtIr::EffectSetSkillContentFlag {
                player,
                bit,
                value,
                span,
            } => {
                self.mark(*span, "set_skill_content_flag");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*bit)));
                self.ops.push(Op::ConstI32(i32::from(*value)));
                self.ops
                    .push(Op::HostEffect(effect_tag::SET_SKILL_CONTENT_FLAG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_490);
            }
            StmtIr::EffectChooseTargetReclaimEquipmentDrawIfRangeReduced {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_target_reclaim_equipment_draw_if_range_reduced",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_RECLAIM_EQUIPMENT_DRAW_IF_RANGE_REDUCED,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_97);
            }
            StmtIr::EffectChooseUseVirtualCard {
                player,
                kind,
                skill,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_card");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_98);
            }
            StmtIr::EffectChooseUseVirtualNanman {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_nanman");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_NANMAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_604);
            }
            StmtIr::EffectResolveQiluan {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_qiluan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_QILUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_605);
            }
            StmtIr::EffectDiscardAllOwnJudge {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_all_own_judge");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_OWN_JUDGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_606);
            }
            StmtIr::EffectStoreEventCardInSkillPile {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "store_event_card_in_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::STORE_EVENT_CARD_IN_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_607);
            }
            StmtIr::EffectResolveOllinjie {
                player,
                source,
                span,
            } => {
                self.mark(*span, "resolve_ollinjie");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_OLLINJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_608);
            }
            StmtIr::EffectChooseTargetGainMaxHp {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_target_gain_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_GAIN_MAX_HP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_646);
            }
            StmtIr::EffectDiscardBasicDrawEqual {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "discard_basic_draw_equal");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_BASIC_DRAW_EQUAL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_657);
            }
            StmtIr::EffectStartDebateAllOthers {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "start_debate_all_others");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::START_DEBATE_ALL_OTHERS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_655);
            }
            StmtIr::EffectStartDebateTargets {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "start_debate_targets");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::START_DEBATE_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_656);
            }
            StmtIr::EffectPinDebateOpinions {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "pin_debate_opinions");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PIN_DEBATE_OPINIONS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_658);
            }
            StmtIr::EffectAlignDebateOpinions {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "align_debate_opinions");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::ALIGN_DEBATE_OPINIONS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_659);
            }
            StmtIr::EffectGainShadowCards {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "gain_shadow_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_SHADOW_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_660);
            }
            StmtIr::EffectChangeGroup {
                player,
                group,
                skill,
                span,
            } => {
                self.mark(*span, "change_group");
                self.emit_expr(player)?;
                let code = match group.as_str() {
                    "wei" => 0,
                    "shu" => 1,
                    "wu" => 2,
                    "qun" => 3,
                    "jin" => 4,
                    _ => 0,
                };
                self.ops.push(Op::ConstI32(code));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHANGE_GROUP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_661);
            }
            StmtIr::EffectDistributeShadowCards {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "distribute_shadow_cards");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISTRIBUTE_SHADOW_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_662);
            }
            StmtIr::EffectRegisterTurnEndDebate {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "register_turn_end_debate");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REGISTER_TURN_END_DEBATE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_663);
            }
            StmtIr::EffectResolveTurnEndDebates {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_turn_end_debates");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_TURN_END_DEBATES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_663);
            }
            StmtIr::EffectChooseGroup {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_group");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GROUP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_664);
            }
            StmtIr::EffectChooseGainOtherSkill {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_gain_other_skill");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_GAIN_OTHER_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_675);
            }
            StmtIr::EffectLoseGainedSkill {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "lose_gained_skill");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::LOSE_GAINED_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_676);
            }
            StmtIr::EffectResolvePotheyun {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_potheyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTHEYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_677);
            }
            StmtIr::EffectPlaceXumou {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "place_xumou");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_665);
            }
            StmtIr::EffectPlaceDeckTopXumou { player, skill, span } => {
                self.mark(*span, "place_deck_top_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_DECK_TOP_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectChoosePlaceHandXumou { player, skill, span } => {
                self.mark(*span, "choose_place_hand_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_PLACE_HAND_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectDiscardOneOwnXumou { player, skill, span } => {
                self.mark(*span, "discard_one_own_xumou");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ONE_OWN_XUMOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_666);
            }
            StmtIr::EffectPlaceDeckTopExpansion {
                player,
                count,
                skill,
                span,
            } => {
                self.mark(*span, "place_deck_top_expansion");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*count));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_DECK_TOP_EXPANSION));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_667);
            }
            StmtIr::EffectCityConsume { player, skill, span } => {
                self.mark(*span, "city_consume");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CITY_CONSUME));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_668);
            }
            StmtIr::EffectQueueExtraTurn {
                player,
                check,
                skill,
                span,
            } => {
                self.mark(*span, "queue_extra_turn");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*check));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QUEUE_EXTRA_TURN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_669);
            }
            StmtIr::EffectChooseOthersInsertExtraTurn {
                player,
                check,
                skill,
                span,
            } => {
                self.mark(*span, "choose_others_insert_extra_turn");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*check));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHERS_INSERT_EXTRA_TURN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_671);
            }
            StmtIr::EffectQueueExtraPhases {
                player,
                phases_mask,
                skill,
                span,
            } => {
                self.mark(*span, "queue_extra_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*phases_mask));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QUEUE_EXTRA_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_672);
            }
            StmtIr::EffectResolveTwjieqiu {
                player,
                target,
                subaction,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_twjieqiu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*subaction));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWJIEQIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_673);
            }
            StmtIr::EffectSkipRemainingPhases {
                player,
                phases_mask,
                skill,
                span,
            } => {
                self.mark(*span, "skip_remaining_phases");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*phases_mask));
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::SKIP_REMAINING_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_674);
            }
            StmtIr::EffectChooseOthersAddMark {
                player,
                mark,
                delta,
                max_count,
                skill,
                span,
            } => {
                self.mark(*span, "choose_others_add_mark");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(*delta));
                self.ops.push(Op::ConstU32(*max_count));
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHERS_ADD_MARK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_670);
            }
            StmtIr::EffectResolveJsrgchaozheng {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgchaozheng");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGCHAOZHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_653);
            }
            StmtIr::EffectResolveJsrgsheju {
                player,
                counterpart,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_jsrgsheju");
                self.emit_expr(player)?;
                self.emit_expr(counterpart)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGSHEJU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_654);
            }
            StmtIr::EffectChooseOtherAddMark {
                player,
                mark,
                delta,
                span,
            } => {
                self.mark(*span, "choose_other_add_mark");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*mark));
                self.ops.push(Op::ConstI32(*delta));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_ADD_MARK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_652);
            }
            StmtIr::EffectGainWholeSkillPile {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "gain_whole_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops.push(Op::HostEffect(effect_tag::GAIN_WHOLE_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_687);
            }
            StmtIr::EffectPlaceHandCardsToDeckTop {
                player,
                skill,
                count,
                span,
            } => {
                self.mark(*span, "place_hand_cards_to_deck_top");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_HAND_CARDS_TO_DECK_TOP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_689);
            }
            StmtIr::EffectRearrangePileToDeckTop {
                player,
                skill,
                pile_def,
                count,
                span,
            } => {
                self.mark(*span, "rearrange_pile_to_deck_top");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::REARRANGE_PILE_TO_DECK_TOP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_690);
            }
            StmtIr::EffectResolveQianlong { player, skill, span } => {
                self.mark(*span, "resolve_qianlong");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANLONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_697);
            }
            StmtIr::EffectResolveJsrgzuozhan { player, skill, span } => {
                self.mark(*span, "resolve_jsrgzuozhan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JSRGZUOZHAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_698);
            }
            StmtIr::EffectResolveJsrgzuozhanGain {
                player,
                skill,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_jsrgzuozhan_gain");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGZUOZHAN_GAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_699);
            }
            StmtIr::EffectResolveTwmouzhi {
                player,
                skill,
                mode,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_twmouzhi");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMOUZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_700);
            }
            StmtIr::EffectResolveTwdaigui {
                player,
                skill,
                stage,
                count,
                span,
            } => {
                self.mark(*span, "resolve_twdaigui");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWDAIGUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_701);
            }
            StmtIr::EffectResolveTwmitu {
                player,
                skill,
                stage,
                count,
                span,
            } => {
                self.mark(*span, "resolve_twmitu");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TWMITU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_702);
            }
            StmtIr::EffectResolveQiantun { player, skill, span } => {
                self.mark(*span, "resolve_qiantun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIANTUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_703);
            }
            StmtIr::EffectResolveYangming { player, skill, span } => {
                self.mark(*span, "resolve_yangming");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_YANGMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_704);
            }
            StmtIr::EffectResolveCanyun { player, skill, span } => {
                self.mark(*span, "resolve_canyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CANYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_705);
            }
            StmtIr::EffectResolveHengwu {
                player,
                skill,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_hengwu");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HENGWU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_706);
            }
            StmtIr::EffectResolveBiaozhao3 { player, skill, span } => {
                self.mark(*span, "resolve_biaozhao3");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIAOZHAO3));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_707);
            }
            StmtIr::EffectResolveChuanyun {
                player,
                skill,
                event_player,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_chuanyun");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHUANYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_708);
            }
            StmtIr::EffectResolveBiaozhao2 {
                player,
                skill,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_biaozhao2");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIAOZHAO2));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_709);
            }
            StmtIr::EffectResolveDcniji {
                player,
                skill,
                mode,
                event_card,
                span,
            } => {
                self.mark(*span, "resolve_dcniji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(event_card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCNIJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_710);
            }
            StmtIr::EffectResolveDcmoukui {
                player,
                skill,
                stage,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcmoukui");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMOUKUI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_711);
            }
            StmtIr::EffectResolveZhongyan { player, skill, stage, span } => {
                self.mark(*span, "resolve_zhongyan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHONGYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_712);
            }
            StmtIr::EffectResolveChumei { player, skill, stage, span } => {
                self.mark(*span, "resolve_chumei");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CHUMEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_713);
            }
            StmtIr::EffectResolveShuaiyan { player, skill, stage, span } => {
                self.mark(*span, "resolve_shuaiyan");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SHUAIYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_715);
            }            StmtIr::EffectResolveMbbeiming { player, skill, stage, span } => {
                self.mark(*span, "resolve_mbbeiming");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBBEIMING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_718);
            }
            StmtIr::EffectResolveReliangyin { player, skill, mode, span } => {
                self.mark(*span, "resolve_reliangyin");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*mode));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RELIANGYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_717);
            }
            StmtIr::EffectResolveZhengjian {
                player,
                skill,
                stage,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_zhengjian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_716);
            }
            StmtIr::EffectResolveQinmian {
                player,
                skill,
                mode,
                amount,
                span,
            } => {
                self.mark(*span, "resolve_qinmian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(mode)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QINMIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_714);
            }
            StmtIr::EffectResolveJsrgdaoren { player, skill, span } => {
                self.mark(*span, "resolve_jsrgdaoren");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_JSRGDAOREN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_696);
            }
            StmtIr::EffectResolveDuwang { player, skill, span } => {
                self.mark(*span, "resolve_duwang");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DUWANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_695);
            }
            StmtIr::EffectResolveQuanjia {
                player,
                skill,
                source,
                target,
                span,
            } => {
                self.mark(*span, "resolve_quanjia");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(source)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QUANJIA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_694);
            }
            StmtIr::EffectResolvePxqixian {
                player,
                skill,
                stage,
                kind,
                span,
            } => {
                self.mark(*span, "resolve_pxqixian");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(kind)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_PXQIXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_693);
            }
            StmtIr::EffectResolveDcbingji {
                player,
                skill,
                stage,
                kind,
                span,
            } => {
                self.mark(*span, "resolve_dcbingji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(stage)?;
                self.emit_expr(kind)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCBINGJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_692);
            }
            StmtIr::EffectChooseHandCardsToSkillPile {
                player,
                skill,
                min_count,
                max_count,
                span,
            } => {
                self.mark(*span, "choose_hand_cards_to_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.emit_expr(min_count)?;
                self.emit_expr(max_count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_HAND_CARDS_TO_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_691);
            }
            StmtIr::EffectChooseTargetHandCardToPile {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "choose_target_hand_card_to_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_TARGET_HAND_CARD_TO_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_688);
            }
            StmtIr::EffectChooseOneFromSkillPileDiscard {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "choose_one_from_skill_pile_discard");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_ONE_FROM_SKILL_PILE_DISCARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_686);
            }
            StmtIr::EffectPlaceTopDeckToExpansion {
                player,
                skill,
                pile_def,
                count,
                span,
            } => {
                self.mark(*span, "place_top_deck_to_expansion");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::PLACE_TOP_DECK_TO_EXPANSION));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_685);
            }
            StmtIr::EffectDiscardSkillPile {
                player,
                skill,
                pile_def,
                span,
            } => {
                self.mark(*span, "discard_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstU32(*pile_def));
                self.ops.push(Op::HostEffect(effect_tag::DISCARD_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_684);
            }
            StmtIr::EffectGainFromSkillPile {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "gain_from_skill_pile");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::GAIN_FROM_SKILL_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_651);
            }
            StmtIr::EffectPlaceHandCardToExpansion {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "place_hand_card_to_expansion");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::PLACE_HAND_CARD_TO_EXPANSION,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_650);
            }
            StmtIr::EffectExtraResolveCurrentTrick {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "extra_resolve_current_trick");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::EXTRA_RESOLVE_CURRENT_TRICK));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_649);
            }
            StmtIr::EffectChooseClanNotMaxHandLimitGain {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "choose_clan_not_max_hand_limit_gain");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CLAN_NOT_MAX_HAND_LIMIT_GAIN,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_648);
            }
            StmtIr::EffectGainTargetHandCard {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "gain_target_hand_card");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::GAIN_TARGET_HAND_CARD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_647);
            }
            StmtIr::EffectIncreaseCurrentShaBaseDamage {
                player,
                card,
                amount,
                span,
            } => {
                self.mark(*span, "increase_current_sha_base_damage");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::INCREASE_CURRENT_SHA_BASE_DAMAGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_99);
            }
            StmtIr::EffectSkipNextPlayPhase { player, span } => {
                self.mark(*span, "skip_next_play_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_PLAY_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_100);
            }
            StmtIr::EffectSkipNextJudgePhase { player, span } => {
                self.mark(*span, "skip_next_judge_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_JUDGE_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_228);
            }
            StmtIr::EffectSkipJudgeAndDrawPhases { player, span } => {
                self.mark(*span, "skip_judge_and_draw_phases");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_JUDGE_AND_DRAW_PHASES));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_450);
            }
            StmtIr::EffectChooseDiscardDamageEndpointCardIncreaseIfSuit {
                player,
                target,
                suit,
                amount,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_discard_damage_endpoint_card_increase_if_suit",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*suit)));
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_DAMAGE_ENDPOINT_CARD_INCREASE_IF_SUIT,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_230);
            }
            StmtIr::EffectChooseOtherDrawOrRecover {
                player,
                draw_count,
                recover_count,
                times,
                span,
            } => {
                self.mark(*span, "choose_other_draw_or_recover");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(recover_count)?;
                self.emit_expr(times)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_DRAW_OR_RECOVER));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_101);
            }
            StmtIr::EffectChooseOtherRecover {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "choose_other_recover");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_RECOVER));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_160);
            }
            StmtIr::EffectEnableEquipSlot {
                player,
                slot,
                source,
                span,
            } => {
                self.mark(*span, "enable_equip_slot");
                self.emit_expr(player)?;
                self.emit_expr(slot)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::ENABLE_EQUIP_SLOT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_161);
            }
            StmtIr::EffectChooseDiscardHandEquipThenRemoveSelfSkill {
                player,
                count,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_remove_self_skill");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_REMOVE_SELF_SKILL,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_102);
            }
            StmtIr::EffectResetLinkedSelectedTargets { source, span } => {
                self.mark(*span, "reset_linked_selected_targets");
                self.emit_expr(source)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESET_LINKED_SELECTED_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_719);
            }
            StmtIr::EffectDrawSelectedTargets { count, span } => {
                self.mark(*span, "draw_selected_targets");
                self.emit_expr(count)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DRAW_SELECTED_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_103);
            }
            StmtIr::EffectRecoverIfMinCurrentHp {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "recover_if_min_current_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECOVER_IF_MIN_CURRENT_HP));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_103);
            }
            StmtIr::EffectIncreaseCurrentCardBaseDamage {
                player,
                card,
                amount,
                span,
            } => {
                self.mark(*span, "increase_current_card_base_damage");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::INCREASE_CURRENT_CARD_BASE_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_104);
            }
            StmtIr::EffectModifyCurrentDamage {
                player,
                delta,
                span,
            } => {
                self.mark(*span, "modify_current_damage");
                self.emit_expr(player)?;
                self.emit_expr(delta)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::MODIFY_CURRENT_DAMAGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_562);
            }
            StmtIr::EffectLoseHpThenDamageOtherLoseMaxHpIfDying {
                player,
                target,
                amount,
                skill,
                span,
            } => {
                self.mark(*span, "lose_hp_then_damage_other_lose_max_hp_if_dying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::LOSE_HP_THEN_DAMAGE_OTHER_LOSE_MAX_HP_IF_DYING,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_108);
            }
            StmtIr::EffectRecoverSelectedTargets { amount, span } => {
                self.mark(*span, "recover_selected_targets");
                self.emit_expr(amount)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RECOVER_SELECTED_TARGETS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_109);
            }
            StmtIr::EffectRecordSelectedPlayPhaseCategory { skill, span } => {
                self.mark(*span, "record_selected_play_phase_category");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::RECORD_SELECTED_PLAY_PHASE_CATEGORY,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_110);
            }
            StmtIr::EffectTakeTargetHandThenRepeatUntilMax {
                player,
                target,
                skill,
                span,
            } => {
                self.mark(*span, "take_target_hand_then_repeat_until_max");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TAKE_TARGET_HAND_THEN_REPEAT_UNTIL_MAX,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_113);
            }
            StmtIr::EffectChooseDrawOrIncreaseCurrentCardDamageOrBothMark {
                player,
                card,
                draw_count,
                damage_amount,
                mark_def,
                mark_count,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_draw_or_increase_current_card_damage_or_both_mark",
                );
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(damage_amount)?;
                self.ops.push(Op::ConstU32(*mark_def));
                self.emit_expr(mark_count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DRAW_OR_INCREASE_CURRENT_CARD_DAMAGE_OR_BOTH_MARK,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_114);
            }
            StmtIr::EffectDrawThenChooseGiveOrDiscardHand {
                player,
                draw_count,
                payment_count,
                span,
            } => {
                self.mark(*span, "draw_then_choose_give_or_discard_hand");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(payment_count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::DRAW_THEN_CHOOSE_GIVE_OR_DISCARD_HAND,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_116);
            }
            StmtIr::EffectGainTargetEquipment {
                player,
                target,
                slot,
                span,
            } => {
                self.mark(*span, "gain_target_equipment");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*slot)));
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_TARGET_EQUIPMENT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_117);
            }
            StmtIr::EffectDiscardAllTargetEquipment {
                actor,
                target,
                span,
            } => {
                self.mark(*span, "discard_all_target_equipment");
                self.emit_expr(actor)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_ALL_TARGET_EQUIPMENT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_118);
            }
            StmtIr::EffectIncreaseCurrentShaTargetDamage {
                source,
                card,
                target,
                amount,
                span,
            } => {
                self.mark(*span, "increase_current_sha_target_damage");
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::INCREASE_CURRENT_SHA_TARGET_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_119);
            }
            StmtIr::EffectChooseTargetDrawThenOwnerIfBelowMaxHp {
                player,
                count,
                span,
            } => {
                self.mark(*span, "choose_target_draw_then_owner_if_below_max_hp");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DRAW_THEN_OWNER_IF_BELOW_MAX_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_120);
            }
            StmtIr::EffectChooseBoundSourceAttackRangeDamage {
                player,
                source,
                amount,
                span,
            } => {
                self.mark(*span, "choose_bound_source_attack_range_damage");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_BOUND_SOURCE_ATTACK_RANGE_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_121);
            }
            StmtIr::EffectChooseTargetDrawMaxHpCapThenDiscardExcess { player, cap, span } => {
                self.mark(*span, "choose_target_draw_max_hp_cap_then_discard_excess");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*cap as i32));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_DISCARD_EXCESS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_122);
            }
            StmtIr::EffectChooseTargetDrawMaxHpCapThenAddSkill {
                player,
                cap,
                skill_def,
                span,
            } => {
                self.mark(*span, "choose_target_draw_to_max_hp_cap_then_add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*cap as i32));
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_455);
            }
            StmtIr::EffectResolveTianxiang {
                player,
                draw_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_heart_redirect_current_damage");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*draw_skill_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_TIANXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_456);
            }
            StmtIr::EffectResolveRetianxiang { player, span } => {
                self.mark(*span, "choose_discard_heart_cancel_damage_branch");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_RETIANXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_457);
            }
            StmtIr::EffectResolveLianzhou { player, span } => {
                self.mark(*span, "choose_link_other_same_hp");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LIANZHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_458);
            }
            StmtIr::EffectResolveStddechong { player, span } => {
                self.mark(*span, "choose_give_hand_equip_to_event_player");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STDDECHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_459);
            }
            StmtIr::EffectDiscardHandEquipCount {
                player,
                count,
                span,
            } => {
                self.mark(*span, "discard_hand_equip_count");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*count as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::DISCARD_HAND_EQUIP_COUNT));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_460);
            }
            StmtIr::EffectResolveJieliangGain { player, span } => {
                self.mark(*span, "resolve_jieliang_gain");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JIELIANG_GAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_461);
            }
            StmtIr::EffectResolveStdjujian { player, span } => {
                self.mark(*span, "choose_other_gain_event_cards");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_STDJUJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_462);
            }
            StmtIr::EffectResolveLulve { player, span } => {
                self.mark(*span, "resolve_lulve");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_LULVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_463);
            }
            StmtIr::EffectResolveDctingxian { player, span } => {
                self.mark(*span, "resolve_dctingxian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCTINGXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_464);
            }
            StmtIr::EffectResolveHfjieying {
                player,
                grant_def,
                span,
            } => {
                self.mark(*span, "choose_other_add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*grant_def));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HFJIEYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_465);
            }
            StmtIr::EffectResolveDcnuchen { player, span } => {
                self.mark(*span, "resolve_dcnuchen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCNUCHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_466);
            }
            StmtIr::EffectResolveDcdanji {
                player,
                skill,
                span,
            } => {
                self.mark(*span, "resolve_dcdanji");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_466);
            }
            StmtIr::EffectResolveDczhiheng { player, span } => {
                self.mark(*span, "resolve_dczhiheng");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCZHIHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_467);
            }
            StmtIr::EffectResolveStdgaobian { player, span } => {
                self.mark(*span, "resolve_stdgaobian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_STDGAOBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_468);
            }
            StmtIr::EffectResolveSpjiedao { player, span } => {
                self.mark(*span, "resolve_spjiedao");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPJIEDAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_469);
            }
            StmtIr::EffectResolveSibian { player, span } => {
                self.mark(*span, "resolve_sibian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SIBIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_470);
            }
            StmtIr::EffectRevealChooseCategoryDistributeSelfThresholdHpLoss {
                player,
                reveal_count,
                recipient_cap,
                span,
            } => {
                self.mark(
                    *span,
                    "reveal_choose_category_distribute_self_threshold_hp_loss",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*reveal_count)));
                self.ops.push(Op::ConstI32(i32::from(*recipient_cap)));
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_CHOOSE_CATEGORY_DISTRIBUTE_SELF_THRESHOLD_HP_LOSS,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_491);
            }
            StmtIr::EffectChooseOtherRandomGainRegionThenDrawCompareColorLoseHp {
                player,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_other_random_gain_region_then_draw_compare_color_lose_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_RANDOM_GAIN_REGION_THEN_DRAW_COMPARE_COLOR_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_492);
            }
            StmtIr::EffectResolveRefuhai { player, span } => {
                self.mark(*span, "resolve_refuhai");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REFUHAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_493);
            }
            StmtIr::EffectChooseGainTargetHandCeilHalfDifference {
                player,
                target,
                span,
            } => {
                self.mark(*span, "choose_gain_target_hand_ceil_half_difference");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GAIN_TARGET_HAND_CEIL_HALF_DIFFERENCE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_494);
            }
            StmtIr::EffectResolveGzjinfa {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_gzjinfa");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GZJINFA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_495);
            }
            StmtIr::EffectResolveWnQifeng {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_wn_qifeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WN_QIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_496);
            }
            StmtIr::EffectResolveDcjue {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcjue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCJUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_583);
            }
            StmtIr::EffectResolveRuanji {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ruanji");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_RUANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_584);
            }
            StmtIr::EffectResolveZhengbi {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhengbi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHENGBI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_585);
            }
            StmtIr::EffectResolveFengying {
                player,
                helper_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_fengying");
                self.emit_expr(player)?;
                self.emit_expr(helper_def)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_FENGYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_585);
            }
            StmtIr::EffectResolveQiaojian {
                player,
                target,
                helper_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_qiaojian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(helper_def)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_QIAOJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_586);
            }
            StmtIr::EffectResolveXicha {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_xicha");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XICHA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_586);
            }
            StmtIr::EffectResolveHuzun {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_huzun");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_HUZUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_587);
            }
            StmtIr::EffectResolveZhugemengxue {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhugemengxue");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_ZHUGEMENGXUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_588);
            }
            StmtIr::EffectResolveBailingyun {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_bailingyun");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_BAILINGYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_593);
            }
            StmtIr::EffectResolveDcshaowei {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dcshaowei");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCSHAOWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_594);
            }
            StmtIr::EffectResolveDcdichou {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcdichou");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDICHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_594);
            }
            StmtIr::EffectResolveDcdufeng {
                player,
                action,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_dcdufeng");
                self.emit_expr(player)?;
                self.emit_expr(action)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCDUFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_595);
            }
            StmtIr::EffectChooseDiscardHandRange {
                player,
                min,
                max,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_range");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*min as i32));
                self.ops.push(Op::ConstI32(*max as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DISCARD_HAND_RANGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_596);
            }
            StmtIr::EffectResolveDcctjiuxian {
                player,
                event_player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dcctjiuxian");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCCTJIUXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_597);
            }
            StmtIr::EffectResolveBianyue {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_bianyue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_BIANYUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_598);
            }
            StmtIr::EffectResolveZhupeilan {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhupeilan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHUPEILAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_600);
            }
            StmtIr::EffectResolveDcSbZhangxiu {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_zhangxiu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_ZHANGXIU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_601);
            }
            StmtIr::EffectResolveDcSbZhouyu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_zhouyu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_ZHOUYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_602);
            }
            StmtIr::EffectResolveDcSbLusu {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dc_sb_lusu");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DC_SB_LUSU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_603);
            }
            StmtIr::EffectResolveZhangjian {
                player,
                card,
                grant_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhangjian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*grant_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHANGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_604);
            }
            StmtIr::EffectResolveZhugeruoxue {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_zhugeruoxue");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_ZHUGERUOXUE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_605);
            }
            StmtIr::EffectResolveCaoyi {
                player,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_caoyi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_CAOYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_606);
            }
            StmtIr::EffectResolveMalingli {
                player,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_malingli");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MALINGLI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_607);
            }
            StmtIr::EffectResolveNsshuaiyan {
                player,
                recovered_player,
                span,
            } => {
                self.mark(*span, "resolve_nsshuaiyan");
                self.emit_expr(player)?;
                self.emit_expr(recovered_player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NSSHUAIYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_608);
            }
            StmtIr::EffectResolveWdbHanlong {
                player,
                target,
                card,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_hanlong");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_HANLONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_609);
            }
            StmtIr::EffectResolveWdbLiufu {
                player,
                target,
                card,
                zhucheng_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liufu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstU32(*zhucheng_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUFU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_610);
            }
            StmtIr::EffectResolveWdbLiuyan {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liuyan");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUYAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_611);
            }
            StmtIr::EffectResolveWdbLiuzan {
                player,
                target,
                helper_skill_def,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_liuzan");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*helper_skill_def));
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_LIUZAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_612);
            }
            StmtIr::EffectResolveWdbTianyu {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_tianyu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_TIANYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_613);
            }
            StmtIr::EffectResolveWdbXizhenxihong {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_xizhenxihong");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_XIZHENXIHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_614);
            }
            StmtIr::EffectResolveWdbYangyi {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_yangyi");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_YANGYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_615);
            }
            StmtIr::EffectResolveWdbZaozhiRenjun {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_wdb_zaozhirenjun");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_WDB_ZAOZHIRENJUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_616);
            }
            StmtIr::EffectResolveDddXianxi {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dddxianxi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIANXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_617);
            }
            StmtIr::EffectResolveDddXiaheng {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_dddxiaheng");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIAHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_618);
            }
            StmtIr::EffectResolveDddLiuchong {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuchong");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUCHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_619);
            }
            StmtIr::EffectResolveDddDuanbing {
                player,
                target,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_duanbing");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_DUANBING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_621);
            }
            StmtIr::EffectResolveDddWuzhi {
                player,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_wuzhi");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_WUZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_622);
            }
            StmtIr::EffectResolveDddXujing {
                player,
                source,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xujing");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XUJING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_623);
            }
            StmtIr::EffectResolveDddCaomao {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_caomao");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_CAOMAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_624);
            }
            StmtIr::EffectResolveDddXinxianying {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xinxianying");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XINXIANYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_625);
            }
            StmtIr::EffectResolveDddXianglang {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xianglang");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIANGLANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_626);
            }
            StmtIr::EffectResolveDddYujin {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_yujin");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_YUJIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_627);
            }
            StmtIr::EffectResolveDddLiuye {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuye");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUYE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_628);
            }
            StmtIr::EffectResolveDddBaosanniang {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_baosanniang");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_BAOSANNIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_629);
            }
            StmtIr::EffectResolveDddZhenji {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_zhenji");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_ZHENJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_630);
            }
            StmtIr::EffectResolveDddZhaoang {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_zhaoang");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_ZHAOANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_631);
            }
            StmtIr::EffectResolveDddLiuba {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuba");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUBA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_632);
            }
            StmtIr::EffectResolveDddJianshuo {
                player,
                target,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_jianshuo");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_JIANSHUO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_633);
            }
            StmtIr::EffectResolveDddGuanning {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_guanning");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_GUANNING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_634);
            }
            StmtIr::EffectResolveDddCaoshuang {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_caoshuang");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_CAOSHUANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_635);
            }
            StmtIr::EffectResolveDddXuelingyun {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xuelingyun");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XUELINGYUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_636);
            }
            StmtIr::EffectResolveDddLiuhong {
                player,
                actor,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liuhong");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIUHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_637);
            }
            StmtIr::EffectResolveDddXiahouxuan {
                player,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_xiahouxuan");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_XIAHOUXUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_638);
            }
            StmtIr::EffectResolveDddZhangkai {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_zhangkai");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_ZHANGKAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_639);
            }
            StmtIr::EffectResolveDddLiangxi {
                player,
                actor,
                card,
                action,
                span,
            } => {
                self.mark(*span, "resolve_ddd_liangxi");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*action)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DDD_LIANGXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_640);
            }
            StmtIr::EffectGuozhanUseZhulianBihe { player, span } => {
                self.mark(*span, "guozhan_use_zhulian_bihe");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GUOZHAN_USE_ZHULIAN_BIHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_641);
            }
            StmtIr::EffectResolveShoulijian {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_shoulijian");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHOULIJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_642);
            }
            StmtIr::EffectCancelIncomingCardUse {
                player,
                source,
                card,
                span,
            } => {
                self.mark(*span, "cancel_incoming_card_use");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CANCEL_INCOMING_CARD_USE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_643);
            }
            StmtIr::EffectChooseOtherGainOneOfTwoSkills {
                player,
                target,
                first_def,
                second_def,
                span,
            } => {
                self.mark(*span, "choose_other_gain_one_of_two_skills");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstU32(*first_def));
                self.ops.push(Op::ConstU32(*second_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_644);
            }
            StmtIr::EffectGwyewuRandomDiscardChain {
                player,
                wuyao_def,
                lang_def,
                span,
            } => {
                self.mark(*span, "gwyewu_random_discard_chain");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*wuyao_def));
                self.ops.push(Op::ConstU32(*lang_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::GWYEWU_RANDOM_DISCARD_CHAIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_645);
            }
            StmtIr::EffectGiveSelectedThenTargetMayUseDrawOwner {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_then_target_may_use_draw_owner");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_TARGET_MAY_USE_DRAW_OWNER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_497);
            }
            StmtIr::EffectChooseCategoryThenTargetGiveMatchingOrDamagePlusOne {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_category_then_target_give_matching_or_damage_plus_one",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_CATEGORY_THEN_TARGET_GIVE_MATCHING_OR_DAMAGE_PLUS_ONE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_498);
            }
            StmtIr::EffectChooseOtherExcludingSourceRandomGainHandEquipThenUseIfEquip {
                player,
                source,
                span,
            } => {
                self.mark(
                    *span,
                    "choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip",
                );
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_EXCLUDING_SOURCE_RANDOM_GAIN_HAND_EQUIP_THEN_USE_IF_EQUIP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_499);
            }
            StmtIr::EffectResolveMobiledanshou {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mobiledanshou");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MOBILEDANSHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_500);
            }
            StmtIr::EffectResolveJsrgchiying {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_jsrgchiying");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_JSRGCHIYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_501);
            }
            StmtIr::EffectResolveMbqingdao {
                player,
                source,
                card,
                damaged,
                span,
            } => {
                self.mark(*span, "resolve_mbqingdao");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.emit_expr(card)?;
                self.emit_expr(damaged)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBQINGDAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_503);
            }
            StmtIr::EffectResolveMbxiugeng {
                player,
                helper_def,
                hand_limit_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbxiugeng");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*helper_def));
                self.ops.push(Op::ConstU32(*hand_limit_def));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXIUGENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_503);
            }
            StmtIr::EffectResolveMbchenshe {
                player,
                dying,
                source,
                span,
            } => {
                self.mark(*span, "resolve_mbchenshe");
                self.emit_expr(player)?;
                self.emit_expr(dying)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBCHENSHE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_503);
            }
            StmtIr::EffectResolveMbquanchong {
                player,
                helper_def,
                span,
            } => {
                self.mark(*span, "resolve_mbquanchong");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*helper_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBQUNCHONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_504);
            }
            StmtIr::EffectResolveMbrenxing { player, span } => {
                self.mark(*span, "resolve_mbrenxing");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBRENXING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_504);
            }
            StmtIr::EffectResolveMbchizhang { player, card, span } => {
                self.mark(*span, "resolve_mbchizhang");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBCHIZHANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_505);
            }
            StmtIr::EffectResolveMbduanyang {
                player,
                target,
                card,
                mode,
                span,
            } => {
                self.mark(*span, "resolve_mbduanyang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*mode)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBDUANYANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_505);
            }
            StmtIr::EffectResolveMbrunwei {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbrunwei");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBRUNWEI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_506);
            }
            StmtIr::EffectResolveMbshuanghuai {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mbshuanghuai");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBSHUANGHUAI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_506);
            }
            StmtIr::EffectResolveMbxiongtu {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbxiongtu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXIONGTU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_507);
            }
            StmtIr::EffectResolveMbganggeng {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbganggeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBGANGGENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_508);
            }
            StmtIr::EffectResolveMbsijian {
                player,
                helper_def,
                span,
            } => {
                self.mark(*span, "resolve_mbsijian");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBSIJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_508);
            }
            StmtIr::EffectResolveMbjianji { player, span } => {
                self.mark(*span, "resolve_mbjianji");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBJIANJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_509);
            }
            StmtIr::EffectResolveMbyuanmo { player, span } => {
                self.mark(*span, "resolve_mbyuanmo");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBYUANMO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_509);
            }
            StmtIr::EffectResolveMbfeijing {
                player,
                target,
                card,
                span,
            } => {
                self.mark(*span, "resolve_mbfeijing");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBFEIJING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_510);
            }
            StmtIr::EffectResolveMbxiaoge {
                player,
                target,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbxiaoge");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXIAOGE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_510);
            }
            StmtIr::EffectResolveMbxuye {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_mbxuye");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBXUYE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_514);
            }
            StmtIr::EffectResolveMbkuangxiang {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbkuangxiang");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_MBKUANGXIANG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_514);
            }
            StmtIr::EffectResolvePotchengpu {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potchengpu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTCHENGPU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_515);
            }
            StmtIr::EffectResolvePotjugu { player, span } => {
                self.mark(*span, "resolve_potjugu");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJUGU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_516);
            }
            StmtIr::EffectResolvePotsifeng {
                player,
                target,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potsifeng");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTSIFENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_517);
            }
            StmtIr::EffectResolvePotchenqun {
                player,
                target,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potchenqun");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTCHENQUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_518);
            }
            StmtIr::EffectResolvePotwanglie {
                player,
                card,
                helper_skill_def,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potwanglie");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTWANGLIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_519);
            }
            StmtIr::EffectResolvePothongyi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_pothongyi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTHONGYI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_519);
            }
            StmtIr::EffectResolvePotguansha {
                player,
                helper_skill_def,
                span,
            } => {
                self.mark(*span, "resolve_potguansha");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*helper_skill_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTGUANSHA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_520);
            }
            StmtIr::EffectResolvePotjiyu {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potjiyu");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTJIYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_520);
            }
            StmtIr::EffectResolveMbzhuji {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_mbzhuji");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_MBZHUJI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_521);
            }
            StmtIr::EffectResolvePotlibing {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potlibing");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTLIBING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            StmtIr::EffectResolvePotpoxi { player, span } => {
                self.mark(*span, "resolve_potpoxi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTPOXI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_522);
            }
            StmtIr::EffectResolvePotfuan {
                player,
                card,
                stage,
                range_helper_def,
                yinxian_def,
                span,
            } => {
                self.mark(*span, "resolve_potfuan");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::ConstI32(*range_helper_def as i32));
                self.ops.push(Op::ConstI32(*yinxian_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTFUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_525);
            }
            StmtIr::EffectResolvePotyinxian {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potyinxian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTYINXIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_525);
            }
            StmtIr::EffectResolvePothuanjie {
                player,
                target,
                stage,
                qice_def,
                kanpo_def,
                span,
            } => {
                self.mark(*span, "resolve_pothuanjie");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::ConstI32(*qice_def as i32));
                self.ops.push(Op::ConstI32(*kanpo_def as i32));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_POTHUANJIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_529);
            }
            StmtIr::EffectResolvePotnigu {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potnigu");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTNIGU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_532);
            }
            StmtIr::EffectResolvePotlulian {
                player,
                card,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_potlulian");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_POTLULIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_532);
            }
            StmtIr::EffectSkipNextDiscardPhase { player, span } => {
                self.mark(*span, "skip_next_discard_phase");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::SKIP_NEXT_DISCARD_PHASE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_471);
            }
            StmtIr::EffectResolveSpfengyin {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_spfengyin");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_SPFENGYIN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_471);
            }
            StmtIr::EffectResolveXinfuLianpian { player, span } => {
                self.mark(*span, "resolve_xinfu_lianpian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_LIANPIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_472);
            }
            StmtIr::EffectResolveGongjian { player, span } => {
                self.mark(*span, "resolve_gongjian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GONGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_473);
            }
            StmtIr::EffectResolveXinfuQinguo { player, span } => {
                self.mark(*span, "resolve_xinfu_qinguo");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINFU_QINGUO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_474);
            }
            StmtIr::EffectResolveDcmanzhi { player, span } => {
                self.mark(*span, "resolve_dcmanzhi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMANZHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_475);
            }
            StmtIr::EffectResolveXinxingluan { player, span } => {
                self.mark(*span, "resolve_xinxingluan");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINXINGLUAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_476);
            }
            StmtIr::EffectResolveDckuizhen { player, span } => {
                self.mark(*span, "resolve_dckuizhen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCKUIZHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_477);
            }
            StmtIr::EffectResolveDckuizhenResolve {
                player,
                winner,
                loser,
                span,
            } => {
                self.mark(*span, "resolve_dckuizhen_resolve");
                self.emit_expr(player)?;
                self.emit_expr(winner)?;
                self.emit_expr(loser)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCKUIZHEN_RESOLVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_477);
            }
            StmtIr::EffectResolveNsdingzhou { player, span } => {
                self.mark(*span, "resolve_nsdingzhou");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NSDINGZHOU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_478);
            }
            StmtIr::EffectResolveSxrmlanjiao { player, span } => {
                self.mark(*span, "resolve_sxrmlanjiao");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SXRMLANJIAO));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_479);
            }
            StmtIr::EffectResolveRekuanshi { player, span } => {
                self.mark(*span, "resolve_rekuanshi");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_REKUANSHI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_480);
            }
            StmtIr::EffectResolveRekuanshiResolve {
                player,
                damaged,
                span,
            } => {
                self.mark(*span, "resolve_rekuanshi_resolve");
                self.emit_expr(player)?;
                self.emit_expr(damaged)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_REKUANSHI_RESOLVE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_480);
            }
            StmtIr::EffectResolveDcmoyu { player, span } => {
                self.mark(*span, "resolve_dcmoyu");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCMOYU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_481);
            }
            StmtIr::EffectResolveXizhen { player, span } => {
                self.mark(*span, "resolve_xizhen");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XIZHEN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_482);
            }
            StmtIr::EffectResolveXuxie { player, span } => {
                self.mark(*span, "resolve_xuxie");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XUXIE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_483);
            }
            StmtIr::EffectResolveXuxieAdd { player, span } => {
                self.mark(*span, "resolve_xuxie_add");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_XUXIE_ADD));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_483);
            }
            StmtIr::EffectResolveDcchiying { player, span } => {
                self.mark(*span, "resolve_dcchiying");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCCHIYING));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_484);
            }
            StmtIr::EffectResolveXinkuangfu { player, span } => {
                self.mark(*span, "resolve_xinkuangfu");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_XINKUANGFU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_485);
            }
            StmtIr::EffectResolveDcxiangshu {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcxiangshu");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCXIANG_SHU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_486);
            }
            StmtIr::EffectResolveDcxiangshuLottery {
                player,
                target,
                span,
            } => {
                self.mark(*span, "resolve_dcxiangshu_lottery");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DCXIANG_SHU_LOTTERY));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_486);
            }
            StmtIr::EffectResolveShuangxiong { player, span } => {
                self.mark(*span, "resolve_shuangxiong");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_SHUANGXIONG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_511);
            }
            StmtIr::EffectResolveZhijian { player, span } => {
                self.mark(*span, "resolve_zhijian");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_512);
            }
            StmtIr::EffectResolveGuzheng {
                player,
                event_player,
                span,
            } => {
                self.mark(*span, "resolve_guzheng");
                self.emit_expr(player)?;
                self.emit_expr(event_player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_GUZHENG));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_513);
            }
            StmtIr::EffectResolveZhiba {
                player,
                actor,
                span,
            } => {
                self.mark(*span, "resolve_zhiba");
                self.emit_expr(player)?;
                self.emit_expr(actor)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_ZHIBA));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_523);
            }
            StmtIr::EffectResolveDrltCongjian { player, span } => {
                self.mark(*span, "resolve_drlt_congjian");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_CONGJIAN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_524);
            }
            StmtIr::EffectResolveNzryFeijun { player, span } => {
                self.mark(*span, "resolve_nzry_feijun");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_NZRY_FEIJUN));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_526);
            }
            StmtIr::EffectResolveDrltYongsi {
                player,
                stage,
                span,
            } => {
                self.mark(*span, "resolve_drlt_yongsi");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*stage)));
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_YONGSI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_527);
            }
            StmtIr::EffectResolveDrltWeidi { player, span } => {
                self.mark(*span, "resolve_drlt_weidi");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::RESOLVE_DRLT_WEIDI));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_528);
            }
            StmtIr::EffectResolveDcyingtu { player, span } => {
                self.mark(*span, "resolve_dcyingtu");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::RESOLVE_DCYINGTU));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_487);
            }
            StmtIr::EffectGiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "give_selected_then_target_choose_discard_same_suit_or_lose_hp",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_THEN_TARGET_CHOOSE_DISCARD_SAME_SUIT_OR_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_123);
            }
            StmtIr::EffectGainEquipmentFromDrawPile { player, span } => {
                self.mark(*span, "gain_equipment_from_draw_pile");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GAIN_EQUIPMENT_FROM_DRAW_PILE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_124);
            }
            StmtIr::EffectTargetChooseSuitGainOwnerHandDamageIfMismatch {
                player,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "target_choose_suit_gain_owner_hand_damage_if_mismatch",
                );
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::TARGET_CHOOSE_SUIT_GAIN_OWNER_HAND_DAMAGE_IF_MISMATCH,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_168);
            }
            StmtIr::EffectJudgeSourceDiscardTwoHandOrDamage {
                player,
                source,
                span,
            } => {
                self.mark(*span, "judge_source_discard_two_hand_or_damage");
                self.emit_expr(player)?;
                self.emit_expr(source)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_SOURCE_DISCARD_TWO_HAND_OR_DAMAGE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_171);
            }
            StmtIr::EffectGiveSelectedRecoverAfterPhaseTotalTwo {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_selected_recover_after_phase_total_two");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::GIVE_SELECTED_RECOVER_AFTER_PHASE_TOTAL_TWO,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_172);
            }
            StmtIr::EffectChooseOtherHandTargetsGainOneReplaceDraw { player, span } => {
                self.mark(*span, "choose_other_hand_targets_gain_one_replace_draw");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REPLACE_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_173);
            }
            StmtIr::EffectChooseOtherHandTargetsGainOneReduceDraw { player, span } => {
                self.mark(*span, "choose_other_hand_targets_gain_one_reduce_draw");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_OTHER_HAND_TARGETS_GAIN_ONE_REDUCE_DRAW,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_311);
            }
            StmtIr::EffectDistributeTopTwoCards { player, span } => {
                self.mark(*span, "distribute_top_two_cards");
                self.emit_expr(player)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::DISTRIBUTE_TOP_TWO_CARDS));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_174);
            }
            StmtIr::EffectGiveAllHandRecoverToOne {
                player,
                target,
                span,
            } => {
                self.mark(*span, "give_all_hand_recover_to_one");
                self.emit_expr(player)?;
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::GIVE_ALL_HAND_RECOVER_TO_ONE));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_125);
            }
            StmtIr::EffectChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp { player, span } => {
                self.mark(
                    *span,
                    "choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_TARGET_SELF_DISCARD_THEN_OWNER_LOSE_HP_IF_HAND_NE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_127);
            }
            StmtIr::EffectChooseGiveOneHandEquipOrRecover {
                owner,
                giver,
                target,
                span,
            } => {
                self.mark(*span, "choose_give_one_hand_equip_or_recover");
                self.emit_expr(owner)?;
                self.emit_expr(giver)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_GIVE_ONE_HAND_EQUIP_OR_RECOVER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_129);
            }
            StmtIr::EffectRemoveSkillByDef {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "remove_skill_by_def");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*skill_def));
                self.ops
                    .push(Op::HostEffect(effect_tag::REMOVE_SKILL_BY_DEF));
            }
            StmtIr::EffectChooseDiscardHandAddSkillByColor {
                player,
                red_skill_def,
                black_skill_def,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_add_skill_by_color");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstU32(*red_skill_def));
                self.ops.push(Op::ConstU32(*black_skill_def));
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_ADD_SKILL_BY_COLOR,
                ));
            }
            StmtIr::EffectRevealTopGainNonBasicOrSwapBasic { player, span } => {
                self.mark(*span, "reveal_top_gain_non_basic_or_swap_basic");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REVEAL_TOP_GAIN_NON_BASIC_OR_SWAP_BASIC,
                ));
            }
            StmtIr::EffectRevealTopGainIfColor {
                player,
                color_mask,
                span,
            } => {
                self.mark(*span, "reveal_top_gain_if_color");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*color_mask)));
                self.ops
                    .push(Op::HostEffect(effect_tag::REVEAL_TOP_GAIN_IF_COLOR));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_352);
            }
            StmtIr::EffectPreventCurrentDamage { span } => {
                self.mark(*span, "prevent_current_damage");
                self.ops
                    .push(Op::HostEffect(effect_tag::PREVENT_CURRENT_DAMAGE));
            }
            StmtIr::EffectJudgeCancelCurrentShaTargetOnRed {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "judge_cancel_current_sha_target_on_red");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::JUDGE_CANCEL_CURRENT_SHA_TARGET_ON_RED,
                ));
            }
            StmtIr::EffectChooseDrawOrRecover {
                player,
                draw_count,
                recover_count,
                allow_cancel,
                span,
            } => {
                self.mark(*span, "choose_draw_or_recover");
                self.emit_expr(player)?;
                self.emit_expr(draw_count)?;
                self.emit_expr(recover_count)?;
                self.emit_expr(allow_cancel)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_DRAW_OR_RECOVER));
            }
            StmtIr::EffectChooseUseVirtualSha {
                player,
                ignore_distance,
                span,
            } => {
                self.mark(*span, "choose_use_virtual_sha");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(i32::from(*ignore_distance)));
                self.ops
                    .push(Op::HostEffect(effect_tag::CHOOSE_USE_VIRTUAL_SHA));
            }
            StmtIr::EffectTakeHand { from, to, span } => {
                self.mark(*span, "take_hand");
                self.emit_expr(from)?;
                self.emit_expr(to)?;
                self.ops.push(Op::HostEffect(effect_tag::TAKE_HAND));
            }
            StmtIr::EffectLoseHp {
                player,
                amount,
                span,
            } => {
                self.mark(*span, "lose_hp");
                self.emit_expr(player)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(effect_tag::LOSE_HP));
            }
            StmtIr::EffectAddSkill {
                player,
                skill_def,
                span,
            } => {
                self.mark(*span, "add_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.ops.push(Op::HostEffect(effect_tag::ADD_SKILL));
            }
            StmtIr::EffectAddBoundSkill {
                player,
                skill_def,
                target,
                span,
            } => {
                self.mark(*span, "add_bound_skill");
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::ADD_BOUND_SKILL));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_130);
            }
            StmtIr::EffectReplaceSkillWithBoundAfterResolutionIfTargetAlive {
                player,
                skill,
                skill_def,
                target,
                span,
            } => {
                self.mark(
                    *span,
                    "replace_skill_with_bound_after_resolution_if_target_alive",
                );
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::ConstI32(*skill_def as i32));
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::REPLACE_SKILL_WITH_BOUND_AFTER_RESOLUTION_IF_TARGET_ALIVE,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_132);
            }
            StmtIr::EffectChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp { player, span } => {
                self.mark(
                    *span,
                    "choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp",
                );
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_ALL_HAND_OR_EQUIP_DRAW_LOST_HP_THEN_LOSE_HP,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_133);
            }
            StmtIr::EffectChooseDiscardHandEquipThenRecover {
                player,
                count,
                amount,
                span,
            } => {
                self.mark(*span, "choose_discard_hand_equip_then_recover");
                self.emit_expr(player)?;
                self.emit_expr(count)?;
                self.emit_expr(amount)?;
                self.ops.push(Op::HostEffect(
                    effect_tag::CHOOSE_DISCARD_HAND_EQUIP_THEN_RECOVER,
                ));
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_134);
            }
            StmtIr::EffectRemoveSkill { skill, span } => {
                self.mark(*span, "remove_skill");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::REMOVE_SKILL));
            }
            StmtIr::EffectRefreshSkill { skill, span } => {
                self.mark(*span, "refresh_skill");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::REFRESH_SKILL));
            }
            StmtIr::EffectTagEventCards { skill, span } => {
                self.mark(*span, "tag_event_cards");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::TAG_CARD));
            }
            StmtIr::EffectUntagEventCards { skill, span } => {
                self.mark(*span, "untag_event_cards");
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::UNTAG_CARD));
            }
            StmtIr::EffectStorageSet {
                skill,
                key,
                value,
                span,
            } => {
                self.mark(*span, "storage_set");
                self.emit_expr(skill)?;
                self.emit_expr(key)?;
                self.emit_expr(value)?;
                self.ops.push(Op::HostEffect(effect_tag::STORAGE_SET));
            }
            StmtIr::EffectDrawAllEventTargets { count, span } => {
                self.mark(*span, "draw_all_event_targets");
                self.emit_expr(count)?;
                self.ops.push(Op::HostEffect(effect_tag::DRAW_ALL_EVENT_TARGETS));
            }
            StmtIr::EffectClanshenjunReveal {
                player,
                card,
                skill,
                span,
            } => {
                self.mark(*span, "clanshenjun_reveal");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CLANSHENJUN_REVEAL));
            }
            StmtIr::EffectClanshenjunUse { player, skill, span } => {
                self.mark(*span, "clanshenjun_use");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CLANSHENJUN_USE));
            }
            StmtIr::EffectClanyuzhiReveal { player, skill, span } => {
                self.mark(*span, "clanyuzhi_reveal");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CLANYUZHI_REVEAL));
            }
            StmtIr::EffectClanyuzhiRoundEnd { player, skill, span } => {
                self.mark(*span, "clanyuzhi_round_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::CLANYUZHI_ROUND_END));
            }
            StmtIr::EffectQieyiPhaseStart { player, skill, span } => {
                self.mark(*span, "qieyi_phase_start");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QIEYI_PHASE_START));
            }
            StmtIr::EffectQieyiUseAfter { player, card, skill, span } => {
                self.mark(*span, "qieyi_use_after");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::QIEYI_USE_AFTER));
            }
            StmtIr::EffectJianzhiPhaseEnd { player, skill, span } => {
                self.mark(*span, "jianzhi_phase_end");
                self.emit_expr(player)?;
                self.emit_expr(skill)?;
                self.ops.push(Op::HostEffect(effect_tag::JIANZHI_PHASE_END));
            }
            StmtIr::EffectMoveCard {
                card,
                to,
                from,
                span,
            } => {
                self.mark(*span, "move_card");
                self.emit_expr(card)?;
                self.emit_expr(to)?;
                self.emit_expr(from)?;
                self.ops.push(Op::HostEffect(effect_tag::MOVE_CARD));
            }
            StmtIr::EffectSetTurnState {
                current,
                phase,
                span,
            } => {
                self.mark(*span, "set_turn_state");
                self.emit_expr(current)?;
                self.ops.push(Op::ConstI32(*phase));
                self.ops.push(Op::HostEffect(effect_tag::SET_TURN_STATE));
            }
            StmtIr::EffectUseCardOn {
                player,
                card,
                target,
                span,
            } => {
                self.mark(*span, "use_card_on");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.emit_expr(target)?;
                self.ops.push(Op::HostEffect(effect_tag::USE_CARD_ON));
            }
            StmtIr::EffectUseVirtualCardOn {
                player,
                kind,
                target,
                span,
            } => {
                self.mark(*span, "use_virtual_card_on");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_13);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.emit_expr(target)?;
                self.ops
                    .push(Op::HostEffect(effect_tag::USE_VIRTUAL_CARD_ON));
            }
            StmtIr::EffectEnterDying { player, span } => {
                self.mark(*span, "enter_dying");
                self.emit_expr(player)?;
                self.ops.push(Op::HostEffect(effect_tag::ENTER_DYING));
            }
            StmtIr::EffectRevealCard { card, span } => {
                self.mark(*span, "reveal_card");
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::REVEAL_CARD));
            }
            StmtIr::EffectUseCard { player, card, span } => {
                self.mark(*span, "use_card");
                self.emit_expr(player)?;
                self.emit_expr(card)?;
                self.ops.push(Op::HostEffect(effect_tag::USE_CARD));
            }
            StmtIr::EffectRemoveCardsByKind { kind, span } => {
                self.mark(*span, "remove_cards_by_kind");
                self.ops.push(Op::ConstI32(*kind));
                self.ops
                    .push(Op::HostEffect(effect_tag::REMOVE_CARDS_BY_KIND));
            }
            StmtIr::EffectGainCardByKind { player, kind, span } => {
                self.mark(*span, "gain_card_by_kind");
                self.required_abi = self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_10);
                self.emit_expr(player)?;
                self.ops.push(Op::ConstI32(*kind));
                self.ops.push(Op::HostEffect(effect_tag::GAIN_CARD_BY_KIND));
            }
            StmtIr::EffectKillPlayer {
                player,
                killer,
                span,
            } => {
                self.mark(*span, "kill_player");
                self.emit_expr(player)?;
                self.emit_expr(killer)?;
                self.ops.push(Op::HostEffect(effect_tag::KILL_PLAYER));
            }
            StmtIr::Let { name, value, span } => {
                self.mark(*span, "let");
                self.emit_expr(value)?;
                let slot = self.local_slot(name)?;
                self.ops.push(Op::Store(slot));
            }
            StmtIr::If {
                cond,
                then_body,
                else_body,
                span,
            } => {
                self.mark(*span, "if");
                self.emit_expr(cond)?;
                // JumpIfFalse to else; then jump over else.
                // Use placeholder offsets that are fixed post-assemble from actual bytes.
                let jif_idx = self.ops.len();
                self.ops.push(Op::JumpIfFalse(0)); // placeholder
                self.emit_stmts(then_body)?;
                let jmp_idx = self.ops.len();
                self.ops.push(Op::Jump(0)); // placeholder
                let else_op_idx = self.ops.len();
                self.emit_stmts(else_body)?;
                let end_op_idx = self.ops.len();
                self.backpatches.push(BackpatchEntry::If {
                    jif_op_idx: jif_idx,
                    jmp_op_idx: jmp_idx,
                    else_op_idx,
                    end_op_idx,
                });
            }
            StmtIr::Loop { times, body, span } => {
                self.mark(*span, "loop");
                // for i in 0..times { body } using local counter
                let slot_name = format!("__loop_{}", self.locals.len());
                let slot = self.local_slot(&slot_name)?;
                self.ops.push(Op::ConstI32(0));
                self.ops.push(Op::Store(slot));
                let loop_start = self.ops.len();
                // if counter >= times jump end
                self.ops.push(Op::Load(slot));
                self.ops.push(Op::ConstI32(*times as i32));
                self.ops.push(Op::Lt); // counter < times
                let jif_idx = self.ops.len();
                self.ops.push(Op::JumpIfFalse(0));
                self.emit_stmts(body)?;
                // counter++
                self.ops.push(Op::Load(slot));
                self.ops.push(Op::ConstI32(1));
                self.ops.push(Op::Add);
                self.ops.push(Op::Store(slot));
                // jump loop_start
                let jmp_back = self.ops.len();
                self.ops.push(Op::Jump(0));
                let end_op_idx = self.ops.len();
                self.backpatches.push(BackpatchEntry::Loop {
                    jif_op_idx: jif_idx,
                    jmp_back_op_idx: jmp_back,
                    loop_start_op_idx: loop_start,
                    end_op_idx,
                });
            }
        }
        Ok(())
    }

    fn patch_jumps_if(
        &mut self,
        jif_idx: usize,
        jmp_idx: usize,
        _end_hint: u32,
    ) -> Result<(), CompileError> {
        // Re-assemble offsets from op indices using estimate_pc
        let after_jif = estimate_pc(&self.ops[..jif_idx + 1]);
        let else_start = estimate_pc(&self.ops[..jmp_idx + 1]);
        // JumpIfFalse should skip then-body to else_start
        // Structure: [jif][then...][jmp][else...]
        // after executing jif operands, pc = after_jif; want target else_start
        let off = (else_start as i32) - (after_jif as i32);
        self.ops[jif_idx] = Op::JumpIfFalse(off as i16);

        let after_jmp = estimate_pc(&self.ops[..jmp_idx + 1]);
        let end = estimate_pc(&self.ops);
        let off2 = (end as i32) - (after_jmp as i32);
        self.ops[jmp_idx] = Op::Jump(off2 as i16);
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
                c if c == op::HOST_EFFECT || c == op::HOST_QUERY || c == op::HOST_CHOICE
                    || c == op::LOAD || c == op::STORE => 2,
                _ => 1,
            };
        }
        op_byte.push(i); // one past end

        for bp in &self.backpatches {
            match bp {
                BackpatchEntry::If { jif_op_idx, jmp_op_idx, else_op_idx, end_op_idx } => {
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
                BackpatchEntry::Loop { jif_op_idx, jmp_back_op_idx, loop_start_op_idx, end_op_idx } => {
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

    fn patch_loop(
        &mut self,
        jif_idx: usize,
        jmp_back: usize,
        loop_start: usize,
        _end: u32,
    ) -> Result<(), CompileError> {
        let after_jif = estimate_pc(&self.ops[..jif_idx + 1]);
        let end_pc = estimate_pc(&self.ops);
        let off_end = (end_pc as i32) - (after_jif as i32);
        self.ops[jif_idx] = Op::JumpIfFalse(off_end as i16);

        let after_jmp = estimate_pc(&self.ops[..jmp_back + 1]);
        let start_pc = estimate_pc(&self.ops[..loop_start]);
        let off_back = (start_pc as i32) - (after_jmp as i32);
        self.ops[jmp_back] = Op::Jump(off_back as i16);
        Ok(())
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
                        self.required_abi =
                            self.required_abi.max(ADDITIVE_RULE_VM_ABI_VERSION_681);
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
            Op::Load(_)
            | Op::Store(_)
            | Op::HostQuery(_)
            | Op::HostChoice(_) => 2,
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
