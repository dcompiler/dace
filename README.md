# Design Notes
## Compile-time Enuerable Programs [Chen et al. PLDI 2018]

This is the class of programs that can be analyzed by static sampling.  These are the conditions:

+ A program is a sequence of statements and loop nests. The loops may be imperfectly nested. A statement is treated as a degenerate loop with just one iteration.
+ The program may have branches, i.e. structured if- statements.
+ The expressions of loop bounds, strides, branch predicates, and array subscripts contain only loop index variables and constants, i.e. all symbols in loop bounds and array index expressions, except for loop indices, are constants

