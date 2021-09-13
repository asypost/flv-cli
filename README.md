# flv-cli
simple flv cli tool

## show metadata info

```
>flv-cli.exe help info
```
```
Show flv file metadata

USAGE:
    flv-cli.exe [FILE] info

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
```

## extract
```
>flv-cli.exe help extract
```
```
Extract video or audio from flv file

USAGE:
    flv-cli.exe [FILE] extract --out <output> --type <type>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --out <output>    output path,- for stdout
    -t, --type <type>     audio,video or all
```
