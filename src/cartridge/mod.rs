pub mod mirroring;

use self::mirroring::Mirroring;

const MAGIC_NUMBERS: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

/// ## [iNES format](https://www.nesdev.org/wiki/INES)
pub struct Cartridge {
  pub mapper: u8,
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub nametable_mirroring: Mirroring,
}

impl Cartridge {
  pub fn new(raw: &Vec<u8>) -> Result<Cartridge, String> {
    if &raw[0..4] != MAGIC_NUMBERS {
      return Err("File is not in iNES file format".to_string());
    }
    if (raw[7] >> 2 & 0x03) != 0 {
      return Err("NES 2.0 format is not supported".to_string());
    }

    let mapper = (raw[7] & 0xF0) | (raw[6] >> 4);

    let mirroring = match (raw[6] & 0x08 == 0x80, raw[6] & 0x01 == 0x01) {
      (false, false) => Mirroring::Horizontal,
      (false, true) => Mirroring::Vertical,
      (true, _) => Mirroring::FourScreen,
    };

    let has_trainer = raw[6] & 0x04 == 0x04;

    // Size of PRG ROM in 16 KB units
    let prg_rom_start = 16 + if has_trainer { 512 } else { 0 };
    let prg_rom_size = (raw[4] as usize) * 16384;
    let prg_rom = raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec();

    // Size of CHR ROM in 8 KB units
    let chr_rom_start = prg_rom_start + prg_rom_size;
    let chr_rom_size = (raw[5] as usize) * 8192;
    let chr_rom = raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec();

    return Ok(Cartridge {
      mapper,
      prg_rom,
      chr_rom,
      nametable_mirroring: mirroring,
    });
  }
}

#[cfg(test)]
mod test {
  use super::*;

  struct TestRom {
    header: Vec<u8>,
    trainer: Option<Vec<u8>>,
    pgp_rom: Vec<u8>,
    chr_rom: Vec<u8>,
  }

  fn create_rom(rom: TestRom) -> Vec<u8> {
    let mut result = Vec::with_capacity(
      rom.header.len()
        + rom.trainer.as_ref().map_or(0, |t| t.len())
        + rom.pgp_rom.len()
        + rom.chr_rom.len(),
    );

    result.extend(&rom.header);
    if let Some(t) = rom.trainer {
      result.extend(t);
    }
    result.extend(&rom.pgp_rom);
    result.extend(&rom.chr_rom);

    result
  }

  #[test]
  fn test() {
    let test_rom = create_rom(TestRom {
      header: vec![
        0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
      ],
      trainer: None,
      pgp_rom: vec![1; 2 * 16384],
      chr_rom: vec![2; 1 * 8192],
    });

    let rom: Cartridge = Cartridge::new(&test_rom).unwrap();

    assert_eq!(rom.chr_rom, vec!(2; 1 * 8192));
    assert_eq!(rom.prg_rom, vec!(1; 2 * 16384));
    assert_eq!(rom.mapper, 3);
    assert_eq!(rom.nametable_mirroring, Mirroring::Vertical);
  }

  #[test]
  fn test_with_trainer() {
    let test_rom = create_rom(TestRom {
      header: vec![
        0x4E,
        0x45,
        0x53,
        0x1A,
        0x02,
        0x01,
        0x31 | 0b100,
        00,
        00,
        00,
        00,
        00,
        00,
        00,
        00,
        00,
      ],
      trainer: Some(vec![0; 512]),
      pgp_rom: vec![1; 2 * 16384],
      chr_rom: vec![2; 1 * 8192],
    });

    let rom: Cartridge = Cartridge::new(&test_rom).unwrap();

    assert_eq!(rom.chr_rom, vec!(2; 1 * 8192));
    assert_eq!(rom.prg_rom, vec!(1; 2 * 16384));
    assert_eq!(rom.mapper, 3);
    assert_eq!(rom.nametable_mirroring, Mirroring::Vertical);
  }

  #[test]
  fn test_nes2_is_not_supported() {
    let test_rom = create_rom(TestRom {
      header: vec![
        0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x31, 0x8, 00, 00, 00, 00, 00, 00, 00, 00,
      ],
      trainer: None,
      pgp_rom: vec![1; 1 * 16384],
      chr_rom: vec![2; 1 * 8192],
    });
    let rom = Cartridge::new(&test_rom);
    match rom {
      Result::Ok(_) => assert!(false, "should not load rom"),
      Result::Err(str) => assert_eq!(str, "NES 2.0 format is not supported"),
    }
  }
}
