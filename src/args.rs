use ethereum_types::H256;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "secret-store-cli", about = "secret-store sample")]
pub enum Args {
    #[structopt(name = "decrypt", about = "Decrypt document")]
    Decrypt {
        #[structopt(help = "Pass a dockey_id", required = true)]
        dockey_id: H256,

        #[structopt(help = "Pass a document hash you want to decrypt", required = true)]
        ipfs_uri: String,
    },
    #[structopt(
        name = "encrypt",
        about = "Encrypt document and receive document key ID and IPFS hash"
    )]
    Encrypt {
        #[structopt(
            help = "Pass a document file path you want to encrypt",
            required = true
        )]
        file: PathBuf,
    },
    // #[structopt(name="vhalist", about="Get VHA List you've purchased")]
    // VhaList,
    #[structopt(name = "address", about = "Get eth address list")]
    Address,
}

pub fn parse() -> Args {
    Args::from_args()
}
