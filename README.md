# filmilhoes
Helper application to pick a file

```
USAGE:
    filmilhoes.exe [OPTIONS] <DIRECTORY>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -x, --exclude <exclude>...    Directories to exclude
    -s, --size <min_size>         Minimum size of files to pick (i.e. 100MiB)
    -n, --number <n>              Set max number of files to pick

ARGS:
    <DIRECTORY>    Directory to analyse
```

The application also loads a `.filmilhoes.yml` YAML file if it exists in the directory to analyze.
The OPTIONS set, if any, override the ones in the YAML file, except the `exclude` option. In case of the `exclude`, it's the union of both options.

Example:
```yaml
files: 5
minsize: 500MiB
exclude:
  - animes
  - work
  - tv series
```

If no options are provided. It picks 1 random file of any size.
