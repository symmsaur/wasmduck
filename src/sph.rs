extern crate num_traits;

use std::f64::consts::PI;
use num_traits::pow;

const REST_DENS: f64 = 1000.0;
const GAS_CONST: f64 = 2000.0;
const H: f64 = 16.0;
const M: f64 = 65.0;
// const VISC: f64 = 250.0;
// const DT: f64 = 0.0008;

#[derive(Clone)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fx: f64,
    fy: f64,
    density: f64,
    pressure: f64
}

fn kernel_2d(r: f64, h: f64) -> f64 {
    let inv_h = 1.0 / h;
    let q = r*inv_h;

    if q > 2.0 {
        return 0.0;
    }

    let alpha_d = 7.0 / 4.0 / PI * pow(inv_h, 2);
    return pow(1.0 - 0.5 * q, 4) * (2.0 * q + 1.0) * alpha_d;
}

fn kernel_3d(r: f64, h: f64) -> f64 {
    let inv_h = 1.0 / h;
    let q = r*inv_h;

    if q > 2.0 {
        return 0.0;
    }

    let alpha_d = 21.0 / 16.0 / PI * pow(inv_h, 3);
    return pow(1.0 - 0.5 * q, 4) * (2.0 * q + 1.0) * alpha_d;
}

fn create_particle(x_: f64, y_: f64) -> Particle {
    Particle {
        x: x_,
        y: y_,
        vx: 0.,
        vy: 0.,
        fx: 0.,
        fy: 0.,
        density: 0.,
        pressure: 0.
    }
}

pub fn density(x: f64, y: f64) -> f64 {
    let mut particles: Vec<Particle> = Vec::new();
    particles.push(create_particle(1.,1.));
    particles.push(create_particle(2.,2.));
    particles.push(create_particle(3.,3.));
    particles.push(create_particle(4.,4.));
    particles.push(create_particle(5.,5.));
    
    for particle in &mut particles {
        particle.density = 0.0;
    }

    let mut cloned_particles = particles.clone();

    for particle1 in &mut cloned_particles {
        for particle2 in &particles {
            let r = f64::sqrt(pow(particle1.x - particle2.x, 2) + pow(particle1.y - particle2.y, 2));
            particle1.density += M*kernel_2d(r, H);
        }
        particle1.pressure = GAS_CONST*(particle1.density-REST_DENS);
    }

    let mut density = 0.0;
    for particle in &cloned_particles {
        let r = f64::sqrt(pow(x - particle.x, 2) + pow(y - particle.y, 2));
        particle1.density += M*kernel_2d(r, H);
    }
    return density;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_2d() {
        let tolerance = 1e-15;
        assert!((kernel_2d(0.5, 0.5)-0.4177817256162253).abs() < tolerance);
        assert!((kernel_2d(1.0, 0.5)-0.0000000000000000).abs() < tolerance);
        assert!((kernel_2d(0.0, 1.0)-0.5570423008216338).abs() < tolerance);
        assert!((kernel_2d(1.0, 1.0)-0.1044454314040563).abs() < tolerance);
        assert!((kernel_2d(2.0, 1.0)-0.0000000000000000).abs() < tolerance);
    }

    #[test]
    fn test_kernel_3d() {
        let tolerance = 1e-15;
        assert!((kernel_3d(0.5, 0.5)-0.6266725884243379).abs() < tolerance);
        assert!((kernel_3d(1.0, 0.5)-0.0000000000000000).abs() < tolerance);
        assert!((kernel_3d(0.0, 1.0)-0.4177817256162253).abs() < tolerance);
        assert!((kernel_3d(1.0, 1.0)-0.0783340735530422).abs() < tolerance);
        assert!((kernel_3d(2.0, 1.0)-0.0000000000000000).abs() < tolerance);
    }
}
