const SBUS_HEADER_BYTE: u8 = 0x0F;
const SBUS_FOOTER_BYTE: u8 = 0b00000000;
const SBUS_PACKET_SIZE: usize = 25;

pub fn encode_sbus(channels: [u16; 16]) -> [u8; 25] {
  let mut buf = [0u8; 25];
  buf[0] = 0x0F;
  buf[1] = (channels[0] & 0x07FF) as u8;
  buf[2] = (((channels[0] & 0x07FF) >> 8) | ((channels[1] & 0x07FF) << 3)) as u8;
  buf[3] = (((channels[1] & 0x07FF) >> 5) | ((channels[2] & 0x07FF) << 6)) as u8;
  buf[4] = ((channels[2] & 0x07FF) >> 2) as u8;
  buf[5] = (((channels[2] & 0x07FF) >> 10) | ((channels[3] & 0x07FF) << 1)) as u8;
  buf[6] = (((channels[3] & 0x07FF) >> 7) | ((channels[4] & 0x07FF) << 4)) as u8;
  buf[7] = (((channels[4] & 0x07FF) >> 4) | ((channels[5] & 0x07FF) << 7)) as u8;
  buf[8] = ((channels[5] & 0x07FF) >> 1) as u8;
  buf[9] = (((channels[5] & 0x07FF) >> 9) | ((channels[6] & 0x07FF) << 2)) as u8;
  buf[10] = (((channels[6] & 0x07FF) >> 6) | ((channels[7] & 0x07FF) << 5)) as u8;
  buf[11] = ((channels[7] & 0x07FF) >> 3) as u8;
  buf[12] = (channels[8] & 0x07FF) as u8;
  buf[13] = (((channels[8] & 0x07FF) >> 8) | ((channels[9] & 0x07FF) << 3)) as u8;
  buf[14] = (((channels[9] & 0x07FF) >> 5) | ((channels[10] & 0x07FF) << 6)) as u8;
  buf[15] = ((channels[10] & 0x07FF) >> 2) as u8;
  buf[16] = (((channels[10] & 0x07FF) >> 10) | ((channels[11] & 0x07FF) << 1)) as u8;
  buf[17] = (((channels[11] & 0x07FF) >> 7) | ((channels[12] & 0x07FF) << 4)) as u8;
  buf[18] = (((channels[12] & 0x07FF) >> 4) | ((channels[13] & 0x07FF) << 7)) as u8;
  buf[19] = ((channels[13] & 0x07FF) >> 1) as u8;
  buf[20] = (((channels[13] & 0x07FF) >> 9) | ((channels[14] & 0x07FF) << 2)) as u8;
  buf[21] = (((channels[14] & 0x07FF) >> 6) | ((channels[15] & 0x07FF) << 5)) as u8;
  buf[22] = ((channels[15] & 0x07FF) >> 3) as u8;
  buf[23] = 0x00;
  buf[24] = 0x00;

  buf
}
