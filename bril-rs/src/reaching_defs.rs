use crate::cfg::{BasicBlock, Graph};
use crate::worklist::Constraints;

use std::collections::HashMap;
use std::collections::HashSet;

fn transfer(
    mut in_constraint: HashMap<String, HashSet<u32>>,
    block: &BasicBlock,
) -> HashMap<String, HashSet<u32>> {
    block.code.iter().for_each(|i| {
        i.get_dest().and_then(|s| {
            let mut loc = HashSet::new();
            loc.insert(block.index);
            in_constraint.insert(s, loc)
        });
    });
    in_constraint
}

fn meet(vec_of_sets: Vec<HashMap<String, HashSet<u32>>>) -> HashMap<String, HashSet<u32>> {
    let mut out = HashMap::new();
    vec_of_sets.into_iter().for_each(|in_map| {
        in_map.into_iter().for_each(|(key, set)| {
            let e = out.entry(key).or_insert(HashSet::new());
            *e = e.union(&set).cloned().collect()
        })
    });
    out
}

pub fn reaching_defs(graph: &mut Graph) -> Constraints<HashMap<String, HashSet<u32>>> {
    graph.worklist_algo(&|_| HashMap::new(), &transfer, &meet, true)
}
