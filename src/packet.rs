use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use failure::Error;
use parking_lot::RwLock;

#[derive(Debug)]
pub struct Packet {
    pub origin: u32,
    pub dest: u32,
    pub packet_id: u32,
    pub length: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Fail)]
pub enum PacketError {
    #[fail(display = "Packet read timed out.")]
    Timeout,
    #[fail(display = "Not enough data to form packet.")]
    NotEnoughData,
}

lazy_static! {
    static ref PACKET_ID: RwLock<u32> = RwLock::new(0);
}

impl Packet {
    pub fn new(dest: u32, data: &[u8]) -> Self {
        let mut id = PACKET_ID.write();
        *id += 1;

        Packet {
            origin: 0,
            dest: dest,
            packet_id: *id,
            length: data.len() as u8,
            data: data.into(),
        }
    }

    pub fn default() -> Self {
        Packet {
            origin: 0,
            dest: 0,
            packet_id: 0,
            length: 0,
            data: Vec::new(),
        }
    }

    pub fn from_data(raw: &[u8]) -> Result<Self, Error> {
        ensure!(raw.len() > 13, PacketError::NotEnoughData);
        ensure!(raw.len() == raw[12] as usize + 13, PacketError::NotEnoughData);

        Ok(Packet {
            origin: LittleEndian::read_u32(&raw),
            dest: LittleEndian::read_u32(&raw[4..]),
            packet_id: LittleEndian::read_u32(&raw[8..]),
            length: raw[12],
            data: raw[13..].into(),
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.write_u32::<LittleEndian>(self.origin).unwrap();
        bytes.write_u32::<LittleEndian>(self.dest).unwrap();
        bytes.write_u32::<LittleEndian>(self.packet_id).unwrap();
        bytes.write_u8(self.length).unwrap();
        bytes.extend_from_slice(&self.data);

        bytes
    }
}