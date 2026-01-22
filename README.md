# Using
Any text prompt either accepts the whole word, or any truncated version
ex. Solve accepts Solve, Solv, S, and s, but does not accept olve
# Modes
## Solving
Input a sodoku board, using 0 or space for unknown cells
You can use pipes('|') as spacers to make it easier to get columns set
After all 9 rows have been entered, if it parsed correctly, choose whether you want to see how to solve it, or if you just want the answer

Solution Help (Sped up for gif):

![sodoku-solve](https://github.com/user-attachments/assets/3c864d37-c9c5-4545-a8f7-465d2568c8fa)

## Generate
WIP
Currently it will just generate a random sodoku board that it can solve, with no regards for difficulty
Support for restricting allowed rules coming soon

## Test
Runs the test sodoku boards in the program to ensure all that have been solved are still solvable
