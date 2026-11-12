//! Stable block identity matching, gap assignment and hashing.
use super::*;

pub(crate) fn assign_stable_ids(
    previous: Option<&RenderSnapshot>,
    candidates: &[PatchCandidate],
) -> PatchAssignment {
    let mut assigned = vec![None::<String>; candidates.len()];
    let mut used_stable_ids = BTreeSet::new();

    if let Some(previous) = previous {
        let lcs_matches = lcs_by(
            &previous.blocks,
            candidates,
            |block| (block.kind.clone(), block.source_hash.clone()),
            |candidate| (candidate.kind.clone(), candidate.source_hash.clone()),
        );

        for (previous_index, candidate_index) in &lcs_matches {
            let stable_id = previous.blocks[*previous_index].stable_id.clone();
            assigned[*candidate_index] = Some(stable_id.clone());
            used_stable_ids.insert(stable_id);
        }

        let mut previous_start = 0usize;
        let mut candidate_start = 0usize;
        for (previous_end, candidate_end) in lcs_matches
            .iter()
            .copied()
            .chain(std::iter::once((previous.blocks.len(), candidates.len())))
        {
            assign_gap_stable_ids(
                &previous.blocks[previous_start..previous_end],
                &candidates[candidate_start..candidate_end],
                candidate_start,
                &mut assigned,
                &mut used_stable_ids,
            );
            previous_start = previous_end.saturating_add(1);
            candidate_start = candidate_end.saturating_add(1);
        }
    }

    let mut next_stable_index = next_stable_index(previous);
    for stable_id in assigned.iter().flatten() {
        used_stable_ids.insert(stable_id.clone());
    }

    for assigned_id in &mut assigned {
        if assigned_id.is_some() {
            continue;
        }
        let stable_id = loop {
            let candidate = format!("oriha-b-{next_stable_index}");
            next_stable_index += 1;
            if !used_stable_ids.contains(&candidate) {
                break candidate;
            }
        };
        used_stable_ids.insert(stable_id.clone());
        *assigned_id = Some(stable_id);
    }

    let mut by_block_id = BTreeMap::new();
    for (candidate, stable_id) in candidates.iter().zip(assigned) {
        let stable_id = stable_id.expect("all patch candidates should have a stable id");
        let assigned_block = AssignedPatchBlock {
            stable_id: stable_id.clone(),
            kind: candidate.kind.clone(),
            source_hash: candidate.source_hash.clone(),
            source_range: candidate.source_range,
        };
        by_block_id.insert(candidate.block_id.clone(), assigned_block);
    }

    PatchAssignment {
        by_block_id,
        next_stable_index,
    }
}

pub(super) fn assign_gap_stable_ids(
    previous: &[RenderSnapshotBlock],
    candidates: &[PatchCandidate],
    candidate_offset: usize,
    assigned: &mut [Option<String>],
    used_stable_ids: &mut BTreeSet<String>,
) {
    let mut previous_by_kind = BTreeMap::<&str, Vec<&RenderSnapshotBlock>>::new();
    for block in previous {
        previous_by_kind
            .entry(block.kind.as_str())
            .or_default()
            .push(block);
    }

    let mut consumed_by_kind = BTreeMap::<&str, usize>::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let assigned_index = candidate_offset + index;
        if assigned[assigned_index].is_some() {
            continue;
        }
        let Some(previous_blocks) = previous_by_kind.get(candidate.kind.as_str()) else {
            continue;
        };
        let consumed = consumed_by_kind.entry(candidate.kind.as_str()).or_default();
        while let Some(previous_block) = previous_blocks.get(*consumed) {
            *consumed += 1;
            if used_stable_ids.insert(previous_block.stable_id.clone()) {
                assigned[assigned_index] = Some(previous_block.stable_id.clone());
                break;
            }
        }
    }
}

pub(super) fn lcs_by<L, R, K, FL, FR>(
    left: &[L],
    right: &[R],
    left_key: FL,
    right_key: FR,
) -> Vec<(usize, usize)>
where
    K: Ord,
    FL: Fn(&L) -> K,
    FR: Fn(&R) -> K,
{
    let mut left_positions = BTreeMap::<K, Vec<usize>>::new();
    for (index, item) in left.iter().enumerate() {
        left_positions
            .entry(left_key(item))
            .or_default()
            .push(index);
    }

    #[derive(Clone, Copy)]
    struct Link {
        left_index: usize,
        right_index: usize,
        previous: Option<usize>,
    }

    let mut links = Vec::<Link>::new();
    let mut tails = Vec::<(usize, usize)>::new();

    for (right_index, item) in right.iter().enumerate() {
        let key = right_key(item);
        let Some(positions) = left_positions.get(&key) else {
            continue;
        };
        for &left_index in positions.iter().rev() {
            let tail_index = lower_bound_by_left_index(&tails, left_index);
            let previous = tail_index
                .checked_sub(1)
                .and_then(|index| tails.get(index).map(|(_, link_index)| *link_index));
            links.push(Link {
                left_index,
                right_index,
                previous,
            });
            let link_index = links.len() - 1;
            if tail_index == tails.len() {
                tails.push((left_index, link_index));
            } else if left_index < tails[tail_index].0 {
                tails[tail_index] = (left_index, link_index);
            }
        }
    }

    let Some((_, mut link_index)) = tails.last().copied() else {
        return Vec::new();
    };

    let mut matches = Vec::new();
    loop {
        let link = links[link_index];
        matches.push((link.left_index, link.right_index));
        let Some(previous) = link.previous else {
            break;
        };
        link_index = previous;
    }
    matches.reverse();
    matches
}

pub(super) fn lower_bound_by_left_index(tails: &[(usize, usize)], left_index: usize) -> usize {
    let mut start = 0usize;
    let mut end = tails.len();
    while start < end {
        let middle = start + (end - start) / 2;
        if tails[middle].0 < left_index {
            start = middle + 1;
        } else {
            end = middle;
        }
    }
    start
}

pub(super) fn next_stable_index(previous: Option<&RenderSnapshot>) -> u64 {
    let mut next = previous
        .map(|snapshot| snapshot.next_stable_index)
        .unwrap_or(1)
        .max(1);

    if let Some(previous) = previous {
        for block in &previous.blocks {
            if let Some(index) = stable_id_index(&block.stable_id) {
                next = next.max(index.saturating_add(1));
            }
        }
    }

    next
}

pub(super) fn stable_id_index(stable_id: &str) -> Option<u64> {
    stable_id.strip_prefix("oriha-b-")?.parse().ok()
}

pub(super) fn stable_hash_hex(value: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}
