//! State-diagram graph layering, positions and transition geometry.
use super::*;

pub(super) fn layout_state(state: &StateDiagram, resolver: &TextResolver<'_>) -> StateLayout {
    let depths = graph_depths(
        state.states.iter().map(|node| node.id.as_str()),
        state
            .transitions
            .iter()
            .map(|transition| (transition.from.as_str(), transition.to.as_str())),
    );
    let mut layers: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, node) in state.states.iter().enumerate() {
        layers
            .entry(*depths.get(&node.id).unwrap_or(&0))
            .or_default()
            .push(index);
    }

    let mut node_layouts = Vec::new();
    let mut layer_sizes = BTreeMap::new();
    for (depth, indexes) in &layers {
        let mut width = 0.0;
        let mut height: f32 = 0.0;
        for node_index in indexes {
            let node = &state.states[*node_index];
            let (text, rect_width, rect_height) = match node.kind {
                StateNodeKind::Initial | StateNodeKind::Final => (None, 22.0, 22.0),
                StateNodeKind::State => {
                    let text = resolver.measure_wrapped(&node.label, STATE_MAX_WIDTH, Some("600"));
                    let width = (text.width + NODE_PAD_X * 2.0)
                        .clamp(120.0, STATE_MAX_WIDTH + NODE_PAD_X * 2.0);
                    let height = (text.height + NODE_PAD_Y).max(52.0);
                    (Some(text), width, height)
                }
            };
            width += rect_width;
            height = height.max(rect_height);
            node_layouts.push(StateNodeLayout {
                id: node.id.clone(),
                kind: node.kind,
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: rect_width,
                    height: rect_height,
                },
                text,
            });
        }
        if !indexes.is_empty() {
            width += NODE_GAP * (indexes.len() - 1) as f32;
        }
        layer_sizes.insert(*depth, (width, height));
    }

    position_state_nodes(&mut node_layouts, &layers, &layer_sizes);

    let by_id = node_layouts
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let transitions = state
        .transitions
        .iter()
        .map(|transition| {
            let from = node_layouts[by_id[&transition.from]].rect;
            let to = node_layouts[by_id[&transition.to]].rect;
            StateTransitionLayout {
                label: transition
                    .label
                    .as_ref()
                    .map(|label| resolver.measure_text(label, None)),
                points: edge_points(from, to),
            }
        })
        .collect::<Vec<_>>();

    let (width, height) = layout_bounds(
        node_layouts.iter().map(|node| node.rect),
        transitions
            .iter()
            .flat_map(|transition| transition.points.iter().copied()),
    );
    StateLayout {
        width,
        height,
        states: node_layouts,
        transitions,
    }
}

pub(super) fn graph_depths<'a>(
    nodes: impl Iterator<Item = &'a str>,
    edges: impl Iterator<Item = (&'a str, &'a str)>,
) -> BTreeMap<String, usize> {
    let nodes = nodes.map(str::to_owned).collect::<Vec<_>>();
    let mut incoming_count = nodes
        .iter()
        .map(|node| (node.clone(), 0usize))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();
    for (from, to) in edges {
        if incoming_count.contains_key(from) && incoming_count.contains_key(to) {
            *incoming_count.entry(to.to_owned()).or_default() += 1;
            outgoing
                .entry(from.to_owned())
                .or_default()
                .push(to.to_owned());
        }
    }

    let mut depths = BTreeMap::new();
    let mut queue = VecDeque::new();
    for node in &nodes {
        if incoming_count.get(node).copied().unwrap_or_default() == 0 {
            depths.insert(node.clone(), 0);
            queue.push_back(node.clone());
        }
    }
    if queue.is_empty() {
        if let Some(first) = nodes.first() {
            depths.insert(first.clone(), 0);
            queue.push_back(first.clone());
        }
    }

    while let Some(node) = queue.pop_front() {
        let next_depth = depths[&node] + 1;
        for next in outgoing.get(&node).into_iter().flatten() {
            if !depths.contains_key(next) {
                depths.insert(next.clone(), next_depth);
                queue.push_back(next.clone());
            }
        }
    }

    let fallback_depth = depths.values().copied().max().unwrap_or(0) + 1;
    for node in nodes {
        depths.entry(node).or_insert(fallback_depth);
    }
    depths
}

pub(super) fn position_state_nodes(
    nodes: &mut [StateNodeLayout],
    layers: &BTreeMap<usize, Vec<usize>>,
    layer_sizes: &BTreeMap<usize, (f32, f32)>,
) {
    let max_width = layer_sizes
        .values()
        .map(|(width, _)| *width)
        .fold(0.0, f32::max);
    let mut y = MARGIN;
    for (depth, indexes) in layers {
        let (layer_width, layer_height) = layer_sizes[depth];
        let mut x = MARGIN + (max_width - layer_width) / 2.0;
        for node_index in indexes {
            nodes[*node_index].rect.x = x;
            nodes[*node_index].rect.y = y + (layer_height - nodes[*node_index].rect.height) / 2.0;
            x += nodes[*node_index].rect.width + NODE_GAP;
        }
        y += layer_height + LAYER_GAP;
    }
}

pub(super) fn edge_points(from: Rect, to: Rect) -> Vec<Point> {
    let start = Point {
        x: from.center_x(),
        y: from.y + from.height,
    };
    let end = Point {
        x: to.center_x(),
        y: to.y,
    };
    let mid_y = (start.y + end.y) / 2.0;
    vec![
        start,
        Point {
            x: start.x,
            y: mid_y,
        },
        Point { x: end.x, y: mid_y },
        end,
    ]
}

pub(super) fn layout_bounds(
    rects: impl Iterator<Item = Rect>,
    points: impl Iterator<Item = Point>,
) -> (f32, f32) {
    let mut width: f32 = MARGIN * 2.0;
    let mut height: f32 = MARGIN * 2.0;
    for rect in rects {
        width = width.max(rect.x + rect.width + MARGIN);
        height = height.max(rect.y + rect.height + MARGIN);
    }
    for point in points {
        width = width.max(point.x + MARGIN);
        height = height.max(point.y + MARGIN);
    }
    (width.ceil(), height.ceil())
}
