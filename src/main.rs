extern crate getopts;

mod cpu;
use cpu::Cpu;
use cpu::Xlen;

use std::env;
use std::fs::File;
use std::io::Read;

use getopts::Options;

fn print_usage(program: &str, opts: Options) {
	let usage = format!("Usage: {} FILE [options]", program);
	print!("{}", opts.usage(&usage));
}

fn main () -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optopt("x", "xlen", "Set bit mode. Default is 32", "32|64");
	opts.optflag("h", "help", "Show this help menu");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => {
			println!("{}", f.to_string());
			print_usage(&program, opts);
			// @TODO: throw error?
			return Ok(());
		}
	};
	if matches.opt_present("h") {
		print_usage(&program, opts);
		return Ok(());
	}
	if args.len() < 2 {
		print_usage(&program, opts);
		// @TODO: throw error?
		return Ok(());
	}

	let filename = args[1].clone();
	let mut file = File::open(filename)?;
	let mut contents = vec![];
	file.read_to_end(&mut contents)?;

	let xlen = match matches.opt_str("x") {
		Some(x) => match x.as_str() {
			"32" => Xlen::Bit32,
			"64" => Xlen::Bit64,
			_ => {
				print_usage(&program, opts);
				// @TODO: throw error?
				return Ok(());
			}
		},
		None => Xlen::Bit32
	};

	let mut cpu = Cpu::new(xlen);
	cpu.run_test(contents);
	Ok(())
}