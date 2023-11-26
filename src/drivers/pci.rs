use super::{network::NetworkSubClass, storage::StorageSubclass};
use alloc::vec::Vec;
use bit_field::BitField;
use core::fmt;
use x86_64::instructions::port::Port;

pub(super) fn get_pci_devices() -> Vec<Pci> {
    let mut devices = Vec::new();
    for bus in 0..=255 {
        for slot in 0..32 {
            for func in 0..8 {
                let vendor =
                    config_read_u16(bus, slot, func, PCIConfigRegisters::PCIVendorID as u8);
                if vendor != 0xFFFF {
                    let header_type =
                        config_read_u8(bus, slot, func, PCIConfigRegisters::PCIHeaderType as u8);
                    let header = Header::new(bus, slot, func);
                    devices.push(Pci {
                        bus,
                        slot,
                        func,
                        header,
                    });
                    if func == 0 && (header_type & 0x80) == 0 {
                        break;
                    }
                }
            }
        }
    }
    devices
}

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

#[allow(dead_code)]
#[repr(u8)]
pub enum ClassCode {
    Unclassified = 0x0,
    MassStorage(StorageSubclass) = 0x1,
    Network(NetworkSubClass) = 0x2,
    Display = 0x3,
    Multimedia = 0x4,
    Memory = 0x5,
    Bridge = 0x6,
    Communication = 0x7,
    BaseSystemPeripheral = 0x8,
    InputDevice = 0x9,
    DockingStation = 0xA,
    Processor = 0xB,
    SerialBus = 0xC,
    Wireless = 0xD,
    Intelligent = 0xE,
    SatelliteCommunication = 0xF,
    Encryption = 0x10,
    SignalProcessing = 0x11,
    ProcessingAccellerator = 0x12,
    NonEssential = 0x13,
}

impl From<u16> for ClassCode {
    fn from(value: u16) -> Self {
        let subclass = value as u8;
        let class = (value >> 8) as u8;
        use ClassCode::*;
        match class {
            0x1 => MassStorage(StorageSubclass::from(subclass)),
            0x2 => Network(NetworkSubClass::from(subclass)),
            0x3 => Display,
            0x4 => Multimedia,
            0x5 => Memory,
            0x6 => Bridge,
            0x7 => Communication,
            0x8 => BaseSystemPeripheral,
            0x9 => InputDevice,
            0xA => DockingStation,
            0xB => Processor,
            0xC => SerialBus,
            0xD => Wireless,
            0xE => Intelligent,
            0xF => SatelliteCommunication,
            0x10 => Encryption,
            0x11 => SignalProcessing,
            0x12 => ProcessingAccellerator,
            0x13 => NonEssential,
            _ => Unclassified,
        }
    }
}

impl fmt::Debug for ClassCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ClassCode::*;
        let st = match self {
            Unclassified => "Unclassified",
            MassStorage(subclass) => {
                subclass.fmt(f)?;
                " Mass Storage"
            }
            Network(subclass) => {
                subclass.fmt(f)?;
                " Network"
            }
            Display => "Display",
            Multimedia => "Multimedia",
            Memory => "Memory",
            Bridge => "Bridge",
            Communication => "Communication",
            BaseSystemPeripheral => "Base System Peripheral",
            InputDevice => "Input Device",
            DockingStation => "Docking Device",
            Processor => "Processor",
            SerialBus => "Serial Bus",
            Wireless => "Wireless",
            Intelligent => "Intelligent",
            SatelliteCommunication => "Satellite Communication",
            Encryption => "Encryption",
            SignalProcessing => "Signal Processing",
            ProcessingAccellerator => "Processing Accellerator",
            NonEssential => "Non Essential",
        };
        f.write_str(st)
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

struct NonBridgeHeader {
    interrupt_line: Option<u8>,
    interrupt_pin: Option<u8>,
}

impl fmt::Debug for NonBridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(interrupt_line) = self.interrupt_line {
            f.write_fmt(format_args!("Interrupt Line : {}", interrupt_line))?;
            f.write_str("\n")?;
        }
        if let Some(interrupt_pin) = self.interrupt_pin {
            f.write_fmt(format_args!("Interrupt Pin : {}", interrupt_pin))?;
        }
        Ok(())
    }
}

pub enum HeaderType {
    Type1,
    Type2,
    Unknown,
}

impl From<u8> for HeaderType {
    fn from(value: u8) -> Self {
        match value {
            0x0 => HeaderType::Type1,
            0x2 => HeaderType::Type2,
            _ => HeaderType::Unknown,
        }
    }
}

impl fmt::Debug for HeaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let st = match self {
            HeaderType::Type1 => "Type 1",
            HeaderType::Type2 => "Type 2",
            HeaderType::Unknown => "Unknown",
        };
        f.write_str(st)
    }
}

pub struct Header {
    pub header_type: HeaderType,
    pub class_code: ClassCode,
    pub non_bridge_header: Option<NonBridgeHeader>,
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.header_type.fmt(f)?;
        f.write_str("\n")?;
        self.class_code.fmt(f)?;
        f.write_str("\n")?;
        if let Some(non_bridge_header) = &self.non_bridge_header {
            non_bridge_header.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Header {
    pub fn new(bus: u8, slot: u8, func: u8) -> Self {
        let val = config_read_u32(bus, slot, func, 0x8);
        let class_code = ClassCode::from((val >> 16) as u16);

        let val = config_read_u8(bus, slot, func, PCIConfigRegisters::PCIHeaderType as u8);
        let header_type = HeaderType::from(val);

        let non_bridge_header = match header_type {
            HeaderType::Type1 => {
                let mut non_bridge_header = NonBridgeHeader {
                    interrupt_pin: None,
                    interrupt_line: None,
                };
                let val = config_read_u32(bus, slot, func, 0x3C);
                let interrupt_line = val as u8;
                let interrupt_pin = (val >> 8) as u8;
                if interrupt_line != 0xFF {
                    non_bridge_header.interrupt_line = Some(interrupt_line);
                }
                if interrupt_pin != 0x0 {
                    non_bridge_header.interrupt_pin = Some(interrupt_pin);
                }
                Some(non_bridge_header)
            }
            _ => None,
        };
        Self {
            class_code,
            header_type,
            non_bridge_header,
        }
    }
}

pub enum Bar {
    Memory32 {
        size: u32,
        address: u32,
        prefetchable: bool,
    },
    Memory64 {
        size: u64,
        address: u64,
        prefetchable: bool,
    },
    Io(u32),
}

pub struct Pci {
    bus: u8,
    slot: u8,
    func: u8,
    pub(super) header: Header,
}

impl Pci {
    pub fn config_read_u8(&self, off: u8) -> u8 {
        config_read_u8(self.bus, self.slot, self.func, off)
    }
    pub fn config_write_u8(&self, off: u8, val: u8) {
        config_write_u8(self.bus, self.slot, self.func, off, val)
    }
    pub fn config_read_u16(&self, off: u8) -> u16 {
        config_read_u16(self.bus, self.slot, self.func, off)
    }
    pub fn config_write_u16(&self, off: u8, val: u16) {
        config_write_u16(self.bus, self.slot, self.func, off, val)
    }
    pub fn config_read_u32(&self, off: u8) -> u32 {
        config_read_u32(self.bus, self.slot, self.func, off)
    }
    pub fn config_write_u32(&self, off: u8, val: u32) {
        config_write_u32(self.bus, self.slot, self.func, off, val)
    }

    pub fn get_bar(&self, bar: u8) -> Option<Bar> {
        let off = 0x10 + bar * 4;
        let bar = self.config_read_u32(off);

        if !bar.get_bit(0) {
            let address = bar.get_bits(4..32) << 4;
            let prefetchable = bar.get_bit(3);

            let size = {
                self.config_write_u32(off, 0xffffffff);
                let mut readback = self.config_read_u32(off);
                self.config_write_u32(off, address);

                if readback == 0x0 {
                    return None;
                }

                readback.set_bits(0..4, 0);

                1 << readback.trailing_zeros()
            };

            match bar.get_bits(1..3) {
                0b00 => Some(Bar::Memory32 {
                    address,
                    prefetchable,
                    size,
                }),
                0b10 => {
                    let address = {
                        let mut address = address as u64;
                        address.set_bits(32..64, self.config_read_u32(off + 4).into());

                        address
                    };
                    Some(Bar::Memory64 {
                        address,
                        size: size as u64,
                        prefetchable,
                    })
                }
                _ => None,
            }
        } else {
            Some(Bar::Io(bar.get_bits(2..32)))
        }
    }

    pub fn enable_bus_mastering(&self) {
        let command = self.config_read_u16(0x4);
        self.config_write_u16(0x4, command | (1 << 2));
    }

    pub fn enable_mmio(&self) {
        let command = self.config_read_u16(0x4);
        self.config_write_u16(0x4, command | (1 << 1));
    }
}

impl fmt::Debug for Pci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.header.fmt(f)
    }
}
