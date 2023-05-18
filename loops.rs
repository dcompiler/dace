/// These are index vectors.
pub type IterVec = Vec<i32>;
pub type ArrAcc = Vec<i32>;

// a single loop (a hidden module)
struct SingleLoop {
    iv: String,
    lb: fn(IterVec)->i32,
    ub: fn(IterVec)->i32,
    // The two arguments are index and upper bound
    test: fn(i32, i32)->bool,
    step: fn(i32)->i32
}

/// A loop nest is perfectly nested.
