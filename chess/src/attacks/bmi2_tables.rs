pub(super) const BISHOP_BMI2: [(usize, u64, u64); 64] = [
    (0x0, 0x40201008040200, 0x8040201008040200),
    (0x40, 0x402010080400, 0x80402010080500),
    (0x60, 0x4020100A00, 0x804020110A00),
    (0x80, 0x40221400, 0x8041221400),
    (0xA0, 0x2442800, 0x182442800),
    (0xC0, 0x204085000, 0x10204885000),
    (0xE0, 0x20408102000, 0x102040810A000),
    (0x100, 0x2040810204000, 0x102040810204000),
    (0x140, 0x20100804020000, 0x4020100804020002),
    (0x160, 0x40201008040000, 0x8040201008050005),
    (0x180, 0x4020100A0000, 0x804020110A000A),
    (0x1A0, 0x4022140000, 0x804122140014),
    (0x1C0, 0x244280000, 0x18244280028),
    (0x1E0, 0x20408500000, 0x1020488500050),
    (0x200, 0x2040810200000, 0x102040810A000A0),
    (0x220, 0x4081020400000, 0x204081020400040),
    (0x240, 0x10080402000200, 0x2010080402000204),
    (0x260, 0x20100804000400, 0x4020100805000508),
    (0x280, 0x4020100A000A00, 0x804020110A000A11),
    (0x300, 0x402214001400, 0x80412214001422),
    (0x380, 0x24428002800, 0x1824428002844),
    (0x400, 0x2040850005000, 0x102048850005088),
    (0x480, 0x4081020002000, 0x2040810A000A010),
    (0x4A0, 0x8102040004000, 0x408102040004020),
    (0x4C0, 0x8040200020400, 0x1008040200020408),
    (0x4E0, 0x10080400040800, 0x2010080500050810),
    (0x500, 0x20100A000A1000, 0x4020110A000A1120),
    (0x580, 0x40221400142200, 0x8041221400142241),
    (0x780, 0x2442800284400, 0x182442800284482),
    (0x980, 0x4085000500800, 0x204885000508804),
    (0xA00, 0x8102000201000, 0x40810A000A01008),
    (0xA20, 0x10204000402000, 0x810204000402010),
    (0xA40, 0x4020002040800, 0x804020002040810),
    (0xA60, 0x8040004081000, 0x1008050005081020),
    (0xA80, 0x100A000A102000, 0x20110A000A112040),
    (0xB00, 0x22140014224000, 0x4122140014224180),
    (0xD00, 0x44280028440200, 0x8244280028448201),
    (0xF00, 0x8500050080400, 0x488500050880402),
    (0xF80, 0x10200020100800, 0x810A000A0100804),
    (0xFA0, 0x20400040201000, 0x1020400040201008),
    (0xFC0, 0x2000204081000, 0x402000204081020),
    (0xFE0, 0x4000408102000, 0x805000508102040),
    (0x1000, 0xA000A10204000, 0x110A000A11204080),
    (0x1080, 0x14001422400000, 0x2214001422418000),
    (0x1100, 0x28002844020000, 0x4428002844820100),
    (0x1180, 0x50005008040200, 0x8850005088040201),
    (0x1200, 0x20002010080400, 0x10A000A010080402),
    (0x1220, 0x40004020100800, 0x2040004020100804),
    (0x1240, 0x20408102000, 0x200020408102040),
    (0x1260, 0x40810204000, 0x500050810204080),
    (0x1280, 0xA1020400000, 0xA000A1120408000),
    (0x12A0, 0x142240000000, 0x1400142241800000),
    (0x12C0, 0x284402000000, 0x2800284482010000),
    (0x12E0, 0x500804020000, 0x5000508804020100),
    (0x1300, 0x201008040200, 0xA000A01008040201),
    (0x1320, 0x402010080400, 0x4000402010080402),
    (0x1340, 0x2040810204000, 0x2040810204080),
    (0x1380, 0x4081020400000, 0x5081020408000),
    (0x13A0, 0xA102040000000, 0xA112040800000),
    (0x13C0, 0x14224000000000, 0x14224180000000),
    (0x13E0, 0x28440200000000, 0x28448201000000),
    (0x1400, 0x50080402000000, 0x50880402010000),
    (0x1420, 0x20100804020000, 0xA0100804020100),
    (0x1440, 0x40201008040200, 0x40201008040201),
];

pub(super) const ROOK_BMI2: [(usize, u64, u64); 64] = [
    (0x1480, 0x101010101017E, 0x1010101010101FE),
    (0x2480, 0x202020202027C, 0x2020202020202FD),
    (0x2C80, 0x404040404047A, 0x4040404040404FB),
    (0x3480, 0x8080808080876, 0x8080808080808F7),
    (0x3C80, 0x1010101010106E, 0x10101010101010EF),
    (0x4480, 0x2020202020205E, 0x20202020202020DF),
    (0x4C80, 0x4040404040403E, 0x40404040404040BF),
    (0x5480, 0x8080808080807E, 0x808080808080807F),
    (0x6480, 0x1010101017E00, 0x10101010101FE01),
    (0x6C80, 0x2020202027C00, 0x20202020202FD02),
    (0x7080, 0x4040404047A00, 0x40404040404FB04),
    (0x7480, 0x8080808087600, 0x80808080808F708),
    (0x7880, 0x10101010106E00, 0x101010101010EF10),
    (0x7C80, 0x20202020205E00, 0x202020202020DF20),
    (0x8080, 0x40404040403E00, 0x404040404040BF40),
    (0x8480, 0x80808080807E00, 0x8080808080807F80),
    (0x8C80, 0x10101017E0100, 0x101010101FE0101),
    (0x9480, 0x20202027C0200, 0x202020202FD0202),
    (0x9880, 0x40404047A0400, 0x404040404FB0404),
    (0x9C80, 0x8080808760800, 0x808080808F70808),
    (0xA080, 0x101010106E1000, 0x1010101010EF1010),
    (0xA480, 0x202020205E2000, 0x2020202020DF2020),
    (0xA880, 0x404040403E4000, 0x4040404040BF4040),
    (0xAC80, 0x808080807E8000, 0x80808080807F8080),
    (0xB480, 0x101017E010100, 0x1010101FE010101),
    (0xBC80, 0x202027C020200, 0x2020202FD020202),
    (0xC080, 0x404047A040400, 0x4040404FB040404),
    (0xC480, 0x8080876080800, 0x8080808F7080808),
    (0xC880, 0x1010106E101000, 0x10101010EF101010),
    (0xCC80, 0x2020205E202000, 0x20202020DF202020),
    (0xD080, 0x4040403E404000, 0x40404040BF404040),
    (0xD480, 0x8080807E808000, 0x808080807F808080),
    (0xDC80, 0x1017E01010100, 0x10101FE01010101),
    (0xE480, 0x2027C02020200, 0x20202FD02020202),
    (0xE880, 0x4047A04040400, 0x40404FB04040404),
    (0xEC80, 0x8087608080800, 0x80808F708080808),
    (0xF080, 0x10106E10101000, 0x101010EF10101010),
    (0xF480, 0x20205E20202000, 0x202020DF20202020),
    (0xF880, 0x40403E40404000, 0x404040BF40404040),
    (0xFC80, 0x80807E80808000, 0x8080807F80808080),
    (0x10480, 0x17E0101010100, 0x101FE0101010101),
    (0x10C80, 0x27C0202020200, 0x202FD0202020202),
    (0x11080, 0x47A0404040400, 0x404FB0404040404),
    (0x11480, 0x8760808080800, 0x808F70808080808),
    (0x11880, 0x106E1010101000, 0x1010EF1010101010),
    (0x11C80, 0x205E2020202000, 0x2020DF2020202020),
    (0x12080, 0x403E4040404000, 0x4040BF4040404040),
    (0x12480, 0x807E8080808000, 0x80807F8080808080),
    (0x12C80, 0x7E010101010100, 0x1FE010101010101),
    (0x13480, 0x7C020202020200, 0x2FD020202020202),
    (0x13880, 0x7A040404040400, 0x4FB040404040404),
    (0x13C80, 0x76080808080800, 0x8F7080808080808),
    (0x14080, 0x6E101010101000, 0x10EF101010101010),
    (0x14480, 0x5E202020202000, 0x20DF202020202020),
    (0x14880, 0x3E404040404000, 0x40BF404040404040),
    (0x14C80, 0x7E808080808000, 0x807F808080808080),
    (0x15480, 0x7E01010101010100, 0xFE01010101010101),
    (0x16480, 0x7C02020202020200, 0xFD02020202020202),
    (0x16C80, 0x7A04040404040400, 0xFB04040404040404),
    (0x17480, 0x7608080808080800, 0xF708080808080808),
    (0x17C80, 0x6E10101010101000, 0xEF10101010101010),
    (0x18480, 0x5E20202020202000, 0xDF20202020202020),
    (0x18C80, 0x3E40404040404000, 0xBF40404040404040),
    (0x19480, 0x7E80808080808000, 0x7F80808080808080),
];