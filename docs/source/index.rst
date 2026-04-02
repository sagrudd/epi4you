epi4you documentation
=====================

``epi4you`` is a toolkit for moving Oxford Nanopore EPI2ME assets between
machines and contexts, especially when portability matters more than a perfect
reproduction of the original workstation.

The original motivation for the project was practical: lifting over
bioinformatics assets for training courses, demonstrations, workshops, and
conference deployments. In those situations you often need more than a single
results folder. You need the workflow context, the expected metadata shape, and
sometimes the associated software artefacts as well.

This documentation is organized around that use case:

* get a quick success path first,
* understand why the archive model exists,
* see how the CLI maps a raw Nextflow run into something EPI2ME Desktop can
  import, and
* learn where the codebase stores those responsibilities.

.. toctree::
   :maxdepth: 2
   :caption: Guide

   quickstart
   why-epi4you
   training-courses
   cli
   archive-format
   architecture
   developer-docs

Indices and tables
==================

* :ref:`genindex`
* :ref:`search`
