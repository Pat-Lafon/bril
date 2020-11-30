# Task 7

For this task, I choose to implement loop invariant code motion on the non-ssa form of bril. Much of this implementation was based on the previous years [blog post](https://www.cs.cornell.edu/courses/cs6120/2019fa/blog/loop-reduction/#strength-reduction). Given that I have a dataflow framework, finding loops is straight forward: Do the worklist algorithm approach for dominators and then identify back edges. We can then do another call to the worklist algorithm to compute the reaching definitions for the function graph. I then need to then have condition checks that a given variable is loop invariant, only defined once, dominates all exits, and isn't live at the preheader. I set up each of the conditions as functions that I then and all together as one single if check. This if check is then run across all instructions in a loop of blocks to find and hoist loop invariant code.

The code for this is in [licm.rs](src/licm.rs), [dominator.rs](src/dominator.rs), and [reaching_defs.rs](src/reaching_defs.rs).

As an evaluation of this, I check the change in the total number of dynamic instructions. This is a good measure as the whole point of this is to move code from something that is run on every loop to something that is run once before the loop. This is run on the benchmarks.

Benchmarks with the memory extension are not implemented. Many of the benchmarks do not improve. It is unclear how much of this is missed optimization opportunities because our identification of loop invariance is too strict, because we aren't running other optimizations, because there is an error in the implentation, or because the benchmarks are not complicated enough for licm to be beneficial.

The following benchmarks were tested using ```bril2json < benchmarks/quadratic.bril | cargo +nightly run -q -- --licm | brili -p -5 8 21```

| Benchmark | Without Licm | With Licm |
| -----|------ | -------- |
| ackermann | 1464231 | 1464231 |
| binary-fmt | 100 | 100 |
| check-primes | 8438 | 8073|
| collatz| 156| 156|
| digital-root | 248 | 248 |
| euclid |562 | 562 |
| gcd | 42|42|
| loopfact | 116 | 108|
| orders | 5447 | 5117|
| perfect |233| 233|
| pythagorean_triple | 61518 | 61518|
| quadratic |785| 701|
| recfact |103|103|
| sqrt |320|320|
| sum-bits |73|73|
| sum-sq-diff | 3037| 2837|