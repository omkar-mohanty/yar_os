use crate::println;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use core::fmt;
use x86_64::{PhysAddr, VirtAddr};

use crate::{drivers::pci::Bar, phys_to_virt_addr};

use super::{
    pci::{ClassCode, Pci},
    Driver, PCI_DEVICES,
};

pub(super) static NETWORK_DEVICES: OnceCell<Vec<NetworkDriver>> = OnceCell::uninit();

pub fn init() {
    let mut network_devices = Vec::new();
    for pci in PCI_DEVICES.get().unwrap() {
        if let ClassCode::Network(_) = pci.header.class_code {
            network_devices.push(NetworkDriver::new(pci));
        }
    }
    NETWORK_DEVICES.init_once(|| network_devices);
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
    pub fn new(pci: &Pci) -> Self {
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

        this.eeprom = this.detect_eeprom();
        this.mac = this.get_mac_address();
        println!(
            "e1000: MAC address {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            this.mac[0], this.mac[1], this.mac[2], this.mac[3], this.mac[4], this.mac[5]
        );
        this
    }
}
