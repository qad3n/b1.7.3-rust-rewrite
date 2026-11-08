use jvm::random::Random;
use std::sync::OnceLock;

const GRADIENTS: [[i32; 3]; 12] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
];

fn skew() -> (f64, f64)
{
    static FACTORS: OnceLock<(f64, f64)> = OnceLock::new();
    *FACTORS.get_or_init(|| (0.5 * (3.0f64.sqrt() - 1.0), (3.0 - 3.0f64.sqrt()) / 6.0))
}

pub struct NoiseGenerator2
{
    m_permutations: [i32; 512],
    pub m_offsetX: f64,
    pub m_offsetY: f64,
    pub m_offsetZ: f64,
}

impl NoiseGenerator2
{
    pub fn new() -> Self { Self::withRandom(&Random::new()) }

    pub fn withRandom(random: &Random) -> Self
    {
        let mut permutations: [i32; 512] = [0; 512];
        let offsetX: f64 = random.nextDouble() * 256.0;
        let offsetY: f64 = random.nextDouble() * 256.0;
        let offsetZ: f64 = random.nextDouble() * 256.0;
        for n in 0..256
        {
            permutations[n] = n as i32;
        }
        for n in 0..256
        {
            let n2: usize = (random.nextIntBounded(256 - n as i32) + n as i32) as usize;
            permutations.swap(n, n2);
            permutations[n + 256] = permutations[n];
        }
        Self { m_permutations: permutations, m_offsetX: offsetX, m_offsetY: offsetY, m_offsetZ: offsetZ }
    }

    fn wrap(x: f64) -> i32
    {
        if x > 0.0
        {
            x as i32
        }
        else
        {
            (x as i32).wrapping_sub(1)
        }
    }

    fn dot(gradient: &[i32; 3], x: f64, y: f64) -> f64 { gradient[0] as f64 * x + gradient[1] as f64 * y }

    fn p(&self, index: i32) -> i32 { self.m_permutations[index as usize] }

    pub fn add(
        &self,
        values: &mut [f64],
        x: f64,
        y: f64,
        sizeX: i32,
        sizeY: i32,
        scaleX: f64,
        scaleY: f64,
        noiseScale: f64,
    )
    {
        let (f2, g2): (f64, f64) = skew();
        let mut i: usize = 0;
        for j in 0..sizeX
        {
            let d0: f64 = (x + j as f64) * scaleX + self.m_offsetX;
            for k in 0..sizeY
            {
                let d1: f64 = (y + k as f64) * scaleY + self.m_offsetY;
                let d5: f64 = (d0 + d1) * f2;
                let l: i32 = Self::wrap(d0 + d5);
                let i1: i32 = Self::wrap(d1 + d5);
                let d6: f64 = l.wrapping_add(i1) as f64 * g2;
                let d7: f64 = l as f64 - d6;
                let d8: f64 = i1 as f64 - d6;
                let d9: f64 = d0 - d7;
                let d10: f64 = d1 - d8;
                let (b0, b1): (i32, i32) = if d9 > d10 { (1, 0) } else { (0, 1) };
                let d11: f64 = d9 - b0 as f64 + g2;
                let d12: f64 = d10 - b1 as f64 + g2;
                let d13: f64 = d9 - 1.0 + 2.0 * g2;
                let d14: f64 = d10 - 1.0 + 2.0 * g2;
                let j1: i32 = l & 0xFF;
                let k1: i32 = i1 & 0xFF;
                let l1: i32 = self.p(j1 + self.p(k1)) % 12;
                let i2: i32 = self.p(j1 + b0 + self.p(k1 + b1)) % 12;
                let j2: i32 = self.p(j1 + 1 + self.p(k1 + 1)) % 12;
                let mut d15: f64 = 0.5 - d9 * d9 - d10 * d10;
                let d2: f64 = if d15 < 0.0
                {
                    0.0
                }
                else
                {
                    d15 *= d15;
                    d15 * d15 * Self::dot(&GRADIENTS[l1 as usize], d9, d10)
                };
                let mut d16: f64 = 0.5 - d11 * d11 - d12 * d12;
                let d3: f64 = if d16 < 0.0
                {
                    0.0
                }
                else
                {
                    d16 *= d16;
                    d16 * d16 * Self::dot(&GRADIENTS[i2 as usize], d11, d12)
                };
                let mut d17: f64 = 0.5 - d13 * d13 - d14 * d14;
                let d4: f64 = if d17 < 0.0
                {
                    0.0
                }
                else
                {
                    d17 *= d17;
                    d17 * d17 * Self::dot(&GRADIENTS[j2 as usize], d13, d14)
                };
                values[i] += 70.0 * (d2 + d3 + d4) * noiseScale;
                i += 1;
            }
        }
    }
}

impl Default for NoiseGenerator2
{
    fn default() -> Self { Self::new() }
}
