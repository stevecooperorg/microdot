# Microdot - a terminal ui for dot and graphviz

[Dot](https://graphviz.org/doc/info/lang.html) is a great DSL for describing graphs, and it works fine until you get to about ten nodes. Beyond that things get tricky: since you write node names and edges using long human names, things like a rename can get really annoyingly complex, with lots of find/replace over a file that's just too noisy.

An interactive editor with full drag and drop would be cool, but... thanks a lot of work. And who doesn't like a good unix-style command line tool?

enter microdot. A repl-driven system for building graphs. The idea is to use language like so;

```
$ microdot my-graph.dot
> n a cool new node
(node added n1)
> n a second node
(node added n2)
> e n1 n2
(edge added; n1->n2)
> r n1 a new name
(n1 renamed)
```

This REPL-style app makes editing a large graph easy and interactive. It outputs `dot`, the graphviz language for rendering directed graphs, and importantly it includs a 'draft mode' output so you can see those node IDs;

[pic to come]

In draft mode, the IDs of nodes and edges are included. This means you can render the graph and show it to the user, and they will see all the IDs they use for their edit commands. This makes it really easy to do things like delete an edge that shouldn't exist, rename a node, or insert a new node onto an edge. The operations that are hard when manually writing dot files.

Once complete, you can render the real artefact; with the right names, for presenting to people. And it's just a switch between, delivered as, say

```
> int
(interactive mode)
> disp
(display mode)
```

Implementation Nodes

- I'm going to write in rust
- I'll be using https://crates.io/crates/rustyline to give a rich repl experience, with history etc.
- microdot pushes complete dot files to stdout, with some separator between them. i.e it produces a stream of dot files
- typically you would redirect stdout to a file descriptor
- ; in a V1 I'm thinking of doing;

```
# console 1
mkfifo md.out
microdot > md.out

# console 2
cat md.out
```

- reading and compiling the dot files is a second app, which would read from the file descriptor. Let's call this `dotcom`, the dot compiler. It'll just invoke graphviz and write the file... somewhere.
- last step, an auto-refreshing preview window shows the 
- 
