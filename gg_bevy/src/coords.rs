//! Shared grid <-> world-space conversion helpers, used by rendering and
//! input-raycast systems alike.

use bevy::prelude::*;

/// Physical world extents for a board of the given cell dimensions.
pub fn world_dimensions(board_width: usize, board_height: usize, cell_size: f32) -> (f32, f32) {
    (board_width as f32 * cell_size, board_height as f32 * cell_size)
}

/// Converts a grid cell to a world-space position, centered on the board.
/// `y` sets the vertical (up) coordinate — 0.0 for ground-level entities,
/// or an offset for entities that render above the walls (e.g. the ghost).
pub fn cell_to_world(
    position: (usize, usize),
    cell_size: f32,
    world_width: f32,
    world_height: f32,
    y: f32,
) -> Vec3 {
    Vec3::new(
        position.0 as f32 * cell_size + cell_size / 2.0 - world_width / 2.0,
        y,
        position.1 as f32 * cell_size + cell_size / 2.0 - world_height / 2.0,
    )
}

/// Raycasts the cursor onto an infinite plane at `plane_transform` and maps
/// the hit point to a grid cell of size `grid_w` x `grid_h`. Shared by the
/// live game's hover tooltip and the editor's click-to-paint input.
pub fn raycast_to_grid_cell(
    window: &Window,
    camera: &Camera,
    cam_transform: &GlobalTransform,
    plane_transform: &GlobalTransform,
    grid_w: u32,
    grid_h: u32,
) -> Option<(UVec2, Vec3)> {
    let cursor = window.cursor_position()?;
    let ray = camera.viewport_to_world(cam_transform, cursor).ok()?;
    let ro = ray.origin;
    let rd = ray.direction;

    // Infinite plane at the ground's origin with its world-space normal
    let plane_point = plane_transform.translation();
    let plane_normal = plane_transform.up().into();

    let denom = rd.dot(plane_normal);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (plane_point - ro).dot(plane_normal) / denom;
    if t < 0.0 {
        return None;
    }

    let hit = ro + t * rd;

    // Convert the world hit to the ground's LOCAL (mesh) space.
    // In mesh space, X/Z are ~[-0.5, 0.5] because the mesh is a unit cuboid.
    let local = plane_transform.affine().inverse().transform_point3(hit);

    // Map mesh-local [-0.5,0.5] → [0, grid_w/grid_h)
    let u_cells = (local.x + 0.5) * grid_w as f32; // columns
    let v_cells = (local.z + 0.5) * grid_h as f32; // rows

    let col = u_cells.floor() as i32;
    let row = v_cells.floor() as i32;

    if col < 0 || row < 0 || col as u32 >= grid_w || row as u32 >= grid_h {
        return None;
    }

    Some((UVec2::new(col as u32, row as u32), hit))
}
