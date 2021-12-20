extern crate secret_store_cli;

use secret_store_cli::{args, args::*, cmd};

fn main() {
    let args = args::parse();
    match args {
        Args::Decrypt {
            dockey_id,
            ipfs_uri,
        } => {
            cmd::decrypt(dockey_id, ipfs_uri);
        }
        Args::Encrypt { file } => {
            cmd::encrypt(file);
        }
        Args::Address => {
            cmd::address();
        }
    }
}
