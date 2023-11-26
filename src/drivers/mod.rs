use alloc::{sync::Arc, vec::Vec};
use conquer_once::spin::OnceCell;
use pci::{get_pci_devices, Pci};
mod network;
mod pci;
mod storage;

pub trait Driver: Sync + Send {
    fn start(&self);
}

static PCI_DEVICES: OnceCell<Vec<Pci>> = OnceCell::uninit();
static DEVICES: OnceCell<Vec<Arc<dyn Driver>>> = OnceCell::uninit();

pub fn init() {
    PCI_DEVICES
        .try_init_once(|| get_pci_devices())
        .expect("Could not initialize PCI devices");
    network::init();
}
