# secret-store-cli

This is SecretStore CLI tool

## architecture

If you are not familliar with how SecretStore works, you can read more details from [Parity's documentation](https://wiki.parity.io/Secret-Store)

## dependency

Here are the local tested versions :

* OS : macos Monterey 12.0.1
* cargo 1.36.0 (c4fcfb725 2019-05-15)
* rustc 1.36.0 (a53f9df32 2019-07-03)

## install

```bash
$ ./build-darwin-universal.sh
$ ./target/release/secret_store_cli.bundle

secret-store-cli

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    address    Get eth address list
    decrypt    Decrypt document
    encrypt    Encrypt document and receive document key ID and IPFS hash
    help       Prints this message or the help of the given subcommand(s)
```
