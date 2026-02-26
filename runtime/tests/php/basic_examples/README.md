# Basic PHP Doc Examples

This folder collects a handful of scripts inspired by the PHP manual so we have simple, independent files that both the native `php` CLI and the `php-router` can run through the `testrunner.js`.

Files:

- `hello.php`: minimal `echo` example from the basic syntax chapter.
- `variables.php`: demonstrates variables, interpolation, and simple math.
- `arrays.php`: exercises indexed arrays, loops, and `print_r`.
- `functions.php`: defines and calls a typed function to show PHP’s function syntax.

Run the whole suite with `bun tests/php/testrunner.js tests/php/basic_examples` from the repo root.
