# TODO

* echo
* fix extra `\r\n` in history
* actually allow navigating history
* file-based tab completion
* selection and copying
    * selection first

* see if we can reasonably cover the program with [`#[no_panic]`](https://github.com/dtolnay/no-panic).
    * Assuming we can't:
        * Write an intentionally panicking program and make sure that running it does not cause `er` to panic.
        * Try sending massive amounts of data down stdout or something like that to try an get `er` to panic.
        * If we found any ways for a command to make `er` panic, then start attempting to save the history on panic.

* start saving history with multiple instances properly
    * As far as I can tell there's no cleaner way to get the history in the right order than giving each instance its own file and merging them (optionally?) on startup
        * What about just storing the time in the file as well? Then scanning through the file from the beginning and sorting it only if something is out of place?
