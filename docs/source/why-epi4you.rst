Why epi4you exists
==================

Oxford Nanopore's EPI2ME Desktop gives users a convenient local environment for
running and browsing nanopore workflows. That convenience is valuable, but it
also means a completed analysis is spread across several concerns:

* the workflow output itself,
* the metadata that lets the GUI present the run coherently,
* the workflow identity and versioning information, and
* sometimes related software assets such as containers or workflow trees.

For everyday use on a single workstation, that is fine.

For training and deployment work, it becomes awkward.

The practical problem
---------------------

If you need to prepare twenty laptops for a workshop, or preload a conference
demo machine, copying one output directory is often not enough.

Typical problems include:

* the target machine has no matching EPI2ME database entry,
* the original workflow version is not obvious,
* the GUI expects helper files that a plain CLI run never created,
* internet access is limited or unavailable, and
* rebuilding analyses from raw data would take too long.

epi4you's answer
----------------

``epi4you`` exists to make that liftover explicit and repeatable.

The software packages a run into a manifest-driven archive so the transfer is
not just "copy these files and hope". Instead the archive includes:

* a typed payload description,
* a file inventory,
* provenance,
* a lightweight integrity check, and
* enough reconstructed metadata to make the imported result behave like an
  EPI2ME-style analysis.

Why the project is broader than the current CLI
-----------------------------------------------

The repository contains logic for several related asset classes:

* Desktop analysis records,
* workflow installations,
* Docker/container artefacts, and
* historical Nextflow CLI runs.

The currently active entry points focus on packaging a CLI Nextflow run and
importing a ``.2me`` archive, but the wider codebase reflects the broader
problem the project was created to solve: moving complete bioinformatics assets
between machines, not just moving raw result files.

Audience
--------

The tool is especially well suited to:

* trainers and course organizers,
* field teams carrying prebuilt examples,
* demo and conference operators,
* support engineers reproducing representative analyses, and
* labs that need to share curated EPI2ME-ready examples internally.
