# Microdot - a REPL and terminal ui for dot and graphviz

A surprisingly useful kind of diagram is the dependency diagram - it shows how one thing leads to or causes another. For example, it could be part of a story, showing how one event lays the groundwork for the next;

![Fellowship of the Ring](./examples/fellowship.svg)

It might also be useful for business analysis, where we examine how problems in a business are inter-related;

![Business Example](./examples/business_example_1.svg)

This kind of diagram is really useful, but tooling to help you make it is pretty hard to come by. Especially if you're not a programmer.

There is a tool called graphviz, which makes these diagrams, which use the [Dot](https://graphviz.org/doc/info/lang.html) language to create these kinds of diagram. It's a system that is widely supported, but frankly becomes really hard to manage once you get above about ten nodes. Beyond that things get tricky: since you write node names and edges using long human names, things like a rename can get really annoyingly complex, with lots of find/replace over a file that's just too noisy.

I've written `microdot` to make this simpler and more interactive. It's a command-line tool you can start to build up the graph node by node, and edge by edge.

It's a [REPL]() -- a Read-evaluate-print-loop -- kind of program. That means you type a command, and the system READs it, then INTERPRETs it by making changes to the graph, and PRINTs the result -- well, outputs a vector graphics file! You can then LOOP back to the start, typing another command.

So, to insert an item you would use something like this;

```
$ microdot --file story.json
>> i Gandalf comes to the shire
(inserted node n0: 'Gandalf comes to the shire')
```

So, there is a command, `i` for insert. Then the text of the command you want. 

Now the SVG will be produced at `story.svg` and you can open it in a browser to see the result. Better, if you want it to be interactive, you can use [Gapplin](http://gapplin.wolfrosch.com/) to automatically refresh the SVG as it changes.

Note how you can see a node ID -- `n0` -- which you can use to refer to that node later. This is really useful when you want to link nodes together, or delete them, or rename them.

For example, to delete the node;

```
>> d n0
(node n0 removed)
```

For the full story example;

```
$ microdot --file story.json
>> i Gandalf comes to the shire
(inserted node n0: 'Gandalf comes to the shire')
>> aft n0 Frodo departs with the ring
(inserted node n1: 'Frodo departs with the ring' after n0)
>> aft n1 the inn at Bree
(inserted node n2: 'the inn at Bree' after n1)
>> aft n2 the hobbits escape with Aragorn
(inserted node n3: 'the hobbits escape with Aragorn' after n2)
>> aft n3 nazghuls catch up at Weathertop; Frodo is injured
(inserted node n4: 'nazghuls catch up at Weathertop; Frodo is injured' after n3)
>> i Nazghuls move to Bree
(inserted node n5: 'Nazghuls move to Bree')
>> l n5 n2
(Added edge e4 from n5 to n2)
>> aft n2 the Nazghuls move to Weathertop
(inserted node n6: 'the Nazghuls move to Weathertop' after n2)
>> l n6 n4
(Added edge e6 from n6 to n4)
>> bef n5 Nazghuls dispatched from Mordor
(inserted node n7: 'Nazghuls dispatched from Mordor' before n5)
>> aft n4 flight to the ford
(inserted node n8: 'flight to the ford' after n4)
>> aft n8 the fellowship meets at Rivendell "one does not simply walk into mordor"
(inserted node n9: 'the fellowship meets at Rivendell "one does not simply walk into mordor"' after n8)
>> l n0 n9
(Added edge e10 from n0 to n9)
>> bef n9 Gimli leaves the mountains
(inserted node n10: 'Gimli leaves the mountains' before n9)
>> bef n9 Legolas travels from Mirkwood
(inserted node n11: 'Legolas travels from Mirkwood' before n9)
>> bef n9 Boromir seeks the sword that is broken
(inserted node n12: 'Boromir seeks the sword that is broken' before n9)
>> bef n12 Boromir fights in the battle for Osgiliath
(inserted node n13: 'Boromir fights in the battle for Osgiliath' before n12)
>> /hobbits
(Search results for: hobbits,
n3: the hobbits escape with Aragorn
)
CTRL-D

```

For the business example;

```
$ microdot --file story.json
>> lr
(Direction changed to LR)
>> i #customers get delivery too slowly #SG_RESULT
(inserted node n0: '#customers get delivery too slowly #SG_RESULT')
>> bef n0 orders need to be processed by hand #SG_CURRENT
(inserted node n1: 'orders need to be processed by hand #SG_CURRENT' before n0)
>> bef n1 no developer capacity to automate orders #SG_CURRENT
(inserted node n2: 'no developer capacity to automate orders #SG_CURRENT' before n1)
>> bef n2 developers engaged in low-value work #SG_CURRENT
(inserted node n3: 'developers engaged in low-value work #SG_CURRENT' before n2)
>> bef n0 shipping labels generated once a day #SG_BLOCKER
(inserted node n4: 'shipping labels generated once a day #SG_BLOCKER' before n0)
>> bef n4 printers need rebooting but everyone in IT is busy util 3pm #SG_BLOCKER
(inserted node n5: 'printers need rebooting but everyone in IT is busy util 3pm #SG_BLOCKER' before n4)
>> l n3 n5
(Added edge e5 from n3 to n5)
CTRL-D

```

See how we're working one line at a time, inserting nodes and linking them together? Each time you make a change, the diagram is regenerated on disk as an SVG file. SVGs can be opened in a browser, making a cheap and cheerful viewer, or you can use a tool like [Gapplin](http://gapplin.wolfrosch.com/) to automatically refresh the SVG as it changes.

This approach can be pretty good for workshops or interactive sessions, where you act as a moderator, and people can call out intructions, like "I think we need to link n3 to n8," and you can add them. Maybe someday I'll make something cooperative, but not today :)


## Installation

Since `microdot` uses the `dot` program from the Graphviz suite, you'll need to have that installed. You can get it from [here](https://graphviz.org/download/), and it can be installed by `brew` on MacOs.

You'll also want to get an SVG viewer, like [Gapplin](http://gapplin.wolfrosch.com/), which will automatically refresh the SVG as it changes.

Lastly, you'll want to check out the source code from github and build it -- it's written in Rust, so you'll need to have that installed. You can get it from [here](https://www.rust-lang.org/tools/install).
