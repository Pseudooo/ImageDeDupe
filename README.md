
### Overview

A small tool for identifying & grouping similar images within a directory and writing the discovered groupings.

Running this program will scan a directory for all supported image files _(`jpg`, `jpeg`, `png` & `heic`)_ before
using DCT to compare them with each other. Images that are identified as similar will be sorted into groups and written
to the output directory.

Images identified as unique will be written directly to the output directory i.e.
```
/my/output/directory/my_unique_img.png
```

Images that are grouped will be written to a numeric directory like this:
```
/my/output/directory/01/my_img_1.png
/my/output/directory/01/my_img_2.png
/my/output/directory/01/my_img_3.png
```

### Usage

The command arguments can be seen with the `--help` option:
```
> .\ImageDeDupe.exe --help
Usage: ImageDeDupe.exe [OPTIONS]

Options:
  -t, --target <TARGET>        The target directory to read images from [default: ./]
  -o, --output <OUTPUT>        The output directory to write groupings too [default: ./output]
  -a, --algorithm <ALGORITHM>  The hash algorithm to use [default: pHash] [possible values: dHash, pHash]
  -d, --distance <DISTANCE>    The hamming distance between two image hashes to be identified as duplicates [default: 8]
  -h, --help                   Print help
  -V, --version                Print version
```
