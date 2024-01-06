use std::collections::{HashMap, HashSet, VecDeque};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use itertools::Itertools;
use tracing::info;

pub fn router() -> Router {
    Router::new()
        .route("/22/health", get(|| async { StatusCode::OK }))
        .route("/22/integers", post(integers))
        .route("/22/rocket", post(rocket))
}

async fn integers(content: String) -> impl IntoResponse {
    let mut result: HashSet<usize> = HashSet::new();
    for line in content.lines() {
        if let Ok(num) = line.trim().parse::<usize>() {
            if result.take(&num).is_none() {
                result.insert(num);
            }
        }
    }
    let rep = result.into_iter().next().unwrap_or_default();
    "🎁".repeat(rep)
}

async fn rocket(content: String) -> impl IntoResponse {
    let mut content_lines = content.lines();
    let star_nums: u32 = content_lines.next().unwrap().parse().unwrap();
    let mut stars = Vec::new();
    let mut portals = Vec::new();

    for _ in 0..star_nums {
        let s = content_lines
            .next()
            .unwrap()
            .splitn(3, ' ')
            .map(|s| s.parse::<i32>().unwrap())
            .collect_tuple::<(i32, i32, i32)>()
            .unwrap();
        stars.push(s);
    }
    let portal_nums: u32 = content_lines.next().unwrap().parse().unwrap();
    for _ in 0..portal_nums {
        let s = content_lines
            .next()
            .unwrap()
            .splitn(2, ' ')
            .map(|s| s.parse::<usize>().unwrap())
            .collect_tuple::<(usize, usize)>()
            .unwrap();
        portals.push(s);
    }

    let mut adjacency_list_graph = AdjacencyListGraph::new();

    for portal in portals.iter() {
        adjacency_list_graph.add_edge(*stars.get(portal.0).unwrap(), *stars.get(portal.1).unwrap())
    }

    let start_vertex = stars.first().unwrap();
    let end_vertex = stars.last().unwrap();

    let shortest_path = adjacency_list_graph
        .bfs_shortest_path(*start_vertex, *end_vertex)
        .unwrap();
    let portals_num_traveled = shortest_path.len() - 1;
    info!("{star_nums}: {:?}", stars);
    info!("{portal_nums}: {:?}", portals);
    info!("{:?}", adjacency_list_graph);
    info!("{:?}", shortest_path);

    format!(
        "{} {:.3}",
        portals_num_traveled,
        AdjacencyListGraph::path_distance_calculator(&shortest_path)
    )
}

#[derive(Debug)]
struct AdjacencyListGraph {
    list: HashMap<(i32, i32, i32), Vec<(i32, i32, i32)>>,
}

type Vertex = (i32, i32, i32);
impl AdjacencyListGraph {
    fn new() -> Self {
        AdjacencyListGraph {
            list: HashMap::new(),
        }
    }

    fn add_edge(&mut self, v1: Vertex, v2: Vertex) {
        self.list
            .entry(v1)
            .and_modify(|ve| ve.push(v2))
            .or_insert(vec![v2]);
    }

    fn bfs_shortest_path(&self, start: Vertex, end: Vertex) -> Option<Vec<Vertex>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        queue.push_back((start, vec![start]));
        while let Some((current_vertex, path)) = queue.pop_front() {
            if visited.contains(&current_vertex) {
                continue;
            }

            visited.insert(current_vertex);

            if current_vertex == end {
                return Some(path);
            }

            if let Some(neighbors) = self.list.get(&current_vertex) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        queue.push_back((neighbor, new_path));
                    }
                }
            }
        }

        None
    }

    fn path_distance_calculator(path: &Vec<Vertex>) -> f32 {
        let mut distance: f32 = 0.0;
        for i in 1..path.len() {
            distance += AdjacencyListGraph::distance_calculator(path[i - 1], path[i])
        }
        distance
    }

    fn distance_calculator(v1: Vertex, v2: Vertex) -> f32 {
        (((v1.0 - v2.0).pow(2) + (v1.1 - v2.1).pow(2) + (v1.2 - v2.2).pow(2)) as f32).sqrt()
    }
}
