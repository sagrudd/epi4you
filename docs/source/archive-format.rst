The .2me archive model
======================

The central transport artifact in ``epi4you`` is a ``.2me`` tarball.

This is the unit that lets a prepared analysis move between machines while
preserving enough structure to be meaningful on the target side.

What the archive contains
-------------------------

At a high level, a ``.2me`` archive contains:

* a manifest file named ``4u_manifest.json``,
* one or more typed payloads,
* file inventory information,
* provenance, and
* a signature over the manifest payload.

Manifest responsibilities
-------------------------

The manifest serves three different roles at once.

Inventory
   It describes which payloads are present and which files belong to them.

Context
   It records the metadata needed to interpret the archive as an EPI2ME-style
   asset rather than as an arbitrary tar file.

Integrity
   It stores a digest-based signature for the serialized manifest so import can
   reject obviously inconsistent archives.

Payload types
-------------

The codebase models three payload classes:

``Epi2mePayload``
   A Desktop-style analysis record with associated files.

``Epi2meWf``
   A workflow installation tree.

``Epi2meContainer``
   A set of exported container artefacts associated with a workflow.

Why manifests matter for liftover
---------------------------------

For training-course and offline deployment use, a manifest is not busywork.
It is what turns a file drop into a transport object with intent.

Without the manifest, the target side has to guess:

* what this bundle represents,
* whether it is complete,
* where files belong,
* how to label the imported analysis, and
* whether the archive still matches what the source side produced.

With the manifest, those questions are answered explicitly.

Current integrity model
-----------------------

The current trust check is manifest-centric:

* the manifest is serialized in a canonical form,
* the signature field is blanked during signature generation, and
* a SHA-256 digest is stored and checked during import.

This is intentionally lightweight. It is enough to detect obvious tampering or
serialization mismatch at the manifest layer, even though it is not yet a full
cryptographic attestation of every unpacked file.
