// extern crate num_traits;
// mod math;
use kernels;
use math;

// use num_traits::pow;

const REST_DENS: f64 = 1000.0;
const GAS_CONST: f64 = 2000.0;
const H: f64 = 16.0;
const M: f64 = 65.0;
// const VISC: f64 = 250.0;
// const DT: f64 = 0.0008;

#[derive(Clone)]
pub struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fx: f64,
    fy: f64,
    density: f64,
    pressure: f64
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

pub fn create_initial_state() -> Vec<Particle> {
    let particles = vec![
        create_particle(1.,1.),
        create_particle(2.,2.),
        create_particle(3.,3.),
        create_particle(4.,4.),
        create_particle(5.,5.),
    ];
    return particles;
}

pub fn update_density(particles: &mut Vec<Particle>) {
    for particle in particles.iter_mut() {
        particle.density = 0.0;
    }

    // Wat?!
    for i in 0..particles.len() {
        let mut density = 0.;
        {
            let particle1 = &particles[i];
            for j in 0..particles.len() {
                let particle2 = &particles[j];
                let r = math::length(particle1.x - particle2.x, particle1.y - particle2.y);
                density += M*kernels::kernel_2d(r, H);
            }
        }
        {
            let particle = &mut particles[i];
            particle.pressure = GAS_CONST*(particle.density-REST_DENS);
            particle.density = density;
        }
    }
}

pub fn density(particles: &Vec<Particle>, x: f64, y: f64) -> f64 {
    let mut density = 0.0;
    for particle in particles.iter() {
        let r = math::length(x - particle.x, y - particle.y);
        density += M*kernels::kernel_2d(r, H);
    }
    return density;
}
