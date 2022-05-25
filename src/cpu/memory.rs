
/// ## [CPU Memory Map](https://www.nesdev.org/wiki/CPU_memory_map)
///
/// | Address range | Size	| Device                                                                  |
/// |---------------|-------|-------------------------------------------------------------------------|
/// | $0000-$07FF	  | $0800 |	2KB internal RAM                                                        |
/// | $0800-$0FFF	  | $0800 |	Mirrors of $0000-$07FF                                                  |
/// | $1000-$17FF	  | $0800 | Mirrors of $0000-$07FF                                                  |
/// | $1800-$1FFF	  | $0800 | Mirrors of $0000-$07FF                                                  |
/// | $2000-$2007	  | $0008 |	NES PPU registers                                                       |
/// | $2008-$3FFF	  | $1FF8 |	Mirrors of $2000-2007 (repeats every 8 bytes)                           |
/// | $4000-$4017	  | $0018 |	NES APU and I/O registers                                               |
/// | $4018-$401F	  | $0008 |	APU and I/O functionality that is normally disabled. See CPU Test Mode. |
/// | $4020-$FFFF	  | $BFE0 |	Cartridge space: PRG ROM, PRG RAM, and mapper registers (See Note)      |
///
/// Some parts of the 2 KiB of internal RAM at $0000-$07FF have predefined purposes dictated by the 6502 architecture.
/// The zero page is $0000-$00FF, and the stack always uses some part of the $0100-$01FF page.
///
/// Note: Most common boards and iNES mappers address ROM and Save/Work RAM in this format:
///
/// - $6000-$7FFF = Battery Backed Save or Work RAM
/// - $8000-$FFFF = Usual ROM, commonly with Mapper Registers (see MMC1 and UxROM for example)
///
/// If using DMC audio:
///
/// - $C000-$FFF1 = DPCM samples
///
/// The CPU expects interrupt vectors in a fixed place at the end of the cartridge space:
///
/// - $FFFA-$FFFB = NMI vector
/// - $FFFC-$FFFD = Reset vector
/// - $FFFE-$FFFF = IRQ/BRK vector
///
/// If a mapper doesn't fix $FFFA-$FFFF to some known bank (typically, along with the rest of the bank containing them,
/// e.g. $C000-$FFFF for a 16KiB banking mapper) or use some sort of reset detection,
/// the vectors need to be stored in all banks.
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
