# epi4you
Some data wrangling for the EPI2ME desktop environment

## Simple manipulation of the EPI2ME Desktop Application's database entries

### List workflow instances

```
epi4you database --list

Listing databases
┌────────────────────────────┬─────────────────────┬─────────────────────┬────────────────────────────────┬───────────┐
│ id                         ┆ name                ┆ workflowRepo        ┆ createdAt                      ┆ status    │
╞════════════════════════════╪═════════════════════╪═════════════════════╪════════════════════════════════╪═══════════╡
│ 01HBVK5GGYB5HENP4R14YPJQHM ┆ flamboyant_nicholls ┆ wf-human-variation  ┆ 2023-10-03 20:20:13.470 +00:00 ┆ COMPLETED │
│ 01HBWF2C13DQPJ3V33S7FHM5G3 ┆ unruffled_liskov    ┆ wf-human-variation  ┆ 2023-10-04 04:27:50.692 +00:00 ┆ COMPLETED │
│ 01HBWYBQ6EQZ7199ZRY1NZESXJ ┆ boring_wright       ┆ wf-metagenomics     ┆ 2023-10-04 08:55:05.680 +00:00 ┆ COMPLETED │
└────────────────────────────┴─────────────────────┴─────────────────────┴────────────────────────────────┴───────────┘
```

### Update workflow instance run status

There are times when computers, workflows, and data do not behave. For users with the anxiety of having either incomplete
or broken analyses in their run folder, exit states can be updated. To correct e.g. a workflow with an `ERROR` status
we can mark this as `COMPLETED` with

```
epi4you database --runid suspicious_khorana --status ERROR
```

### Rename a workflow instance

While it is possible to rename a workflow instance through the GUI, sometimes things are just cleaner through the command-line.
Using the `--rename` option it is possible to quickly rename an existing workflow based on either the workflow ID or name; please
note that a restart of the desktop application is required for the changes to be seen.

```
epi4you database --runid 01HEQSF512CK2VK4YY07CEY2B9 --rename ARTIC_DEMO
```

### Delete an unsuccessful or redundant analysis

As with the `--rename` functionality presented above, it is simple to delete workflows from the GUI; it is perhaps simpler to
run from the command line. This will remove the corresponding entry from the database and will remove the linked folder and its
files.

```
epi4you database --runid 01HEQR5KECW3KENBY9JFKEKTYE --delete
```

### Workflow instance housekeeping

Nextflow places a collection of intermediate files in its working directory (in a subfolder named `work` in each
instance). These intermediate files are kept by EPI2ME (intentionally) whilst the result files are copied into a separate folder. 
While there may some useful files within the `work` intermediate folder, a significant amount of disk space can be freed by
removing these folders - this may be hundreds of Gb for `wf-human-variation` analyses. A `housekeeping` exercise will remove
these intermediate files for analysis instances that have completed successfully or been stopped by a user.

```
epi4you database --housekeeping
```

### Duplicating existing analysis runs

There are times when you'd like to be creative with existing datasets. This is often centred around customer demonstrations or
the need to package existing command line runs in the GUI. There is a `--clone` parameter that can be used to clone existing
workflows; this just copies one analysis into a new folder, updates the metadata and adds the corresponding entry to the
database.

```
epi4you database --runid 01HESF8SQ43RT9MVEFARS3SW14 --clone cloned_workflow
```


## Docker containers and EPI2ME

There is a lot of interest in tools that can be used to facilitate the offline analysis of biological data. At the heart of the
software is a massive collection of prepackaged data that includes a variety of containers that are supported for both docker
and singularity. These software containers have the challenge that they are large, frequently updated, and are only available
through internet connected resources. The `docker` tools provided here are intended to simplify the identification of docker
containers associated with individual projects. These containers can then be installed and further packaged as offline
accessible artifacts. 

### List workflow required containers

It is trivial to list the container(s) required by an installed workflow revision. Please note that the example below is
both incomplete - and versions will evolve and adapt quickly - this output was correct in Nov'23.

```
epi4you docker --workflow wf-human-variation --list
...
ontresearch/wf-human-variation-snp:sha0d7e7e8e8207d9d23fdf50a34ceb577da364373e
ontresearch/snpeff:sha4f289afaf754c7a3e0b9ffb6c0b5be0f89a5cf04
nanoporetech/dorado:sha1433bfc3146fd0dc94ad9648452364f2327cf1b0
ontresearch/wf-cnv:sha428cb19e51370020ccf29ec2af4eead44c6a17c2
...
```