# Microdot - a REPL and terminal ui for dot and graphviz

A surprisingly useful kind of diagram is the dependency diagram - it shows how one thing leads to or causes another. For example, it could be part of a story, showing how one event lays the groundwork for the next;

![Fellowship of the Ring](./examples/fellowship.svg)

It might also be useful for business analysis, where we examine how problems in a business are inter-related;

![Business Example](./examples/business_example_1.svg)

This kind of diagram is really useful, but tooling to help you make it is pretty hard to come by. Especially if you're not a programer.

There is a tool called graphviz, which makes these diagrams, which use the [Dot](https://graphviz.org/doc/info/lang.html) language to create these kinds of diagram. It's a system that is widely supported, but frankly becomes really hard to manage once you get above about ten nodes. Beyond that things get tricky: since you write node names and edges using long human names, things like a rename can get really annoyingly complex, with lots of find/replace over a file that's just too noisy.

I've written `microdot` to make this simpler and more interactive. It's a command-line tool you can start to build up the graph node by node, and edge by edge.

For the story example;

```
$ microdot --file story.json
{{fellowship_content}}
```

For the business example;

```
$ microdot --file story.json
{{business_content}}
```

See how we're working one line at a time, inserting nodes and linking them together? Each time you make a change, the diagram is regenerated on disk as an SVG file. SVGs can be opened in a browser, making a cheap and cheerful viewer, or you can use a tool like [Gapplin](http://gapplin.wolfrosch.com/) to automatically refresh the SVG as it changes.

This approach can be pretty good for workshops or interactive sessions, where you act as a moderator, and people can call out intructions, like "I think we need to link n3 to n8," and you can add them. Maybe someday I'll make something cooperative, but not today :)

---

Enter microdot. A repl-driven system for building graphs. The idea is to use language like so;

```
$ microdot --file my-graph.json
{{example_content}}
```

This REPL-style app makes editing a large graph easy and interactive. It outputs `dot`, and compiles it to `svg` if you have graphviz installed and on your path. Importantly it defaults to a 'draft mode' output so you can see those node IDs;

![Fellowship of the Ring](./examples/readme_example_1.svg)

In draft mode, the IDs of nodes and edges are included. This means we render a version where every node and edge can be referred to by a very short ID, like `n34` or `e16`. This makes it really easy to do things like delete an edge that shouldn't exist, rename a node, or insert a new node onto an edge. The operations that are hard when manually writing dot files.

Once complete, you can render the real artefact; with the right names, for presenting to people. And it's just a switch between, delivered as, say

```
> int
(interactive mode)
> disp
(display mode)
```


--

Color Palettes:

- go to http://khroma.co/train/
- train it with the 50-color selection
- on the trained page, pick a bunch of favourites - as many as you want for different node types
- download trained data (settings cog, download Kharma data button, get json file)
- copy it into src/my_khroma_data.json

https://applecolors.com/palettes

#ED4145 #F1B02F #EADE84 #A3D064 #11B2AA #177C99