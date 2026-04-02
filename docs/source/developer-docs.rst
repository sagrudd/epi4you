Developer documentation builds
==============================

``epi4you`` now links Rust compilation and documentation production.

Build behavior
--------------

When you run:

.. code-block:: bash

   cargo build

the package build script will also try to build the Sphinx documentation.

The build order is:

1. try local ``sphinx-build``,
2. if Sphinx is not installed locally, try a container runtime, and
3. use the dedicated AlmaLinux 9 image described by ``docs/Dockerfile.docs``.

Why use a container
-------------------

The docs toolchain is not part of the Rust crate itself. Using a dedicated docs
container keeps the host machine light while still making documentation
production reproducible and easy to trigger from the normal Cargo workflow.

The image is intentionally narrow in scope:

* base image: AlmaLinux 9,
* Python available,
* Sphinx installed from ``docs/requirements.txt``, and
* default command aimed at ``make -C docs html``.

Skipping docs intentionally
---------------------------

If you need a fast Rust-only build or you are in an environment where neither
Sphinx nor a usable container runtime is available, you can skip docs with:

.. code-block:: bash

   EPI4YOU_SKIP_DOCS=1 cargo build

Relevant files
--------------

* ``build.rs``
* ``docs/Dockerfile.docs``
* ``docs/Makefile``
* ``docs/requirements.txt``
