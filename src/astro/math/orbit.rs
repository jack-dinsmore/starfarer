use cgmath::{Vector3, InnerSpace};

/// Untested code to get the position of a satellite given the time since it passed through perigee.
pub fn get_pos(mu: f64, time_since_perigee: f64, eccentricity: Vector3<f64>, ang_mom: Vector3<f64>) -> Vector3<f64> {
    const TOL: f64 = 1.0e-5;

    let e = eccentricity.magnitude();
    let a = ang_mom.magnitude2() / mu / (1.0 - e*e);
    let scale = (a.abs().powi(3) / mu).sqrt();
    let target = if e < 1.0 {
        let period = 2.0 * std::f64::consts::PI * (a.powi(3) / mu).sqrt();
        (time_since_perigee % period) / scale
    } else {
        time_since_perigee / scale
    };

    let nu = if e > 1.0 {
        // Hyperbola: solve e sinh F - F = target
        // cosh F = (e + cos v) / (1 + e * cos v)
        // (cosh F - e) / (1 - cosh F * e) = cos v

        let mut val = TOL + 1.0;
        let mut f = target / (e - 1.0);

        while val.abs() > TOL {
            let derivative = e * f.cosh() - 1.0;
            f -= val / derivative;
            val = e * f.sinh() - f - target;
        }

        ((f.cosh() - e) / (1.0 - f.cosh() * e)).acos()
    } else {
        // Ellipse: solve F - e sin F = target
        // cos E = (e + cos v) / (1 + e * cos v)
        // (cos E - e) / (1 - cos E * e) = cos v 

        let mut val = TOL + 1.0;
        let mut f = target / (1.0 - e);

        while val.abs() > TOL {
            let derivative = 1.0 - e * f.cos();
            f -= val / derivative;
            val = f - e * f.sin() - target;
        }

        ((f.cos() - e) / (1.0 - f.cos() * e)).acos()
    };

    (eccentricity * nu.cos() + ang_mom.normalize().cross(eccentricity) * nu.sin()) / e
}