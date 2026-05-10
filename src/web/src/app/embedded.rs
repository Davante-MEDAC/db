pub struct EmbeddedDb {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub static EMBEDDED_DBS: &[EmbeddedDb] = &[
    EmbeddedDb {
        name: "chile-seismological-records.sqlite",
        data: include_bytes!("../../../../databases/chile-seismological-records.sqlite"),
    },
    EmbeddedDb {
        name: "qa.sqlite",
        data: include_bytes!("../../../../databases/qa.sqlite"),
    },
];
