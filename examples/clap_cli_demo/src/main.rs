use cli_infra::{AppArgs, Parser};

fn main() {
	let args = AppArgs::parse();
	println!("{:?}", args.commit);
}
