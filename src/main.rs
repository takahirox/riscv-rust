extern crate getopts;

mod cpu;
mod display;
mod standalone_display;

use cpu::Cpu;
use cpu::Xlen;
use standalone_display::StandaloneDisplay;

use std::env;
use std::fs::File;
use std::io::Read;

use getopts::Options;

fn print_usage(program: &str, opts: Options) {
	let usage = format!("Usage: {} KERNEL_FILE IMAGE_FILE [options]", program);
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
	if args.len() < 3 {
		print_usage(&program, opts);
		// @TODO: throw error?
		return Ok(());
	}

	let kernel_filename = args[1].clone();
	let mut kernel_file = File::open(kernel_filename)?;
	let mut kernel_contents = vec![];
	kernel_file.read_to_end(&mut kernel_contents)?;

	let image_filename = args[2].clone();
	let mut image_file = File::open(image_filename)?;
	let mut image_contents = vec![];
	image_file.read_to_end(&mut image_contents)?;

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

	let display = Box::new(StandaloneDisplay::new());
	let mut cpu = Cpu::new(xlen, display);
	cpu.init(kernel_contents, image_contents);
	cpu.run();
	Ok(())
}