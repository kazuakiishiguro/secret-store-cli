# secret-store-cli

This is SecretStore CLI tool

## architecture

If you are not familliar with how SecretStore works, you can read more details at [Open Ethereum's documentation](https://openethereum.github.io/Secret-Store)

## dependency

Here are the local tested versions :

* Ubuntu : Ubuntu 20.04.3 LTS
* OS : macos Monterey 12.0.1
* cargo 1.56.0 (4ed5d137b 2021-10-04)

## build

### for linux

```bash
$ cargo build --resease
$ ./target/resease/secret-store-cli
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

### for OSX

```bash
$ ./scripts/build-darwin-universal.sh
$ ./target/release/secret-store-cli.bundle
```
