use {
    crate::{geometry::Geometry, material::Material, prelude::*},
    std::fmt::{Debug, Formatter},
};

pub struct HitRecord<'m> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: &'m dyn Material,
    pub unit: f64,
    pub u: f64,
    pub v: f64,
    pub outside: bool,
}

impl Debug for HitRecord<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "HitRecord {{ unit: {}, hit: {:?}, normal: {:?}, outside: {} }}",
            self.unit, self.point, self.normal, self.outside
        ))
    }
}

impl<'m> HitRecord<'m> {
    pub fn new<G: Geometry>(r: &Ray, object: &'m G, unit: f64) -> Self {
        let point = r.position_after(unit);
        let mut normal = object.normal(&point);
        let outside = r.direction.dot(&normal) < 0.0;
        if !outside {
            normal.reverse();
        }
        let material = object.material();
        Self {
            point,
            normal,
            material,
            unit,
            // TODO: Fill u v value
            u: 0.0,
            v: 0.0,
            outside,
        }
    }
}
