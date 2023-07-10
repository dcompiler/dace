# DACE (达思) Programs

DACE stands for Data Access Compile-time Enumerable.  Its Chinese name is 达思 which means far reaching thoughts. 

Based on Compile-time Enumerable Programs [Chen et al. PLDI 2018]

This is the class of programs whose data accesses can be enumerated by a compiler, or data access compiler enumerable (DACE) programs.  These are the conditions:

+ A program is a sequence of statements and loop nests. The loops may be imperfectly nested. A statement is treated as a degenerate loop with just one iteration.
+ The program may have branches, i.e. structured if- statements.
+ The expressions of loop bounds, strides, branch predicates, and array subscripts contain only loop index variables and constants, i.e. all symbols in loop bounds and array index expressions, except for loop indices, are constants


```
dace
├─ .git
├─ .gitattributes
├─ .github
│  └─ workflows
│     └─ build.yaml
├─ .gitignore
├─ Cargo.toml
├─ LICENSE
├─ README.md
├─ aws_utilities
│  ├─ Cargo.toml
│  └─ src
│     ├─ batch.rs
│     ├─ ec2.rs
│     ├─ lib.rs
│     ├─ rds.rs
│     └─ s3.rs
├─ benches
│  ├─ lruvec_bench
│  │  ├─ Cargo.toml
│  │  └─ src
│  │     └─ main.rs
│  └─ stack_alg_sim_bench
│     ├─ Cargo.toml
│     └─ src
│        ├─ bin
│        │  └─ main.rs
│        └─ main.rs
├─ dace
│  ├─ Cargo.toml
│  └─ src
│     ├─ arybase.rs
│     ├─ ast.rs
│     ├─ iter.rs
│     └─ lib.rs
├─ dace_tests
│  ├─ Cargo.toml
│  └─ src
│     ├─ lib.rs
│     └─ polybench.rs
├─ database
│  ├─ create.sql
│  ├─ load.sql
│  └─ run.sql
├─ hist
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
├─ list_serializable
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
├─ polybenchrun
│  ├─ Cargo.toml
│  └─ src
│     └─ main.rs
├─ stack_alg_sim
│  ├─ Cargo.toml
│  ├─ LICENSE
│  ├─ README.md
│  └─ src
│     ├─ lib.rs
│     ├─ olken.rs
│     ├─ stack.rs
│     └─ vec.rs
├─ stack_test_cases
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
└─ tracer
   ├─ Cargo.toml
   ├─ LICENSE
   └─ src
      ├─ calculate.rs
      ├─ lib.rs
      └─ trace.rs

```