use crate::sph;

use std::convert::TryFrom;

use nom::{
    self,
    IResult,
    number::complete::le_f64,
    number::complete::le_u64,
    multi::count,
};

use nalgebra ::{
    self as na,
    VectorN,
    U3,
};

use sph::Particle;
use std::io::prelude::*;

#[derive(Debug, PartialEq)]
struct ParticleDto {
    position: VectorN<f64, U3>,
    velocity: VectorN<f64, U3>,
    density:  f64,
    pressure: f64,
}

impl From<Particle> for ParticleDto {
    fn from(particle: Particle) -> Self {
        ParticleDto {
            position: VectorN::<f64,U3>::new (particle.x, particle.y, 0.0),
            velocity: VectorN::<f64,U3>::new (particle.vx, particle.vy, 0.0),
            density:  particle.density,
            pressure: particle.pressure,
        }
    }
}

impl From<ParticleDto> for Particle {
    fn from(particle: ParticleDto) -> Self {
        Particle {
            x: particle.position.x,
            y: particle.position.y,
            vx: particle.velocity.x,
            vy: particle.velocity.y,
            fx: 0.0,
            fy: 0.0,
            ofx: 0.0,
            ofy: 0.0,
            density:  particle.density,
            pressure: particle.pressure,
        }
    }
}

fn to_le_bytes(input: &VectorN<f64, U3>) -> [u8; 24] {
    let mut result = [0; 24];
    result[..8].copy_from_slice(&input.x.to_le_bytes());
    result[8..16].copy_from_slice(&input.y.to_le_bytes());
    result[16..24].copy_from_slice(&input.z.to_le_bytes());
    result
}

pub fn write_to_io(particles: &[Particle],
                   buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let length = particles.len();

    buffer.write(&(length as u64).to_le_bytes())?;
    for particle in particles {
        let position = VectorN::<f64,U3>::new(particle.x, particle.y, 0.0);
        let velocity = VectorN::<f64,U3>::new(particle.vx, particle.vy, 0.0);
        buffer.write(&to_le_bytes(&position))?;
        buffer.write(&to_le_bytes(&velocity))?;
        buffer.write(&particle.density.to_le_bytes())?;
        buffer.write(&particle.pressure.to_le_bytes())?;
    }
    buffer.flush()?;

    Ok(())
}

// This is in preparation for trying to reimplement
// particle using geometric vector types from nalgebra
fn future_write_to_io(particles: &[ParticleDto],
               buffer: &mut impl std::io::Write) -> std::io::Result<()> {
    let length = particles.len();

    buffer.write(&(length as u64).to_le_bytes())?;
    for particle in particles {
        buffer.write(&to_le_bytes(&particle.position))?;
        buffer.write(&to_le_bytes(&particle.velocity))?;
        buffer.write(&particle.density.to_le_bytes())?;
        buffer.write(&particle.pressure.to_le_bytes())?;
    }
    buffer.flush()?;

    Ok(())
}

fn take_particle(input: &[u8]) -> IResult<&[u8], ParticleDto> {
    let (input, position) = le_vector3(input)?;
    let (input, velocity) = le_vector3(input)?;
    let (input, density)  = le_f64(input)?;
    let (input, pressure) = le_f64(input)?;

    Ok((input, ParticleDto {
        position,
        velocity,
        density,
        pressure
    }))
}

fn take_particles(input: &[u8]) -> IResult<&[u8], Vec<ParticleDto>> {
    let (input, length) = le_u64(input)?;
    let length = length as usize;

    let (input, particles) = count(take_particle, length)(input)?;

    Ok((input, particles))
}

fn le_vector3(input: &[u8]) -> IResult<&[u8], VectorN<f64, U3>> {
    let (input, (x, y, z)) = nom::sequence::tuple((le_f64, le_f64, le_f64))(input)?;
    Ok((input, VectorN::<f64, U3>::new(x, y, z)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialize_and_deserialize_vector() {
        let initial = VectorN::<f64, U3>::new(1.0, 2.0, 3.0);

        let data = to_le_bytes(&initial);
        let (_, result) = le_vector3(&data).unwrap();

        assert_eq!(result, initial);
    }

    #[test]
    fn map() {
        let initial = Particle::new(1.0, 0.0);

        let converted: ParticleDto = initial.into();
        let result: Particle = converted.into();

        assert_eq!(result, Particle::new(1.0, 0.0));
    }

    #[test]
    fn write_to_io_writes_length_first() {
        let particles = vec![ParticleDto {
            position: na::VectorN::<f64, U3>::new(1.0, 0.0, 0.0),
            velocity: na::VectorN::<f64, U3>::new(1.0, 0.0, 0.0),
            density:  0.0,
            pressure: 0.0,
        }];
        let expected = (particles.len() as u64).to_le_bytes();

        let mut result = Vec::<u8>::new();
        write_to_io_internal(&particles, &mut result).unwrap();

        let u64_size = std::mem::size_of::<u64>();
        assert_eq!(result[..u64_size], expected);
    }

    #[test]
    fn serialize_and_deserialize_particles() {
        let particles = vec![ParticleDto {
            position: na::VectorN::<f64, U3>::new(1.0, 0.0, 0.0),
            velocity: na::VectorN::<f64, U3>::new(1.0, 0.0, 0.0),
            density:  0.0,
            pressure: 0.0,
        }];
    
        let mut data = Vec::<u8>::new();
        write_to_io_internal(&particles, &mut data).unwrap();
    
        let (_, result) = take_particles(&data).unwrap();
    
        assert_eq!(result, particles);
    }
}
