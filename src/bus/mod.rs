
pub struct Bus {
  cpu_vram: [u8; 0x800],
}

impl Bus {

  pub fn new<'a>() -> Self {
    return Bus {
      cpu_vram: [0; 0x800],
    };
  }

  pub fn read(&self, address: u16) -> u8 {
    return match address {
      // internal RAM
      0x0000..=0x1FFF => self.cpu_vram[(address & 0x7FF) as usize],
      // NES PPU registers
      0x2000..=0x3FFF => {
        todo!("PPU memory not impl {:04X}", address);
      }
      // // NES APU and I/O registers
      // 0x4000..=0x4017 => {

      // }
      // // APU and I/O functionality that is normally disabled. See CPU Test Mode.
      // 0x4018..=0x401F => {

      // }
      // // Cartridge space: PRG ROM, PRG RAM, and mapper registers
      // 0x4020..=0xFFFF => {

      // }
      _ => {
        println!("Ignoring mem access at {:04X}", address);
        return 0;
      }
    };
  }

  pub fn write(&mut self, address: u16, data: u8) {
    match address {
      // internal RAM
      0x0000..=0x1FFF => self.cpu_vram[(address & 0x7FF) as usize] = data,
      0x2000..=0x3FFF => {
        todo!("PPU memory not impl {:04X}", address);
      }
      _ => {
        println!("Ignoring mem write-access at {:04X}", address);
      }
    };
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
