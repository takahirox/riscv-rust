extern crate getopts;

mod application;
mod cpu;
mod mmu;
mod plic;
mod clint;
mod uart;
mod virtio_block_disk;
mod terminal;
mod dummy_terminal;
mod popup_terminal;

use cpu::Xlen;
use terminal::Terminal;
use popup_terminal::PopupTerminal;
use dummy_terminal::DummyTerminal;
use application::Application;

use std::env;
use std::fs::File;
use std::io::Read;

use getopts::Options;

enum TerminalType {
	PopupTerminal,
	DummyTerminal
}

fn print_usage(program: &str, opts: Options) {
	let usage = format!("Usage: {} program_file [options]", program);
	print!("{}", opts.usage(&usage));
}

fn get_terminal(terminal_type: TerminalType) -> Box<dyn Terminal> {
	match terminal_type {
		TerminalType::PopupTerminal => Box::new(PopupTerminal::new()),
		TerminalType::DummyTerminal => Box::new(DummyTerminal::new()),
	}
}

fn main () -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();

	let mut opts = Options::new();
	opts.optopt("x", "xlen", "Set bit mode. Default is auto detect from elf file", "32|64");
	opts.optopt("f", "fs", "File system image file", "xv6/fs.img");
	opts.optopt("d", "dtb", "Device tree file", "linux/dtb");
	opts.optflag("n", "no_terminal", "No popup terminal");
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

	let fs_contents = match matches.opt_str("f") {
		Some(path) => {
			let mut file = File::open(path)?;
			let mut contents = vec![];
			file.read_to_end(&mut contents)?;
			contents
		}
		None => vec![]
	};

	let dtb_contents = match matches.opt_str("d") {
		Some(path) => {
			let mut file = File::open(path)?;
			let mut contents = vec![];
			file.read_to_end(&mut contents)?;
			contents
		}
		None => vec![]
	};

	let elf_filename = args[1].clone();
	let mut elf_file = File::open(elf_filename)?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;

	let terminal_type = match matches.opt_present("n") {
		true => {
			println!("No popup terminal mode. Output will be flushed on your terminal but you can not input.");
			TerminalType::DummyTerminal
		},
		false => TerminalType::PopupTerminal
	};

	let mut application = Application::new(get_terminal(terminal_type));
	application.setup_from_elf(elf_contents);
	
	match matches.opt_str("x") {
		Some(x) => match x.as_str() {
			"32" => {
				println!("Force to 32-bit mode.");
				application.update_xlen(Xlen::Bit32);
			},
			"64" => {
				println!("Force to 64-bit mode.");
				application.update_xlen(Xlen::Bit64);
			},
			_ => {
				print_usage(&program, opts);
				// @TODO: throw error?
				return Ok(());
			}
		},
		None => {}
	};

	application.setup_filesystem(fs_contents);
	application.setup_dtb(dtb_contents);
	application.run();
	Ok(())
}
