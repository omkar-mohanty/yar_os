use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::fmt;
use x86_64::{structures::paging::FrameAllocator, PhysAddr, VirtAddr};

use crate::{
    drivers::pci::Bar, memory::FRAME_ALLOCATOR, phys_to_virt_addr, read_virt_addr, ReadError,
};

use super::{
    pci::{ClassCode, Pci},
    Driver, PCI_DEVICES,
};

const TX_DESC_NUM: u32 = 32;
const TX_DESC_SIZE: u32 = TX_DESC_NUM * core::mem::size_of::<TxRegister>() as u32;

pub(super) static NETWORK_DEVICES: OnceCell<Vec<NetworkDriver>> = OnceCell::uninit();

pub fn init() -> Result<(), Error> {
    let mut network_devices = Vec::new();
    for pci in PCI_DEVICES.get().unwrap() {
        if let ClassCode::Network(_) = pci.header.class_code {
            network_devices.push(NetworkDriver::new(pci)?);
        }
    }
    NETWORK_DEVICES.init_once(|| network_devices);

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    OutOfMemory,
}

impl From<ReadError> for Error {
    fn from(_value: ReadError) -> Self {
        Error::OutOfMemory
    }
}

pub enum NetworkSubClass {
    Ethernet,
    TokenRing,
    FDDI,
    ATM,
    ISDN,
    WordFip,
    PICMGMultiComputing,
    Infiniband,
    Fabric,
    Other,
}

impl From<u8> for NetworkSubClass {
    fn from(value: u8) -> Self {
        use NetworkSubClass::*;
        match value {
            0x0 => Ethernet,
            0x1 => TokenRing,
            0x2 => FDDI,
            0x3 => ATM,
            0x4 => ISDN,
            0x5 => WordFip,
            0x6 => PICMGMultiComputing,
            0x7 => Infiniband,
            0x8 => Fabric,
            _ => Other,
        }
    }
}

impl fmt::Debug for NetworkSubClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use NetworkSubClass::*;
        let st = match self {
            Ethernet => "Ethernet",
            TokenRing => "Token Ring",
            FDDI => "FDDI",
            ATM => "ATM",
            ISDN => "ISDN",
            WordFip => "Word FIP",
            PICMGMultiComputing => "PIC MG Multi Computing",
            Infiniband => "Infiniband",
            Fabric => "Fabric",
            Other => "Other",
        };
        f.write_str(st)
    }
}

enum Register {
    Eeprom = 0x14,
}

impl Driver for NetworkDriver {
    fn start(&self) {
        unimplemented!()
    }
}

#[derive(Default)]
#[repr(C, packed)]
struct RxRegister {
    addr: u64,
    length: u16,
    checksum: u16,
    status: u8,
    errors: u8,
    special: u16,
}

#[derive(Default)]
#[repr(C, packed)]
struct TxRegister {
    addr: u64,
    len: u16,
    cso: u8,
    cmd: u8,
    status: u8,
    css: u8,
    special: u16,
}

pub struct NetworkDriver {
    base_register: VirtAddr,
    mac: [u8; 6],
    tx_curr: usize,
    tx_ring: VirtAddr,
    rx_curr: usize,
    rx_ring: VirtAddr,
    eeprom: bool,
}

impl NetworkDriver {
    fn write(&self, register: Register, value: u32) {
        unsafe {
            let register = self.base_register.as_mut_ptr::<u8>().add(register as usize);
            core::ptr::write_volatile(register as *mut u32, value);
        }
    }
    fn read(&self, register: Register) -> u32 {
        unsafe {
            let register = self.base_register.as_mut_ptr::<u8>().add(register as usize);
            core::ptr::read_volatile(register as *const u32)
        }
    }
    fn detect_eeprom(&self) -> bool {
        self.write(Register::Eeprom, 0x1);
        for _ in 0..1000 {
            let value = self.read(Register::Eeprom);
            if value & (1 << 4) > 0 {
                return true;
            }
        }

        return false;
    }
    fn read_eeprom(&self, addr: u8) -> u32 {
        self.write(Register::Eeprom, 1 | ((addr as u32) << 8));
        loop {
            let value = self.read(Register::Eeprom);
            if value & (1 << 4) > 0 {
                return (value >> 16) & 0xffff;
            }
        }
    }
    fn get_mac_address(&mut self) -> [u8; 6] {
        let mut mac: [u8; 6] = [0, 0, 0, 0, 0, 0];
        for i in 0..3 {
            let tmp = self.read_eeprom(i);
            mac[i as usize * 2] = (tmp & 0xff) as u8;
            mac[i as usize * 2 + 1] = (tmp >> 8) as u8;
        }
        mac
    }

    fn init_tx(&mut self) -> Result<(), Error> {
        let mut frame_allocator = FRAME_ALLOCATOR.try_get().unwrap().lock();

        let frame = frame_allocator.allocate_frame().unwrap();
        let phys = frame.start_address();
        let mut addr = phys_to_virt_addr(phys);
        let descriptors = read_virt_addr::<[TxRegister; TX_DESC_NUM as usize]>(&mut addr)?;
        Ok(())
    }

    pub fn new(pci: &Pci) -> Result<Self, Error> {
        pci.enable_mmio();
        pci.enable_bus_mastering();
        let bar = pci.get_bar(0);

        let phys_addr = match bar {
            Some(Bar::Memory32 { address, .. }) => PhysAddr::new(address as u64),
            Some(Bar::Memory64 { address, .. }) => PhysAddr::new(address),
            _ => panic!("Unknown Base Address register for Network Driver"),
        };
        let base_register = phys_to_virt_addr(phys_addr);

        let mut this = Self {
            base_register,
            mac: [0; 6],
            tx_curr: 0,
            tx_ring: VirtAddr::zero(),
            rx_curr: 0,
            rx_ring: VirtAddr::zero(),
            eeprom: false,
        };

        this.init_tx()?;

        this.eeprom = this.detect_eeprom();
        this.mac = this.get_mac_address();
        Ok(this)
    }
}
