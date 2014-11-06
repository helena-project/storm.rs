/*typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t en         : 1;
        uint32_t pclr       : 1;
        uint32_t cal        : 1;
        uint32_t reserved0  : 5;
        uint32_t ca0        : 1;
        uint32_t ca1        : 1;
        uint32_t reserved1  : 6;
        uint32_t psel       : 5;
        uint32_t reserved2  : 11;
    } __attribute__((__packed__)) bits;
} ast_cr_t;

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t ovf        : 1;
        uint32_t reserved0  : 7;
        uint32_t alarm0     : 1;
        uint32_t reserved1  : 7;
        uint32_t per0       : 1;
        uint32_t reserved2  : 7;
        uint32_t busy       : 1;
        uint32_t ready      : 1;
        uint32_t reserved3  : 2;
        uint32_t clkbusy    : 1;
        uint32_t clkrdy     : 1;
        uint32_t reserved4  : 2;
    } __attribute__((__packed__)) bits;
} ast_sr_t;

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t ovf        : 1;
        uint32_t reserved0  : 7;
        uint32_t alarm0     : 1;
        uint32_t reserved1  : 7;
        uint32_t per0       : 1;
        uint32_t reserved2  : 8;
        uint32_t ready      : 1;
        uint32_t reserved3  : 3;
        uint32_t clkrdy     : 1;
        uint32_t reserved4  : 2;
    } __attribute__((__packed__)) bits;
} ast_sr_write_t;

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t ovf        : 1;
        uint32_t reserved0  : 7;
        uint32_t alarm0     : 1;
        uint32_t reserved1  : 7;
        uint32_t per0       : 1;
        uint32_t reserved2  : 15;
    } __attribute__((__packed__)) bits;
} ast_wer_t;

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t cen        : 1;
        uint32_t reserved0  : 7;
        uint32_t cssel      : 3;
        uint32_t reserved1  : 21;
    } __attribute__((__packed__)) bits;
} ast_clock_t;

enum {
    CSSEL_RCSYS,
    CSSEL_OSC32,
    CSSEL_APB,
    CSSEL_GCLK2,
    CSSEL_CLK1K
};

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t exp        : 5;
        uint32_t add        : 1;
        uint32_t reserved0  : 2;
        uint32_t value      : 8;
        uint32_t reserved1  : 16;
    } __attribute__((__packed__)) bits;
} ast_dtr_t;

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t ovf        : 1;
        uint32_t reserved0  : 7;
        uint32_t alarm0     : 1;
        uint32_t reserved1  : 7;
        uint32_t per0       : 1;
        uint32_t reserved2  : 15;
    } __attribute__((__packed__)) bits;
} ast_evx_t;

typedef union
{
    uint32_t flat;
    struct
    {
        uint32_t sec        : 6;
        uint32_t min        : 6;
        uint32_t hour       : 5;
        uint32_t day        : 5;
        uint32_t month      : 4;
        uint32_t year       : 6;
    } __attribute__((__packed__)) bits;
} ast_calv_t;
*/

struct Ast {
    cr : i32,
    cv : i32,
    sr : i32,
    scr : i32,
    ier : i32,
    idr : i32,
    imr : i32,
    wer : i32,
    //0x20
    ar0 : i32,
    ar1 : i32,
    reserved0 : [i32, ..2],
    pir0 : i32,
    pir1 : i32,
    reserved1 : [i32, ..2],
    //0x40
    clock : i32,
    dtr : i32,
    eve : i32,
    evd : i32,
    evm : i32,
    calv : i32
    //we leave out parameter and version
}

pub const AST_BASE : int = 0x400F0800;

enum Clock {
    CSSEL_RCSYS = 0,
    CSSEL_OSC32 = 1,
    CSSEL_APB = 2,
    CSSEL_GCLK2 = 3,
    CSSEL_CLK1K = 4
}

fn clock_busy(ast : &mut Ast) -> bool {
    return ast.sr & (1 << 28) != 0;
}

fn busy(ast : &mut Ast) -> bool {
    return ast.sr & (1 << 24) != 0;
}

pub fn select_clock(ast : &mut Ast, clock : Clock) {
    // Disable clock by setting first bit to zero
    ast.clock ^= 1;
    while clock_busy(ast) {}

    // Select clock
    ast.clock = (clock as i32) << 8;
    while clock_busy(ast) {}

    // Re-enable clock
    ast.clock |= 1;
}

pub fn setup() {
    let ast = unsafe { &mut *(AST_BASE as int as *mut Ast) };

    // Select clock
    select_clock(ast, CSSEL_OSC32);
}

pub fn start_periodic() {
    let ast = unsafe { &mut *(AST_BASE as int as *mut Ast) };

    ast.pir0 = 4;

    ast.ier = 1 << 16;
}

pub fn stop_periodic() {
    let ast = unsafe { &mut *(AST_BASE as int as *mut Ast) };
    ast.idr = 1 << 16;
}

