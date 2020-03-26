// Based on Virtual I/O Device (VIRTIO) Version 1.1
// https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html

pub struct VirtioBlockDisk {
	id: u16,
	clock: u64,
	device_features: u64, // read only
	device_features_sel: u32, // write only
	driver_features: u32, // write only
	driver_features_sel: u32, // write only
	guest_page_size: u32, // write only
	queue_select: u32, // write only
	queue_num: u32, // write only
	queue_align: u32, // write only
	queue_pfn: u32, // read and write
	queue_notify: u32, // write only
	interrupt_status: u32, // read only
	status: u32, // read and write
	notify_clock: u64,
	contents: Vec<u8>
}

impl VirtioBlockDisk {
	pub fn new() -> Self {
		VirtioBlockDisk {
			id: 0,
			clock: 0,
			device_features: 0,
			device_features_sel: 0,
			driver_features: 0,
			driver_features_sel: 0,
			guest_page_size: 0,
			queue_select: 0,
			queue_num: 0,
			queue_align: 0,
			queue_pfn: 0,
			queue_notify: 0,
			status: 0,
			interrupt_status: 0,
			notify_clock: 0,
			contents: vec![]
		}
	}

	pub fn is_interrupting(&mut self) -> bool {
		(self.interrupt_status & 0x1) == 1
	}

	pub fn init(&mut self, contents: Vec<u8>) {
		for i in 0..contents.len() {
			self.contents.push(contents[i]);
		}
	}

	pub fn tick(&mut self) {
		// Disk access should be much slower than CPU. To simulate that rising interrupt
		// 500 cpu clocks away from the notification for now. Maybe disk access is further
		// slower in reality but we don't support /request queue yet then
		// we want to finish the first request before next request comes.
		// @TODO: Support request queue and rise interrupt slower
		if self.notify_clock > 0 && self.clock > self.notify_clock + 500 {
			// bit 0 in interrupt_status register indicates
			// the interrupt was asserted because the device has used a buffer
			// in at least one of the active virtual queues.
			self.interrupt_status |= 0x1;
		}
		self.clock = self.clock.wrapping_add(1);
	}

	// Load/Store registers.
	// From 4.2.4 Legacy interface in the specification

	pub fn load(&mut self, address: u64) -> u8 {
		//println!("Disk Load AD:{:X}", address);
		// Legacy virtio Interface
		match address {
			// Magic number: 0x74726976
			0x10001000 => 0x76,
			0x10001001 => 0x69,
			0x10001002 => 0x72,
			0x10001003 => 0x74,
			// Device version: 1 (Legacy device)
			0x10001004 => 1,
			// Virtio Subsystem Device id: 2 (Block device)
			0x10001008 => 2,
			// Virtio Subsystem Vendor id: 0x554d4551
			0x1000100c => 0x51,
			0x1000100d => 0x45,
			0x1000100e => 0x4d,
			0x1000100f => 0x55,
			// Flags representing features the device supports
			0x10001010 => ((self.device_features >> (self.device_features_sel * 32)) & 0xff) as u8,
			0x10001011 => (((self.device_features >> (self.device_features_sel * 32)) >> 8) & 0xff) as u8,
			0x10001012 => (((self.device_features >> (self.device_features_sel * 32)) >> 16) & 0xff) as u8,
			0x10001013 => (((self.device_features >> (self.device_features_sel * 32)) >> 24) & 0xff) as u8,
			// Maximum virtual queue size: 8 so far
			0x10001034 => 8,
			// Guest physical page number of the virtual queue
			0x10001040 => self.queue_pfn as u8,
			0x10001041 => (self.queue_pfn >> 8) as u8,
			0x10001042 => (self.queue_pfn >> 16) as u8,
			0x10001043 => (self.queue_pfn >> 24) as u8,
			// Interrupt status
			0x10001060 => self.interrupt_status as u8,
			0x10001061 => (self.interrupt_status >> 8) as u8,
			0x10001062 => (self.interrupt_status >> 16) as u8,
			0x10001063 => (self.interrupt_status >> 24) as u8,
			// Device status
			0x10001070 => self.status as u8,
			0x10001071 => (self.status >> 8) as u8,
			0x10001072 => (self.status >> 16) as u8,
			0x10001073 => (self.status >> 24) as u8,
			// Configurations @TODO: Implement properly
			0x10001100 => 0x00,
			0x10001101 => 0x20,
			0x10001102 => 0x03,
			0x10001103 => 0,
			0x10001104 => 0,
			0x10001105 => 0,
			0x10001106 => 0,
			0x10001107 => 0,
			_ => 0
		}
	}

	pub fn store(&mut self, address: u64, value: u8) {
		//println!("Disk Store AD:{:X} VAL:{:X}", address, value);
		match address {
			0x10001014 => {
				self.device_features_sel = (self.device_features_sel & !0xff) | (value as u32);
			},
			0x10001015 => {
				self.device_features_sel = (self.device_features_sel & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x10001016 => {
				self.device_features_sel = (self.device_features_sel & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x10001017 => {
				self.device_features_sel = (self.device_features_sel & !(0xff << 24)) | ((value as u32) << 24);
			},
			0x10001020 => {
				self.driver_features = (self.driver_features & !0xff) | (value as u32);
			},
			0x10001021 => {
				self.driver_features = (self.driver_features & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x10001022 => {
				self.driver_features = (self.driver_features & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x10001023 => {
				self.driver_features = (self.driver_features & !(0xff << 24)) | ((value as u32) << 24);
			},
			0x10001028 => {
				self.guest_page_size = (self.guest_page_size & !0xff) | (value as u32);
			},
			0x10001029 => {
				self.guest_page_size = (self.guest_page_size & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x1000102a => {
				self.guest_page_size = (self.guest_page_size & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x1000102b => {
				self.guest_page_size = (self.guest_page_size & !(0xff << 24)) | ((value as u32) << 24);
			},
			0x10001030 => {
				self.queue_select = (self.queue_select & !0xff) | (value as u32);
			},
			0x10001031 => {
				self.queue_select = (self.queue_select & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x10001032 => {
				self.queue_select = (self.queue_select & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x10001033 => {
				self.queue_select = (self.queue_select & !(0xff << 24)) | ((value as u32) << 24);
				if self.queue_select != 0 {
					panic!("Virtio: No multi queue support yet.");
				}
			},
			0x10001038 => {
				self.queue_num = (self.queue_num & !0xff) | (value as u32);
			},
			0x10001039 => {
				self.queue_num = (self.queue_num & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x1000103a => {
				self.queue_num = (self.queue_num & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x1000103b => {
				self.queue_num = (self.queue_num & !(0xff << 24)) | ((value as u32) << 24);
			},
			0x1000103c => {
				self.queue_align = (self.queue_align & !0xff) | (value as u32);
			},
			0x1000103d => {
				self.queue_align = (self.queue_align & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x1000103e => {
				self.queue_align = (self.queue_align & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x1000103f => {
				self.queue_align = (self.queue_align & !(0xff << 24)) | ((value as u32) << 24);
			},
			0x10001040 => {
				self.queue_pfn = (self.queue_pfn & !0xff) | (value as u32);
			},
			0x10001041 => {
				self.queue_pfn = (self.queue_pfn & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x10001042 => {
				self.queue_pfn = (self.queue_pfn & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x10001043 => {
				self.queue_pfn = (self.queue_pfn & !(0xff << 24)) | ((value as u32) << 24);
			},
			// @TODO: Queue request support
			0x10001050 => {
				self.queue_notify = (self.queue_notify & !0xff) | (value as u32);
			},
			0x10001051 => {
				self.queue_notify = (self.queue_notify & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x10001052 => {
				self.queue_notify = (self.queue_notify & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x10001053 => {
				self.queue_notify = (self.queue_notify & !(0xff << 24)) | ((value as u32) << 24);
				if self.notify_clock != 0 || (self.interrupt_status & 0x1) == 1 {
					panic!("Virtio: Overlap notification. Queue request is not supported yet.");
				}
				self.notify_clock = self.clock;
			},
			0x10001064 => {
				// interrupt ack
				if (value & 0x1) == 1 {
					self.interrupt_status &= !0x1;
					self.notify_clock = 0;
				}
			},
			0x10001070 => {
				self.status = (self.status & !0xff) | (value as u32);
			},
			0x10001071 => {
				self.status = (self.status & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x10001072 => {
				self.status = (self.status & !(0xff << 16)) | ((value as u32) << 16);			
			},
			0x10001073 => {
				self.status = (self.status & !(0xff << 24)) | ((value as u32) << 24);
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

	pub fn get_new_id(&mut self) -> u16 {
		self.id = self.id.wrapping_add(1);
		self.id
	}
}
