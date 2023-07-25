use dace::ast::AryRef;

const DS: usize = 8;
const CLS: usize = 64;

pub fn access3addr(ary_ref: &AryRef, ivec: &[i32]) -> usize {
    let ary_index = (ary_ref.sub)(ivec);
    if ary_index.len() != ary_ref.dim.len() {
        panic!("array index and dimension do not match");
    }

    let offset = ary_index
        .iter()
        .zip(ary_ref.dim.iter())
        .fold(0, |acc, (&i, &d)| acc * d + i);

    (ary_ref.base.unwrap() + offset) * DS / CLS
}

