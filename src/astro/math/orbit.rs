use cgmath::Vector3;

pub fn get_pos(mu: f64, time_since_perigee: f64, eccentricity: Vector3<f64>, ang_mom: Vector3<f64>) -> Vector3<f64> {
    let e = eccentricity.magnitude();
    let scale = ((ang_mom.magnitude2() / mu / (1 - e*e)).abs().powi(3) / mu).sqrt();
    let target = time_since_perigee / scale;
    let mut val = f32::MAX;
    while (val - target).abs() < 
    if e2 > 1 {
        // Hyperbola: solve e sinh F - F = target

    } else {
        // Ellipse: solve e F - sin F = target
        
    }
}