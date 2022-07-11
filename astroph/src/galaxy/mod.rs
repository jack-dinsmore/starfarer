// https://iopscience.iop.org/article/10.1088/0004-637X/783/2/138/pdf
use noise::{OpenSimplex, NoiseFn};

const RENDER_PRECISION: f32 = 0.012;
const PI: f32 = 3.141592653589793238462643383;
const ONE_OVER_E: f32 = 0.36787944117;
const INTENSITY_THRESHOLD: f32 = 2e-5;
const MIN_DISTANCE: f32 = 0.05;

const BULGE_SED: (f32, f32, f32) = (1.0, 0.7, 0.4);
const DISK_SED: (f32, f32, f32) = (0.7, 0.6, 1.0);
const STAR_SED_YOUNG: (f32, f32, f32) = (0.8, 0.7, 1.0);
const STAR_SED_OLD: (f32, f32, f32) = (1.0, 0.3, 0.1);
const DUST_SED: (f32, f32, f32) = (0.65, 0.85, 1.0);

mod alg {
    use num_traits::Float;
    pub fn norm2<T: Float>(v: [T; 3]) -> T {
        v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
    }
    pub fn norm<T: Float>(v: [T; 3]) -> T {
        norm2(v).sqrt()
    }
    pub fn add<T: Float>(v1: [T; 3], v2: [T; 3]) -> [T; 3] {
        [
            v1[0] + v2[0],
            v1[1] + v2[1],
            v1[2] + v2[2],
        ]
    }
    pub fn sub<T: Float>(v1: [T; 3], v2: [T; 3]) -> [T; 3] {
        [
            v1[0] - v2[0],
            v1[1] - v2[1],
            v1[2] - v2[2],
        ]
    }
    pub fn mul<T: Float>(s: T, v: [T; 3]) -> [T; 3] {
        [
            s * v[0],
            s * v[1],
            s * v[2],
        ]
    }
    pub fn dot<T: Float> (v1: [T; 3], v2: [T; 3]) -> T {
        v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2]
    }
}
use alg::*;

pub struct Galaxy {
    radius: f32, // Parsecs
    arm_winding_angle: f32,
    arm_winding_number: usize,
    arm_count: usize,
    arm_width_young: f32,
    arm_width_old: f32,
    arm_old_intensity: f32,
    bulge_intensity: f32,
    bulge_size_frac: f32,
    disk_intensity: f32,
    disk_height_frac: f32,
    disk_dropoff_frac: f32,
    twirl_star: f32,
    twirl_disk: f32,
    twirl_dust: f32,
    dust_intensity: f32,
    dust_shift: f32,
}

pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down
}

fn angle_diff(a: f32, b: f32, max_angle: f32) -> f32 {
    let diff = (a - b).abs() % (2.0 * max_angle);
    if diff > max_angle {
        2.0 * max_angle - diff
    } else {
        diff
    }
}

impl Galaxy {
    pub fn default() -> Galaxy {
        Galaxy {
            radius: 30_000.0,
            arm_winding_angle: 0.9,
            arm_winding_number: 4,
            arm_count: 3,
            arm_width_young: 2.8,
            arm_width_old: 1.4,
            arm_old_intensity: 0.3,
            bulge_intensity: 4.5,
            bulge_size_frac: 0.2,
            disk_intensity: 60.0,
            disk_height_frac: 0.06,
            disk_dropoff_frac: 0.3,
            twirl_star: 0.1,
            twirl_disk: 0.4,
            twirl_dust: 0.5,
            dust_intensity: 1.0,
            dust_shift: -0.3,
        }
    }

    pub fn render(&self, width: usize, height: usize, direction: Direction, pos: [f32; 3]) -> Vec<Vec<(f32, f32, f32)>> {
        let mut output = (0..height).collect::<Vec<_>>().iter().map(|_| { vec![(0.0, 0.0, 0.0); width] }).collect::<Vec<_>>();
        let disk_map = Self::general_map::<8>(12.0, 1.0, 1);
        let star_map = Self::general_map::<8>(80.0, 0.5, 16);
        let dust_map = Self::general_map::<8>(8.0, 0.8, 2);
        //let filament_map = Self::general_map::<7>(Theta::Linear, 0.0, 1.0, 4.0);
        for x in 0..width {
            for y in 0..height {
                let dir = match direction {
                    Direction::Forward => [1.0, (x as f32 / width as f32) * 2.0 - 1.0, (y as f32 / height as f32) * 2.0 - 1.0],
                    Direction::Backward => [-1.0, (x as f32 / width as f32) * 2.0 - 1.0, (y as f32 / height as f32) * 2.0 - 1.0],
                    Direction::Left => [(x as f32 / width as f32) * 2.0 - 1.0, 1.0, (y as f32 / height as f32) * 2.0 - 1.0],
                    Direction::Right => [(x as f32 / width as f32) * 2.0 - 1.0, -1.0, (y as f32 / height as f32) * 2.0 - 1.0],
                    Direction::Up => [(x as f32 / width as f32) * 2.0 - 1.0, (y as f32 / height as f32) * 2.0 - 1.0, 1.0],
                    Direction::Down => [(x as f32 / width as f32) * 2.0 - 1.0, (y as f32 / height as f32) * 2.0 - 1.0, -1.0],
                };
                let dir = mul(1.0 / norm(dir), dir);
                let scaled_pos = mul(1.0 / self.radius, pos);

                let mut poses_far = Vec::new();
                let mut this_pos = add(scaled_pos, mul(MIN_DISTANCE, dir));
                if norm2(this_pos) > 1.0 {
                    // Move this_pos to sphere
                    let alpha = {
                        let d = dot(scaled_pos, dir);
                        let discriminant = d*d - norm2(scaled_pos) + 1.0;
                        if discriminant < 0.0 {
                            continue;
                        }
                        - d - discriminant.sqrt()
                    };
                    this_pos = add(scaled_pos, mul(alpha, dir));
                }
                poses_far.push((this_pos, 0.0));
                loop {
                    let length_scale = self.get_length_scale(this_pos[2]);
                    this_pos = add(this_pos, mul(length_scale, dir));
                    if norm2(this_pos) > 1.0 {
                        // Add a pos that's on the sphere
                        let beta = {
                            let d = dot(scaled_pos, dir);
                            - d + (d*d - norm2(scaled_pos) + 1.0).sqrt()
                        };
                        let sphere_pos = add(scaled_pos, mul(beta, dir));
                        let length_scale = norm(sub(this_pos, sphere_pos));
                        poses_far.iter_mut().last().unwrap().1 = length_scale;
                        break;
                    }
                    poses_far.iter_mut().last().unwrap().1 = length_scale;
                    poses_far.push((this_pos, 0.0));
                }
                poses_far.reverse();

                let mut color = (0.0, 0.0, 0.0);

                for (light_pos, length_scale) in poses_far {
                    // Find brightness
                    let r_disk_now = (light_pos[1]*light_pos[1] + light_pos[0]*light_pos[0]).sqrt();
                    let theta_now = f32::atan2(light_pos[1], light_pos[0]);
                    let arm_theta = self.log_spiral_inv(r_disk_now); //// Takes some time

                    let max_angle = PI / self.arm_count as f32;
                    let arm_scale_young = (1.0 - 0.9 / max_angle * angle_diff(arm_theta, theta_now, max_angle)).powf(self.arm_width_young);
                    let arm_scale_old = (1.0 - 0.9 / max_angle * angle_diff(arm_theta, theta_now, max_angle)).powf(self.arm_width_old);
                    let arm_scale_dust = (1.0 - 0.9 / max_angle * angle_diff(arm_theta + self.dust_shift, theta_now, max_angle)).powf(self.arm_width_young);
                    let disk_scale = self.disk_lum(light_pos[2], r_disk_now);
                    
                    let twirled_pos_disk = [
                        r_disk_now * (theta_now - arm_theta * self.twirl_disk).cos(),
                        r_disk_now * (theta_now - arm_theta * self.twirl_disk).sin(),
                        light_pos[2]
                    ];
                    let twirled_pos_dust = [
                        r_disk_now * (theta_now - arm_theta * self.twirl_dust).cos(),
                        r_disk_now * (theta_now - arm_theta * self.twirl_dust).sin(),
                        light_pos[2]
                    ];
                    let twirled_pos_star_young = [
                        r_disk_now * (theta_now - arm_theta * self.twirl_star).cos(),
                        r_disk_now * (theta_now - arm_theta * self.twirl_star).sin(),
                        light_pos[2]
                    ];
                    let twirled_pos_star_old = [
                        r_disk_now * (theta_now + PI / self.arm_count as f32 - arm_theta * self.twirl_star).cos(),
                        r_disk_now * (theta_now + PI / self.arm_count as f32 - arm_theta * self.twirl_star).sin(),
                        light_pos[2]
                    ];
                                
                    let bulge_intensity = length_scale * self.bulge_lum(norm(light_pos)) * self.bulge_intensity;

                    let mut disk_intensity = length_scale * arm_scale_old * disk_scale * self.disk_intensity;
                    if disk_intensity >= INTENSITY_THRESHOLD {
                        disk_intensity *= disk_map(twirled_pos_disk);
                    }
                    let mut star_intensity_young = length_scale * arm_scale_young * disk_scale;
                    if star_intensity_young >= INTENSITY_THRESHOLD {
                        star_intensity_young *= star_map(twirled_pos_star_young)
                    }
                    let mut star_intensity_old = length_scale * arm_scale_old * disk_scale * self.arm_old_intensity;
                    if star_intensity_old >= INTENSITY_THRESHOLD {
                        star_intensity_old *= star_map(twirled_pos_star_old);
                    }
                    let mut dust_intensity = length_scale * arm_scale_dust * disk_scale * self.dust_intensity;
                    if dust_intensity >= INTENSITY_THRESHOLD {
                        dust_intensity *= dust_map(twirled_pos_dust);
                    } 
                    dust_intensity *= 1000.0;
                    
                    color = (
                        color.0
                            + (bulge_intensity * BULGE_SED.0 
                            + disk_intensity * DISK_SED.0 
                            + star_intensity_young * STAR_SED_YOUNG.0 
                            + star_intensity_old * STAR_SED_OLD.0
                        ),
                        color.1 
                            + (bulge_intensity * BULGE_SED.1 
                            + disk_intensity * DISK_SED.1
                            + star_intensity_young * STAR_SED_YOUNG.1
                            + star_intensity_old * STAR_SED_OLD.1
                        ),
                        color.2
                            + (bulge_intensity * BULGE_SED.2
                            + disk_intensity * DISK_SED.2
                            + star_intensity_young * STAR_SED_YOUNG.2
                            + star_intensity_old * STAR_SED_OLD.2
                        ),
                    );

                    color = (
                        color.0 * (-dust_intensity * DUST_SED.0).exp(),
                        color.1 * (-dust_intensity * DUST_SED.1).exp(),
                        color.2 * (-dust_intensity * DUST_SED.2).exp(),
                    );
                }
                output[y][x] = color;
            }
        }
        output
    }
}

impl Galaxy {
    fn general_map<const NUM_OCTAVES: usize>(freq_scale: f32, spectral_index: f32, power: i32) -> impl Fn([f32; 3]) -> f32 {

        let noise = OpenSimplex::new();
        let mut scales = [0.0; NUM_OCTAVES];
        let mut power_scales = [0.0; NUM_OCTAVES];
        for octave_num in 1..NUM_OCTAVES {
            scales[octave_num] = octave_num as f32 * freq_scale;
            power_scales[octave_num] = (octave_num as f32).powf(-spectral_index);
        }
        move |pos| {
            let mut sum = 0.5f32;
            for octave_num in 1..NUM_OCTAVES {
                let new_pos = mul(scales[octave_num], pos);
                sum += power_scales[octave_num] * noise.get([new_pos[0] as f64, new_pos[1] as f64, new_pos[2] as f64]) as f32;
            }
            let p = sum.powi(power);
            p.max(0.0)
        }
    }

    fn log_spiral_inv(&self, r: f32) -> f32 {
        ((1.0 / r).exp() / self.arm_winding_angle).atan() * 2.0 * self.arm_winding_number as f32
    }

    fn bulge_lum(&self, r: f32) -> f32 {
        let r_scale = r / self.bulge_size_frac;
        r_scale.powf(-0.855) * (-r_scale).exp()
    }

    fn disk_lum(&self, z: f32, r: f32) -> f32 {
        let ratio = r / self.disk_dropoff_frac;
        (z / self.disk_height_frac).cosh().powi(-2) * (if ratio > 1.0 { (-ratio).exp() } else { ratio * ONE_OVER_E })
    }
    fn get_length_scale(&self, z: f32) -> f32 {
        //((z / self.disk_height_frac).cosh().powi(2) * RENDER_PRECISION).min(0.2)
        //(((z / self.disk_height_frac).powi(2) + 1.0)* RENDER_PRECISION).min(0.3)
        (((z / self.disk_height_frac).abs() + 1.0)* RENDER_PRECISION).min(0.3)
    }
}