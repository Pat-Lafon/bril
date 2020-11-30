# Task 4

Based on previous experience, I implemented the generic worklist algorithm located at [worklist.rs](src/worklist.rs). It creates a worklist from a list of nodes in the graph and iterates over a set of constraints until nothing changes. The constraints can be parameterized over a clonable data structure of your choice and are iterated on by meet and transfer functions the user provides. Getting these functions to type-check was a little tricky due to differences in passing a function as an argument versus passing a closure as an argument.

This framework was used to implement dead code elimination by doing a cool backwards, dataflow analysis to check the liveliness of variables. Doing it in reverse means that you can find all of the dead code in one go and then you just need a second pass to remove all of the instructions.
