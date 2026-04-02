CLI reference
=============

The current top-level CLI emphasizes two active flows:

* package a local CLI Nextflow run as a portable archive, and
* import a previously built archive.

Package a CLI Nextflow run
--------------------------

Subcommand:

.. code-block:: text

   epi4you nextflow-run

List candidate runs:

.. code-block:: bash

   epi4you nextflow-run --nxf_work /data/nextflow_runs --list

Bundle one run:

.. code-block:: bash

   epi4you nextflow-run \
       --nxf_work /data/nextflow_runs \
       --runid clever_ampere \
       --twome /tmp/clever_ampere.2me.tar

Relevant options:

``--nxf_work``
   Path to the directory containing the historical Nextflow run context.

``--nxf_bin``
   Optional explicit path to the ``nextflow`` executable. If omitted,
   ``epi4you`` falls back to ``which nextflow``.

``--list``
   Lists successful runs parsed from ``nextflow log``.

``--runid``
   The Nextflow ``run_name`` to package.

``--twome``
   Destination path for the generated ``.2me`` archive.

``--force``
   Allows overwriting an existing destination archive.

Import an archive
-----------------

Subcommand:

.. code-block:: text

   epi4you import

Import example:

.. code-block:: bash

   epi4you import --twome /tmp/clever_ampere.2me.tar

Relevant options:

``--twome``
   Path to the archive being imported.

``--force``
   Reserved for higher-level overwrite semantics in the import flow.

Notes on older capabilities
---------------------------

The repository still contains code for a broader asset-management vision:

* workflow packaging,
* Docker/container capture,
* database operations, and
* legacy Desktop-oriented archive flows.

Those areas help explain the structure of the codebase and manifest model, even
though the current top-level CLI is centered on ``nextflow-run`` and ``import``.
