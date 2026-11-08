use jvm::random::Random;

pub struct NoiseGeneratorPerlin
{
    m_permutations: [i32; 512],
    pub m_xCoord: f64,
    pub m_yCoord: f64,
    pub m_zCoord: f64,
}

fn fade(t: f64) -> f64 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }

impl NoiseGeneratorPerlin
{
    pub fn new() -> Self { Self::withRandom(&Random::new()) }

    pub fn withRandom(random: &Random) -> Self
    {
        let mut permutations: [i32; 512] = [0; 512];
        let xCoord: f64 = random.nextDouble() * 256.0;
        let yCoord: f64 = random.nextDouble() * 256.0;
        let zCoord: f64 = random.nextDouble() * 256.0;
        for i in 0..256
        {
            permutations[i] = i as i32;
        }
        for j in 0..256
        {
            let n: usize = (random.nextIntBounded(256 - j as i32) + j as i32) as usize;
            permutations.swap(j, n);
            permutations[j + 256] = permutations[j];
        }
        Self { m_permutations: permutations, m_xCoord: xCoord, m_yCoord: yCoord, m_zCoord: zCoord }
    }

    fn p(&self, index: i32) -> i32 { self.m_permutations[index as usize] }

    pub fn generateNoise(&self, x: f64, y: f64, z: f64) -> f64
    {
        let n: f64 = x + self.m_xCoord;
        let n2: f64 = y + self.m_yCoord;
        let n3: f64 = z + self.m_zCoord;
        let mut n4: i32 = n as i32;
        let mut n5: i32 = n2 as i32;
        let mut n6: i32 = n3 as i32;
        if n < n4 as f64
        {
            n4 = n4.wrapping_sub(1);
        }
        if n2 < n5 as f64
        {
            n5 = n5.wrapping_sub(1);
        }
        if n3 < n6 as f64
        {
            n6 = n6.wrapping_sub(1);
        }
        let n7: i32 = n4 & 0xFF;
        let n8: i32 = n5 & 0xFF;
        let n9: i32 = n6 & 0xFF;
        let n10: f64 = n - n4 as f64;
        let n11: f64 = n2 - n5 as f64;
        let n12: f64 = n3 - n6 as f64;
        let n13: f64 = fade(n10);
        let n14: f64 = fade(n11);
        let delta: f64 = fade(n12);
        let n15: i32 = self.p(n7) + n8;
        let n16: i32 = self.p(n15) + n9;
        let n17: i32 = self.p(n15 + 1) + n9;
        let n18: i32 = self.p(n7 + 1) + n8;
        let n19: i32 = self.p(n18) + n9;
        let n20: i32 = self.p(n18 + 1) + n9;
        self.lerp(
            delta,
            self.lerp(
                n14,
                self.lerp(n13, self.grad(self.p(n16), n10, n11, n12), self.grad(self.p(n19), n10 - 1.0, n11, n12)),
                self.lerp(
                    n13,
                    self.grad(self.p(n17), n10, n11 - 1.0, n12),
                    self.grad(self.p(n20), n10 - 1.0, n11 - 1.0, n12),
                ),
            ),
            self.lerp(
                n14,
                self.lerp(
                    n13,
                    self.grad(self.p(n16 + 1), n10, n11, n12 - 1.0),
                    self.grad(self.p(n19 + 1), n10 - 1.0, n11, n12 - 1.0),
                ),
                self.lerp(
                    n13,
                    self.grad(self.p(n17 + 1), n10, n11 - 1.0, n12 - 1.0),
                    self.grad(self.p(n20 + 1), n10 - 1.0, n11 - 1.0, n12 - 1.0),
                ),
            ),
        )
    }

    pub fn lerp(&self, delta: f64, min: f64, max: f64) -> f64 { min + delta * (max - min) }

    pub fn gradientDot(&self, hash: i32, x: f64, z: f64) -> f64
    {
        let n: i32 = hash & 0xF;
        let n2: f64 = (1 - ((n & 0x8) >> 3)) as f64 * x;
        let n3: f64 = if n < 4
        {
            0.0
        }
        else if n == 12 || n == 14
        {
            x
        }
        else
        {
            z
        };
        (if (n & 0x1) == 0 { n2 } else { -n2 }) + (if (n & 0x2) == 0 { n3 } else { -n3 })
    }

    pub fn grad(&self, hash: i32, x: f64, y: f64, z: f64) -> f64
    {
        let n: i32 = hash & 0xF;
        let n2: f64 = if n < 8 { x } else { y };
        let n3: f64 = if n < 4
        {
            y
        }
        else if n == 12 || n == 14
        {
            x
        }
        else
        {
            z
        };
        (if (n & 0x1) == 0 { n2 } else { -n2 }) + (if (n & 0x2) == 0 { n3 } else { -n3 })
    }

    pub fn getValue(&self, x: f64, y: f64) -> f64 { self.generateNoise(x, y, 0.0) }

    pub fn add(
        &self,
        values: &mut [f64],
        x: f64,
        y: f64,
        z: f64,
        sizeX: i32,
        sizeY: i32,
        sizeZ: i32,
        scaleX: f64,
        scaleY: f64,
        scaleZ: f64,
        noiseScale: f64,
    )
    {
        if sizeY == 1
        {
            let mut n: usize = 0;
            let n2: f64 = 1.0 / noiseScale;
            for i in 0..sizeX
            {
                let n3: f64 = (x + i as f64) * scaleX + self.m_xCoord;
                let mut n4: i32 = n3 as i32;
                if n3 < n4 as f64
                {
                    n4 = n4.wrapping_sub(1);
                }
                let n5: i32 = n4 & 0xFF;
                let n6: f64 = n3 - n4 as f64;
                let n7: f64 = fade(n6);
                for j in 0..sizeZ
                {
                    let n8: f64 = (z + j as f64) * scaleZ + self.m_zCoord;
                    let mut n9: i32 = n8 as i32;
                    if n8 < n9 as f64
                    {
                        n9 = n9.wrapping_sub(1);
                    }
                    let n10: i32 = n9 & 0xFF;
                    let n11: f64 = n8 - n9 as f64;
                    let delta: f64 = fade(n11);
                    let n12: i32 = self.p(self.p(n5) + 0) + n10;
                    let n13: i32 = self.p(self.p(n5 + 1) + 0) + n10;
                    let lerp: f64 = self.lerp(
                        delta,
                        self.lerp(
                            n7,
                            self.gradientDot(self.p(n12), n6, n11),
                            self.grad(self.p(n13), n6 - 1.0, 0.0, n11),
                        ),
                        self.lerp(
                            n7,
                            self.grad(self.p(n12 + 1), n6, 0.0, n11 - 1.0),
                            self.grad(self.p(n13 + 1), n6 - 1.0, 0.0, n11 - 1.0),
                        ),
                    );
                    values[n] += lerp * n2;
                    n += 1;
                }
            }
            return;
        }

        let mut n15: usize = 0;
        let n16: f64 = 1.0 / noiseScale;
        let mut n17: i32 = -1;
        let mut lerp2: f64 = 0.0;
        let mut lerp3: f64 = 0.0;
        let mut lerp4: f64 = 0.0;
        let mut lerp5: f64 = 0.0;
        for k in 0..sizeX
        {
            let n18: f64 = (x + k as f64) * scaleX + self.m_xCoord;
            let mut n19: i32 = n18 as i32;
            if n18 < n19 as f64
            {
                n19 = n19.wrapping_sub(1);
            }
            let n20: i32 = n19 & 0xFF;
            let n21: f64 = n18 - n19 as f64;
            let n22: f64 = fade(n21);
            for l in 0..sizeZ
            {
                let n23: f64 = (z + l as f64) * scaleZ + self.m_zCoord;
                let mut n24: i32 = n23 as i32;
                if n23 < n24 as f64
                {
                    n24 = n24.wrapping_sub(1);
                }
                let n25: i32 = n24 & 0xFF;
                let n26: f64 = n23 - n24 as f64;
                let delta2: f64 = fade(n26);
                for n27 in 0..sizeY
                {
                    let n28: f64 = (y + n27 as f64) * scaleY + self.m_yCoord;
                    let mut n29: i32 = n28 as i32;
                    if n28 < n29 as f64
                    {
                        n29 = n29.wrapping_sub(1);
                    }
                    let n30: i32 = n29 & 0xFF;
                    let n31: f64 = n28 - n29 as f64;
                    let n32: f64 = fade(n31);
                    if n27 == 0 || n30 != n17
                    {
                        n17 = n30;
                        let n33: i32 = self.p(n20) + n30;
                        let n34: i32 = self.p(n33) + n25;
                        let n35: i32 = self.p(n33 + 1) + n25;
                        let n36: i32 = self.p(n20 + 1) + n30;
                        let n37: i32 = self.p(n36) + n25;
                        let n38: i32 = self.p(n36 + 1) + n25;
                        lerp2 = self.lerp(
                            n22,
                            self.grad(self.p(n34), n21, n31, n26),
                            self.grad(self.p(n37), n21 - 1.0, n31, n26),
                        );
                        lerp3 = self.lerp(
                            n22,
                            self.grad(self.p(n35), n21, n31 - 1.0, n26),
                            self.grad(self.p(n38), n21 - 1.0, n31 - 1.0, n26),
                        );
                        lerp4 = self.lerp(
                            n22,
                            self.grad(self.p(n34 + 1), n21, n31, n26 - 1.0),
                            self.grad(self.p(n37 + 1), n21 - 1.0, n31, n26 - 1.0),
                        );
                        lerp5 = self.lerp(
                            n22,
                            self.grad(self.p(n35 + 1), n21, n31 - 1.0, n26 - 1.0),
                            self.grad(self.p(n38 + 1), n21 - 1.0, n31 - 1.0, n26 - 1.0),
                        );
                    }
                    let lerp6: f64 = self.lerp(delta2, self.lerp(n32, lerp2, lerp3), self.lerp(n32, lerp4, lerp5));
                    values[n15] += lerp6 * n16;
                    n15 += 1;
                }
            }
        }
    }
}

impl Default for NoiseGeneratorPerlin
{
    fn default() -> Self { Self::new() }
}
