use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use failure::Error;
use parking_lot::RwLock;

#[derive(Clone, Debug)]
pub struct Packet {
    pub origin: u32,
    pub dest: u32,
    pub packet_id: u32,
    pub checksum: u16,
    pub length: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Fail)]
pub enum PacketError {
    #[fail(display = "Packet read timed out.")]
    Timeout,
    #[fail(display = "Not enough data to form packet.")]
    NotEnoughData,
    #[fail(display = "Checksum does not match packet.")]
    InvalidChecksum,
}

lazy_static! {
    static ref PACKET_ID: RwLock<u32> = RwLock::new(0);
}

fn calculate_checksum(origin: u32, dest: u32, id: u32, data: &[u8]) -> u16 {
    let sum = data.iter().fold(0u32, |sum, &elem| sum + elem as u32);
    
    (origin + dest + id + data.len() as u32 + sum) as u16
}

impl Packet {
    pub fn new(dest: u32, data: &[u8]) -> Self {
        let mut id = PACKET_ID.write();
        *id += 1;

        let checksum = calculate_checksum(0, dest, *id, data);

        Packet {
            origin: 0,
            dest: dest,
            packet_id: *id,
            checksum,
            length: data.len() as u8,
            data: data.into(),
        }
    }

    pub fn default() -> Self {
        Packet {
            origin: 0,
            dest: 0,
            packet_id: 0,
            checksum: 0,
            length: 0,
            data: Vec::new(),
        }
    }

    pub fn from_data(raw: &[u8]) -> Result<Self, Error> {
        ensure!(raw.len() > 13, PacketError::NotEnoughData);
        ensure!(raw.len() >= raw[12] as usize + 13, PacketError::NotEnoughData);

        let packet = Packet {
            origin: LittleEndian::read_u32(&raw),
            dest: LittleEndian::read_u32(&raw[4..]),
            packet_id: LittleEndian::read_u32(&raw[8..]),
            checksum: LittleEndian::read_u16(&raw[12..]),
            length: raw[14],
            data: if raw[14] > 0 { 
                    raw[15..15 + raw[14] as usize].into() 
                } else { 
                    Vec::new()
                },
        };

        ensure!(packet.is_valid(), PacketError::InvalidChecksum);

        Ok(packet)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.write_u8(0xA3).unwrap();
        bytes.write_u8(0xFF).unwrap();
        bytes.write_u32::<LittleEndian>(self.origin).unwrap();
        bytes.write_u32::<LittleEndian>(self.dest).unwrap();
        bytes.write_u32::<LittleEndian>(self.packet_id).unwrap();
        bytes.write_u16::<LittleEndian>(self.checksum).unwrap();
        bytes.write_u8(self.length).unwrap();
        bytes.extend_from_slice(&self.data);

        bytes
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn is_valid(&self) -> bool {
        self.checksum == calculate_checksum(self.origin, self.dest, self.packet_id, &self.data)
    }
}