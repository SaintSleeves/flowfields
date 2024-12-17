// use std::sync::Barrier;

use std::{clone, collections::HashSet, os::windows::process};

use macroquad::prelude::*;

// Size of each grid cell
const CELL_SIZE: f32 = 30.0; 
// Amount the highlighted cells are dimmed when hovered. Lower value = more dim.
const HIGHLIGHT_DIM_AMOUNT: f32 = 0.75; 
const CELLS_HORIZONTAL: usize = 10;
const CELLS_VERTICAL: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellType {
    Barrier,
    Inactive,
    Active,
    Source,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Inactive
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
struct Cell {
    cell_type: CellType,
    cell_number: Option<i32>,
    x_position: usize,
    y_position: usize,
    highlighted: bool
}

impl Cell {
    fn get_color(self) -> macroquad::color::Color {
        let color = match self.cell_type {
            CellType::Barrier => macroquad::color::colors::BLACK,

            CellType::Source => macroquad::color::colors::RED, 
            
            matched_type @ (CellType::Active | CellType::Inactive) => {
                match self.cell_number {
                    None => macroquad::color::colors::WHITE,

                    Some(value) => match matched_type {

                        CellType::Active => {
                            macroquad::color::Color {
                                r: 1.0 - (1.0 / value as f32),
                                g: 1.0,
                                b: 1.0 - (1.0 / value as f32),
                                a: 1.0,
                            }
                        },

                        CellType::Inactive => {
                            macroquad::color::Color {
                                r: 1.0 - (1.0 / value as f32),
                                g: 1.0 - (1.0 / value as f32),
                                b: 1.0,
                                a: 1.0,
                            }
                        },

                        _ => unreachable!()
                    }
                }
            },
        };

        if self.highlighted {
            return macroquad::color::Color {
                r: color.r * HIGHLIGHT_DIM_AMOUNT,
                g: color.g * HIGHLIGHT_DIM_AMOUNT,
                b: color.b * HIGHLIGHT_DIM_AMOUNT,
                a: 1.0,
            }
        }
        color
    }
}


#[derive(Debug)]
struct Grid {
    grid: Vec<Vec<Cell>>,
    row_count_y: usize,
    column_count_x: usize,
}

impl Grid {
    fn new(row_count: usize, column_count: usize) -> Self {
        let mut grid = Vec::with_capacity(row_count);
        for y in 0..row_count {
            let mut row = Vec::with_capacity(column_count);
            for x in 0..column_count {
                row.push(Cell {
                    x_position: x,
                    y_position: y,
                    ..Default::default()
                });
            }
            grid.push(row);
        }
        
        Grid {
            grid,
            row_count_y: row_count,
            column_count_x: column_count,
        }
    }

    fn get_neighbor_coordinates(&self, target: &Cell) -> Vec<(usize, usize)> {
        let mut adjacent = Vec::new();
        if target.x_position > 0 {
            adjacent.push((target.x_position - 1, target.y_position));
        }
        if target.y_position > 0 {
            adjacent.push((target.x_position, target.y_position - 1));
        }
        if target.x_position + 1 < self.column_count_x {  // Fixed boundary check
            adjacent.push((target.x_position + 1, target.y_position));
        }
        if target.y_position + 1 < self.row_count_y {    // Fixed boundary check
            adjacent.push((target.x_position, target.y_position + 1));
        }
        adjacent
    }

    fn get_cell_with_lowest_cell_number(cells: Vec<Cell>) -> Option<Cell> {
        cells.into_iter().min_by_key(|x| x.cell_number)
    }


    fn get_cell_from_coordinate(&mut self, col_x: usize, row_y: usize) -> &mut Cell {
        &mut self.grid[row_y][col_x]
    }



    fn source_cells(&mut self, source_coordinates: &Vec<(usize, usize)>) -> () {
        let mut neighbor_cells = Vec::<(usize, usize)>::new();
        
        for &(col_x, row_y) in source_coordinates {
            let cell = &self.grid[row_y][col_x];
            neighbor_cells.append(&mut self.get_neighbor_coordinates(cell));
        }

        self.populate_cells(&neighbor_cells, 2, &mut neighbor_cells.clone());
    }
    

    fn populate_cells(
        &mut self, 
        unpopulated_coordinates: &Vec<(usize, usize)>, 
        new_cell_number: i32, 
        processed_cells: &mut Vec<(usize, usize)>
    ) -> () {

        // let mut reached_coordinates = Vec::new();
        let mut new_unpopulated_coordinates = Vec::new();
        
        for &(col_x, row_y) in unpopulated_coordinates {
            let cell = self.get_cell_from_coordinate(col_x, row_y);
            let should_process = match cell.cell_type {
                CellType::Barrier => false,
                _ => true,
            };

            if should_process {
                cell.cell_number = Some(new_cell_number);
                
                let immutable_cell = &self.grid[row_y][col_x];  // temporary immutable borrow
                let neighbors = self.get_neighbor_coordinates(immutable_cell);

                println!("{:?}", unpopulated_coordinates);

                for coord in neighbors {
                    if !new_unpopulated_coordinates.contains(&coord) && !processed_cells.contains(&coord){
                        new_unpopulated_coordinates.push(coord);
                        processed_cells.push(coord);
                    }
                }

            }
        }
    if !new_unpopulated_coordinates.is_empty() {
        self.populate_cells(&new_unpopulated_coordinates, new_cell_number + 1, processed_cells);
    }
    }
}


#[macroquad::main("Grid")]
async fn main() {
    let mut source_cells = Vec::<(usize, usize)>::new();

    let grid = &mut Grid::new(CELLS_HORIZONTAL, CELLS_VERTICAL);

    grid.grid[0][0].cell_type = CellType::Barrier;

    let mut action_blocked = false;
    let mut last_hovered_cell = (0, 0);
    // println!("{:#?}", grid);

    loop {
        // clear_background(WHITE);
        let (mouse_x, mouse_y) = mouse_position();
        let mut grid_recalculation_needed = false;
        
        for row_y in &mut grid.grid {
            for cell in row_y {
                let cell_position_x = cell.x_position as f32 * CELL_SIZE;
                let cell_position_y = cell.y_position as f32 * CELL_SIZE;
                
                let is_hovered = mouse_x >= cell_position_x 
                    && mouse_x < (cell_position_x + CELL_SIZE) 
                    && mouse_y >= cell_position_y 
                    && mouse_y < (cell_position_y + CELL_SIZE);

                cell.highlighted = is_hovered;               

                if is_hovered && !action_blocked {

                    if is_mouse_button_down(MouseButton::Right) {
                        match cell.cell_type {
                            CellType::Source => {
                                source_cells.retain(|&x| x != (cell.x_position, cell.y_position));
                                cell.cell_type = CellType::Inactive;
                            }

                            _ => {
                                source_cells.push((cell.x_position, cell.y_position));
                                cell.cell_number = Some(1);
                                cell.cell_type = CellType::Source;
                            }
                        }
                        action_blocked = true;
                        grid_recalculation_needed = true;
                    }

                    if is_mouse_button_down(MouseButton::Left) {
                        match cell.cell_type {
                            CellType::Barrier => {
                                cell.cell_type = CellType::Inactive;
                            }

                            _ => {
                                cell.cell_type = CellType::Barrier;
                            }
                        }
                        action_blocked = true;
                        grid_recalculation_needed = true;
                    }
                }

                if is_hovered && action_blocked {
                    if (cell.x_position, cell.y_position) != last_hovered_cell {
                        last_hovered_cell = (cell.x_position, cell.y_position);
                        action_blocked = false;
                    }
                }

                // println!("{}\n{:#?}", action_blocked, last_cell);

                
                // Draw the cell
                draw_rectangle(
                    cell_position_x,
                    cell_position_y,
                    CELL_SIZE,
                    CELL_SIZE,
                    cell.get_color(),
                );

                
                // Draw cell border
                draw_rectangle_lines(
                    cell_position_x,
                    cell_position_y,
                    CELL_SIZE,
                    CELL_SIZE,
                    1.0,
                    DARKGRAY,
                );

                draw_text(
                    &cell.cell_number.unwrap_or(0).to_string(),
                    // cell_position_x + CELL_SIZE / 2.0,
                    cell_position_x,
                    cell_position_y + CELL_SIZE / 2.0,
                    25.0,
                    BLACK,
                );


                // if is_mouse_button_down(MouseButton::Left) {
                // }
            }
        }

        if grid_recalculation_needed {
            // *grid = Grid::new(CELLS_HORIZONTAL, CELLS_VERTICAL);
            grid.source_cells(&source_cells);
            // grid_recalculation_needed = false;
        }

        next_frame().await;
    }
}