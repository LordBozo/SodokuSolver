# Overview
I just wanted to make a sodoku solver, both because it seemed like an interesting problem to solve, and to get better at Sodoku myself

It also works as a generator for making sodokus that use specific rules (though that is a much harder problem to solve)

# Using
Any text prompt either accepts the whole word, or any abbreviated version

## Commandline
Run with -h to get all the commandline args, currently all args are accessible from within the program, so running with args is just to help re-running the program multiple times

ex. Solve accepts Solve, Solv, S, and s
# Modes
## Solving
Input a sodoku board, using 0 or space for unknown cells

You can use pipes('|') as spacers to make it easier to get columns set

After all 9 rows have been entered, if it parsed correctly, choose whether you want to see how to solve it, or if you just want the answer

Solution Help Example(Sped up for gif):

![sodoku-solve](https://github.com/user-attachments/assets/3c864d37-c9c5-4545-a8f7-465d2568c8fa)

## Generate
Accepts inputs for what rules are allowed to be used when solving. Naked Single is always enabled, but others are opt-in




## Test
Runs the test sodoku boards in the program to ensure all that have been solved are still solvable
