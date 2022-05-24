mod addressing_mode;
mod memory;
mod opcodes;
mod register;
mod status_flags;

use std::collections::HashMap;

use self::addressing_mode::AddressingMode;
use self::memory::Memory;
use self::opcodes::{Opcode, OPCODES_MAP};
use self::register::Registers;
use self::status_flags::Flags;

pub struct CPU {
  pub registers: Registers,
  memory: Memory,
}

impl CPU {
  pub fn new() -> Self {
    return CPU {
      registers: Registers::new(),
      memory: Memory::new(),
    }
  }

  fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
    use AddressingMode::*;
    match mode {
      Immediate => self.registers.program_counter,
      Absolute => self.memory.read_u16(self.registers.program_counter),
      AbsoluteX => self.memory.read_u16(self.registers.program_counter).wrapping_add(self.registers.x as u16),
      AbsoluteY => self.memory.read_u16(self.registers.program_counter).wrapping_add(self.registers.y as u16),
      ZeroPage => self.memory.read(self.registers.program_counter) as u16,
      ZeroPageX => self.memory.read(self.registers.program_counter).wrapping_add(self.registers.x) as u16,
      ZeroPageY => self.memory.read(self.registers.program_counter).wrapping_add(self.registers.y) as u16,
      // Indirect => self.memory.read_u16(self.memory.read_u16(self.program_counter)),
      IndexedIndirect => self.memory.read_u16(self.memory.read(self.registers.program_counter).wrapping_add(self.registers.x) as u16),
      IndirectIndexed => self.memory.read_u16(self.memory.read(self.registers.program_counter) as u16).wrapping_add(self.registers.y as u16),
      _ => panic!("addressing mode {:?} is not support", mode),
    }
  }

  fn load(&mut self, program: Vec<u8>) {
    // TODO
    self.memory.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
    self.memory.write_u16(0xFFFC, 0x8000);
  }

  /// NES 平台有一个特殊的机制来标记 CPU 应该从哪里开始执行。
  /// 插入新卡带后，CPU 会收到一个称为 `Reset interrupt` 的特殊信号，指示 CPU：
  ///
  /// - 重置状态（寄存器和标志）
  /// - 将 `program_counter` 寄存器设置为存储在 `0xFFFC` 的 16 位地址
  fn reset(&mut self) {
    self.registers.reset(self.memory.read_u16(0xFFFC));
  }

  pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run();
  }

  pub fn run(&mut self) {
    let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODES_MAP;

    loop {
      let code = self.memory.read(self.registers.program_counter);
      let opcode = opcodes.get(&code).expect(&format!("Opcode {:x} is not recognized", code));

      self.registers.program_counter += 1;

      let program_counter_state = self.registers.program_counter;

      match code {
        // LDA
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.load_accumulator_with_memory(&opcode.mode),

        // LDX
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.load_index_x_with_memory(&opcode.mode),

        // LDY
        0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.load_index_y_with_memory(&opcode.mode),

        // LSR
        0x4A => self.shift_one_bit_right_accumulator(),
        0x46 | 0x56 | 0x4E | 0x5E => self.shift_one_bit_right_memory(&opcode.mode),

        // NOP
        // 0xEA => return,



        // BRK
        0x00 => return,


        // 0xAA => self.tax(),
        // 0xe8 => self.inx(),

        // TAX
        0xAA => {
          self.registers.x = self.registers.a;
          self.registers.set_nz_flags(self.registers.x);
        }
        // INX
        0xE8 => {
          self.registers.x = self.registers.x.wrapping_add(1);
          self.registers.set_nz_flags(self.registers.x);
        }
        _ => {
          todo!();
        }
      }

      if program_counter_state == self.registers.program_counter {
        self.registers.program_counter += (opcode.length - 1) as u16;
      }
    }
  }

}

impl CPU {
  /// LDA
  fn load_accumulator_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let param = self.memory.read(address);

    self.registers.a = param;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// LDX
  fn load_index_x_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let param = self.memory.read(address);
    self.registers.x = param;
    self.registers.set_nz_flags(self.registers.x);
  }

  // LDY
  fn load_index_y_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let param = self.memory.read(address);
    self.registers.y = param;
    self.registers.set_nz_flags(self.registers.y);
  }

  /// LSR
  fn shift_one_bit_right_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut param = self.memory.read(address);
    if param & 0x01 == 1 {
      self.registers.status.insert(Flags::C);
    } else {
      self.registers.status.remove(Flags::C);
    }
    param = param >> 1;
    self.memory.write(address, param);
    self.registers.set_nz_flags(param);
  }

  fn shift_one_bit_right_accumulator(&mut self) {
    let mut a = self.registers.a;
    if a & 0x01 == 1 {
      self.registers.status.insert(Flags::C);
    } else {
      self.registers.status.remove(Flags::C);
    }
    a = a >> 1;
    self.registers.a = a;
    self.registers.set_nz_flags(self.registers.a);
  }


}

#[cfg(test)]
mod test {
  use super::*;
  use super::status_flags::*;

  #[test]
  fn test_0xa9_lda_immidiate_load_data() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
    assert_eq!(cpu.registers.a, 0x05);
    assert!(cpu.registers.a & 0x02 == 0);
    assert!(cpu.registers.a & 0x80 == 0);
  }

  #[test]
  fn test_0xa9_lda_zero_flag() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
    assert!(cpu.registers.status.contains(Flags::Z));
  }

  #[test]
  fn test_inx_increment_index_x_by_one() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);
    assert_eq!(cpu.registers.x, 2);
  }

  #[test]
  fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

    assert_eq!(cpu.registers.x, 0xc1);
  }

}
