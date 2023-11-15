use pci::PCI_DEVICES;

use pci::get_pci_devices;

mod network;
mod pci;
mod storage;

pub fn init() {
    PCI_DEVICES
        .try_init_once(|| get_pci_devices())
        .expect("Could not initialize PCI devices");
}
