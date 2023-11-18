# Backdoor searcher

> Evolutionary algorithm for searching rho-backdoors for a given SAT formula.

## Usage

### CLI

```
search [OPTIONS] <CNF>

Arguments:
  <CNF>  Input file with CNF in DIMACS format

Options:
      --backdoor-size <INT>       Backdoor size
      --num-iters <INT>           Number of EA iterations
      --num-runs <INT>            Number of EA runs [default: 1]
      --seed <INT>                Random seed [default: 42]
      --results <FILE>            Output file with results
  -h, --help                      Print help
  -V, --version                   Print version
```

### Example

Search for 3 backdoors, each of size 10, using 1000 iterations of EA and the random seed 42 (default):

```sh
cargo run -p backdoor --bin search --release -- data/mult/lec_CvK_12.cnf --backdoor-size 10 --num-iters 1000 --num-runs 3 --seed 42 --results output.txt
```

`output.txt` might look like:
```
Backdoor [1180, 2163, 3625, 3695, 3911, 3980, 4020, 4071, 5341, 5387] of size 10 on iter 887 with fitness = 0.0078125, rho = 0.9921875, hard = 8 in 612.654 ms
Backdoor [824, 2095, 2688, 3447, 3732, 3787, 3876, 3890, 4005, 4475] of size 10 on iter 533 with fitness = 0.0078125, rho = 0.9921875, hard = 8 in 295.766 ms
Backdoor [959, 975, 1248, 1902, 1946, 1994, 2313, 2414, 3885, 4913] of size 10 on iter 624 with fitness = 0.01953125, rho = 0.98046875, hard = 20 in 603.379 ms
```

Note: variables in the reported backdoors (in file/console) are 1-based.
