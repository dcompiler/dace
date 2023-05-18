/// An array reference
/// Each array reference resides in a loop nest.
pub struct ArrayRef {
    name: String,
    /// Subscript expressions: one function for each data dimension.  
    /// Each function takes the indices of its loop nest and returns indices 
    /// of the array access.
    subexprs: Vec<fn(Vec<i32>) -> Vec<i32>>
}
