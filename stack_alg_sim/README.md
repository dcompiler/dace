# stack-alg-sim

Simulation of stack algorithms, originally defined by Mattson et al. 1970.  For the LRU distance (called reuse distance by a share of the literature), 
more efficient algorithms include Olken 1981 and Ding and Zhong PLDI 2003 (Zhong et al. TOPLAS 2009). 



```
stack-alg-sim
├─ .git
├─ .gitattributes
├─ .gitignore
├─ LICENSE
├─ README.md
├─ bench
│  ├─ Cargo.toml
│  └─ src
│     ├─ bin
│     │  └─ main.rs
│     └─ main.rs
├─ create.sql
├─ database.db
├─ hist
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
├─ lru_stack
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
├─ lru_trait
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
├─ lru_vec
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
├─ olken
│  ├─ Cargo.toml
│  └─ src
│     └─ lib.rs
└─ test_cases
   ├─ Cargo.toml
   └─ src
      └─ lib.rs

```