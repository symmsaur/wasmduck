use na::{Point2, Vector2};
use num_traits::identities::Zero;

use grid;
use kernels;

const N: u32 = 30;
pub const N_PARTICLES: u32 = N * N;
pub const MAX_X: f64 = 4.95;
pub const MIN_X: f64 = 0.05;
pub const MAX_Y: f64 = 4.95;
pub const MIN_Y: f64 = 0.05;
const START_MIN_X: f64 = 0.0;
const START_MAX_X: f64 = 2.0;
const START_MIN_Y: f64 = 2.5;
const START_MAX_Y: f64 = 4.5;
const GAS_CONST: f64 = 2000.0;
const H: f64 = 4.0 * (START_MAX_X - START_MIN_X) / N as f64;
const M: f64 = 65.0;
const MU: f64 = 0.1;
const DAMPING: f64 = 0.9;
const REST_DENS: f64 =
    M * (N * N) as f64 / ((START_MAX_X - START_MIN_X) * (START_MAX_Y - START_MIN_Y));
const GRAVITY: f64 = 50.0; // Acceleration * Area ?

const DUCK_X: f64 = 2.5;
const DUCK_Y: f64 = 4.5;
const DUCK_RADIUS: f64 = 0.4;
const DUCK_MASS: f64 = 10. * M;

pub struct State {
    pub particles: Vec<Particle>,
    duck: Duck,
}

pub struct Duck {
    pos: Point2<f64>,
    v: Vector2<f64>,
}

impl Duck {
    fn new() -> Duck {
        Duck {
            pos: Point2::<f64>::new(DUCK_X, DUCK_Y),
            v: Vector2::<f64>::zero(),
        }
    }
}

#[derive(Clone)]
pub struct Particle {
    pub pos: Point2<f64>,
    v: Vector2<f64>,
    f: Vector2<f64>,
    of: Vector2<f64>,
    density: f64,
    pressure: f64,
}

impl Particle {
    fn new(p: Point2<f64>) -> Particle {
        Particle {
            pos: p,
            v: Vector2::<f64>::zero(),
            f: Vector2::<f64>::zero(),
            of: Vector2::<f64>::zero(),
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
            let particle = Particle::new(Point2::<f64>::new(x, y));
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
            let neighbours = grid.get_neighbours(particle1.pos);
            if neighbours.len() > n_neighbours {
                n_neighbours = neighbours.len()
            }
            for j in neighbours {
                let particle2 = &particles[j as usize];
                let r = (particle1.pos - particle2.pos).norm();
                density += M * kernels::kernel_2d(r, H);
            }
        }
        {
            let particle = &mut particles[i];
            particle.density = density;
            if density > max_density {
                max_density = density;
            }
            particle.pressure = GAS_CONST * f64::max(particle.density - REST_DENS, 0.0);
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
    let temp_particles = particles.clone();
    let new_forces: Vec<_> = (0..particles.len())
        .into_iter()
        .map(|i| {
            let mut f = Vector2::<f64>::zero();
            {
                let particle1 = &temp_particles[i];
                f = Vector2::<f64>::new(0.0, GRAVITY * particle1.density);
                let neighbours = grid.get_neighbours(particle1.pos);
                for j in neighbours {
                    if i as u32 != j {
                        let particle2 = &temp_particles[j as usize];
                        let r = particle1.pos - particle2.pos;
                        let p_over_rho_1 = particle1.pressure / particle1.density.powi(2);
                        let p_over_rho_2 = particle2.pressure / particle2.density.powi(2);
                        let grad = kernels::grad_kernel_2d(r, H);
                        let laplacian = kernels::laplace_kernel_2d(r.norm(), H);
                        let advection = -M * particle1.density * (p_over_rho_1 + p_over_rho_2);
                        let diffusion = -laplacian * MU * M / particle2.density;
                        f += grad * advection + diffusion * (particle2.v - particle1.v);
                    }
                }
            }
            f
        })
        .collect();
    for (i, f) in new_forces.into_iter().enumerate() {
        let particle = &mut particles[i];
        particle.of = particle.f;
        particle.f = f;
    }
    debug
}

pub fn update_state(state: &mut State, dt: f64, debug: SPHDebug) -> (grid::Grid, SPHDebug) {
    let mut grid = grid::create_grid(H, MIN_X, MAX_X, MIN_Y, MAX_Y);

    let duck = &mut state.duck;

    duck.pos += duck.v * dt;

    let duck_x = duck.pos[0];
    let duck_y = duck.pos[1];

    if duck_x < MIN_X + DUCK_RADIUS {
        let refl = na::geometry::Reflection::new(Vector2::<f64>::x_axis(), MIN_X);
        refl.reflect(&mut duck.v);
        duck.pos = Point2::new(MIN_X + DUCK_RADIUS, duck_y);
    }
    if duck_x > MAX_X - DUCK_RADIUS {
        let refl = na::geometry::Reflection::new(Vector2::<f64>::x_axis(), MAX_X);
        refl.reflect(&mut duck.v);
        duck.pos = Point2::new(MAX_X - DUCK_RADIUS, duck_y);
    }
    if duck_y < MIN_Y + DUCK_RADIUS {
        let refl = na::geometry::Reflection::new(Vector2::<f64>::y_axis(), MIN_Y);
        refl.reflect(&mut duck.v);
        duck.pos = Point2::new(duck_x, MIN_Y + DUCK_RADIUS);
    }
    if duck_y > MAX_Y - DUCK_RADIUS {
        let refl = na::geometry::Reflection::new(Vector2::<f64>::y_axis(), MAX_Y);
        refl.reflect(&mut duck.v);
        duck.pos = Point2::new(duck_x, MAX_Y - DUCK_RADIUS);
    }

    duck.v += Vector2::<f64>::y() * GRAVITY * dt;

    for (index, particle) in state.particles.iter_mut().enumerate() {
        // Velocity Verlet (position update)
        particle.pos += particle.v * dt + 0.5 * (particle.f / particle.density) * dt * dt;
        let px = particle.pos[0];
        let py = particle.pos[1];

        if px > MAX_X {
            particle.pos = Point2::<f64>::new(MAX_X, py);
            let refl = na::geometry::Reflection::new(Vector2::<f64>::x_axis(), MAX_X);
            refl.reflect(&mut particle.v);
            particle.v *= DAMPING;
        }
        if px < MIN_X {
            particle.pos = Point2::<f64>::new(MIN_X, py);
            let refl = na::geometry::Reflection::new(Vector2::<f64>::x_axis(), MIN_X);
            refl.reflect(&mut particle.v);
            particle.v *= DAMPING;
        }
        if py > MAX_Y {
            particle.pos = Point2::<f64>::new(px, MAX_Y);
            let refl = na::geometry::Reflection::new(Vector2::<f64>::y_axis(), MAX_Y);
            refl.reflect(&mut particle.v);
            particle.v *= DAMPING;
        }
        if py < MIN_Y {
            particle.pos = Point2::<f64>::new(py, MIN_Y);
            let refl = na::geometry::Reflection::new(Vector2::<f64>::y_axis(), MIN_Y);
            refl.reflect(&mut particle.v);
            particle.v *= DAMPING;
        }
        if (particle.pos - duck.pos).norm() < DUCK_RADIUS {
            let displacement = particle.pos - duck.pos;
            let distance = displacement.norm();
            let normal = displacement / distance;
            let dot = na::dot(&normal, &particle.v);
            particle.v -= (1.0 + DAMPING) * dot * normal;
            particle.pos += normal * (DUCK_RADIUS - distance);
            duck.v += M / DUCK_MASS * (1.0 + DAMPING) * dot * normal;
        }
        grid.add_particle(index as u32, particle.pos);
    }
    let debug1 = update_density(&mut state.particles, &grid, debug);
    let debug2 = calculate_forces(&mut state.particles, &grid, debug1);

    for particle in state.particles.iter_mut() {
        // Velocity Verlet (velocity update)
        particle.v += (particle.of + particle.f) / particle.density / 2.0 * dt;
    }
    return (grid, SPHDebug { h: H, ..debug2 });
}

pub fn density(particles: &Vec<Particle>, grid: &grid::Grid, p: Point2<f64>) -> f64 {
    let mut density = 0.0;
    let neighbours = grid.get_neighbours(p);
    for i in neighbours {
        let particle = &particles[i as usize];
        let r = (p - particle.pos).norm();
        density += M * kernels::kernel_2d(r, H);
    }
    return density;
}
