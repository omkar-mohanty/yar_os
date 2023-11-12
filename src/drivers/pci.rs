use alloc::vec::Vec;
use core::ops::Deref;
use x86_64::instructions::port::Port;

#[allow(dead_code)]
pub enum PCIConfigRegisters {
    PCIDeviceID = 0x2,
    PCIVendorID = 0x0,
    PCIStatus = 0x6,
    PCICommand = 0x4,
    PCIClassCode = 0xB,
    PCISubclass = 0xA,
    PCIProgIF = 0x9,
    PCIRevisionID = 0x8,
    PCIBIST = 0xF,
    PCIHeaderType = 0xE,
    PCILatencyTimer = 0xD,
    PCICacheLineSize = 0xC,
    PCIBAR0 = 0x10,
    PCIBAR1 = 0x14,
    PCIBAR2 = 0x18,
    PCIBAR3 = 0x1C,
    PCIBAR4 = 0x20,
    PCIBAR5 = 0x24,
    PCICardbusCISPointer = 0x28,
    PCISubsystemID = 0x2E,
    PCISubsystemVendorID = 0x2C,
    PCIExpansionROMBaseAddress = 0x30,
    PCICapabilitiesPointer = 0x34,
    PCIMaxLatency = 0x3F,
    PCIMinGrant = 0x3E,
    PCIInterruptPIN = 0x3D,
    PCIInterruptLine = 0x3C,
}

fn config_address(bus: u8, slot: u8, func: u8, offset: u8) {
    let mut config_port: Port<u32> = Port::new(0xCF8);
    let address: u32 = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xfc)
        | 0x80000000;
    unsafe {
        config_port.write(address);
    }
}

pub fn config_read_u32(bus: u8, slot: u8, func: u8, off: u8) -> u32 {
    config_address(bus, slot, func, off);
    let mut port_config_data = Port::new(0xCFC);
    let read: u32 = unsafe { port_config_data.read() };
    read
}

pub fn config_read_u16(bus: u8, slot: u8, func: u8, off: u8) -> u16 {
    config_address(bus, slot, func, off);
    let mut port_config_data = Port::new(0xCFC);
    let read: u32 = unsafe { port_config_data.read() };
    let read = (read >> ((off & 2) * 8)) & 0xffff;
    read as u16
}

pub fn config_read_u8(bus: u8, slot: u8, func: u8, off: u8) -> u8 {
    config_address(bus, slot, func, off);
    let mut port_config_data = Port::new(0xCFC);
    let read: u32 = unsafe { port_config_data.read() };
    let read = (read >> ((off & 3) * 8)) & 0xff;
    read as u8
}

pub fn config_write_u32(bus: u8, slot: u8, func: u8, off: u8, data: u32) {
    config_address(bus, slot, func, off);
    let mut port_config_data = Port::new(0xCFC);
    unsafe {
        port_config_data.write(data);
    };
}

pub fn config_write_u16(bus: u8, slot: u8, func: u8, off: u8, data: u16) {
    config_address(bus, slot, func, off);
    let mut port_config_data = Port::new(0xCFC);
    let read: u32 = unsafe { port_config_data.read() };
    let val = (read & (!(0xFFFF << ((off & 2) * 8)))) | ((data as u32) << ((off & 2) * 8));
    unsafe {
        port_config_data.write(val);
    };
}

pub fn config_write_u8(bus: u8, slot: u8, func: u8, off: u8, data: u8) {
    config_address(bus, slot, func, off);
    let mut port_config_data = Port::new(0xCFC);
    let read: u32 = unsafe { port_config_data.read() };
    let val = (read & (!(0xFF << ((off & 3) * 8)))) | ((data as u32) << ((off & 3) * 8));
    unsafe {
        port_config_data.write(val);
    };
}

pub struct PCI {
    bus: u8,
    slot: u8,
    func: u8,
}

impl PCI {
    pub fn config_read_u8(&self, off: u8) -> u8 {
        config_read_u8(self.bus, self.slot, self.func, off as u8)
    }
    pub fn config_write_u8(&self, off: u8, val: u8) {
        config_write_u8(self.bus, self.slot, self.func, off as u8, val)
    }
    pub fn config_read_u16(&self, off: u8) -> u16 {
        config_read_u16(self.bus, self.slot, self.func, off as u8)
    }
    pub fn config_write_u16(&self, off: u8, val: u16) {
        config_write_u16(self.bus, self.slot, self.func, off as u8, val)
    }
    pub fn config_read_u32(&self, off: u8) -> u32 {
        config_read_u32(self.bus, self.slot, self.func, off as u8)
    }
}

pub struct PCIS {
    devices: Vec<PCI>,
}

impl PCIS {
    pub fn new() -> Self {
        let mut devices = Vec::new();
        for bus in 0..=255 {
            for slot in 0..32 {
                for func in 0..8 {
                    let vendor =
                        config_read_u16(bus, slot, func, PCIConfigRegisters::PCIVendorID as u8);
                    if vendor != 0xFFFF {
                        let header_type = config_read_u8(
                            bus,
                            slot,
                            func,
                            PCIConfigRegisters::PCIHeaderType as u8,
                        );
                        devices.push(PCI { bus, slot, func });
                        if func == 0 && (header_type & 0x80) == 0 {
                            break;
                        }
                    }
                }
            }
        }
        Self { devices }
    }
}

impl Deref for PCIS {
    type Target = Vec<PCI>;
    fn deref(&self) -> &Self::Target {
        &self.devices
    }
}
