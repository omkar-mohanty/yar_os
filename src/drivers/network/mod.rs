use core::fmt;

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
