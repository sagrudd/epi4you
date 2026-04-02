Architecture
============

This page explains the main code paths in terms of responsibilities rather than
individual source files.

High-level flow
---------------

The active project flow looks like this:

.. code-block:: text

   nextflow working directory
     -> run discovery
     -> run selection
     -> analysis/log staging
     -> desktop-style metadata synthesis
     -> manifest creation
     -> .2me tarball
     -> import

Main areas of the codebase
--------------------------

CLI entry points
   ``src/create_2me/create_from_cli_run.rs`` and
   ``src/importer/import_from_2me.rs`` define the active command-line flows.

Nextflow capture
   ``src/nextflow/nextflow_toolkit.rs`` indexes historical runs with
   ``nextflow log`` and orchestrates CLI-run packaging.

Analysis staging
   ``src/nextflow/nextflow_analysis.rs`` resolves output directories, finds
   matching logs, distills ``nextflow.stdout``, and synthesizes
   ``progress.json``.

Metadata extraction
   ``src/nextflow_log_parser.rs`` parses the reduced Nextflow transcript into
   workflow identity fields such as project, repository, revision, and version.

Desktop analysis model
   ``src/epi2me_desktop_analysis.rs`` defines the EPI2ME-style analysis record
   that is serialized into the archive payload.

Workflow payload model
   ``src/epi2me_workflow.rs`` inventories installed workflow files for packaging
   and import.

Manifest and archive semantics
   ``src/xmanifest.rs`` defines the portable archive structure, provenance, and
   manifest verification logic.

Design intent
-------------

The recurring design theme is translation.

``epi4you`` is not trying to replace Nextflow or EPI2ME Desktop. Instead it
translates between:

* raw CLI-oriented run artifacts,
* EPI2ME-style metadata expectations, and
* a portable archive form suitable for transfer.

This translation is why the codebase contains both low-level filesystem work and
higher-level domain models such as manifests and desktop analyses.

Relationship to the broader repository
--------------------------------------

The repository contains additional code for workflows, containers, and database
operations that reflects the wider original ambition of the project.

Even where those paths are not the current main CLI entry points, they still
matter architecturally because they explain why the manifest supports multiple
payload types and why the project thinks in terms of "bioinformatics assets"
rather than only "analysis result folders".
