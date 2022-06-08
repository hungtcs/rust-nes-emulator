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
    // self.status = Flags::from_bits_truncate(0x34);
    self.status = Flags::from_bits_truncate(0x24);
    self.stack_pointer = 0xFD;
    self.program_counter = program_counter;
  }

  pub fn set_nz_flags(&mut self, result: u8) {
    // 是否为0
    self.status.set(Flags::Z, result == 0);
    // 是否为负数
    self.status.set(Flags::N, result & 0x80 == 0x80);
  }

  /// ## [The 6502 overflow flag explained mathematically](http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html)
  ///
  /// Note on the overflow flag: The overflow flag indicates overflow with signed
  /// binary arithmetcis. As a signed byte represents a range of -128 to +127, an
  /// overflow can never occure when the operands are of opposite sign, since the
  /// result will never exceed this range. Thus, overflow may only occure, if both
  /// operands are of the same sign. Then, the result must be also of the same sign.
  /// Otherwise, overflow is detected and the overflow flag is set.
  /// (I.e., both operands have a zero in the sign position at bit 7, but bit 7 of the
  /// result is 1, or, both operands have the sign-bit set, but the result is positive.)
  ///
  /// 只有当两个相同符号数相加，而运算结果的符号与原数据符号相反时，产生溢出，此时的运算结果显然不正确。其他情况下，则不会产生溢出。
  ///
  /// 1. 如果 `data` 与 `result` 的符号位不同，则 `data ^ result` 最高位为 `1`
  /// 2. 如果 `self.a` 与 `result` 的符号位不同，则 `self.a ^ result` 最高位为 `1`
  /// 3. 如果 `(data ^ result) & (self.a ^ result)` 最高位为 `1`，则 data 和 self.a 的符号位与 result 皆不同。
  ///
  pub fn add_to_a(&mut self, data: u8) {
    let a = self.a as u16;
    let sum = a + (data as u16) + (if self.status.contains(Flags::C) { 1 } else { 0 });
    let result = sum as u8;

    self.status.set(Flags::C, sum > 0xFF);
    self.status.set(Flags::V, (data ^ result) & (self.a ^ result) & 0x80 != 0);

    self.a = result;
    self.set_nz_flags(self.a);
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
