# microdot-termgraph

ASCII/Unicode graph drawing for microdot’s TUI.

Rendering code (character grid, node boxes, edge routing) is adapted from
[ascii-petgraph](https://github.com/elefthei/ascii-petgraph) (MIT). Force-directed
layout from that crate is **not** included — callers supply absolute positions
(typically from Graphviz `-Tplain`).

See `LICENSE-MIT` for the upstream license text.
