use super::status_flags::Flags;

/// NES CPU 上的寄存器和 6502 上的一样。寄存器分别有：
///
/// - Accumulator 累加器
/// - 2 Indexes
/// - Program Counter 程序计数器
/// - Stack Pointer 堆栈指针
/// - Status 状态寄存器
///
/// NES Dev 文档地址：[CPU_registers](https://www.nesdev.org/wiki/CPU_registers)
pub struct Registers {
  /// accumulator
  pub a: u8,

  /// X - Indexes
  pub x: u8,

  /// Y - Indexes
  pub y: u8,

  /// status register [NV-B DIZC]
  ///
  /// also call name **P**
  pub status: Flags,

  /// stack pointer
  pub stack_pointer: u8,

  /// program counter
  pub program_counter: u16,
}

impl Registers {
  pub fn new() -> Self {
    return Registers {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: Flags::from_bits_truncate(0x34),
      stack_pointer: 0x00,
      program_counter: 0x0000,
    };
  }

  /// [CPU power up state](https://www.nesdev.org/wiki/CPU_power_up_state)
  pub fn reset(&mut self, program_counter: u16) {
    self.a = 0x00;
    self.x = 0x00;
    self.y = 0x00;
    // NV-B DIZC
    // 0011 0100
    self.status = Flags::from_bits_truncate(0x34);
    self.stack_pointer = 0xFD;
    self.program_counter = program_counter;
  }

  pub fn set_nz_flags(&mut self, result: u8) {
    // 是否为0
    if result == 0 {
      self.status.insert(Flags::Z);
    } else {
      self.status.remove(Flags::Z);
    }

    // 是否为负数
    if result & 0x80 != 0 {
      self.status.insert(Flags::N);
    } else {
      self.status.remove(Flags::N);
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_update_zero_and_negative_flags() {
    let mut registers = Registers::new();

    registers.set_nz_flags(0x00);
    assert!(registers.status.contains(Flags::Z));

    registers.set_nz_flags(0x80);
    assert!(registers.status.contains(Flags::N));
  }

}
