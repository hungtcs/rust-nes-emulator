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

  /// LIFO, top-down, 8 bit range, 0x0100 - 0x01FF
  fn stack_push(&mut self, data: u8) {
    self.memory.write(0x0100 + (self.registers.stack_pointer as u16), data);
    self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
  }

  fn stack_pop(&mut self) -> u8 {
    self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
    return self.memory.read(0x0100 + (self.registers.stack_pointer as u16));
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
        // Transfer Instructions
        // LDA
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.load_accumulator_with_memory(&opcode.mode),
        // LDX
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.load_index_x_with_memory(&opcode.mode),
        // LDY
        0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.load_index_y_with_memory(&opcode.mode),
        // STA
        0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.store_accumulator_in_memory(&opcode.mode),
        // STX
        0x86 | 0x96 | 0x8E => self.store_index_x_in_memory(&opcode.mode),
        // STY
        0x84 | 0x94 | 0x8C => self.store_index_y_in_memory(&opcode.mode),
        // TAX
        0xAA => self.transfer_accumulator_to_index_x(),
        // TAY
        0xA8 => self.transfer_accumulator_to_index_y(),
        // TSX
        0xBA => self.transfer_stack_pointer_to_index_x(),
        // TXA
        0x8A => self.transfer_index_x_to_accumulator(),
        // TXS
        0x9A => self.transfer_index_x_to_stack_register(),
        // TYA
        0x98 => self.transfer_index_y_to_accumulator(),

        // Stack Instructions
        // PHA
        0x48 => self.push_accumulator_on_stack(),
        // PHP
        0x08 => self.push_processor_status_on_stack(),
        // PLA
        0x68 => self.pull_accumulator_from_stack(),
        // PLP
        0x28 => self.pull_processor_status_from_stack(),

        // Decrements & Increments
        // DEC
        0xC6 | 0xD6 | 0xCE | 0xDE => self.decrement_memory_by_one(&opcode.mode),
        // DEX
        0xCA => self.decrement_index_x_by_one(),
        // DEY
        0x88 => self.decrement_index_y_by_one(),
        // INC
        0xE6 | 0xF6 | 0xEE | 0xFE => self.increment_memory_by_one(&opcode.mode),
        // INX
        0xE8 => self.increment_index_x_by_one(),
        // INY
        0xC8 => self.increment_index_y_by_one(),

        // Arithmetic Operations
        // ADC
        0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.add_memory_to_accumulator_with_carry(&opcode.mode),
        // SBC
        0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.subtract_memory_from_accumulator_with_borrow(&opcode.mode),

        // Logical Operations
        // AND
        0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and_memory_with_accumulator(&opcode.mode),
        // EOR
        0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.exclusive_or_memory_with_accumulator(&opcode.mode),
        // ORA
        0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.or_memory_with_accumulator(&opcode.mode),

        // Shift & Rotate Instructions
        // ASL
        0x0A => self.shift_left_one_bit_accumulator(),
        0x06 | 0x16 | 0x0E | 0x1E => self.shift_left_one_bit_memory(&opcode.mode),
        // LSR
        0x4A => self.shift_one_bit_right_accumulator(),
        0x46 | 0x56 | 0x4E | 0x5E => self.shift_one_bit_right_memory(&opcode.mode),
        // ROL
        0x2A => self.rotate_one_bit_left_accumulator(),
        0x26 | 0x36 | 0x2E | 0x3E => self.rotate_one_bit_left_memory(&opcode.mode),
        // ROR
        0x6A => self.rotate_one_bit_right_accumulator(),
        0x66 | 0x76 | 0x6E | 0x7E => self.rotate_one_bit_right_memory(&opcode.mode),


        _ => {
          panic!("opcode {:x} not support", code);
        }
      }

      if program_counter_state == self.registers.program_counter {
        self.registers.program_counter += (opcode.length - 1) as u16;
      }
    }
  }

}

impl CPU {
  /// Transfer Instructions

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

  /// LDY
  fn load_index_y_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let param = self.memory.read(address);
    self.registers.y = param;
    self.registers.set_nz_flags(self.registers.y);
  }

  /// STA
  fn store_accumulator_in_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    self.memory.write(address, self.registers.a);
  }

  /// STX
  fn store_index_x_in_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    self.memory.write(address, self.registers.x);
  }

  /// STY
  fn store_index_y_in_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    self.memory.write(address, self.registers.y);
  }

  /// TAX
  fn transfer_accumulator_to_index_x(&mut self) {
    self.registers.x = self.registers.a;
    self.registers.set_nz_flags(self.registers.x);
  }

  /// TAY
  fn transfer_accumulator_to_index_y(&mut self) {
    self.registers.y = self.registers.a;
    self.registers.set_nz_flags(self.registers.y);
  }

  /// TSX
  fn transfer_stack_pointer_to_index_x(&mut self) {
    self.registers.x = self.registers.stack_pointer;
    self.registers.set_nz_flags(self.registers.x);
  }

  /// TXA
  fn transfer_index_x_to_accumulator(&mut self) {
    self.registers.a = self.registers.x;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// TXS
  fn transfer_index_x_to_stack_register(&mut self) {
    self.registers.stack_pointer = self.registers.x;
  }

  /// TYA
  fn transfer_index_y_to_accumulator(&mut self) {
    self.registers.a = self.registers.y;
    self.registers.set_nz_flags(self.registers.a);
  }


  /// Stack Instructions
  /// PHA
  fn push_accumulator_on_stack(&mut self) {
    self.stack_push(self.registers.a);
  }

  /// PHP
  ///
  /// - [the B flag](https://www.nesdev.org/wiki/Status_flags#The_B_flag)
  fn push_processor_status_on_stack(&mut self) {
    let mut status = self.registers.status.clone();
    status.insert(Flags::B);
    status.insert(Flags::U);
    self.stack_push(status.bits());
  }

  /// PLA
  fn pull_accumulator_from_stack(&mut self) {
    self.registers.a = self.stack_pop();
    self.registers.set_nz_flags(self.registers.a);
  }

  /// PLP
  fn pull_processor_status_from_stack(&mut self) {
    let data = self.stack_pop();
    self.registers.status = Flags::from_bits(data).expect(&format!("Status {:x} is not valid", data));
    self.registers.status.remove(Flags::B);
    self.registers.status.insert(Flags::U);
  }


  /// Decrements & Increments
  /// DEC
  fn decrement_memory_by_one(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.memory.read(address);
    data = data.wrapping_sub(1);
    self.memory.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// DEX
  fn decrement_index_x_by_one(&mut self) {
    self.registers.x = self.registers.x.wrapping_sub(1);
    self.registers.set_nz_flags(self.registers.x);
  }

  /// DEY
  fn decrement_index_y_by_one(&mut self) {
    self.registers.y = self.registers.y.wrapping_sub(1);
    self.registers.set_nz_flags(self.registers.y);
  }

  /// INC
  fn increment_memory_by_one(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.memory.read(address);
    data = data.wrapping_add(1);
    self.memory.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// INX
  fn increment_index_x_by_one(&mut self) {
    self.registers.x = self.registers.x.wrapping_add(1);
    self.registers.set_nz_flags(self.registers.x);
  }

  /// INY
  fn increment_index_y_by_one(&mut self) {
    self.registers.y = self.registers.y.wrapping_add(1);
    self.registers.set_nz_flags(self.registers.y);
  }


  /// Arithmetic Operations
  /// ADC
  fn add_memory_to_accumulator_with_carry(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.memory.read(address);
    self.registers.add_to_a(data);
  }

  /// SBC
  /// `A - B = A + (-B)`, `-B = !B + 1`
  fn subtract_memory_from_accumulator_with_borrow(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.memory.read(address);
    self.registers.add_to_a((data as i8).wrapping_neg().wrapping_sub(-1) as u8);
  }


  /// Logical Operations
  /// AND
  fn and_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.memory.read(address);
    self.registers.a = self.registers.a & data;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// EOR
  fn exclusive_or_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.memory.read(address);
    self.registers.a = self.registers.a ^ data;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// ORA
  fn or_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.memory.read(address);
    self.registers.a = self.registers.a | data;
    self.registers.set_nz_flags(self.registers.a);
  }


  /// ### Shift & Rotate Instructions
  ///
  /// All shift and rotate instructions preserve the bit shifted out in the carry flag.
  ///
  /// ASL
  fn shift_left_one_bit_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.memory.read(address);

    self.registers.status.set(Flags::C, data & 0x80 == 0x80);
    data = data << 1;
    self.memory.write(address, data);
    self.registers.set_nz_flags(data);
  }

  fn shift_left_one_bit_accumulator(&mut self) {
    self.registers.status.set(Flags::C, self.registers.a & 0x80 == 0x80);
    self.registers.a = self.registers.a << 1;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// LSR
  fn shift_one_bit_right_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.memory.read(address);
    self.registers.status.set(Flags::C, data & 0x01 == 1);
    data = data >> 1;
    self.memory.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// LSR accumulator
  fn shift_one_bit_right_accumulator(&mut self) {
    self.registers.status.set(Flags::C, self.registers.a & 0x01 == 1);
    self.registers.a = self.registers.a >> 1;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// ROL
  fn rotate_one_bit_left_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.memory.read(address);
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, data & 0x80 == 0x80);
    data = (data << 1) | (if carry { 0x01 } else { 0x00 });
    self.memory.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// ROL accumulator
  fn rotate_one_bit_left_accumulator(&mut self) {
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, self.registers.a & 0x80 == 0x80);
    self.registers.a = (self.registers.a << 1) | (if carry { 0x01 } else { 0x00 });
    self.registers.set_nz_flags(self.registers.a);
  }

  /// ROR
  fn rotate_one_bit_right_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.memory.read(address);
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, data & 0x01 == 0x01);

    data = (data >> 1) | (if carry { 0x80 } else { 0x00 });
    self.memory.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// ROR accumulator
  fn rotate_one_bit_right_accumulator(&mut self) {
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, self.registers.a & 0x01 == 0x01);
    self.registers.a = (self.registers.a >> 1) | (if carry { 0x80 } else { 0x00 });
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
