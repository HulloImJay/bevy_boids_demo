pub struct Bounds {
    pub cell_size: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
    pub z_min: f32,
    pub z_max: f32,
    pub x_size: f32,
    pub y_size: f32,
    pub z_size: f32,
    pub margin: f32,
    pub cells_x: usize,
    pub cells_z: usize,
}

impl Bounds {
    pub fn new(cell_size: f32, x_min: f32, x_max: f32, y_min: f32, y_max: f32, z_min: f32, z_max: f32, margin: f32) -> Bounds {
        let x_size = x_max - x_min;
        let y_size = y_max - y_min;
        let z_size = z_max - z_min;
        let cells_x = (x_size / cell_size) as usize;
        let cells_z = (z_size / cell_size) as usize;
        Bounds {
            cell_size,
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
            margin,
            x_size,
            y_size,
            z_size,
            cells_x,
            cells_z,
        }
    }
}