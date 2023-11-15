use core::fmt;
mod ahci;

#[allow(dead_code)]
pub enum StorageSubclass {
    SCSI = 0x0,
    IDE = 0x1,
    Floppy = 0x2,
    IPIBus = 0x3,
    RAID = 0x4,
    ATA = 0x5,
    SATA = 0x6,
    SerialAttachedSCSI = 0x7,
    NonVolatileMemory = 0x8,
    Other,
}

impl From<u8> for StorageSubclass {
    fn from(value: u8) -> Self {
        use StorageSubclass::*;
        match value {
            0x0 => SCSI,
            0x1 => IDE,
            0x2 => Floppy,
            0x3 => IPIBus,
            0x4 => RAID,
            0x5 => ATA,
            0x6 => SATA,
            0x7 => SerialAttachedSCSI,
            0x8 => NonVolatileMemory,
            _ => Other,
        }
    }
}

impl fmt::Debug for StorageSubclass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use StorageSubclass::*;
        let st = match self {
            SCSI => "SCSI",
            IDE => "IDE",
            Floppy => "Floppy",
            IPIBus => "IPIBus",
            RAID => "Raid",
            ATA => "ATA",
            SATA => "SATA",
            SerialAttachedSCSI => "Serial Attached SCSI",
            NonVolatileMemory => "Non Volatile Memory",
            Other => "Other",
        };
        f.write_str(st)
    }
}
