use crate::grid;
use crate::kernels;
use crate::math;

const N: u32 = 30;
pub const N_PARTICLES: u32 = N * N;
pub const MAX_X: f64 = 4.95;
pub const MIN_X: f64 = 0.05;
pub const MAX_Y: f64 = 4.95;
pub const MIN_Y: f64 = 0.05;
const START_MIN_X: f64 = 0.1;
const START_MAX_X: f64 = 4.9;
const START_MIN_Y: f64 = 1.5;
const START_MAX_Y: f64 = 3.9;
const GAS_CONST: f64 = 1000.0;
pub const H: f64 = 4.0 * (START_MAX_X - START_MIN_X) / N as f64;
const M: f64 = 65.0;
const MU: f64 = 0.1;
const DAMPING: f64 = 0.9;
const REST_DENS: f64 =
    M * (N * N) as f64 / ((START_MAX_X - START_MIN_X) * (START_MAX_Y - START_MIN_Y));
const GRAVITY: f64 = 100.0; // Acceleration * Area ?

const DUCK_X: f64 = 2.5;
const DUCK_Y: f64 = 1.0;
const DUCK_RADIUS: f64 = 0.4;
const DUCK_MASS: f64 = 10. * M;

pub struct State {
    pub particles: Vec<Particle>,
    pub duck: Duck,
}

pub struct Duck {
    pub x: f64,
    pub y: f64,
    vx: f64,
    vy: f64,
}

impl Duck {
    pub fn new() -> Duck {
        Duck {
            x: DUCK_X,
            y: DUCK_Y,
            vx: 0.0,
            vy: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub fx: f64,
    pub fy: f64,
    pub ofx: f64,
    pub ofy: f64,
    pub density: f64,
    pub pressure: f64,
}

impl Particle {
    pub fn new(x: f64, y: f64) -> Particle {
        Particle {
            x: x,
            y: y,
            vx: 0.,
            vy: 0.,
            fx: 0.,
            fy: 0.,
            ofx: 0.,
            ofy: 0.,
            density: 1.,
            pressure: 0.,
        }
    }
}

pub struct SPHDebug {
    pub max_density: f64,
    pub n_neighbours: usize,
    pub frame_time: u128,
    pub h: f64,
    pub grid_width: u64,
}

impl SPHDebug {
    pub fn new() -> SPHDebug {
        SPHDebug {
            max_density: 0.0,
            n_neighbours: 0,
            frame_time: 0,
            h: 0.0,
            grid_width: 0,
        }
    }
}

pub fn create_initial_state() -> State {
    let mut particles = Vec::new();
    let width = START_MAX_X - START_MIN_X;
    let height = START_MAX_Y - START_MIN_Y;
    let dx = width / N as f64;
    let dy = height / N as f64;
    for x in 0..N {
        for y in 0..N {
            let x = START_MIN_X + (x as f64) * dx;
            let y = START_MIN_Y + (y as f64) * dy;
            let particle = Particle::new(x, y);
            particles.push(particle);
        }
    }

    State {
        particles: particles,
        duck: Duck::new(),
    }
}

pub fn update_density(
    particles: &mut Vec<Particle>,
    grid: &grid::Grid,
    debug: SPHDebug,
) -> SPHDebug {
    for particle in particles.iter_mut() {
        particle.density = 0.0;
    }
    let mut max_density = 0.0;
    let mut n_neighbours = 0;
    // Wat?!
    for i in 0..particles.len() {
        let mut density = 0.;
        {
            let particle1 = &particles[i];
            let neighbours = grid.get_neighbours(particle1.x, particle1.y);
            if neighbours.len() > n_neighbours {
                n_neighbours = neighbours.len()
            }
            for j in neighbours {
                let particle2 = &particles[j as usize];
                let r = math::length(particle1.x - particle2.x, particle1.y - particle2.y);
                density += M * kernels::kernel_2d(r, H);
            }
        }
        {
            let particle = &mut particles[i];
            particle.density = density;
            if density > max_density {
                max_density = density;
            }
            // particle.pressure = GAS_CONST * f64::max(particle.density - REST_DENS, 0.0);
            particle.pressure = GAS_CONST * (particle.density - REST_DENS);
        }
    }
    return SPHDebug {
        max_density,
        n_neighbours,
        ..debug
    };
}

pub fn calculate_forces(
    particles: &mut Vec<Particle>,
    grid: &grid::Grid,
    debug: SPHDebug,
) -> SPHDebug {
    let new_forces: Vec<_> = (0..particles.len())
        .into_iter()
        .map(|i| {
            let mut fx = 0.;
            let mut fy: f64;
            {
                let particle1 = &particles[i];
                fy = GRAVITY * particle1.density;
                let neighbours = grid.get_neighbours(particle1.x, particle1.y);
                for j in neighbours {
                    if i as u32 != j {
                        let particle2 = &particles[j as usize];
                        let rx = particle1.x - particle2.x;
                        let ry = particle1.y - particle2.y;
                        let p_over_rho_1 = particle1.pressure / particle1.density.powi(2);
                        let p_over_rho_2 = particle2.pressure / particle2.density.powi(2);
                        let (grad_x, grad_y) = kernels::grad_kernel_2d(rx, ry, H);
                        let laplacian = kernels::laplace_kernel_2d(math::length(rx, ry), H);
                        let advection = -M * particle1.density * (p_over_rho_1 + p_over_rho_2);
                        let diffusion = -laplacian * MU * M / particle2.density;
                        fx += grad_x * advection + diffusion * (particle2.vx - particle1.vx);
                        fy += grad_y * advection + diffusion * (particle2.vy - particle1.vy);
                    }
                }
            }
            (fx, fy)
        })
        .collect();
    for (i, (fx, fy)) in new_forces.into_iter().enumerate() {
        let particle = &mut particles[i];
        particle.ofx = particle.fx;
        particle.ofy = particle.fy;
        particle.fx = fx;
        particle.fy = fy;
    }
    debug
}

pub fn update_state(state: &mut State, dt: f64, debug: SPHDebug) -> (grid::Grid, SPHDebug) {
    let mut grid = grid::create_grid(H, MIN_X, MAX_X, MIN_Y, MAX_Y);

    let duck = &mut state.duck;

    duck.x += duck.vx * dt;
    duck.y += duck.vy * dt;

    if duck.y > MAX_Y - DUCK_RADIUS {
        duck.vy = -duck.vy;
        duck.y = MAX_Y - DUCK_RADIUS;
    }
    if duck.x > MAX_X - DUCK_RADIUS {
        duck.vx = -duck.vx;
        duck.x = MAX_X - DUCK_RADIUS;
    }
    if duck.x < MIN_X + DUCK_RADIUS {
        duck.vx = -duck.vx;
        duck.x = MIN_X + DUCK_RADIUS;
    }

    duck.vy += GRAVITY * dt;

    for (index, particle) in state.particles.iter_mut().enumerate() {
        // Velocity Verlet (position update)
        particle.x =
            particle.x + particle.vx * dt + 0.5 * (particle.fx / particle.density) * dt * dt;
        particle.y =
            particle.y + particle.vy * dt + 0.5 * (particle.fy / particle.density) * dt * dt;

        if particle.x > MAX_X {
            particle.x = MAX_X;
            particle.vx = -DAMPING * particle.vx;
        }
        if particle.x < MIN_X {
            particle.x = MIN_X;
            particle.vx = -DAMPING * particle.vx;
        }
        if particle.y > MAX_Y {
            particle.y = MAX_Y;
            particle.vy = -DAMPING * particle.vy;
        }
        if particle.y < MIN_Y {
            particle.y = MIN_Y;
            particle.vy = -DAMPING * particle.vy;
        }
        if (particle.x - duck.x).powi(2) + (particle.y - duck.y).powi(2) < DUCK_RADIUS.powi(2) {
            let distance_x = particle.x - duck.x;
            let distance_y = particle.y - duck.y;
            let distance = f64::sqrt(distance_x.powi(2) + distance_y.powi(2));
            let normal_x = distance_x / distance;
            let normal_y = distance_y / distance;
            let dot = normal_x * particle.vx + normal_y * particle.vy;
            particle.vx -= (1.0 + DAMPING) * dot * normal_x;
            particle.vy -= (1.0 + DAMPING) * dot * normal_y;
            particle.x += normal_x * (DUCK_RADIUS - distance);
            particle.y += normal_y * (DUCK_RADIUS - distance);

            duck.vx += M / DUCK_MASS * (1.0 + DAMPING) * dot * normal_x;
            duck.vy += M / DUCK_MASS * (1.0 + DAMPING) * dot * normal_y;
        }
        grid.add_particle(index as u32, particle.x, particle.y);
    }
    let debug1 = update_density(&mut state.particles, &grid, debug);
    let debug2 = calculate_forces(&mut state.particles, &grid, debug1);

    for particle in state.particles.iter_mut() {
        // Velocity Verlet (velocity update)
        particle.vx = particle.vx + (particle.ofx + particle.fx) / particle.density / 2.0 * dt;
        particle.vy = particle.vy + (particle.ofy + particle.fy) / particle.density / 2.0 * dt;
    }
    return (grid, SPHDebug { h: H, ..debug2 });
}

#[allow(dead_code)]
pub fn density(particles: &Vec<Particle>, grid: &grid::Grid, x: f64, y: f64) -> f64 {
    let mut density = 0.0;
    let neighbours = grid.get_neighbours(x, y);
    for i in neighbours {
        let particle = &particles[i as usize];
        let r = math::length(x - particle.x, y - particle.y);
        density += M * kernels::kernel_2d(r, H);
    }
    return density;
}
