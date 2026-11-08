use crate::noise::NoiseGenerator2;
use jvm::random::Random;

pub struct NoiseGeneratorOctaves2
{
    m_noiseLevels: Vec<NoiseGenerator2>,
    m_levels: i32,
}

impl NoiseGeneratorOctaves2
{
    pub fn new(random: &Random, levels: i32) -> Self
    {
        let generators: Vec<NoiseGenerator2> = (0..levels).map(|_| NoiseGenerator2::withRandom(random)).collect();
        Self { m_noiseLevels: generators, m_levels: levels }
    }

    pub fn getRegion(
        &self,
        values: Option<Vec<f64>>,
        x: f64,
        y: f64,
        sizeX: i32,
        sizeY: i32,
        scaleX: f64,
        scaleY: f64,
        scaleExponent: f64,
    ) -> Vec<f64>
    {
        self.getRegionScaled(values, x, y, sizeX, sizeY, scaleX, scaleY, scaleExponent, 0.5)
    }

    pub fn getRegionScaled(
        &self,
        values: Option<Vec<f64>>,
        x: f64,
        y: f64,
        sizeX: i32,
        sizeY: i32,
        scaleX: f64,
        scaleY: f64,
        scaleExponentX: f64,
        scaleExponentY: f64,
    ) -> Vec<f64>
    {
        let scaleX: f64 = scaleX / 1.5;
        let scaleY: f64 = scaleY / 1.5;
        let mut values: Vec<f64> = match values
        {
            Some(mut values) if values.len() as i32 >= sizeX * sizeY =>
            {
                values.iter_mut().for_each(|value| *value = 0.0);
                values
            }
            _ => vec![0.0; (sizeX * sizeY) as usize],
        };
        let mut d: f64 = 1.0;
        let mut d2: f64 = 1.0;
        for i in 0..self.m_levels
        {
            self.m_noiseLevels[i as usize].add(&mut values, x, y, sizeX, sizeY, scaleX * d2, scaleY * d2, 0.55 / d);
            d2 *= scaleExponentX;
            d *= scaleExponentY;
        }
        values
    }
}
