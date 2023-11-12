use conquer_once::spin::OnceCell;

use crate::println;

use pci::PCIS;
mod pci;

static PCI_DEVICES: OnceCell<PCIS> = OnceCell::uninit();
pub fn init() {
    let pci_devices = pci::PCIS::new();
    println!("Total {}", &pci_devices.len());
}
