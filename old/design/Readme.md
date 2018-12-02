# Idea:

Something that runs executables, and displays their output, but is not a terminal emulator. Partially inspired by https://www.destroyallsoftware.com/talks/a-whole-new-world .

Personally 99% of what I do in a terminal ever is run executables. I don't need all the legacy stuff. I don't care if it couldn't be done on a VT1000. I don't need it to support every weird feature or kernel call. I don't care id it can't support playing Rogue or if t displays `htop` correctly. I can write my own `less` if I need to. If I encounter that 1%, I can use a "real terminal".

## Possible Drawbacks and mitigations
Many programs assume you are in a terminal. I may end up being surprised by how many times `\x33m` etc appears when I run things. This could be mitigated by either filtering all of these, or implementing the most commonly used ones.

A more worrying drawback is some programs change their behavior when being run in a terminal, suggesting that we may need to set up some kind of environment the tells executables that they are or are not in a terminal. It also suggests a larger prevalence of programs which manipulate the terminal so excessively that simply ignoring or barely implementing things will not suffice, than I initially thought.

## Names:

Just F-ing Run Executables (JFRE)

Type To Run Executables (TTRE)

e (short for executables)

er (short for executable runner)
    I like this one. You go to run something and you tell the computer `er` and then you say what you want to run.

## Bare minimum features

* ability to type executable names and arguments, and execute that command.
* ability to halt spawned processes. Preferably with a keyboard shortcut.
* ability to see output of executed programs
* command history for the current session
* ability to copy text from the window
* ability to paste text into the command prompt

## Extra Features

* I'll also want to be able to sequence commands together. The bash `&&`, `||` and `;` notation seem as good as any. Also, many executables will want to be bash compatible, so they are unlikely to require input containing those characters. 
* command history that is saved to disk on close
* command history that is automatically saved to disk periodically

