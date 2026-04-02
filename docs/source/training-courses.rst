Training courses and asset liftover
===================================

The original development driver for ``epi4you`` was training delivery.

In a training course, the goal is rarely to prove that an analysis can be run
from scratch under ideal conditions. The goal is to make sure participants can
see, inspect, discuss, and learn from realistic bioinformatics assets on the
machines in front of them.

Why training environments are awkward
-------------------------------------

Training and workshop setups tend to magnify deployment friction:

* there may be many target laptops,
* there is limited time before the session starts,
* internet access may be unreliable,
* downloading large software artefacts repeatedly is wasteful, and
* re-running full analyses may not fit into the teaching schedule.

For nanopore workflows, that friction can include:

* large result trees,
* workflow metadata that matters to the GUI,
* versioned workflow assets,
* container dependencies, and
* a need for visually coherent imported runs rather than raw folders.

How epi4you helps
-----------------

``epi4you`` makes training-course preparation a packaging task rather than a
manual reconstruction task.

The intended pattern is:

1. produce or curate a representative run on one source machine,
2. package it into a ``.2me`` archive,
3. copy that archive to training machines, and
4. import it into their local EPI2ME-style environment.

This has a few important benefits:

* the transfer artifact is explicit,
* the payload is documented by a manifest,
* metadata travels with the run,
* validation happens before import, and
* the target machine receives something closer to a GUI-ready analysis.

Typical course-preparation pattern
----------------------------------

One sensible workflow for trainers is:

.. code-block:: text

   source workstation
     -> run or curate a representative Nextflow analysis
     -> package it with epi4you
     -> distribute the .2me archive

   trainee workstation
     -> import the archive
     -> open and discuss the analysis in its local EPI2ME-style context

What to package
---------------

Good candidate assets for course liftover include:

* small, representative completed analyses,
* canonical examples for one workflow version,
* "known good" demonstration runs,
* intentionally curated edge cases for teaching troubleshooting, and
* prebuilt exercises where the learning goal is interpretation rather than raw
  compute execution.

What this does not replace
--------------------------

``epi4you`` is not a substitute for workflow validation, reproducible pipeline
development, or formal package management. It is a pragmatic transport tool for
bioinformatics assets that need to survive a move from one environment into
another with the right shape and context.
