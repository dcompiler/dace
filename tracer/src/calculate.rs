use fxhash::FxHashMap;
use hist::Hist;
use list_serializable::ListSerializable;
use stack_alg_sim::olken::LRUSplay;
use stack_alg_sim::stack::LRUStack;
use stack_alg_sim::vec::LRUVec;
use stack_alg_sim::LRU;
use std::collections::hash_map::Entry;
type Reuse = ListSerializable<(usize, Option<usize>)>;

pub fn calculate_trace(
    trace_input: &ListSerializable<usize>,
    lru_type: &str,
) -> (Hist, Hist, Reuse, Reuse) {
    let mut lru: Box<dyn LRU<usize>> = match lru_type {
        "Olken" => Box::new(LRUSplay::<usize>::new()),
        "Stack" => Box::new(LRUStack::<usize>::new()),
        "Vec" => Box::new(LRUVec::<usize>::new()),
        _ => Box::new(LRUSplay::<usize>::new()),
    };

    let mut hist_rd = Hist::new();
    let mut hist_ri = Hist::new();
    let mut dist_rd: ListSerializable<(usize, Option<usize>)> =
        ListSerializable::<(usize, Option<usize>)>::new();
    let mut dist_ri: ListSerializable<(usize, Option<usize>)> =
        ListSerializable::<(usize, Option<usize>)>::new();
    let mut timestamps: FxHashMap<usize, usize> = FxHashMap::default();

    for (i, element) in trace_input.get_vec().iter().enumerate() {
        let dist = lru.rec_access(*element);
        hist_rd.add_dist(dist);
        dist_rd.add((*element, dist));
        match timestamps.entry(*element) {
            Entry::Occupied(mut entry) => {
                let prev_i = *entry.get();
                dist_ri.add((*element, Some(i - prev_i)));
                hist_ri.add_dist(Some(i - prev_i));
                *entry.get_mut() = i;
            }
            Entry::Vacant(entry) => {
                dist_ri.add((*element, Some(0)));
                hist_ri.add_dist(Some(0));
                entry.insert(i);
            }
        }
    }

    (hist_rd, hist_ri, dist_rd, dist_ri)
}
