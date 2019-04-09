#[derive(Clone)]
pub struct Cell {
    particles: Vec<u32>
}

#[derive(Clone)]
pub struct Grid {
    grid: Vec<Cell>,
    grid_width: u64,
    grid_height: u64,
    sx: f64,
    sy: f64,
    h: f64
}

fn world_to_grid(h: f64, sx: f64, sy: f64, x: f64, y: f64) -> (u64, u64) {
    let gx = ((x - sx) / (2.0 * h)).floor() as u64;
    let gy = ((y - sy) / (2.0 * h)).floor() as u64;
    (gx, gy)
}

pub fn create_grid(h: f64, sx: f64, ex: f64, sy: f64, ey: f64) -> Grid {
    let (last_gx, last_gy) = world_to_grid(h, sx, sy, ex, ey);
    let (grid_width, grid_height) = (last_gx + 1, last_gy + 1);
    Grid {
        grid: vec![Cell { particles: Vec::new() }; (grid_width * grid_height) as usize],
        grid_width,
        grid_height,
        sx,
        sy,
        h: h
    }
}

impl Grid {
    fn grid_index(&self, (gx, gy): (u64, u64)) -> usize {
        (gy * self.grid_width + gx) as usize
    }

    fn grid_get(&self, gx: u64, gy: u64) -> &Cell {
        &self.grid[self.grid_index((gx, gy))]
    }

    pub fn add_particle(&mut self, index: u32, x: f64, y: f64) {
        let (gx, gy) = world_to_grid(self.h, self.sx, self.sy, x, y);
        let grid_index = self.grid_index((gx, gy));
        self.grid[grid_index].particles.push(index);
    }

    pub fn get_neighbours(&self, x: f64, y: f64) -> Vec<u32> {
        let mut neighbours = Vec::new();
        let (gx, gy) = world_to_grid(self.h, self.sx, self.sy, x, y);
        neighbours.extend(&self.grid_get(gx, gy).particles);
        if gx + 1 < self.grid_width {
            neighbours.extend(&self.grid_get(gx + 1, gy).particles);
        }
        if gy + 1 < self.grid_height {
            neighbours.extend(&self.grid_get(gx, gy + 1).particles);
        }
        if gx + 1 < self.grid_width && gy + 1 < self.grid_height {
            neighbours.extend(&self.grid_get(gx + 1, gy + 1).particles);
        }
        if gx > 0 {
            neighbours.extend(&self.grid_get(gx - 1, gy).particles);
        }
        if gy > 0 {
            neighbours.extend(&self.grid_get(gx, gy - 1).particles);
        }
        if gx > 0 && gy > 0 {
            neighbours.extend(&self.grid_get(gx - 1, gy - 1).particles);
        }

        if gx + 1 < self.grid_width && gy > 0 {
            neighbours.extend(&self.grid_get(gx + 1, gy - 1).particles);
        }
        if gx > 0 && gy + 1 < self.grid_height {
            neighbours.extend(&self.grid_get(gx - 1, gy + 1).particles);
        }
        return neighbours;
    }

    pub fn get_neighbours_grid(&self, gx: u64, gy: u64) -> Vec<u32> {
        let mut neighbours = Vec::new();
        neighbours.extend(&self.grid_get(gx, gy).particles);
        if gx + 1 < self.grid_width {
            neighbours.extend(&self.grid_get(gx + 1, gy).particles);
        }
        if gy + 1 < self.grid_height {
            neighbours.extend(&self.grid_get(gx, gy + 1).particles);
        }
        if gx + 1 < self.grid_width && gy + 1 < self.grid_height {
            neighbours.extend(&self.grid_get(gx + 1, gy + 1).particles);
        }
        if gx > 0 {
            neighbours.extend(&self.grid_get(gx - 1, gy).particles);
        }
        if gy > 0 {
            neighbours.extend(&self.grid_get(gx, gy - 1).particles);
        }
        if gx > 0 && gy > 0 {
            neighbours.extend(&self.grid_get(gx - 1, gy - 1).particles);
        }

        if gx + 1 < self.grid_width && gy > 0 {
            neighbours.extend(&self.grid_get(gx + 1, gy - 1).particles);
        }
        if gx > 0 && gy + 1 < self.grid_height {
            neighbours.extend(&self.grid_get(gx - 1, gy + 1).particles);
        }
        return neighbours;
    }

    pub fn particles_and_neighbours(&self) -> ParticleNeighbourIterator {
        ParticleNeighbourIterator {
            grid: self.clone(),
            x: 0,
            y: 0,
        }
    }
}

pub struct ParticleNeighbourIterator {
    grid: Grid,
    x: u64,
    y: u64,
}

impl Iterator for ParticleNeighbourIterator {
    type Item = (Vec<u32>, Vec<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.grid.grid_width - 1 {
            self.x += 1;
        } else if self.y < self.grid.grid_height - 1 {
            self.y += 1;
            self.x = 0;
        } else {
            return None
        }
        let particles = self.grid.grid_get(self.x, self.y).particles.clone();
        let neighbours = self.grid.get_neighbours_grid(self.x, self.y);

        Some((particles, neighbours))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_grid() {
        let (gx, gy) = world_to_grid(0.5, -1.0, -1.0, -0.75, -0.75);
        assert!(gx == 0);
        assert!(gy == 0);
    }
    #[test]
    fn test_grid() {
        let mut grid = create_grid(0.5, -2.0, 2.0, -2.0, 2.0);
        grid.add_particle(1, -0.5, -0.5);
        grid.add_particle(2, -0.2, -0.2);
        grid.add_particle(3, 1.0, 1.0);
        grid.add_particle(4, 1.5, 1.5);
        let neighbours = grid.get_neighbours(-0.4, -0.4);
        assert!(neighbours.contains(&1));
        assert!(neighbours.contains(&2));
        let neighbours2 = grid.get_neighbours(1.0, 1.0);
        assert!(neighbours2.contains(&3));
        assert!(neighbours2.contains(&4));
        let neighbours3 = grid.get_neighbours(0.0, 0.0);
        assert!(neighbours3.contains(&1));
        assert!(neighbours3.contains(&2));
    }
}
