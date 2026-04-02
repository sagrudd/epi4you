Quickstart
==========

This page is the shortest route from source tree to a useful ``.2me`` archive.

What epi4you is for
-------------------

The most important workflow today is:

1. take an existing command-line Nextflow run,
2. package it as a portable ``.2me`` archive, and
3. import that archive into an EPI2ME-style environment elsewhere.

This is especially useful for:

* training courses,
* classroom laptops,
* demo machines,
* conference booths,
* disconnected or low-bandwidth environments, and
* sharing representative analyses with colleagues.

Build the binary
----------------

Clone the repository and build it with Cargo:

.. code-block:: bash

   cargo build

Or run the CLI without a separate install:

.. code-block:: bash

   cargo run -- --help

Package a local Nextflow run
----------------------------

If you already have a directory containing prior Nextflow runs, list them first:

.. code-block:: bash

   cargo run -- nextflow-run \
       --nxf_work /data/nextflow_runs \
       --list

Once you know the ``run_name`` you want, bundle it:

.. code-block:: bash

   cargo run -- nextflow-run \
       --nxf_work /data/nextflow_runs \
       --runid clever_ampere \
       --twome /tmp/clever_ampere.2me.tar

What happens during bundling
---------------------------

The CLI capture flow does more than tar up a folder.

It:

* resolves the original analysis output directory,
* finds the matching ``.nextflow.log`` entry,
* extracts the log lines needed to reconstruct workflow metadata,
* synthesizes helper files such as ``nextflow.stdout`` and ``progress.json``,
* stages output files into an EPI2ME-like layout, and
* writes a manifest-driven ``.2me`` tarball.

Import the archive
------------------

On the target machine, import the bundle:

.. code-block:: bash

   cargo run -- import --twome /tmp/clever_ampere.2me.tar

At import time, ``epi4you`` verifies the embedded manifest before unpacking and
then routes the payload into local EPI2ME-style storage.

Install Sphinx docs locally
---------------------------

``cargo build`` is now wired to generate the Sphinx documentation as part of the
build process. The build tries local ``sphinx-build`` first and then falls back
to a dedicated AlmaLinux 9 container image defined in ``docs/Dockerfile.docs``.

To build the documentation directly yourself:

.. code-block:: bash

   python3 -m venv .venv
   . .venv/bin/activate
   pip install -r docs/requirements.txt
   make -C docs html

The generated HTML will be in ``docs/build/html``.

If you explicitly need to skip documentation generation for one build, set:

.. code-block:: bash

   EPI4YOU_SKIP_DOCS=1 cargo build

What to read next
-----------------

* :doc:`why-epi4you`
* :doc:`training-courses`
* :doc:`cli`
* :doc:`archive-format`
