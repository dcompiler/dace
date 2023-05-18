/// An array reference
pub struct ArrayRef {
    name: String,
    /// subscript expressions: one function for each array dimension
    subexprs: Vec<fn(Vec<i32>) -> Vec<i32>>
}
