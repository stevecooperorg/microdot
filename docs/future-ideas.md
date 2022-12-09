Reusing existing code
===


I've written this at quite a low level but it turns out there's some useful code out there it could be worth using

- petgraph - the standard for representing graph data structures - https://crates.io/crates/petgraph
- fdg - a force-directed graph layout algorithm. Maybe not as useful as dot/graphviz, but if it's wasm-enabled it opens up the possibility of having a live server for shared edits. https://github.com/grantshandy/fdg

Network Capable
===

It's a single-user system at the moment, which means it's not as good as it could be for interactive sessions 

- zeromq - a way to create a networked REPL so multiple users can contribue to a single running server over terminal - https://zeromq.org/languages/rust/

Multiple UIs
===

While _I_ really love a REPL, the bulk of the app is actually more general and new 'heads' could be added if there was a neatly extracted interface -- say, through the GraphCommand enum? -- that could be used.
