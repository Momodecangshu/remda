use {super::Texture, crate::prelude::*};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SmoothType {
    None,
    Interp,
    Hermitian,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TextureType {
    Normal,
    Turbulence(u8),
    Marble(u8),
}

#[derive(Debug, Clone, PartialEq)]
enum RanType {
    Float(f64),
    Vector(Vec3),
}

#[derive(Debug, Clone)]
pub struct Perlin {
    texture_type: TextureType,
    smooth_type: SmoothType,
    point_count: usize,
    scale: f64,
    ran: Vec<RanType>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
}

impl Default for RanType {
    fn default() -> Self {
        Self::Float(0.0)
    }
}

impl Perlin {
    #[must_use]
    pub fn new(point_count: usize, vector: bool) -> Self {
        let ran = (0..point_count)
            .map(|_| {
                if vector {
                    RanType::Vector(Vec3::random_unit())
                } else {
                    RanType::Float(Random::normal())
                }
            })
            .collect();
        let mut perm_x = (0..point_count).collect();
        Random::shuffle(&mut perm_x);
        let mut perm_y = (0..point_count).collect();
        Random::shuffle(&mut perm_y);
        let mut perm_z = (0..point_count).collect();
        Random::shuffle(&mut perm_z);
        Self {
            texture_type: TextureType::Normal,
            smooth_type: SmoothType::Hermitian,
            point_count,
            scale: 1.0,
            ran,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    #[must_use]
    pub const fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    #[must_use]
    pub const fn smooth(mut self, smooth: SmoothType) -> Self {
        self.smooth_type = smooth;
        self
    }

    #[must_use]
    pub const fn turbulence(mut self, depth: u8) -> Self {
        self.texture_type = TextureType::Turbulence(depth);
        self
    }

    #[must_use]
    pub const fn marble(mut self, depth: u8) -> Self {
        self.texture_type = TextureType::Marble(depth);
        self
    }

    #[allow(clippy::cast_sign_loss)] // because we bit and with positive number before cast
    #[allow(clippy::cast_possible_wrap)] // because di dj dk and point_count is small enough
    #[allow(clippy::cast_precision_loss)] // scene is not so big
    #[allow(clippy::many_single_char_names)]
    fn noise(&self, point: &Point3) -> f64 {
        match self.smooth_type {
            SmoothType::None => {
                let i = (((4.0 * point.x) as isize) & (self.point_count - 1) as isize) as usize;
                let j = (((4.0 * point.y) as isize) & (self.point_count - 1) as isize) as usize;
                let k = (((4.0 * point.z) as isize) & (self.point_count - 1) as isize) as usize;

                match &self.ran[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]] {
                    RanType::Vector(v) => v.x,
                    RanType::Float(x) => *x,
                }
            }
            SmoothType::Interp | SmoothType::Hermitian => {
                let i = point.x.floor() as isize;
                let j = point.y.floor() as isize;
                let k = point.z.floor() as isize;
                let u = point.x - i as f64;
                let v = point.y - j as f64;
                let w = point.z - k as f64;

                let mut grays: [[[RanType; 2]; 2]; 2] = Default::default();

                (0..2).for_each(|di| {
                    (0..2).for_each(|dj| {
                        (0..2).for_each(|dk| {
                            let xi = ((i + di as isize) & (self.point_count - 1) as isize) as usize;
                            let yi = ((j + dj as isize) & (self.point_count - 1) as isize) as usize;
                            let zi = ((k + dk as isize) & (self.point_count - 1) as isize) as usize;
                            let index = self.perm_x[xi] ^ self.perm_y[yi] ^ self.perm_z[zi];
                            grays[di][dj][dk] = self.ran[index].clone();
                        })
                    })
                });

                interp(&grays, u, v, w, &self.smooth_type)
            }
        }
    }

    fn turb(&self, point: &Point3, depth: usize) -> f64 {
        let mut p = point.clone();
        let mut weight = 1.0;

        (0..depth)
            .map(|_| {
                let res = weight * self.noise(&p);
                weight *= 0.5;
                p *= 2.0;
                res
            })
            .sum::<f64>()
            .abs()
    }
}

#[allow(clippy::cast_precision_loss)] // i j k is small enough
fn interp(c: &[[[RanType; 2]; 2]; 2], u: f64, v: f64, w: f64, smooth: &SmoothType) -> f64 {
    let (mut uu, mut vv, mut ww) = (u, v, w);
    if smooth == &SmoothType::Hermitian {
        uu = u * u * (3.0 - 2.0 * u);
        vv = v * v * (3.0 - 2.0 * v);
        ww = w * w * (3.0 - 2.0 * w);
    }

    (0..2)
        .map(|i| {
            (0..2)
                .map(|j| {
                    (0..2)
                        .map(|k| {
                            (i as f64).mul_add(uu, (1 - i) as f64 * (1.0 - uu))
                                * (j as f64).mul_add(vv, (1 - j) as f64 * (1.0 - vv))
                                * (k as f64).mul_add(ww, (1 - k) as f64 * (1.0 - ww))
                                * match &c[i][j][k] {
                                    RanType::Vector(vec) => {
                                        let weight =
                                            Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                                        vec.dot(&weight)
                                    }
                                    RanType::Float(x) => *x,
                                }
                        })
                        .sum::<f64>()
                })
                .sum::<f64>()
        })
        .sum()
}

impl Texture for Perlin {
    fn color(&self, _u: f64, _v: f64, point: &Point3) -> Color {
        Color::newf(1.0, 1.0, 1.0)
            * match &self.texture_type {
                TextureType::Normal => {
                    let p = self.scale * point;
                    let mut noise = self.noise(&p);

                    if let RanType::Vector(_) = &self.ran[0] {
                        noise = 0.5 * (noise + 1.0);
                    }
                    noise
                }
                TextureType::Turbulence(depth) => self.turb(point, *depth as usize),
                TextureType::Marble(depth) => {
                    let noise = self.turb(point, *depth as usize);
                    (self.scale.mul_add(point.z, 10.0 * noise).sin() + 1.0) * 0.5
                }
            }
    }
}
