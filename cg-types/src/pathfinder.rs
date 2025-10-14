use i_cg_types_proc::protocol;

/// A 2D point with x and y coordinates
#[protocol("cg")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// An edge connecting two points
#[protocol("cg")]
pub struct Edge {
    pub from: usize,
    pub to: usize,
}

/// Result of a shortest path computation
#[protocol("cg")]
pub struct PathResult {
    pub path: Vec<usize>,
    pub distance: f64,
}

/// Parameters for finding the shortest path between two points
#[protocol("cg")]
#[codegen(fn = "find_shortest_path() -> PathResult")]
pub struct ShortestPathParams {
    pub points: Vec<Point>,
    pub edges: Vec<Edge>,
    pub start_idx: usize,
    pub end_idx: usize,
}
