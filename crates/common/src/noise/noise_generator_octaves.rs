use crate::noise::NoiseGeneratorPerlin;
use jvm::random::Random;

pub struct NoiseGeneratorOctaves
{
    m_generatorCollection: Vec<NoiseGeneratorPerlin>,
    m_levels: i32,
}

impl NoiseGeneratorOctaves
{
    pub fn new(random: &Random, levels: i32) -> Self
    {
        let generators: Vec<NoiseGeneratorPerlin> =
            (0..levels).map(|_| NoiseGeneratorPerlin::withRandom(random)).collect();
        Self { m_generatorCollection: generators, m_levels: levels }
    }

    pub fn getValue(&self, x: f64, y: f64) -> f64
    {
        let mut d: f64 = 0.0;
        let mut d2: f64 = 1.0;
        for i in 0..self.m_levels
        {
            d += self.m_generatorCollection[i as usize].getValue(x * d2, y * d2) / d2;
            d2 /= 2.0;
        }
        d
    }

    pub fn generateNoiseOctaves(
        &self,
        values: Option<Vec<f64>>,
        x: f64,
        y: f64,
        z: f64,
        sizeX: i32,
        sizeY: i32,
        sizeZ: i32,
        scaleX: f64,
        scaleY: f64,
        scaleZ: f64,
    ) -> Vec<f64>
    {
        let mut values: Vec<f64> = match values
        {
            None => vec![0.0; (sizeX * sizeY * sizeZ) as usize],
            Some(mut values) =>
            {
                values.iter_mut().for_each(|value| *value = 0.0);
                values
            }
        };
        let mut d: f64 = 1.0;
        for i in 0..self.m_levels
        {
            self.m_generatorCollection[i as usize].add(
                &mut values,
                x,
                y,
                z,
                sizeX,
                sizeY,
                sizeZ,
                scaleX * d,
                scaleY * d,
                scaleZ * d,
                d,
            );
            d /= 2.0;
        }
        values
    }

    pub fn getRegion(
        &self,
        values: Option<Vec<f64>>,
        x: i32,
        z: i32,
        sizeX: i32,
        sizeZ: i32,
        scaleX: f64,
        scaleZ: f64,
        _d: f64,
    ) -> Vec<f64>
    {
        self.generateNoiseOctaves(values, x as f64, 10.0, z as f64, sizeX, 1, sizeZ, scaleX, 1.0, scaleZ)
    }
}
