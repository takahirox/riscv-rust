pub struct VirtioBlockDisk {
	id: u8,
	clock: u64,
	driver_features: u32,
	guest_page_size: u32,
	queue_select: u32,
	queue_num: u32,
	queue_pfn: u32,
	queue_notify: u32,
	status: u32,
	notify_clock: u64,
	interrupting: bool,
	contents: Vec<u8>
}

impl VirtioBlockDisk {
	pub fn new() -> Self {
		VirtioBlockDisk {
			id: 0,
			clock: 0,
			driver_features: 0,
			guest_page_size: 0,
			queue_select: 0,
			queue_num: 0,
			queue_pfn: 0,
			queue_notify: 0,
			status: 0,
			notify_clock: 0,
			interrupting: false,
			contents: vec![]
		}
	}

	pub fn is_interrupting(&mut self) -> bool {
		self.interrupting
	}

	// @TODO: Rename
	pub fn reset_interrupting(&mut self) {
		self.interrupting = false;
		self.notify_clock = 0;
	}

	pub fn init(&mut self, contents: Vec<u8>) {
		for i in 0..contents.len() {
			self.contents.push(contents[i]);
		}
	}

	pub fn tick(&mut self) {
		if self.notify_clock > 0 && self.clock > self.notify_clock + 500 {
			self.interrupting = true;
		}
		self.clock = self.clock.wrapping_add(1);
	}

	pub fn load(&self, address: u64) -> u8 {
		match address {
			0x10001000 => 0x76, // vertio disk magic value: 0x74726976
			0x10001001 => 0x69,
			0x10001002 => 0x72,
			0x10001003 => 0x74,
			0x10001004 => 1, // vertio version: 1
			0x10001008 => 2, // vertio device id: 2
			0x1000100c => 0x51, // vertio vendor id: 0x554d4551
			0x1000100d => 0x45,
			0x1000100e => 0x4d,
			0x1000100f => 0x55,
			0x10001034 => 8, // vertio  queue num max: At least 8
			_ => 0
		}
	}
	
	pub fn store(&mut self, address: u64, value: u8) {
		match address {
			0x10001020 => {
				self.driver_features = (self.driver_features & !0xff) | (value as u32);
			},
			0x10001021 => {
				self.driver_features = (self.driver_features & !0xff00) | ((value as u32) << 8);
			},
			0x10001022 => {
				self.driver_features = (self.driver_features & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001023 => {
				self.driver_features = (self.driver_features & !0xff000000) | ((value as u32) << 24);
			},
			0x10001028 => {
				self.guest_page_size = (self.guest_page_size & !0xff) | (value as u32);
			},
			0x10001029 => {
				self.guest_page_size = (self.guest_page_size & !0xff00) | ((value as u32) << 8);
			},
			0x1000102a => {
				self.guest_page_size = (self.guest_page_size & !0xff0000) | ((value as u32) << 16);			
			},
			0x1000102b => {
				self.guest_page_size = (self.guest_page_size & !0xff000000) | ((value as u32) << 24);
			},
			0x10001030 => {
				self.queue_select = (self.queue_select & !0xff) | (value as u32);
			},
			0x10001031 => {
				self.queue_select = (self.queue_select & !0xff00) | ((value as u32) << 8);
			},
			0x10001032 => {
				self.queue_select = (self.queue_select & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001033 => {
				self.queue_select = (self.queue_select & !0xff000000) | ((value as u32) << 24);
			},
			0x10001038 => {
				self.queue_num = (self.queue_num & !0xff) | (value as u32);
			},
			0x10001039 => {
				self.queue_num = (self.queue_num & !0xff00) | ((value as u32) << 8);
			},
			0x1000103a => {
				self.queue_num = (self.queue_num & !0xff0000) | ((value as u32) << 16);			
			},
			0x1000103b => {
				self.queue_num = (self.queue_num & !0xff000000) | ((value as u32) << 24);
			},
			0x10001040 => {
				self.queue_pfn = (self.queue_pfn & !0xff) | (value as u32);
			},
			0x10001041 => {
				self.queue_pfn = (self.queue_pfn & !0xff00) | ((value as u32) << 8);
			},
			0x10001042 => {
				self.queue_pfn = (self.queue_pfn & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001043 => {
				self.queue_pfn = (self.queue_pfn & !0xff000000) | ((value as u32) << 24);
			},
			0x10001050 => {
				self.queue_notify = (self.queue_notify & !0xff) | (value as u32);
			},
			0x10001051 => {
				self.queue_notify = (self.queue_notify & !0xff00) | ((value as u32) << 8);
			},
			0x10001052 => {
				self.queue_notify = (self.queue_notify & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001053 => {
				self.queue_notify = (self.queue_notify & !0xff000000) | ((value as u32) << 24);
				self.notify_clock = self.clock;
			},
			0x10001070 => {
				self.status = (self.status & !0xff) | (value as u32);
			},
			0x10001071 => {
				self.status = (self.status & !0xff00) | ((value as u32) << 8);
			},
			0x10001072 => {
				self.status = (self.status & !0xff0000) | ((value as u32) << 16);			
			},
			0x10001073 => {
				self.status = (self.status & !0xff000000) | ((value as u32) << 24);
			},
			_ => {}
		};
	}

	pub fn get_page_address(&self) -> u64 {
		self.queue_pfn as u64 * self.guest_page_size as u64
	}

	// desc = pages -- num * VRingDesc
	// avail = pages + 0x40 -- 2 * uint16, then num * uint16
	// used = pages + 4096 -- 2 * uint16, then num * vRingUsedElem
	
	pub fn get_desc_address(&self) -> u64 {
		self.get_page_address()
	}

	pub fn get_avail_address(&self) -> u64 {
		self.get_page_address() + 0x40
	}

	pub fn get_used_address(&self) -> u64 {
		self.get_page_address() + 4096
	}

	pub fn read_from_disk(&mut self, address: u64) -> u8 {
		self.contents[address as usize]
	}
	
	pub fn write_to_disk(&mut self, address: u64, value: u8) {
		self.contents[address as usize] = value
	}

	pub fn get_new_id(&mut self) -> u8 {
		self.id += 1;
		self.id
	}
}
