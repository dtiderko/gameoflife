use bevy::prelude::*;
use bevy_egui::prelude::*;

const CELL_LIVE_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const CELL_DEAD_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
const CELL_HOVER_DELTA: f32 = 0.2;
const MARGIN_PERCENT: f32 = 0.1;
const STARTING_GRID_SIZE: usize = 10;

#[derive(Component)]
struct StepSim;
#[derive(Component)]
struct RunSim;
#[derive(Component)]
struct StopSim;

#[derive(Resource, PartialEq)]
pub struct GridSize(usize);

#[derive(Resource, Clone)]
pub struct GridState(Vec<Vec<bool>>);
impl GridState {
    pub fn new(size: &GridSize) -> Self {
        Self(vec![vec![false; size.0]; size.0])
    }

    pub fn at(&self, row: usize, col: usize) -> &bool {
        assert!(row < self.0.len());
        assert!(col < self.0.len());

        &self.0[row][col]
    }

    pub fn at_mut(&mut self, row: usize, col: usize) -> &mut bool {
        assert!(row < self.0.len());
        assert!(col < self.0.len());

        &mut self.0[row][col]
    }

    pub fn neighbours(&self, row: usize, col: usize) -> usize {
        let lcol = if col > 0 { col - 1 } else { col };
        let rcol = if col + 1 < self.0.len() { col + 1 } else { col };
        let urow = if row > 0 { row - 1 } else { row };
        let lrow = if row + 1 < self.0.len() { row + 1 } else { row };

        let neighbours = (urow..=lrow)
            .map(|r| (lcol..=rcol).map(move |c| (r, c)))
            .flatten()
            .filter(|&(r, c)| self.0[r][c])
            .count();

        if self.0[row][col] {
            neighbours - 1
        } else {
            neighbours
        }
    }

    pub fn resize(&mut self, size: &GridSize) {
        if self.0.len() == size.0 {
            return;
        }

        let mut grid = vec![vec![false; size.0]; size.0];

        let sync_size = size.0.min(self.0.len());
        for row in 0..sync_size {
            for col in 0..sync_size {
                grid[row][col] = self.0[row][col];
            }
        }

        self.0 = grid;
    }
}

#[derive(Resource)]
struct Simulation {
    timer: Timer,
    running: bool,
}

#[derive(Component)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
    pub alive: bool,
    hovered: bool,
}

pub struct Grid;
impl Plugin for Grid {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_grid, setup_simulation))
            .add_systems(EguiPrimaryContextPass, grid_controls)
            .add_systems(Update, rebuild_grid_on_resize)
            .add_systems(Update, update_cell_color)
            .add_systems(
                Update,
                (press_cell, over_cell, out_cell).before(update_cell_color),
            )
            .add_systems(
                Update,
                (
                    step_sim,
                    sync_cells
                        .run_if(resource_changed::<GridState>.and(not(resource_added::<GridState>)))
                        .after(rebuild_grid_on_resize),
                    run_sim,
                    stop_sim,
                    simulate,
                ),
            );
    }
}

fn spawn_grid_cells(commands: &mut Commands, grid_size: usize, window: &Window, grid: &GridState) {
    let margin_w = window.width() * MARGIN_PERCENT;
    let margin_h = window.height() * MARGIN_PERCENT;

    let usable = if window.height() <= window.width() {
        window.height() - margin_h * 2.0
    } else {
        window.width() - margin_w * 2.0
    };
    let cell_size = usable / grid_size as f32;

    let real_grid_size = grid_size as f32 * cell_size;
    let start_w = cell_size / 2.0 - real_grid_size / 2.0;
    let start_h = cell_size / 2.0 - real_grid_size / 2.0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            let w = start_w + col as f32 * cell_size;
            let h = start_h + row as f32 * cell_size;

            commands.spawn((
                Sprite {
                    color: grid
                        .at(row, col)
                        .then(|| CELL_LIVE_COLOR)
                        .unwrap_or(CELL_DEAD_COLOR),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(w, h, 0.0),
                    scale: Vec3::splat(cell_size),
                    ..default()
                },
                Cell {
                    alive: false,
                    hovered: false,
                    row: row as usize,
                    col: col as usize,
                },
                Pickable::default(),
            ));
        }
    }
}

fn setup_grid(mut commands: Commands, window: Single<&Window>) {
    let grid_size = GridSize(STARTING_GRID_SIZE);
    let grid = GridState::new(&grid_size);

    spawn_grid_cells(&mut commands, grid_size.0, &window, &grid);

    commands.insert_resource(grid);
    commands.insert_resource(grid_size);
}

fn setup_simulation(mut commands: Commands) {
    commands.insert_resource(Simulation {
        running: false,
        timer: Timer::from_seconds(0.2, TimerMode::Repeating),
    });
}

fn rebuild_grid_on_resize(
    mut commands: Commands,
    grid_size: Res<GridSize>,
    mut grid: ResMut<GridState>,
    window: Single<&Window>,
    cells: Query<Entity, With<Cell>>,
) {
    if !grid_size.is_changed() {
        return;
    }

    grid.resize(&grid_size);

    for c in cells {
        commands.entity(c).despawn();
    }

    spawn_grid_cells(&mut commands, grid_size.0, &window, &grid);
}

fn grid_controls(
    mut contexts: EguiContexts,
    mut grid: ResMut<GridState>,
    mut grid_size: ResMut<GridSize>,
    mut sim: ResMut<Simulation>,
) -> Result {
    egui::Window::new("Controls").show(contexts.ctx_mut()?, |ui| {
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );

        ui.label("Controls: Left-click to create cells and right-click to destroy them.");
        ui.separator();

        if ui.button("Step").clicked() {
            step_grid(&mut grid, grid_size.0);
        }

        if sim.running {
            if ui.button("Pause").clicked() {
                sim.running = false;
            }
        } else {
            if ui.button("Run").clicked() {
                sim.running = true;
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Grid size");

            if ui.button("-").clicked() {
                grid_size.0 = grid_size.0.saturating_sub(1);
            }

            let mut buf = grid_size.0;
            ui.add(egui::DragValue::new(&mut buf));
            grid_size.set_if_neq(GridSize(buf));

            if ui.button("+").clicked() {
                grid_size.0 = grid_size.0.saturating_add(1);
            }
        });
    });
    Ok(())
}

fn press_cell(
    mut msgs: MessageReader<Pointer<Press>>,
    mut query: Query<&mut Cell>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut grid: ResMut<GridState>,
) {
    for msg in msgs.read() {
        if let Ok(mut cell) = query.get_mut(msg.event_target()) {
            if mouse.pressed(MouseButton::Left) {
                cell.alive = true;
                *grid.at_mut(cell.row, cell.col) = true;
            } else if mouse.pressed(MouseButton::Right) {
                cell.alive = false;
                *grid.at_mut(cell.row, cell.col) = false;
            }
        }
    }
}

fn over_cell(
    mut msgs: MessageReader<Pointer<Over>>,
    mut query: Query<&mut Cell>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut grid: ResMut<GridState>,
) {
    for msg in msgs.read() {
        if let Ok(mut cell) = query.get_mut(msg.event_target()) {
            cell.hovered = true;

            if mouse.pressed(MouseButton::Left) {
                cell.alive = true;
                *grid.at_mut(cell.row, cell.col) = true;
            } else if mouse.pressed(MouseButton::Right) {
                cell.alive = false;
                *grid.at_mut(cell.row, cell.col) = false;
            }
        }
    }
}

fn out_cell(mut msgs: MessageReader<Pointer<Out>>, mut query: Query<&mut Cell>) {
    for msg in msgs.read() {
        if let Ok(mut cell) = query.get_mut(msg.event_target()) {
            cell.hovered = false;
        }
    }
}

fn update_cell_color(mut query: Query<(&Cell, &mut Sprite), Changed<Cell>>) {
    for (cell, mut sprite) in &mut query {
        let base = if cell.alive {
            CELL_LIVE_COLOR
        } else {
            CELL_DEAD_COLOR
        };

        sprite.color = if cell.hovered {
            base.lighter(CELL_HOVER_DELTA)
        } else {
            base
        }
    }
}

fn step_grid(grid: &mut ResMut<GridState>, grid_size: usize) {
    let mut new_grid = grid.clone();

    for row in 0..grid_size {
        for col in 0..grid_size {
            let neighbours = grid.neighbours(row, col);

            if neighbours < 2 || neighbours > 3 {
                *new_grid.at_mut(row, col) = false;
            } else if neighbours == 3 {
                *new_grid.at_mut(row, col) = true;
            }
        }
    }

    **grid = new_grid;
}

fn step_sim(
    query: Query<&Interaction, (Changed<Interaction>, With<StepSim>)>,
    mut grid: ResMut<GridState>,
    grid_size: Res<GridSize>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        step_grid(&mut grid, grid_size.0);
    }
}

fn sync_cells(mut cells: Query<&mut Cell>, grid: Res<GridState>) {
    for mut cell in &mut cells {
        cell.alive = *grid.at(cell.row, cell.col);
    }
}

fn run_sim(
    query: Query<&Interaction, (Changed<Interaction>, With<RunSim>)>,
    mut sim: ResMut<Simulation>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        sim.running = true;
    }
}

fn stop_sim(
    query: Query<&Interaction, (Changed<Interaction>, With<StopSim>)>,
    mut sim: ResMut<Simulation>,
) {
    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        sim.running = false;
    }
}

fn simulate(
    time: Res<Time>,
    mut sim: ResMut<Simulation>,
    mut grid: ResMut<GridState>,
    grid_size: Res<GridSize>,
) {
    if !sim.running {
        return;
    }

    if sim.timer.tick(time.delta()).just_finished() {
        step_grid(&mut grid, grid_size.0);
    }
}
