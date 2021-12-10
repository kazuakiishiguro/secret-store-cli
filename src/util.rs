use std::{path::PathBuf, fs::File};
use std::io::{self, BufReader, Read, Write};

use ethereum_types::{H160, H256};
use ethkey::Password;
use ring::digest::{Context, Digest, SHA256};

pub fn password_prompt() -> Result<Password, String> {
	use rpassword::read_password;
	const STDIN_ERROR: &'static str = "Unable to ask for password on non-interactive terminal.";
	println!("Please note that password is NOT RECOVERABLE.");
	print!("Type password: ");
	flush_stdout();

	let password = read_password().map_err(|_| STDIN_ERROR.to_owned())?.into();

	print!("Repeat password: ");
	flush_stdout();

	let password_repeat = read_password().map_err(|_| STDIN_ERROR.to_owned())?.into();

	if password != password_repeat {
		return Err("Passwords do not match!".into());
	}
	Ok(password)
}

pub fn accounts_prompt(accounts: Vec<H160>) -> u32 {
	let mut input = String::new();
	println!("Account lists: \n");
	for i in 0..accounts.len() {
		println!("{}: {:?}", i, accounts[0]);
	}
	println!("\n");
	loop {
		println!("Select an account: ");
		match io::stdin().read_line(&mut input) {
			Ok(_) => {
				let input_u32: u32 = match input.trim().parse() {
					Ok(n) => n,
					Err(_) => continue,
				};
				// something wrokng on break statement when you pick non-number
				break
				input_u32
			}
			Err(error) => println!("error: {}", error),
		}
	}
}

pub fn create_dockey_id(file: PathBuf) -> io::Result<H256> {
	let f = File::open(file)?;
	let digest = sha256_digest(BufReader::new(f))?;
	let message = H256::from_slice(digest.as_ref());
	Ok(message)
}

fn flush_stdout() {
	io::stdout().flush().expect("stdout is flushable; qed");
}

fn sha256_digest<R: Read>(mut reader: R) -> io::Result<Digest> {
	let mut context = Context::new(&SHA256);
	let mut buffer = [0; 1024];
	loop {
		let count = reader.read(&mut buffer)?;
		if count == 0 {
			break;
		}
		context.update(&buffer[..count]);
	}
	Ok(context.finish())
}