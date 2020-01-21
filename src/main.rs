mod cpu;
use cpu::Cpu;

use std::env;
use std::fs::File;
use std::io::Read;

fn main () -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();

	if args.len() < 2 {
		// @TODO: throw error
		return Ok(());
	}

	let filename = &args[1];
	let mut file = File::open(filename)?;
	let mut contents = vec![];
	file.read_to_end(&mut contents)?;

	let mut cpu = Cpu::new();
	cpu.run_test(contents);
	Ok(())
}