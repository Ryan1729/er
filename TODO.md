# TODO

* Write an intentionally panicking program and make sure that running it does not cause `er` to panic.
* Try sending massive amounts of data down stdout or something like that to try an get `er` to panic.
* If we found any ways for a command to make `er` panic, then start attempting to save the history on panic.
* start saving history with multiple instances properly
    * As far as I can tell there's no cleaner way to get the history in the right order than giving each instance its own file and merging them (optionally?) on startup
