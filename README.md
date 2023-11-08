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