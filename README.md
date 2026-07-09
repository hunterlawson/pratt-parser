# Rust Pratt Parser

Parses strings into a token tree using opereator precidence with the following operators and features supported:

- Single character variables: `a, b, c`
- Operators: `+ - * / ^ ()`

## Examples

The output of the following examples is a prefix notation representation of the tree output from the parser:

`"1 + 2 * 3" -> (+ 1 (* 2 3))`

`"1 + 2 * Z^2 -> (+ 1 (* 2 (pow Z 2)))`

`"(1 + 2) * 3" -> (* 3 (+ 1 2))`
