# Rust Pratt Parser

Parses strings into an expression tree using opereator precidence with the following operators and features supported:

- Variables: `X, Y, num_1, abc`
- Functions: `abs(x)` `complex_multiply(a, b)`
- Operators: `+ - * / ^`

## Examples

The output of the following examples is a prefix notation representation of the tree output from the parser:

`"1 + 2 * 3" -> (+ 1 (* 2 3))`

`"1 + 2 * Z^2 -> (+ 1 (* 2 (pow Z 2)))`

`"(1 + 2) * 3" -> (* 3 (+ 1 2))`
