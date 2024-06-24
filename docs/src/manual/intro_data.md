# Managing Data

Kerblam! has a bunch of utilities to help you manage the local data for your
project.
If you follow open science guidelines, chances are that a lot of your data is
FAIR, and you can fetch it remotely.

Kerblam! is perfect to work with such data. The next tutorial sections outline what
Kerblam! can do to help you work with data.

Remember that Kerblam! recognizes what data is what by the location where you 
save the data in.
If you need a refresher, read [this section of the book](../quickstart.html).

`kerblam data` will give you an overview of the status of local data:
```
> kerblam data
./data       500 KiB [2]
└── in       1.2 MiB [8]
└── out      823 KiB [2]
──────────────────────
Total        2.5 Mib [12]
└── cleanup  2.3 Mib [9] (92.0%)
└── remote   1.0 Mib [5]
! There are 3 undownloaded files.   
```
The first lines highlight the size (`500 KiB`) and amount (`2`) of files in the
`./data/in` (input), `./data/out` (output) and `./data` (intermediate) folders.

The total size of all the files in the `./data/` folder is then broken down
between categories: the `Total` data size, how much data can be removed with
`kerblam data clean` or `kerblam data pack`, and how many files are specified
to be downloaded but are not yet present locally.
