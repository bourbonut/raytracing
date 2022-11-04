use glam::Vec3A;

static BASE: [Vec3A; 3] = [Vec3A::X, Vec3A::Y, Vec3A::Z];

#[derive(Debug, Default, Clone)]
pub struct Path{
    keys: [u32; 2],
    direction: Vec3A,
    unit: f32,
    pub current_key: u32,
    pub current_point: Vec3A,
    dl: [u32; 3],
    div: [i32; 3],
    signnum: Vec3A, 
    borders: Vec3A,
    first: bool,
}

fn unit_round(value: f32, sign: f32, unit:f32) -> f32 {
    if sign > 0. {
        (value.div_euclid(unit) + 1.) * unit 
    } else {
        let divisor = value.div_euclid(unit);
        divisor * unit - if value - divisor * unit < 2. * f32::EPSILON { unit } else { 0. }
    }
}

fn round_value(v: Vec3A, signnum: Vec3A, unit: f32) -> Vec3A {
    Vec3A::new(
        unit_round(v.x, signnum.x, unit),
        unit_round(v.y, signnum.y, unit),
        unit_round(v.z, signnum.z, unit)
    )
}

impl Path {
    pub fn new(in_key: u32, in_point:Vec3A, out_key:u32, out_point:Vec3A, unit: f32, dx:u32, dy:u32, dz:u32) -> Self {
        let keys = [in_key, out_key];
        let direction = out_point.clone() - in_point.clone();
        let signnum = Vec3A::new(direction.x.signum(), direction.y.signum(), direction.z.signum());
        let div = [
            (direction.x.abs().div_euclid(unit) * direction.x.signum()) as i32, 
            (direction.y.abs().div_euclid(unit) * direction.y.signum()) as i32,
            (direction.z.abs().div_euclid(unit) * direction.z.signum()) as i32
        ];
        let dl = [dx, dy, dz];
        let borders = round_value(in_point, signnum, unit);
        // println!("in_point = {:?}", in_point);
        // println!("out_point = {:?}", out_point);
        Path {keys, direction, unit, current_key: in_key, current_point: in_point, dl, div, signnum, borders, first:true}
    }

    fn _cut_value(&self, value: f32, sign: f32) -> u32 {
        let divisor = value.div_euclid(self.unit);
        let rest = value - divisor * self.unit;
        if divisor == 0. {
            return divisor as u32
        } else if sign > 0. {
            return divisor as u32
        } else if rest > 0. {
            return divisor as u32
        } else {
            return divisor as u32 - 1
        }
    }

    fn _compute_key(&self, v:Vec3A) -> u32 {
        // println!("1");
        let x = self._cut_value(v.x, self.signnum.x);
        let y = self._cut_value(v.y, self.signnum.y);
        let z = self._cut_value(v.z, self.signnum.z);
        // println!("2");
        self.dl[0] * self.dl[1] * z + self.dl[0] * y + x
    }

    pub fn next(&mut self) -> bool {
        // println!("self.current_key = {:?}", self.current_key);
        // println!("self.direction = {:?}", self.direction);
        if self.first { self.first = false; return true }
        if self.current_key == self.keys[1] { return false }
        // println!("self.div = {:?}", self.div);
        // println!("self.borders = {:?}", self.borders);
        // println!("self.current_point = {:?}", self.current_point);
        let diffzero = |x:&(usize, &f32)| -> bool { self.div[x.0] != 0 };
        let min = |a:&(usize, &f32), b:&(usize, &f32)| a.1.partial_cmp(b.1).unwrap();
        let all_t = (self.borders - self.current_point) / self.direction;
        // println!("all_t = {:?}", all_t);
        if let Some((i, &t)) = all_t.to_array().iter().enumerate().filter(diffzero).min_by(min) {
            self.current_point = self.current_point + self.direction * t;
            self.current_key = self._compute_key(self.current_point);
            self.div[i] -= self.signnum[i] as i32;
            self.borders += self.signnum * BASE[i] * if self.div[i] != 0 { 1. } else { 0. };
            // println!("ok");
            // println!("self.current_key = {:?}", self.current_key);
            // println!("self.div = {:?}", self.div);
            // println!("self.current_key = {:?}", self.current_key);
            true
        } else { false }
    }
}
