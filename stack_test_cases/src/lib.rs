use stack_alg_sim::stack::LRUStack;
use stack_alg_sim::vec::LRUVec;
use stack_alg_sim::LRU;

pub fn nmm(
    a_size_row: usize,
    a_size_col: usize,
    b_size_row: usize,
    b_size_col: usize,
    lru_type: String,
) -> Vec<(String, Option<usize>)> {
    assert_eq!(
        a_size_col, b_size_row,
        "The number of A columns must be equal to the number of B rows for matrix multiplication."
    );

    let mut cache: Box<dyn LRU<(usize, usize, char)>> = if lru_type == "Vec" {
        Box::new(LRUVec::<(usize, usize, char)>::new())
    } else {
        Box::new(LRUStack::<(usize, usize, char)>::new())
    };

    let mut dists: Vec<(String, Option<usize>)> = Vec::new();

    for i in 0..a_size_row {
        for j in 0..b_size_col {
            for k in 0..a_size_col {
                // println!("Here");
                let a_tuple = (i, k, 'A');
                let cur_a = cache.rec_access(a_tuple);
                dists.push((format!("{:?}", a_tuple), cur_a));

                let b_tuple = (k, j, 'B');
                let cur_b = cache.rec_access(b_tuple);
                dists.push((format!("{:?}", b_tuple), cur_b));

                let c_tuple = (i, j, 'C');
                let cur_c = cache.rec_access(c_tuple);
                dists.push((format!("{:?}", c_tuple), cur_c));

                let c_tuple = (i, j, 'C');
                let cur_c = cache.rec_access(c_tuple);
                dists.push((format!("{:?}", c_tuple), cur_c));
            }
        }
    }

    dists
}
