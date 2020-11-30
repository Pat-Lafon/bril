# Introduction

## Task 3

I added new tests for task3 in [lvn](test/lvn) and [tdce](test/tdce). Currently the ones that only use a single basic block work and that reduce to there simplest form using ```--lvn --dce```. A specific example is [clobber](test/lvn/clobber.bril) with when running ```bril2json < test/lvn/clobber.bril | cargo +nightly run -q -- --lvn --dce | bril2txt``` produces:

```python
@main {
  prod1: int = const 36;
  print prod1;
}
```
