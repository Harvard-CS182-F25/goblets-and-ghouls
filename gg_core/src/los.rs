use std::collections::HashSet;

/// Cells visited by a Bresenham line walk from `from` to `to`, inclusive of
/// both endpoints.
fn bresenham_line(from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
    let mut x0 = from.0 as isize;
    let mut y0 = from.1 as isize;
    let x1 = to.0 as isize;
    let y1 = to.1 as isize;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut points = Vec::new();
    loop {
        points.push((x0 as usize, y0 as usize));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }

    points
}

/// Whether `to` is visible from `from` on a grid where `walls` block sight.
/// Only intermediate cells (excluding both endpoints) are checked against
/// `walls`, since the agent's own cell and the ghost's cell are never walls.
pub fn has_line_of_sight(
    from: (usize, usize),
    to: (usize, usize),
    walls: &HashSet<(usize, usize)>,
) -> bool {
    if from == to {
        return true;
    }

    let line = bresenham_line(from, to);
    line[1..line.len() - 1]
        .iter()
        .all(|cell| !walls.contains(cell))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_cell_is_always_visible() {
        let walls = HashSet::new();
        assert!(has_line_of_sight((3, 3), (3, 3), &walls));
    }

    #[test]
    fn adjacent_cell_is_visible() {
        let walls = HashSet::new();
        assert!(has_line_of_sight((3, 3), (4, 3), &walls));
    }

    #[test]
    fn clear_straight_line_is_visible() {
        let walls = HashSet::new();
        assert!(has_line_of_sight((0, 0), (5, 0), &walls));
        assert!(has_line_of_sight((0, 0), (0, 5), &walls));
    }

    #[test]
    fn clear_diagonal_line_is_visible() {
        let walls = HashSet::new();
        assert!(has_line_of_sight((0, 0), (5, 5), &walls));
    }

    #[test]
    fn wall_directly_on_straight_path_blocks() {
        let mut walls = HashSet::new();
        walls.insert((2, 0));
        assert!(!has_line_of_sight((0, 0), (5, 0), &walls));
    }

    #[test]
    fn wall_directly_on_diagonal_path_blocks() {
        let mut walls = HashSet::new();
        walls.insert((2, 2));
        assert!(!has_line_of_sight((0, 0), (5, 5), &walls));
    }

    #[test]
    fn wall_off_path_does_not_block() {
        let mut walls = HashSet::new();
        walls.insert((0, 5));
        walls.insert((5, 0));
        assert!(has_line_of_sight((0, 0), (5, 5), &walls));
    }

    #[test]
    fn wall_at_endpoint_is_not_checked() {
        // Neither endpoint can actually be a wall in-game, but the function
        // shouldn't treat the endpoints themselves as blocking.
        let mut walls = HashSet::new();
        walls.insert((0, 0));
        walls.insert((5, 5));
        assert!(has_line_of_sight((0, 0), (5, 5), &walls));
    }
}
