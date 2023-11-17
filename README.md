# epi4you
Some data wrangling for the EPI2ME desktop environment

## Headline features

The ambition for this application was a toolkit to faciliate the export and import of analyses
that have been run using the EPI2ME desktop application. Why would anyone care about this functionality?
This could be useful to share runs between computers, between users, or to populate a demonstration
computer with exciting and colourful datasets. This section on the `headline` features includes the core
functionality.

### Backup an EPI2ME Desktop run to a .2me format archive

First step - have a look and see which analysis runs are available on your computer. 

```
epi4you epi2me --list # can be used to identify the analyses that have been run
```

Pick the `id` from the run that you would like to archive; then pack the dataset into a `.2me` file.

```
epi4you epi2me --runid 01HBWYY322RMWACRMGX70BMMPB --twome /tmp/wf-clone-validation.2me.tar
```

### Import an EPI2ME Desktop run from a .2me format archive

```
epi4you import --twome /tmp/wf-clone-validation.2me.tar
```

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

### Pull workflow required containers

Using the docker containers as identified in the code above, this code can populate the docker resources with the appropriate
containers. This will create a large amount of internet usage for a few minutes ... The progress is a bit ugly but the
functionality is present for canonical samples

```
epi4you docker --workflow wf-human-variation --pull
```

### Export containers as tar files to defined location

One of the biggest challenges to moving files around offline is getting the files exported as a sensible format - docker provides
native functionality for export and load ...

```
epi4you docker --workflow wf-human-variation --export /tmp
creating object = wf-human-variation.1.8.1.x86_64
exporting [ontresearch/wf-human-variation-str:sha28799bc3058fa256c01c1f07c87f04e4ade1fcc1]
exporting [ontresearch/wf-human-variation-snp:sha0d7e7e8e8207d9d23fdf50a34ceb577da364373e]
...
```

## Nextflow CLI mischief

There is an argument for retrospectively packaging nextflow CLI runs into the GUI - especially if working with users who prefer to
keep away from the wilds of the linux computer.

### List localised nextflow analyses

This is performed in a very similar way to the other functionality presented. A major difference is that `epi4you` needs to be pointed in the right
direction - this requires the `--nxf-work` parameter that should point to a path where bulk analyses have been run.

```
epi4you nextflow --nxf-work /data/CAACB_presentation/nextflow/ --list

nextflow candidate at [/usr/local/bin/nextflow]
Using nxf_bin found at ["/usr/local/bin/nextflow"]
Looking for nxf artifacts at [/data/CAACB_presentation/nextflow/]
┌─────────────────────┬────────────┬───────────────────────────┬────────┬─────────────┬───────────────────────────────────┬───────────────────────────────────┐
│ timestamp           ┆ duration   ┆ run_name                  ┆ status ┆ revision_id ┆ session_id                        ┆ command                           │
╞═════════════════════╪════════════╪═══════════════════════════╪════════╪═════════════╪═══════════════════════════════════╪═══════════════════════════════════╡
│ 2023-10-06 10:54:56 ┆ 2m 34s     ┆ pedantic_stonebraker      ┆ OK     ┆ 9d640fe641  ┆ 153844e1-4cdd-4239-8441-d24b5691… ┆ nextflow run epi2me-labs/wf-meta… │
│ 2023-10-06 10:57:55 ┆ 2m 26s     ┆ big_lamarck               ┆ OK     ┆ 9d640fe641  ┆ bdbb974f-e5f7-43eb-9728-22dd42a1… ┆ nextflow run epi2me-labs/wf-meta… │
│ 2023-10-06 11:02:01 ┆ 2m 1s      ┆ disturbed_kowalevski      ┆ OK     ┆ 9d640fe641  ┆ 45075f3a-a554-48a5-b982-05537e33… ┆ nextflow run epi2me-labs/wf-meta… │
│ 2023-10-06 11:04:34 ┆ 1m 37s     ┆ clever_ampere             ┆ OK     ┆ 9d640fe641  ┆ e817418c-f134-4545-bc01-b91d2099… ┆ nextflow run epi2me-labs/wf-meta… │
│ …                   ┆ …          ┆ …                         ┆ …      ┆ …           ┆ …                                 ┆ …                                 │
│ 2023-10-06 16:04:10 ┆ 48m 42s    ┆ boring_cantor             ┆ OK     ┆ 9d640fe641  ┆ 80db0730-94f3-43df-8ff4-ebfa1fd7… ┆ nextflow run epi2me-labs/wf-meta… │
│ 2023-10-06 16:54:35 ┆ 29m 35s    ┆ amazing_brown             ┆ OK     ┆ 9d640fe641  ┆ 7a88812e-75f2-48cb-8643-1c695f52… ┆ nextflow run epi2me-labs/wf-meta… │
│ 2023-10-06 17:35:59 ┆ 22m 56s    ┆ lethal_booth              ┆ OK     ┆ 9d640fe641  ┆ 2424adad-ed53-480e-a008-b839afa4… ┆ nextflow run epi2me-labs/wf-meta… │
│ 2023-10-06 20:42:29 ┆ 1h 7m 39s  ┆ berserk_sanger            ┆ OK     ┆ 9d640fe641  ┆ ec0c6f3d-acc1-4eb0-839e-5f25c7ab… ┆ nextflow run epi2me-labs/wf-meta… │
└─────────────────────┴────────────┴───────────────────────────┴────────┴─────────────┴───────────────────────────────────┴───────────────────────────────────┘
```