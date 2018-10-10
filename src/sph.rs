use kernels;
use math;

const MAX_X: f64 = 5.0;
const MIN_X: f64 = 0.0;
const MAX_Y: f64 = 5.0;
const REST_DENS: f64 = 5.0;
const GAS_CONST: f64 = 2000.0;
const H: f64 = 5.0;
const M: f64 = 65.0;
const MU: f64 = 250.0;

#[derive(Clone)]
pub struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    fx: f64,
    fy: f64,
    ofx: f64,
    ofy: f64,
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
        ofx: 0.,
        ofy: 0.,
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

pub fn update_density(particles: &mut Vec<Particle>) -> f64 {
    for particle in particles.iter_mut() {
        particle.density = 0.0;
    }
    let mut max_density = 0.0;
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
            particle.density = density;
            if density > max_density {
                max_density = density;
            }
            particle.pressure = GAS_CONST*(particle.density-REST_DENS);
        }
    }
    {
        js! {
            console.log("Max density: " + @{max_density});
        }
    }
    return max_density;
}

pub fn update_state(particles: &mut Vec<Particle>, dt: f64) {
    for particle in particles.iter_mut() {
        particle.fx = 0.0;
        particle.fy = 50.0;

        // Velocity Verlet (position update)
        particle.x = particle.x + particle.vx * dt + 0.5*(particle.fx / M) * dt * dt;
        particle.y = particle.y + particle.vy * dt + 0.5*(particle.fy / M) * dt * dt;
        if particle.x > MAX_X {
            particle.x = MAX_X;
            particle.vx = 0.0;
        }
        if particle.x < MIN_X {
            particle.x = MIN_X;
            particle.vx = 0.0;
        }
        if particle.y > MAX_Y {
            particle.y = MAX_Y;
            particle.vy = 0.0;
        }
    }

    for i in 0..particles.len() {
        let mut fx = 0.;
        let mut fy = 0.;
        {
            let particle1 = &particles[i];
            for j in 0..particles.len() {
                let particle2 = &particles[j];
                let rx = particle1.x - particle2.x;
                let ry = particle1.y - particle2.y;
                let p_over_rho_1 = particle1.pressure / math::pow(particle1.density, 2);
                let p_over_rho_2 = particle2.pressure / math::pow(particle2.density, 2);
                let (grad_x, grad_y) = kernels::grad_kernel_2d(rx, ry, H);
                let advection = -M*(p_over_rho_1 + p_over_rho_2);
                let diffusion = MU / particle1.density * M / particle2.density;
                fx += grad_x * advection + diffusion * (particle2.vx - particle1.vx);
                fy += grad_y * advection + diffusion * (particle2.vy - particle1.vy);
            }
        }
        {
            let particle = &mut particles[i];
            particle.ofx = particle.fx;
            particle.ofy = particle.fy;
            particle.fx = fx;
            particle.fy = fy;
        }
    }

    {
        let val = particles[0].vy;
        js! {
            console.log("Particle 1 vy: " + @{val});
        }
    }
    for particle in particles.iter_mut() {
        // Velocity Verlet (velocity update)
        particle.vx = particle.vx + (particle.ofx + particle.fx) / M / 2.0 * dt;
        particle.vy = particle.vy + (particle.ofy + particle.fy) / M / 2.0 * dt;
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
