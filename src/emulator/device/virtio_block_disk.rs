use emulator::memory::Memory;

// Based on Virtual I/O Device (VIRTIO) Version 1.1
// https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html

// 0x2000 is an arbitary number.
const MAX_QUEUE_SIZE: u64 = 0x2000;

// To simulate disk access time.
// @TODO: Set more proper number. 500 core clocks may be too short.
const DISK_ACCESS_DELAY: u64 = 500;

const VIRTQ_DESC_F_NEXT: u64 = 1;

// 0: buffer is write-only = read from disk operation
// 1: buffer is read-only = write to disk operation
const VIRTQ_DESC_F_WRITE: u64 = 2;

const SECTOR_SIZE: u64 = 512;

pub struct VirtioBlockDisk {
	used_ring_index: u16,
	clock: u64,
	device_features: u64, // read only
	device_features_sel: u32, // write only
	driver_features: u32, // write only
	_driver_features_sel: u32, // write only
	guest_page_size: u32, // write only
	queue_select: u32, // write only
	queue_size: u32, // write only
	queue_align: u32, // write only
	queue_pfn: u32, // read and write
	queue_notify: u32, // write only
	interrupt_status: u32, // read only
	status: u32, // read and write
	notify_clocks: Vec::<u64>,
	contents: Vec<u8>
}

impl VirtioBlockDisk {
	pub fn new() -> Self {
		VirtioBlockDisk {
			used_ring_index: 0,
			clock: 0,
			device_features: 0,
			device_features_sel: 0,
			driver_features: 0,
			_driver_features_sel: 0,
			guest_page_size: 0,
			queue_select: 0,
			queue_size: 0,
			queue_align: 0x1000, // xv6 seems to expect this default value
			queue_pfn: 0,
			queue_notify: 0,
			status: 0,
			interrupt_status: 0,
			notify_clocks: Vec::new(),
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

	pub fn tick(&mut self, memory: &mut Memory) {
		if self.notify_clocks.len() > 0 && (self.clock == self.notify_clocks[0] + DISK_ACCESS_DELAY) {
			// bit 0 in interrupt_status register indicates
			// the interrupt was asserted because the device has used a buffer
			// in at least one of the active virtual queues.
			self.interrupt_status |= 0x1;
			self.handle_disk_access(memory);
			self.notify_clocks.remove(0);
		}
		self.clock = self.clock.wrapping_add(1);
	}

	// Load/Store registers.
	// From 4.2.4 Legacy interface in the specification

	pub fn load(&mut self, address: u64) -> u8 {
		//println!("Disk Load AD:{:X}", address);
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
			// Maximum virtual queue size
			0x10001034 => MAX_QUEUE_SIZE as u8,
			0x10001035 => (MAX_QUEUE_SIZE >> 8) as u8,
			0x10001036 => (MAX_QUEUE_SIZE >> 16) as u8,
			0x10001037 => (MAX_QUEUE_SIZE >> 24) as u8,
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
				self.queue_size = (self.queue_size & !0xff) | (value as u32);
			},
			0x10001039 => {
				self.queue_size = (self.queue_size & !(0xff << 8)) | ((value as u32) << 8);
			},
			0x1000103a => {
				self.queue_size = (self.queue_size & !(0xff << 16)) | ((value as u32) << 16);
			},
			0x1000103b => {
				self.queue_size = (self.queue_size & !(0xff << 24)) | ((value as u32) << 24);
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
				self.notify_clocks.push(self.clock);
			},
			0x10001064 => {
				// interrupt ack
				if (value & 0x1) == 1 {
					self.interrupt_status &= !0x1;
				} else {
					panic!("Unknown ack {:X}", value);
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

	fn read_from_disk(&mut self, address: u64) -> u8 {
		self.contents[address as usize]
	}

	fn write_to_disk(&mut self, address: u64, value: u8) {
		self.contents[address as usize] = value
	}

	fn get_page_address(&self) -> u64 {
		self.queue_pfn as u64 * self.guest_page_size as u64
	}

	// Virtqueue layout: Starting at page address
	//
	// struct virtq {
	//   struct virtq_desc desc[queue_size]; // queue_size * 16bytes
	//   struct virtq_avail avail;           // 2 * 2bytes + queue_size * 2bytes
	//   uint8 pad[padding];                 // until queue_align
	//   struct virtq_used used;             // 2 * 2bytes + queue_size * 8bytes
	// }
	//
	// struct virtq_desc {
	//   uint64 addr;
	//   uint32 len;
	//   uint16 flags;
	//   uint16 next;
	// }
	//
	// struct virtq_avail {
	//   uint16 flags;
	//   uint16 idx;
	//   uint16 ring[queue_size];
	// }
	//
	// struct virtq_used {
	//   uint16 flags;
	//   uint16 idx;
	//   struct virtq_used_elem ring[queue_size];
	// }
	//
	// struct virtq_used_elem {
	//   uint32 id;
	//   uint32 len;
	// }

	fn get_base_desc_address(&self) -> u64 {
		self.get_page_address()
	}

	fn get_base_avail_address(&self) -> u64 {
		self.get_base_desc_address() + self.queue_size as u64 * 16
	}

	fn get_base_used_address(&self) -> u64 {
		let align = self.queue_align as u64;
		let queue_size = self.queue_size as u64;
		((self.get_base_avail_address() + 4 + queue_size * 2 + align - 1) / align) * align
	}

	// @TODO: Follow the virtio block specification more propertly.
	fn handle_disk_access(&mut self, memory: &mut Memory) {
		let base_desc_address = self.get_base_desc_address();
		let base_avail_address = self.get_base_avail_address();
		let base_used_address = self.get_base_used_address();
		let queue_size = self.queue_size as u64;

		let _avail_flag = memory.read_bytes(base_avail_address, 2);
		let _avail_index = memory.read_bytes(base_avail_address.wrapping_add(2), 2) % queue_size;
		let desc_index_address = base_avail_address.wrapping_add(4).wrapping_add(self.used_ring_index as u64 * 2);
		let desc_head_index = memory.read_bytes(desc_index_address, 2) % queue_size;

		/*
		println!("Desc AD:{:X}", base_desc_address);
		println!("Avail AD:{:X}", base_avail_address);
		println!("Used AD:{:X}", base_used_address);
		println!("Avail flag:{:X}", _avail_flag);
		println!("Avail index:{:X}", _avail_index);
		println!("Used ring index:{:X}", self.used_ring_index);
		println!("Desc head index:{:X}", desc_head_index);
		*/

		let mut _blk_type = 0;
		let mut _blk_reserved = 0;
		let mut blk_sector = 0;
		let mut desc_num = 0;
		let mut desc_next = desc_head_index;
		loop {
			let desc_element_address = base_desc_address + 16 * desc_next;
			let desc_addr = memory.read_bytes(desc_element_address, 8);
			let desc_len = memory.read_bytes(desc_element_address.wrapping_add(8), 4);
			let desc_flags = memory.read_bytes(desc_element_address.wrapping_add(12), 2);
			desc_next = memory.read_bytes(desc_element_address.wrapping_add(14), 2) % queue_size;

			/*
			println!("Desc addr:{:X}", desc_addr);
			println!("Desc len:{:X}", desc_len);
			println!("Desc flags:{:X}", desc_flags);
			println!("Desc next:{:X}", desc_next);
			*/

			match desc_num {
				0 => {
					// First descriptor: Block description
					// struct virtio_blk_req {
					//   uint32 type;
					//   uint32 reserved;
					//   uint64 sector;
					// }

					// Read/Write operation can be distinguished with the second descriptor flags
					// so we can ignore blk_type?
					_blk_type = memory.read_bytes(desc_addr, 4);
					_blk_reserved = memory.read_bytes(desc_addr.wrapping_add(4), 4);
					blk_sector = memory.read_bytes(desc_addr.wrapping_add(8), 8);
					/*
					println!("Blk type:{:X}", _blk_type);
					println!("Blk reserved:{:X}", _blk_reserved);
					println!("Blk sector:{:X}", blk_sector);
					*/
				},
				1 => {
					// Second descriptor: Read/Write disk
					match (desc_flags & VIRTQ_DESC_F_WRITE) == 0 {
						true => { // write to disk
							for i in 0..desc_len as u64 {
								let data = memory.read_byte(desc_addr + i);
								self.write_to_disk(blk_sector * SECTOR_SIZE + i, data);
							}
						},
						false => { // read from disk
							for i in 0..desc_len as u64 {
								let data = self.read_from_disk(blk_sector * SECTOR_SIZE + i);
								memory.write_byte(desc_addr + i, data);
							}
						}
					};
				},
				2 => {
					// Third descriptor: Result status
					if (desc_flags & VIRTQ_DESC_F_WRITE) == 0 {
						panic!("Third descriptor should be write.");
					}
					if desc_len != 1 {
						panic!("Third descriptor length should be one.");
					}
					memory.write_byte(desc_addr, 0); // 0 means succeeded
				},
				_ => {}
			};

			desc_num += 1;

			if (desc_flags & VIRTQ_DESC_F_NEXT) == 0 {
				break;
			}
		}

		if desc_num != 3 {
			panic!("Descript chain length should be three.");
		}

		memory.write_bytes(base_used_address.wrapping_add(4).wrapping_add(self.used_ring_index as u64 * 8), desc_head_index, 4);

		self.used_ring_index = self.used_ring_index.wrapping_add(1) % self.queue_size as u16;
		memory.write_bytes(base_used_address.wrapping_add(2), self.used_ring_index as u64, 2);
	}
}
