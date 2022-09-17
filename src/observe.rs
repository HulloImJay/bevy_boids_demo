use bevy::{
    prelude::*,
};

// Our own plugin:
pub struct Observe;

impl Plugin for Observe {
    fn build(&self, app: &mut App) {
        app
            .add_system(observation_system_update_cells)
            .add_system(observation_system_update_observed.after(observation_system_update_cells))
            .add_system(observation_system_update_hashmap.after(observation_system_update_observed));
    }
}

#[derive(Component, Debug, Default)]
pub struct Observer {
    pub cell: usize,
    pub observed: Vec<Entity>,
}

// A resource which collects observable thingies by spatial hashing.
pub struct StuffsToObserve {
    stuff: Vec<Vec<Entity>>,
    cell_size: f32,
    width: usize,
    depth: usize,
}

impl StuffsToObserve {
    pub fn new(width: usize, depth: usize, cell_size: f32) -> StuffsToObserve {
        let mut stuff = Vec::new();
        let size = width * depth;
        for _ in 0..size {
            stuff.push(Vec::new());
        }
        StuffsToObserve {
            stuff,
            cell_size,
            width,
            depth,
        }
    }
}

impl StuffsToObserve {
    fn collect_cells(&self, cell: usize) -> Vec<usize>
    {
        let mut all_cells = Vec::new();

        let me = cell as isize;
        let w = self.width as isize;
        let d = self.depth as isize;

        let x_me = me % w;
        let z_me = me / w;

        for x in x_me - 1..x_me + 2 // note ranges are [min..max)
        {
            for z in z_me - 1..z_me + 2
            {
                if x >= 0 && x < w && z >= 0 && z < d
                {
                    all_cells.push((x + z * w) as usize);
                }
            }
        }

        all_cells
    }
}

// Our crude spatial-hash function.
fn hash_function(pos: Vec3, cell_size: f32, width: usize, depth: usize) -> usize
{
    if cell_size <= 0.
    { return 0; }

    let x = (f32::floor(pos.x / cell_size) as usize).clamp(0, width - 1);
    let z = (f32::floor(pos.z / cell_size) as usize).clamp(0, depth - 1);

    x + z * width
}

fn observation_system_update_cells(
    stuff_to_observe: Res<StuffsToObserve>,
    mut observables: Query<(&mut Observer, &Transform)>)
{
    for (mut obs, transform) in observables.iter_mut() {
        obs.cell = hash_function(transform.translation, stuff_to_observe.cell_size, stuff_to_observe.width, stuff_to_observe.depth);
    }
}

fn observation_system_update_observed(
    stuff_to_observe: Res<StuffsToObserve>,
    mut observers: Query<&mut Observer>)
{
    // let mut observed_count = 0;
    // let mut observer_count = 0;
    // let mut observed_cells = 0;
    for mut obs in observers.iter_mut() {
        obs.observed.clear();
        let near_cells = stuff_to_observe.collect_cells(obs.cell);
        for near_cell in near_cells.iter()
        {
            for entity in stuff_to_observe.stuff[*near_cell].iter() {
                obs.observed.push(*entity);
            }
            // observed_cells += 1;
        }
        // observed_count += obs.observed.len();
        // observer_count += 1;
    }
    // let observed_avg = observed_count as f32 / observer_count as f32;
    // let observed_cells_avg = observed_cells as f32 / observer_count as f32;
    // println!("observers: {:?}; observed_avg: {:?}; observed_cells_avg: {:?}", observer_count, observed_avg, observed_cells_avg);
}

fn observation_system_update_hashmap(
    mut stuff_to_observe: ResMut<StuffsToObserve>,
    observables: Query<(&Observer, Entity)>)
{
    for thing in stuff_to_observe.stuff.iter_mut() {
        thing.clear();
    }
    for (obs, entity) in observables.iter() {
        if let Some(set) = stuff_to_observe.stuff.get_mut(obs.cell)
        {
            set.push(entity);
        }
    }
}