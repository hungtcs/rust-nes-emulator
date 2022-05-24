
pub struct Memory {
  pub memory: [u8; 0xFFFF],
}

impl Memory {

  pub fn new() -> Self {
    return Memory {
      memory: [0x00; 0xFFFF],
    };
  }

  pub fn read(&self, address: u16) -> u8 {
    return self.memory[address as usize];
  }

  pub fn write(&mut self, address: u16, data: u8) {
    self.memory[address as usize] = data;
  }

  pub fn read_u16(&self, address: u16) -> u16 {
    let lo = self.read(address) as u16;
    let hi = self.read(address + 1) as u16;
    return (hi << 8) | lo;
  }

  pub fn write_u16(&mut self, address: u16, data: u16) {
    let lo = (data & 0x00FF) as u8;
    let hi = (data >> 8) as u8;
    self.write(address, lo);
    self.write(address + 1, hi);
  }
}
