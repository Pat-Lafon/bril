# Task 11

## Implementation

I kept my implementation and assumptions super simple. I keep track of the indices of instructions until one of them repeats. At this point, I know that I have completed a loop. I can then duplicate the repeated sequence of instructions, and edit it with speculates/commits and rename the labels. This can then be inserted before the usual sequence runs as a trace of that sequence. Any previous control flow to this section is redirected to the trace. The interpreter spits out a new program with this change and quits. You can then go and run this code like normal.

## Running the interpreter
``` bril2json < while.bril| cargo +nightly run -- --tracing 2 ```

## Things that would improve tracing

Given enough time, I would like to:

- Collect more traces to determine which paths are most likely to be taken.

- Move guard operations as early in the trace as possible and then run an optimizer over the code.

- Try and create trace trees so that if a trace fails, it can try and run on some other optimized trace.

- Would be cool if speculation could support function calls to get inlining for free.

- Make sure this works for all tracing scenarios and doesn't fail on any weird edge cases(besides when including functions which isn't currently supported)
