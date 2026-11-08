// fdlibm 5.3 as compiled into the JDK 6 StrictMath natives, statement for statement

mod sine
{
    pub const S1: f64 = f64::from_bits(0xBFC5555555555549);
    pub const S2: f64 = f64::from_bits(0x3F8111111110F8A6);
    pub const S3: f64 = f64::from_bits(0xBF2A01A019C161D5);
    pub const S4: f64 = f64::from_bits(0x3EC71DE357B1FE7D);
    pub const S5: f64 = f64::from_bits(0xBE5AE5E68A2B9CEB);
    pub const S6: f64 = f64::from_bits(0x3DE5D93A5ACFD57C);
}

mod cosine
{
    pub const C1: f64 = f64::from_bits(0x3FA555555555554C);
    pub const C2: f64 = f64::from_bits(0xBF56C16C16C15177);
    pub const C3: f64 = f64::from_bits(0x3EFA01A019CB1590);
    pub const C4: f64 = f64::from_bits(0xBE927E4F809C52AD);
    pub const C5: f64 = f64::from_bits(0x3E21EE9EBDB4B1C4);
    pub const C6: f64 = f64::from_bits(0xBDA8FAE9BE8838D4);
}

mod reduce
{
    pub const TWO24: f64 = f64::from_bits(0x4170000000000000);
    pub const TWON24: f64 = f64::from_bits(0x3E70000000000000);
    pub const INVPIO2: f64 = f64::from_bits(0x3FE45F306DC9C883);
    pub const PIO2_1: f64 = f64::from_bits(0x3FF921FB54400000);
    pub const PIO2_1T: f64 = f64::from_bits(0x3DD0B4611A626331);
    pub const PIO2_2: f64 = f64::from_bits(0x3DD0B4611A600000);
    pub const PIO2_2T: f64 = f64::from_bits(0x3BA3198A2E037073);
    pub const PIO2_3: f64 = f64::from_bits(0x3BA3198A2E000000);
    pub const PIO2_3T: f64 = f64::from_bits(0x397B839A252049C1);

    pub const NPIO2_HW: [i32; 32] = [
        0x3FF921FB, 0x400921FB, 0x4012D97C, 0x401921FB, 0x401F6A7A, 0x4022D97C, 0x4025FDBB, 0x402921FB, 0x402C463A,
        0x402F6A7A, 0x4031475C, 0x4032D97C, 0x40346B9C, 0x4035FDBB, 0x40378FDB, 0x403921FB, 0x403AB41B, 0x403C463A,
        0x403DD85A, 0x403F6A7A, 0x40407E4C, 0x4041475C, 0x4042106C, 0x4042D97C, 0x4043A28C, 0x40446B9C, 0x404534AC,
        0x4045FDBB, 0x4046C6CB, 0x40478FDB, 0x404858EB, 0x404921FB,
    ];

    pub const TWO_OVER_PI: [i32; 66] = [
        0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163, 0xABDEBB, 0xC561B7,
        0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129, 0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C,
        0x7026B4, 0x5F7E41, 0x3991D6, 0x398353, 0x39F49C, 0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11,
        0x8B5A0A, 0x6D1F6D, 0x367ECF, 0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7,
        0x8A5292, 0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3, 0x91615E,
        0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA, 0x73A8C9, 0x60E27B, 0xC08C6B,
    ];

    pub const PIO2: [f64; 8] = [
        f64::from_bits(0x3FF921FB40000000),
        f64::from_bits(0x3E74442D00000000),
        f64::from_bits(0x3CF8469880000000),
        f64::from_bits(0x3B78CC5160000000),
        f64::from_bits(0x39F01B8380000000),
        f64::from_bits(0x387A252040000000),
        f64::from_bits(0x36E3822280000000),
        f64::from_bits(0x3569F31D00000000),
    ];
}

mod arctan
{
    pub const HI: [f64; 4] = [
        f64::from_bits(0x3FDDAC670561BB4F),
        f64::from_bits(0x3FE921FB54442D18),
        f64::from_bits(0x3FEF730BD281F69B),
        f64::from_bits(0x3FF921FB54442D18),
    ];
    pub const LO: [f64; 4] = [
        f64::from_bits(0x3C7A2B7F222F65E2),
        f64::from_bits(0x3C81A62633145C07),
        f64::from_bits(0x3C7007887AF0CBBD),
        f64::from_bits(0x3C91A62633145C07),
    ];
    pub const AT: [f64; 11] = [
        f64::from_bits(0x3FD555555555550D),
        f64::from_bits(0xBFC999999998EBC4),
        f64::from_bits(0x3FC24924920083FF),
        f64::from_bits(0xBFBC71C6FE231671),
        f64::from_bits(0x3FB745CDC54C206E),
        f64::from_bits(0xBFB3B0F2AF749A6D),
        f64::from_bits(0x3FB10D66A0D03D51),
        f64::from_bits(0xBFADDE2D52DEFD9A),
        f64::from_bits(0x3FA97B4B24760DEB),
        f64::from_bits(0xBFA2B4442C6A6C2F),
        f64::from_bits(0x3F90AD3AE322DA11),
    ];
    pub const PI_O_4: f64 = f64::from_bits(0x3FE921FB54442D18);
    pub const PI_O_2: f64 = f64::from_bits(0x3FF921FB54442D18);
    pub const PI: f64 = f64::from_bits(0x400921FB54442D18);
    pub const PI_LO: f64 = f64::from_bits(0x3CA1A62633145C07);
}

mod logarithm
{
    pub const LN2_HI: f64 = f64::from_bits(0x3FE62E42FEE00000);
    pub const LN2_LO: f64 = f64::from_bits(0x3DEA39EF35793C76);
    pub const TWO54: f64 = f64::from_bits(0x4350000000000000);
    pub const LG1: f64 = f64::from_bits(0x3FE5555555555593);
    pub const LG2: f64 = f64::from_bits(0x3FD999999997FA04);
    pub const LG3: f64 = f64::from_bits(0x3FD2492494229359);
    pub const LG4: f64 = f64::from_bits(0x3FCC71C51D8E78AF);
    pub const LG5: f64 = f64::from_bits(0x3FC7466496CB03DE);
    pub const LG6: f64 = f64::from_bits(0x3FC39A09D078C69F);
    pub const LG7: f64 = f64::from_bits(0x3FC2F112DF3E5244);
}

mod exponent
{
    pub const HALF: [f64; 2] = [0.5, -0.5];
    pub const TWOM1000: f64 = f64::from_bits(0x0170000000000000);
    pub const O_THRESHOLD: f64 = f64::from_bits(0x40862E42FEFA39EF);
    pub const U_THRESHOLD: f64 = f64::from_bits(0xC0874910D52D3051);
    pub const LN2_HI: [f64; 2] = [f64::from_bits(0x3FE62E42FEE00000), f64::from_bits(0xBFE62E42FEE00000)];
    pub const LN2_LO: [f64; 2] = [f64::from_bits(0x3DEA39EF35793C76), f64::from_bits(0xBDEA39EF35793C76)];
    pub const INVLN2: f64 = f64::from_bits(0x3FF71547652B82FE);
    pub const P1: f64 = f64::from_bits(0x3FC555555555553E);
    pub const P2: f64 = f64::from_bits(0xBF66C16C16BEBD93);
    pub const P3: f64 = f64::from_bits(0x3F11566AAF25DE2C);
    pub const P4: f64 = f64::from_bits(0xBEBBBD41C5D26BF1);
    pub const P5: f64 = f64::from_bits(0x3E66376972BEA4D0);
}

mod power
{
    pub const BP: [f64; 2] = [1.0, 1.5];
    pub const DP_H: [f64; 2] = [0.0, f64::from_bits(0x3FE2B80340000000)];
    pub const DP_L: [f64; 2] = [0.0, f64::from_bits(0x3E4CFDEB43CFD006)];
    pub const TWO53: f64 = f64::from_bits(0x4340000000000000);
    pub const L1: f64 = f64::from_bits(0x3FE3333333333303);
    pub const L2: f64 = f64::from_bits(0x3FDB6DB6DB6FABFF);
    pub const L3: f64 = f64::from_bits(0x3FD55555518F264D);
    pub const L4: f64 = f64::from_bits(0x3FD17460A91D4101);
    pub const L5: f64 = f64::from_bits(0x3FCD864A93C9DB65);
    pub const L6: f64 = f64::from_bits(0x3FCA7E284A454EEF);
    pub const LG2: f64 = f64::from_bits(0x3FE62E42FEFA39EF);
    pub const LG2_H: f64 = f64::from_bits(0x3FE62E4300000000);
    pub const LG2_L: f64 = f64::from_bits(0xBE205C610CA86C39);
    pub const OVT: f64 = 8.0085662595372944372e-17;
    pub const CP: f64 = f64::from_bits(0x3FEEC709DC3A03FD);
    pub const CP_H: f64 = f64::from_bits(0x3FEEC709E0000000);
    pub const CP_L: f64 = f64::from_bits(0xBE3E2FE0145B01F5);
    pub const IVLN2: f64 = f64::from_bits(0x3FF71547652B82FE);
    pub const IVLN2_H: f64 = f64::from_bits(0x3FF7154760000000);
    pub const IVLN2_L: f64 = f64::from_bits(0x3E54AE0BF85DDF44);
}

const HUGE: f64 = 1.0e300;
const TINY: f64 = 1.0e-300;
const TWOM54: f64 = f64::from_bits(0x3C90000000000000);

fn hi(x: f64) -> i32 { (x.to_bits() >> 32) as i32 }

fn lo(x: f64) -> u32 { x.to_bits() as u32 }

fn words(high: i32, low: u32) -> f64 { f64::from_bits(((high as u32 as u64) << 32) | low as u64) }

fn withHi(x: f64, high: i32) -> f64 { words(high, lo(x)) }

fn withLo(x: f64, low: u32) -> f64 { words(hi(x), low) }

fn copysign(x: f64, y: f64) -> f64 { words((hi(x) & 0x7FFFFFFF) | (hi(y) & 0x80000000u32 as i32), lo(x)) }

pub fn scalbn(x: f64, n: i32) -> f64
{
    let mut x: f64 = x;
    let mut high: i32 = hi(x);
    let low: u32 = lo(x);
    let mut k: i32 = (high & 0x7FF00000) >> 20;
    if k == 0
    {
        if (low as i32 | (high & 0x7FFFFFFF)) == 0
        {
            return x;
        }

        x *= logarithm::TWO54;
        high = hi(x);
        k = ((high & 0x7FF00000) >> 20) - 54;
        if n < -50000
        {
            return TINY * x;
        }
    }

    if k == 0x7FF
    {
        return x + x;
    }

    k += n;
    if k > 0x7FE
    {
        return HUGE * copysign(HUGE, x);
    }

    if k > 0
    {
        return withHi(x, (high & 0x800FFFFFu32 as i32) | (k << 20));
    }

    if k <= -54
    {
        if n > 50000
        {
            return HUGE * copysign(HUGE, x);
        }
        return TINY * copysign(TINY, x);
    }

    k += 54;
    withHi(x, (high & 0x800FFFFFu32 as i32) | (k << 20)) * TWOM54
}

fn kernelSin(x: f64, y: f64, iy: i32) -> f64
{
    let ix: i32 = hi(x) & 0x7FFFFFFF;
    if ix < 0x3E400000 && x as i32 == 0
    {
        return x;
    }

    let z: f64 = x * x;
    let v: f64 = z * x;
    let r: f64 = sine::S2 + z * (sine::S3 + z * (sine::S4 + z * (sine::S5 + z * sine::S6)));
    if iy == 0
    {
        return x + v * (sine::S1 + z * r);
    }
    x - ((z * (0.5 * y - v * r) - y) - v * sine::S1)
}

fn kernelCos(x: f64, y: f64) -> f64
{
    let ix: i32 = hi(x) & 0x7FFFFFFF;
    if ix < 0x3E400000 && x as i32 == 0
    {
        return 1.0;
    }

    let z: f64 = x * x;
    let r: f64 =
        z * (cosine::C1 + z * (cosine::C2 + z * (cosine::C3 + z * (cosine::C4 + z * (cosine::C5 + z * cosine::C6)))));
    if ix < 0x3FD33333
    {
        return 1.0 - (0.5 * z - (z * r - x * y));
    }

    let qx: f64 = if ix > 0x3FE90000
    {
        0.28125
    }
    else
    {
        words(ix - 0x00200000, 0)
    };
    let hz: f64 = 0.5 * z - qx;
    let a: f64 = 1.0 - qx;
    a - (hz - (z * r - x * y))
}

fn kernelRemPio2(x: &[f64; 3], y: &mut [f64; 2], e0: i32, nx: i32) -> i32
{
    let jk: i32 = 4;
    let jp: i32 = jk;
    let mut iq: [i32; 20] = [0; 20];
    let mut f: [f64; 20] = [0.0; 20];
    let mut fq: [f64; 20] = [0.0; 20];
    let mut q: [f64; 20] = [0.0; 20];

    let jx: i32 = nx - 1;
    let mut jv: i32 = (e0 - 3) / 24;
    if jv < 0
    {
        jv = 0;
    }
    let mut q0: i32 = e0 - 24 * (jv + 1);

    let mut j: i32 = jv - jx;
    let m: i32 = jx + jk;
    for i in 0..=m
    {
        f[i as usize] = if j < 0
        {
            0.0
        }
        else
        {
            reduce::TWO_OVER_PI[j as usize] as f64
        };
        j += 1;
    }

    for i in 0..=jk
    {
        let mut fw: f64 = 0.0;
        for j in 0..=jx
        {
            fw += x[j as usize] * f[(jx + i - j) as usize];
        }
        q[i as usize] = fw;
    }

    let mut jz: i32 = jk;
    let mut z: f64;
    let mut n: i32;
    let mut ih: i32;
    loop
    {
        let mut i: i32 = 0;
        let mut j: i32 = jz;
        z = q[jz as usize];
        while j > 0
        {
            let fw: f64 = ((reduce::TWON24 * z) as i32) as f64;
            iq[i as usize] = (z - reduce::TWO24 * fw) as i32;
            z = q[(j - 1) as usize] + fw;
            i += 1;
            j -= 1;
        }

        z = scalbn(z, q0);
        z -= 8.0 * (z * 0.125).floor();
        n = z as i32;
        z -= n as f64;
        ih = 0;
        if q0 > 0
        {
            let i: i32 = iq[(jz - 1) as usize] >> (24 - q0);
            n += i;
            iq[(jz - 1) as usize] -= i << (24 - q0);
            ih = iq[(jz - 1) as usize] >> (23 - q0);
        }
        else if q0 == 0
        {
            ih = iq[(jz - 1) as usize] >> 23;
        }
        else if z >= 0.5
        {
            ih = 2;
        }

        if ih > 0
        {
            n += 1;
            let mut carry: i32 = 0;
            for i in 0..jz
            {
                let j: i32 = iq[i as usize];
                if carry == 0
                {
                    if j != 0
                    {
                        carry = 1;
                        iq[i as usize] = 0x1000000 - j;
                    }
                }
                else
                {
                    iq[i as usize] = 0xFFFFFF - j;
                }
            }

            match q0
            {
                1 => iq[(jz - 1) as usize] &= 0x7FFFFF,
                2 => iq[(jz - 1) as usize] &= 0x3FFFFF,
                _ => (),
            }

            if ih == 2
            {
                z = 1.0 - z;
                if carry != 0
                {
                    z -= scalbn(1.0, q0);
                }
            }
        }

        if z != 0.0
        {
            break;
        }

        let mut j: i32 = 0;
        let mut i: i32 = jz - 1;
        while i >= jk
        {
            j |= iq[i as usize];
            i -= 1;
        }
        if j != 0
        {
            break;
        }

        let mut k: i32 = 1;
        while iq[(jk - k) as usize] == 0
        {
            k += 1;
        }

        for i in (jz + 1)..=(jz + k)
        {
            f[(jx + i) as usize] = reduce::TWO_OVER_PI[(jv + i) as usize] as f64;
            let mut fw: f64 = 0.0;
            for j in 0..=jx
            {
                fw += x[j as usize] * f[(jx + i - j) as usize];
            }
            q[i as usize] = fw;
        }
        jz += k;
    }

    if z == 0.0
    {
        jz -= 1;
        q0 -= 24;
        while iq[jz as usize] == 0
        {
            jz -= 1;
            q0 -= 24;
        }
    }
    else
    {
        z = scalbn(z, -q0);
        if z >= reduce::TWO24
        {
            let fw: f64 = ((reduce::TWON24 * z) as i32) as f64;
            iq[jz as usize] = (z - reduce::TWO24 * fw) as i32;
            jz += 1;
            q0 += 24;
            iq[jz as usize] = fw as i32;
        }
        else
        {
            iq[jz as usize] = z as i32;
        }
    }

    let mut fw: f64 = scalbn(1.0, q0);
    let mut i: i32 = jz;
    while i >= 0
    {
        q[i as usize] = fw * iq[i as usize] as f64;
        fw *= reduce::TWON24;
        i -= 1;
    }

    let mut i: i32 = jz;
    while i >= 0
    {
        let mut fw: f64 = 0.0;
        let mut k: i32 = 0;
        while k <= jp && k <= jz - i
        {
            fw += reduce::PIO2[k as usize] * q[(i + k) as usize];
            k += 1;
        }
        fq[(jz - i) as usize] = fw;
        i -= 1;
    }

    let mut fw: f64 = 0.0;
    let mut i: i32 = jz;
    while i >= 0
    {
        fw += fq[i as usize];
        i -= 1;
    }
    y[0] = if ih == 0 { fw } else { -fw };
    fw = fq[0] - fw;
    for i in 1..=jz
    {
        fw += fq[i as usize];
    }
    y[1] = if ih == 0 { fw } else { -fw };
    n & 7
}

fn remPio2(x: f64, y: &mut [f64; 2]) -> i32
{
    let hx: i32 = hi(x);
    let ix: i32 = hx & 0x7FFFFFFF;
    if ix <= 0x3FE921FB
    {
        y[0] = x;
        y[1] = 0.0;
        return 0;
    }

    if ix < 0x4002D97C
    {
        if hx > 0
        {
            let mut z: f64 = x - reduce::PIO2_1;
            if ix != 0x3FF921FB
            {
                y[0] = z - reduce::PIO2_1T;
                y[1] = (z - y[0]) - reduce::PIO2_1T;
            }
            else
            {
                z -= reduce::PIO2_2;
                y[0] = z - reduce::PIO2_2T;
                y[1] = (z - y[0]) - reduce::PIO2_2T;
            }
            return 1;
        }

        let mut z: f64 = x + reduce::PIO2_1;
        if ix != 0x3FF921FB
        {
            y[0] = z + reduce::PIO2_1T;
            y[1] = (z - y[0]) + reduce::PIO2_1T;
        }
        else
        {
            z += reduce::PIO2_2;
            y[0] = z + reduce::PIO2_2T;
            y[1] = (z - y[0]) + reduce::PIO2_2T;
        }
        return -1;
    }

    if ix <= 0x413921FB
    {
        let mut t: f64 = x.abs();
        let n: i32 = (t * reduce::INVPIO2 + 0.5) as i32;
        let fn_: f64 = n as f64;
        let mut r: f64 = t - fn_ * reduce::PIO2_1;
        let mut w: f64 = fn_ * reduce::PIO2_1T;
        if n < 32 && ix != reduce::NPIO2_HW[(n - 1) as usize]
        {
            y[0] = r - w;
        }
        else
        {
            let j: i32 = ix >> 20;
            y[0] = r - w;
            let mut i: i32 = j - ((hi(y[0]) >> 20) & 0x7FF);
            if i > 16
            {
                t = r;
                w = fn_ * reduce::PIO2_2;
                r = t - w;
                w = fn_ * reduce::PIO2_2T - ((t - r) - w);
                y[0] = r - w;
                i = j - ((hi(y[0]) >> 20) & 0x7FF);
                if i > 49
                {
                    t = r;
                    w = fn_ * reduce::PIO2_3;
                    r = t - w;
                    w = fn_ * reduce::PIO2_3T - ((t - r) - w);
                    y[0] = r - w;
                }
            }
        }

        y[1] = (r - y[0]) - w;
        if hx < 0
        {
            y[0] = -y[0];
            y[1] = -y[1];
            return -n;
        }
        return n;
    }

    if ix >= 0x7FF00000
    {
        y[0] = x - x;
        y[1] = y[0];
        return 0;
    }

    let e0: i32 = (ix >> 20) - 1046;
    let mut z: f64 = words(ix - (e0 << 20), lo(x));
    let mut tx: [f64; 3] = [0.0; 3];
    for slot in tx.iter_mut().take(2)
    {
        *slot = (z as i32) as f64;
        z = (z - *slot) * reduce::TWO24;
    }
    tx[2] = z;

    let mut nx: i32 = 3;
    while tx[(nx - 1) as usize] == 0.0
    {
        nx -= 1;
    }

    let n: i32 = kernelRemPio2(&tx, y, e0, nx);
    if hx < 0
    {
        y[0] = -y[0];
        y[1] = -y[1];
        return -n;
    }
    n
}

pub fn sin(x: f64) -> f64
{
    let ix: i32 = hi(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB
    {
        return kernelSin(x, 0.0, 0);
    }

    if ix >= 0x7FF00000
    {
        return x - x;
    }

    let mut y: [f64; 2] = [0.0; 2];
    let n: i32 = remPio2(x, &mut y);
    match n & 3
    {
        0 => kernelSin(y[0], y[1], 1),
        1 => kernelCos(y[0], y[1]),
        2 => -kernelSin(y[0], y[1], 1),
        _ => -kernelCos(y[0], y[1]),
    }
}

pub fn cos(x: f64) -> f64
{
    let ix: i32 = hi(x) & 0x7FFFFFFF;
    if ix <= 0x3FE921FB
    {
        return kernelCos(x, 0.0);
    }

    if ix >= 0x7FF00000
    {
        return x - x;
    }

    let mut y: [f64; 2] = [0.0; 2];
    let n: i32 = remPio2(x, &mut y);
    match n & 3
    {
        0 => kernelCos(y[0], y[1]),
        1 => -kernelSin(y[0], y[1], 1),
        2 => -kernelCos(y[0], y[1]),
        _ => kernelSin(y[0], y[1], 1),
    }
}

pub fn atan(x: f64) -> f64
{
    let mut x: f64 = x;
    let hx: i32 = hi(x);
    let ix: i32 = hx & 0x7FFFFFFF;
    let id: i32;
    if ix >= 0x44100000
    {
        if ix > 0x7FF00000 || (ix == 0x7FF00000 && lo(x) != 0)
        {
            return x + x;
        }
        if hx > 0
        {
            return arctan::HI[3] + arctan::LO[3];
        }
        return -arctan::HI[3] - arctan::LO[3];
    }

    if ix < 0x3FDC0000
    {
        if ix < 0x3E200000 && HUGE + x > 1.0
        {
            return x;
        }
        id = -1;
    }
    else
    {
        x = x.abs();
        if ix < 0x3FF30000
        {
            if ix < 0x3FE60000
            {
                id = 0;
                x = (2.0 * x - 1.0) / (2.0 + x);
            }
            else
            {
                id = 1;
                x = (x - 1.0) / (x + 1.0);
            }
        }
        else if ix < 0x40038000
        {
            id = 2;
            x = (x - 1.5) / (1.0 + 1.5 * x);
        }
        else
        {
            id = 3;
            x = -1.0 / x;
        }
    }

    let at: &[f64; 11] = &arctan::AT;
    let z: f64 = x * x;
    let w: f64 = z * z;
    let s1: f64 = z * (at[0] + w * (at[2] + w * (at[4] + w * (at[6] + w * (at[8] + w * at[10])))));
    let s2: f64 = w * (at[1] + w * (at[3] + w * (at[5] + w * (at[7] + w * at[9]))));
    if id < 0
    {
        return x - x * (s1 + s2);
    }

    let z: f64 = arctan::HI[id as usize] - ((x * (s1 + s2) - arctan::LO[id as usize]) - x);
    if hx < 0 { -z } else { z }
}

pub fn atan2(y: f64, x: f64) -> f64
{
    let hx: i32 = hi(x);
    let ix: i32 = hx & 0x7FFFFFFF;
    let lx: u32 = lo(x);
    let hy: i32 = hi(y);
    let iy: i32 = hy & 0x7FFFFFFF;
    let ly: u32 = lo(y);
    if (ix as u32 | ((lx | lx.wrapping_neg()) >> 31)) > 0x7FF00000
        || (iy as u32 | ((ly | ly.wrapping_neg()) >> 31)) > 0x7FF00000
    {
        return x + y;
    }

    if ((hx.wrapping_sub(0x3FF00000)) as u32 | lx) == 0
    {
        return atan(y);
    }

    let m: i32 = ((hy >> 31) & 1) | ((hx >> 30) & 2);
    if (iy as u32 | ly) == 0
    {
        return match m
        {
            0 | 1 => y,
            2 => arctan::PI + TINY,
            _ => -arctan::PI - TINY,
        };
    }

    if (ix as u32 | lx) == 0
    {
        return if hy < 0
        {
            -arctan::PI_O_2 - TINY
        }
        else
        {
            arctan::PI_O_2 + TINY
        };
    }

    if ix == 0x7FF00000
    {
        if iy == 0x7FF00000
        {
            return match m
            {
                0 => arctan::PI_O_4 + TINY,
                1 => -arctan::PI_O_4 - TINY,
                2 => 3.0 * arctan::PI_O_4 + TINY,
                _ => -3.0 * arctan::PI_O_4 - TINY,
            };
        }
        return match m
        {
            0 => 0.0,
            1 => -0.0,
            2 => arctan::PI + TINY,
            _ => -arctan::PI - TINY,
        };
    }

    if iy == 0x7FF00000
    {
        return if hy < 0
        {
            -arctan::PI_O_2 - TINY
        }
        else
        {
            arctan::PI_O_2 + TINY
        };
    }

    let k: i32 = (iy - ix) >> 20;
    let z: f64 = if k > 60
    {
        arctan::PI_O_2 + 0.5 * arctan::PI_LO
    }
    else if hx < 0 && k < -60
    {
        0.0
    }
    else
    {
        atan((y / x).abs())
    };
    match m
    {
        0 => z,
        1 => withHi(z, hi(z) ^ 0x80000000u32 as i32),
        2 => arctan::PI - (z - arctan::PI_LO),
        _ => (z - arctan::PI_LO) - arctan::PI,
    }
}

pub fn log(x: f64) -> f64
{
    let mut x: f64 = x;
    let mut hx: i32 = hi(x);
    let lx: u32 = lo(x);
    let mut k: i32 = 0;
    if hx < 0x00100000
    {
        if ((hx & 0x7FFFFFFF) as u32 | lx) == 0
        {
            return -logarithm::TWO54 / 0.0;
        }
        if hx < 0
        {
            return (x - x) / 0.0;
        }
        k -= 54;
        x *= logarithm::TWO54;
        hx = hi(x);
    }

    if hx >= 0x7FF00000
    {
        return x + x;
    }

    k += (hx >> 20) - 1023;
    hx &= 0x000FFFFF;
    let mut i: i32 = (hx + 0x95F64) & 0x100000;
    x = withHi(x, hx | (i ^ 0x3FF00000));
    k += i >> 20;
    let f: f64 = x - 1.0;
    if (0x000FFFFF & (2 + hx)) < 3
    {
        if f == 0.0
        {
            if k == 0
            {
                return 0.0;
            }
            let dk: f64 = k as f64;
            return dk * logarithm::LN2_HI + dk * logarithm::LN2_LO;
        }

        let r: f64 = f * f * (0.5 - 0.33333333333333333 * f);
        if k == 0
        {
            return f - r;
        }
        let dk: f64 = k as f64;
        return dk * logarithm::LN2_HI - ((r - dk * logarithm::LN2_LO) - f);
    }

    let s: f64 = f / (2.0 + f);
    let dk: f64 = k as f64;
    let z: f64 = s * s;
    i = hx - 0x6147A;
    let w: f64 = z * z;
    let j: i32 = 0x6B851 - hx;
    let t1: f64 = w * (logarithm::LG2 + w * (logarithm::LG4 + w * logarithm::LG6));
    let t2: f64 = z * (logarithm::LG1 + w * (logarithm::LG3 + w * (logarithm::LG5 + w * logarithm::LG7)));
    i |= j;
    let r: f64 = t2 + t1;
    if i > 0
    {
        let hfsq: f64 = 0.5 * f * f;
        if k == 0
        {
            return f - (hfsq - s * (hfsq + r));
        }
        return dk * logarithm::LN2_HI - ((hfsq - (s * (hfsq + r) + dk * logarithm::LN2_LO)) - f);
    }

    if k == 0
    {
        return f - s * (f - r);
    }
    dk * logarithm::LN2_HI - ((s * (f - r) - dk * logarithm::LN2_LO) - f)
}

pub fn exp(x: f64) -> f64
{
    let mut x: f64 = x;
    let mut hx: u32 = hi(x) as u32;
    let xsb: usize = ((hx >> 31) & 1) as usize;
    hx &= 0x7FFFFFFF;
    if hx >= 0x40862E42
    {
        if hx >= 0x7FF00000
        {
            if ((hx & 0xFFFFF) | lo(x)) != 0
            {
                return x + x;
            }
            return if xsb == 0 { x } else { 0.0 };
        }
        if x > exponent::O_THRESHOLD
        {
            return HUGE * HUGE;
        }
        if x < exponent::U_THRESHOLD
        {
            return exponent::TWOM1000 * exponent::TWOM1000;
        }
    }

    let mut hi_part: f64 = 0.0;
    let mut lo_part: f64 = 0.0;
    let mut k: i32 = 0;
    if hx > 0x3FD62E42
    {
        if hx < 0x3FF0A2B2
        {
            hi_part = x - exponent::LN2_HI[xsb];
            lo_part = exponent::LN2_LO[xsb];
            k = 1 - xsb as i32 - xsb as i32;
        }
        else
        {
            k = (exponent::INVLN2 * x + exponent::HALF[xsb]) as i32;
            let t: f64 = k as f64;
            hi_part = x - t * exponent::LN2_HI[0];
            lo_part = t * exponent::LN2_LO[0];
        }
        x = hi_part - lo_part;
    }
    else if hx < 0x3E300000
    {
        if HUGE + x > 1.0
        {
            return 1.0 + x;
        }
    }

    let t: f64 = x * x;
    let c: f64 =
        x - t * (exponent::P1 + t * (exponent::P2 + t * (exponent::P3 + t * (exponent::P4 + t * exponent::P5))));
    if k == 0
    {
        return 1.0 - ((x * c) / (c - 2.0) - x);
    }

    let y: f64 = 1.0 - ((lo_part - (x * c) / (2.0 - c)) - hi_part);
    if k >= -1021
    {
        return withHi(y, hi(y).wrapping_add(k << 20));
    }
    withHi(y, hi(y).wrapping_add((k + 1000) << 20)) * exponent::TWOM1000
}

pub fn pow(x: f64, y: f64) -> f64
{
    let hx: i32 = hi(x);
    let lx: u32 = lo(x);
    let hy: i32 = hi(y);
    let ly: u32 = lo(y);
    let mut ix: i32 = hx & 0x7FFFFFFF;
    let iy: i32 = hy & 0x7FFFFFFF;

    if (iy as u32 | ly) == 0
    {
        return 1.0;
    }

    if ix > 0x7FF00000 || (ix == 0x7FF00000 && lx != 0) || iy > 0x7FF00000 || (iy == 0x7FF00000 && ly != 0)
    {
        return x + y;
    }

    let mut yisint: i32 = 0;
    if hx < 0
    {
        if iy >= 0x43400000
        {
            yisint = 2;
        }
        else if iy >= 0x3FF00000
        {
            let k: i32 = (iy >> 20) - 0x3FF;
            if k > 20
            {
                let j: u32 = ly >> (52 - k);
                if (j << (52 - k)) == ly
                {
                    yisint = 2 - (j & 1) as i32;
                }
            }
            else if ly == 0
            {
                let j: i32 = iy >> (20 - k);
                if (j << (20 - k)) == iy
                {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }

    if ly == 0
    {
        if iy == 0x7FF00000
        {
            if ((ix - 0x3FF00000) as u32 | lx) == 0
            {
                return y - y;
            }
            if ix >= 0x3FF00000
            {
                return if hy >= 0 { y } else { 0.0 };
            }
            return if hy < 0 { -y } else { 0.0 };
        }

        if iy == 0x3FF00000
        {
            if hy < 0
            {
                return 1.0 / x;
            }
            return x;
        }

        if hy == 0x40000000
        {
            return x * x;
        }

        if hy == 0x3FE00000 && hx >= 0
        {
            return x.sqrt();
        }
    }

    let mut ax: f64 = x.abs();
    if lx == 0 && (ix == 0x7FF00000 || ix == 0 || ix == 0x3FF00000)
    {
        let mut z: f64 = ax;
        if hy < 0
        {
            z = 1.0 / z;
        }
        if hx < 0
        {
            if ((ix - 0x3FF00000) | yisint) == 0
            {
                z = (z - z) / (z - z);
            }
            else if yisint == 1
            {
                z = -z;
            }
        }
        return z;
    }

    let mut n: i32 = (hx >> 31) + 1;
    if (n | yisint) == 0
    {
        return (x - x) / (x - x);
    }

    let mut s: f64 = 1.0;
    if (n | (yisint - 1)) == 0
    {
        s = -1.0;
    }

    let t1: f64;
    let t2: f64;
    if iy > 0x41E00000
    {
        if iy > 0x43F00000
        {
            if ix <= 0x3FEFFFFF
            {
                return if hy < 0 { HUGE * HUGE } else { TINY * TINY };
            }
            if ix >= 0x3FF00000
            {
                return if hy > 0 { HUGE * HUGE } else { TINY * TINY };
            }
        }

        if ix < 0x3FEFFFFF
        {
            return if hy < 0
            {
                s * HUGE * HUGE
            }
            else
            {
                s * TINY * TINY
            };
        }
        if ix > 0x3FF00000
        {
            return if hy > 0
            {
                s * HUGE * HUGE
            }
            else
            {
                s * TINY * TINY
            };
        }

        let t: f64 = ax - 1.0;
        let w: f64 = (t * t) * (0.5 - t * (0.3333333333333333333333 - t * 0.25));
        let u: f64 = power::IVLN2_H * t;
        let v: f64 = t * power::IVLN2_L - w * power::IVLN2;
        t1 = withLo(u + v, 0);
        t2 = v - (t1 - u);
    }
    else
    {
        n = 0;
        if ix < 0x00100000
        {
            ax *= power::TWO53;
            n -= 53;
            ix = hi(ax);
        }
        n += (ix >> 20) - 0x3FF;
        let j: i32 = ix & 0x000FFFFF;
        ix = j | 0x3FF00000;
        let k: usize;
        if j <= 0x3988E
        {
            k = 0;
        }
        else if j < 0xBB67A
        {
            k = 1;
        }
        else
        {
            k = 0;
            n += 1;
            ix -= 0x00100000;
        }
        ax = withHi(ax, ix);

        let mut u: f64 = ax - power::BP[k];
        let mut v: f64 = 1.0 / (ax + power::BP[k]);
        let ss: f64 = u * v;
        let s_h: f64 = withLo(ss, 0);
        let mut t_h: f64 = words(((ix >> 1) | 0x20000000) + 0x00080000 + ((k as i32) << 18), 0);
        let mut t_l: f64 = ax - (t_h - power::BP[k]);
        let s_l: f64 = v * ((u - s_h * t_h) - s_h * t_l);

        let mut s2: f64 = ss * ss;
        let mut r: f64 = s2
            * s2
            * (power::L1 + s2 * (power::L2 + s2 * (power::L3 + s2 * (power::L4 + s2 * (power::L5 + s2 * power::L6)))));
        r += s_l * (s_h + ss);
        s2 = s_h * s_h;
        t_h = withLo(3.0 + s2 + r, 0);
        t_l = r - ((t_h - 3.0) - s2);

        u = s_h * t_h;
        v = s_l * t_h + t_l * ss;

        let p_h: f64 = withLo(u + v, 0);
        let p_l: f64 = v - (p_h - u);
        let z_h: f64 = power::CP_H * p_h;
        let z_l: f64 = power::CP_L * p_h + p_l * power::CP + power::DP_L[k];

        let t: f64 = n as f64;
        t1 = withLo(((z_h + z_l) + power::DP_H[k]) + t, 0);
        t2 = z_l - (((t1 - t) - power::DP_H[k]) - z_h);
    }

    let y1: f64 = withLo(y, 0);
    let p_l: f64 = (y - y1) * t1 + y * t2;
    let mut p_h: f64 = y1 * t1;
    let mut z: f64 = p_l + p_h;
    let mut j: i32 = hi(z);
    let i: i32 = lo(z) as i32;
    if j >= 0x40900000
    {
        if (j.wrapping_sub(0x40900000) | i) != 0
        {
            return s * HUGE * HUGE;
        }
        if p_l + power::OVT > z - p_h
        {
            return s * HUGE * HUGE;
        }
    }
    else if (j & 0x7FFFFFFF) >= 0x4090CC00
    {
        if (j.wrapping_sub(0xC090CC00u32 as i32) | i) != 0
        {
            return s * TINY * TINY;
        }
        if p_l <= z - p_h
        {
            return s * TINY * TINY;
        }
    }

    let i: i32 = j & 0x7FFFFFFF;
    let mut k: i32 = (i >> 20) - 0x3FF;
    n = 0;
    if i > 0x3FE00000
    {
        n = j + (0x00100000 >> (k + 1));
        k = ((n & 0x7FFFFFFF) >> 20) - 0x3FF;
        let t: f64 = words(n & !(0x000FFFFF >> k), 0);
        n = ((n & 0x000FFFFF) | 0x00100000) >> (20 - k);
        if j < 0
        {
            n = -n;
        }
        p_h -= t;
    }

    let t: f64 = withLo(p_l + p_h, 0);
    let u: f64 = t * power::LG2_H;
    let v: f64 = (p_l - (t - p_h)) * power::LG2 + t * power::LG2_L;
    z = u + v;
    let w: f64 = v - (z - u);
    let t: f64 = z * z;
    let t1: f64 =
        z - t * (exponent::P1 + t * (exponent::P2 + t * (exponent::P3 + t * (exponent::P4 + t * exponent::P5))));
    let r: f64 = (z * t1) / (t1 - 2.0) - (w + z * w);
    z = 1.0 - (r - z);
    j = hi(z);
    j = j.wrapping_add(n << 20);
    if (j >> 20) <= 0
    {
        z = scalbn(z, n);
    }
    else
    {
        z = withHi(z, hi(z).wrapping_add(n << 20));
    }
    s * z
}
