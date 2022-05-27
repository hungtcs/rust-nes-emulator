mod addressing_mode;
mod opcodes;
mod register;
mod status_flags;

use std::collections::HashMap;
use crate::bus::Bus;
use self::addressing_mode::AddressingMode;
use self::opcodes::{Opcode, OPCODES_MAP};
use self::register::Registers;
use self::status_flags::Flags;

pub struct CPU {
  pub bus: Bus,
  pub registers: Registers,
}

impl CPU {
  pub fn new(bus: Bus) -> Self {
    return CPU {
      bus,
      registers: Registers::new(),
    }
  }

  fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
    use AddressingMode::*;
    match mode {
      Immediate => self.registers.program_counter,
      Absolute => self.bus.read_u16(self.registers.program_counter),
      AbsoluteX => self.bus.read_u16(self.registers.program_counter).wrapping_add(self.registers.x as u16),
      AbsoluteY => self.bus.read_u16(self.registers.program_counter).wrapping_add(self.registers.y as u16),
      ZeroPage => self.bus.read(self.registers.program_counter) as u16,
      ZeroPageX => self.bus.read(self.registers.program_counter).wrapping_add(self.registers.x) as u16,
      ZeroPageY => self.bus.read(self.registers.program_counter).wrapping_add(self.registers.y) as u16,
      Indirect => {
        // http://www.6502.org/tutorials/6502opcodes.html#JMP
        // Indirect 仅适用于 JMP 指令
        // **AN INDIRECT JUMP MUST NEVER USE A VECTOR BEGINNING ON THE LAST BYTE OF A PAGE**
        // For example if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
        // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
        // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000.
        let indirect_address = self.bus.read_u16(self.registers.program_counter);
        if indirect_address & 0x00FF == 0x00FF {
          let lo = self.bus.read(indirect_address);
          let hi = self.bus.read(indirect_address & 0xFF00);
          return self.bus.read_u16((hi as u16) << 8 | (lo as u16));
        } else {
          return self.bus.read_u16(indirect_address);
        }
      },
      IndexedIndirect => self.bus.read_u16(self.bus.read(self.registers.program_counter).wrapping_add(self.registers.x) as u16),
      IndirectIndexed => self.bus.read_u16(self.bus.read(self.registers.program_counter) as u16).wrapping_add(self.registers.y as u16),
      _ => panic!("addressing mode {:?} is not support", mode),
    }
  }

  /// LIFO, top-down, 8 bit range, 0x0100 - 0x01FF
  fn stack_push(&mut self, data: u8) {
    self.bus.write(0x0100 + (self.registers.stack_pointer as u16), data);
    self.registers.stack_pointer = self.registers.stack_pointer.wrapping_sub(1);
  }

  fn stack_pop(&mut self) -> u8 {
    self.registers.stack_pointer = self.registers.stack_pointer.wrapping_add(1);
    return self.bus.read(0x0100 + (self.registers.stack_pointer as u16));
  }

  fn stack_push_u16(&mut self, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.stack_push(hi);
    self.stack_push(lo);
  }

  fn stack_pop_u16(&mut self) -> u16 {
    let lo = self.stack_pop() as u16;
    let hi = self.stack_pop() as u16;
    return hi << 8 | lo;
  }

  pub fn load(&mut self, program: Vec<u8>) {
    for i in 0..(program.len() as u16) {
      self.bus.write(0x0600 + i, program[i as usize]);
    }
    self.bus.write_u16(0xFFFC, 0x0600);

    // TODO
    // self.bus.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    // self.bus.write_u16(0xFFFC, 0x8000);
  }

  /// NES 平台有一个特殊的机制来标记 CPU 应该从哪里开始执行。
  /// 插入新卡带后，CPU 会收到一个称为 `Reset interrupt` 的特殊信号，指示 CPU：
  ///
  /// - 重置状态（寄存器和标志）
  /// - 将 `program_counter` 寄存器设置为存储在 `0xFFFC` 的 16 位地址
  pub fn reset(&mut self) {
    self.registers.reset(self.bus.read_u16(0xFFFC));
  }

  pub fn load_and_run(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run();
  }

  pub fn run(&mut self) {
    self.run_with_callback(
      |_| {

      },
    );
  }

  pub fn run_with_callback<C>(&mut self, mut callback: C) where C: FnMut(&mut CPU) {
    let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODES_MAP;

    println!("PC   Code A  X  Y  Status");

    loop {
      let code = self.bus.read(self.registers.program_counter);
      let opcode = opcodes.get(&code).expect(&format!("Opcode {:x} is not recognized", code));
      let mode = &opcode.mode;

      print!(
        "{:04X} {:02X}   {:02X} {:02X} {:02X} {:08b} \t",
        self.registers.program_counter,
        code,
        self.registers.a,
        self.registers.x,
        self.registers.y,
        self.registers.status.bits(),
      );

      self.registers.program_counter += 1;

      let program_counter_state = self.registers.program_counter;

      match code {
        // Transfer Instructions
        // LDA
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.load_accumulator_with_memory(mode),
        // LDX
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.load_index_x_with_memory(mode),
        // LDY
        0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.load_index_y_with_memory(mode),
        // STA
        0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.store_accumulator_in_memory(mode),
        // STX
        0x86 | 0x96 | 0x8E => self.store_index_x_in_memory(mode),
        // STY
        0x84 | 0x94 | 0x8C => self.store_index_y_in_memory(mode),
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
        0xC6 | 0xD6 | 0xCE | 0xDE => self.decrement_memory_by_one(mode),
        // DEX
        0xCA => self.decrement_index_x_by_one(),
        // DEY
        0x88 => self.decrement_index_y_by_one(),
        // INC
        0xE6 | 0xF6 | 0xEE | 0xFE => self.increment_memory_by_one(mode),
        // INX
        0xE8 => self.increment_index_x_by_one(),
        // INY
        0xC8 => self.increment_index_y_by_one(),

        // Arithmetic Operations
        // ADC
        0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.add_memory_to_accumulator_with_carry(mode),
        // SBC
        0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.subtract_memory_from_accumulator_with_borrow(mode),

        // Logical Operations
        // AND
        0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and_memory_with_accumulator(mode),
        // EOR
        0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.exclusive_or_memory_with_accumulator(mode),
        // ORA
        0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.or_memory_with_accumulator(mode),

        // Shift & Rotate Instructions
        // ASL
        0x0A => self.shift_left_one_bit_accumulator(),
        0x06 | 0x16 | 0x0E | 0x1E => self.shift_left_one_bit_memory(mode),
        // LSR
        0x4A => self.shift_one_bit_right_accumulator(),
        0x46 | 0x56 | 0x4E | 0x5E => self.shift_one_bit_right_memory(mode),
        // ROL
        0x2A => self.rotate_one_bit_left_accumulator(),
        0x26 | 0x36 | 0x2E | 0x3E => self.rotate_one_bit_left_memory(mode),
        // ROR
        0x6A => self.rotate_one_bit_right_accumulator(),
        0x66 | 0x76 | 0x6E | 0x7E => self.rotate_one_bit_right_memory(mode),

        // Flag Instructions
        // CLC
        0x18 => self.clear_carry_flag(),
        // CLD
        0xD8 => self.clear_decimal_mode(),
        // CLI
        0x58 => self.clear_interrupt_disable_bit(),
        // CLV
        0xB8 => self.clear_overflow_flag(),
        // SEC
        0x38 => self.set_carry_flag(),
        // SED
        0xF8 => self.set_decimal_mode(),
        // SEI
        0x78 => self.set_interrupt_disable_bit(),

        // Comparisons
        // CMP
        0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => self.compare_memory_with_accumulator(mode),
        // CPX
        0xE0 | 0xE4 | 0xEC => self.compare_memory_and_index_x(mode),
        // CPY
        0xC0 | 0xC4 | 0xCC => self.compare_memory_and_index_y(mode),

        // Conditional Branch Instructions
        // BCC
        0x90 => self.branch_on_carry_clear(),
        // BCS
        0xB0 => self.branch_on_carry_set(),
        // BEQ
        0xF0 => self.branch_on_result_zero(),
        // BMI
        0x30 => self.branch_on_result_minus(),
        // BNE
        0xD0 => self.branch_on_result_not_zero(),
        // BPL
        0x10 => self.branch_on_result_plus(),
        // BVC
        0x50 => self.branch_on_overflow_clear(),
        // BVS
        0x70 => self.branch_on_overflow_set(),

        // Jumps & Subroutines
        // JMP
        0x4C | 0x6C => self.jump_to_new_location(mode),
        // JSR
        0x20 => self.jump_to_new_location_saving_return_address(),
        // RTS
        0x60 => self.return_from_subroutine(),

        // Interrupts
        // BRK
        // 0x00 => self.force_break(),
        0x00 => return,
        // RTI
        0x40 => self.return_from_interrupt(),

        // Other
        // BIT
        0x24 | 0x2C => self.test_bits_in_memory_with_accumulator(mode),
        // NOP
        0xEA => {},

        // Illegal Opcodes
        // MOP implied 1byte	2cycles
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => {},
        // MOP immediate 2byte	2cycles
        0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {},
        // NOP need read data
        0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 | 0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
          todo!();
        },
        _ => {
          panic!("opcode {:x} not support", code);
        }
      }

      if program_counter_state == self.registers.program_counter {
        self.registers.program_counter += (opcode.length - 1) as u16;
      }

      callback(self);

      print!("\n");
    }
  }

}

impl CPU {
  /// Transfer Instructions

  /// LDA
  fn load_accumulator_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);

    self.registers.a = data;
    self.registers.set_nz_flags(self.registers.a);

    print!("LDA {:02X}\t{:?}", data, mode);
  }

  /// LDX
  fn load_index_x_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let param = self.bus.read(address);
    self.registers.x = param;
    self.registers.set_nz_flags(self.registers.x);
  }

  /// LDY
  fn load_index_y_with_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let param = self.bus.read(address);
    self.registers.y = param;
    self.registers.set_nz_flags(self.registers.y);
  }

  /// STA
  fn store_accumulator_in_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    self.bus.write(address, self.registers.a);
  }

  /// STX
  fn store_index_x_in_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    self.bus.write(address, self.registers.x);
  }

  /// STY
  fn store_index_y_in_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    self.bus.write(address, self.registers.y);
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
    let mut data = self.bus.read(address);
    data = data.wrapping_sub(1);
    self.bus.write(address, data);
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
    let mut data = self.bus.read(address);
    data = data.wrapping_add(1);
    self.bus.write(address, data);
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
    let data = self.bus.read(address);
    self.registers.add_to_a(data);
  }

  /// SBC
  /// `A - B = A + (-B)`, `-B = !B + 1`
  fn subtract_memory_from_accumulator_with_borrow(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);
    // WHY
    self.registers.add_to_a((data as i8).wrapping_neg().wrapping_sub(1) as u8);

    print!("SBC {:02X}\t{:?}", data, mode);
  }


  /// Logical Operations
  /// AND
  fn and_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);
    self.registers.a = self.registers.a & data;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// EOR
  fn exclusive_or_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);
    self.registers.a = self.registers.a ^ data;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// ORA
  fn or_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);
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
    let mut data = self.bus.read(address);

    self.registers.status.set(Flags::C, data & 0x80 == 0x80);
    data = data << 1;
    self.bus.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// ASL accumulator
  fn shift_left_one_bit_accumulator(&mut self) {
    self.registers.status.set(Flags::C, self.registers.a & 0x80 == 0x80);
    self.registers.a = self.registers.a << 1;
    self.registers.set_nz_flags(self.registers.a);
  }

  /// LSR
  fn shift_one_bit_right_memory(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let mut data = self.bus.read(address);
    self.registers.status.set(Flags::C, data & 0x01 == 1);
    data = data >> 1;
    self.bus.write(address, data);
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
    let mut data = self.bus.read(address);
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, data & 0x80 == 0x80);
    data = (data << 1) | (if carry { 0x01 } else { 0x00 });
    self.bus.write(address, data);
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
    let mut data = self.bus.read(address);
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, data & 0x01 == 0x01);

    data = (data >> 1) | (if carry { 0x80 } else { 0x00 });
    self.bus.write(address, data);
    self.registers.set_nz_flags(data);
  }

  /// ROR accumulator
  fn rotate_one_bit_right_accumulator(&mut self) {
    let carry = self.registers.status.contains(Flags::C);
    self.registers.status.set(Flags::C, self.registers.a & 0x01 == 0x01);
    self.registers.a = (self.registers.a >> 1) | (if carry { 0x80 } else { 0x00 });
    self.registers.set_nz_flags(self.registers.a);
  }


  /// Flag Instructions
  /// CLC
  fn clear_carry_flag(&mut self) {
    self.registers.status.remove(Flags::C);
  }

  /// CLD
  fn clear_decimal_mode(&mut self) {
    self.registers.status.remove(Flags::D);
  }

  /// CLI
  fn clear_interrupt_disable_bit(&mut self) {
    self.registers.status.remove(Flags::I);
  }

  /// CLV
  fn clear_overflow_flag(&mut self) {
    self.registers.status.remove(Flags::V);
  }

  /// SEC
  fn set_carry_flag(&mut self) {
    self.registers.status.insert(Flags::C);
  }

  /// SED
  fn set_decimal_mode(&mut self) {
    self.registers.status.insert(Flags::D);
  }

  /// SEI
  fn set_interrupt_disable_bit(&mut self) {
    self.registers.status.insert(Flags::I);
  }


  /// Comparisons
  /// Generally, comparison instructions subtract the operand from the given register without affecting this register.
  /// Flags are still set as with a normal subtraction and thus the relation of the two values becomes accessible by
  /// the Zero, Carry and Negative flags.
  /// (See the branch instructions below for how to evaluate flags.)
  ///
  /// | Relation R − Op    | Z | C |	N                 |
  /// |--------------------|---|---|--------------------|
  /// | Register < Operand | 0 | 0 | sign bit of result |
  /// | Register = Operand | 1 | 1 | 0                  |
  /// | Register > Operand | 0 | 1 | sign bit of result |
  fn compare_memory_with(&mut self, mode: &AddressingMode, rv: u8) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);
    self.registers.status.set(Flags::C, rv >= data);
    self.registers.set_nz_flags(rv.wrapping_sub(data));
  }

  /// CMP
  fn compare_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    self.compare_memory_with(mode, self.registers.a);
  }

  /// CPX
  fn compare_memory_and_index_x(&mut self, mode: &AddressingMode) {
    self.compare_memory_with(mode, self.registers.x);
  }

  /// CPY
  fn compare_memory_and_index_y(&mut self, mode: &AddressingMode) {
    self.compare_memory_with(mode, self.registers.y);
  }


  /// Conditional Branch Instructions
  ///
  /// Branch targets are relative, signed 8-bit address offsets.
  /// (An offset of #0 corresponds to the immedately following address — or a rather odd and expensive NOP.)
  fn branch(&mut self, condition: bool) {
    if condition {
      let offset = self.bus.read(self.registers.program_counter) as i8;
      self.registers.program_counter = self.registers.program_counter
        .wrapping_add(1).wrapping_add(offset as u16);
    }
  }

  /// BCC
  fn branch_on_carry_clear(&mut self) {
    self.branch(!self.registers.status.contains(Flags::C));
  }

  /// BCS
  fn branch_on_carry_set(&mut self) {
    self.branch(self.registers.status.contains(Flags::C));
  }

  /// BEQ
  fn branch_on_result_zero(&mut self) {
    self.branch(self.registers.status.contains(Flags::Z));
  }

  /// BMI
  fn branch_on_result_minus(&mut self) {
    self.branch(self.registers.status.contains(Flags::N));
  }

  /// BNE
  fn branch_on_result_not_zero(&mut self) {
    self.branch(!self.registers.status.contains(Flags::Z));
  }

  /// BPL
  fn branch_on_result_plus(&mut self) {
    self.branch(!self.registers.status.contains(Flags::N));
  }

  /// BVC
  fn branch_on_overflow_clear(&mut self) {
    self.branch(!self.registers.status.contains(Flags::V));
  }

  /// BVS
  fn branch_on_overflow_set(&mut self) {
    self.branch(self.registers.status.contains(Flags::V));
  }


  /// Jumps & Subroutines
  ///
  /// JSR and RTS affect the stack as the return address is pushed onto or pulled from the stack, respectively.
  /// (JSR will first push the high-byte of the return address [PC+2] onto the stack, then the low-byte.
  /// The stack will then contain, seen from the bottom or from the most recently added byte, [PC+2]-L [PC+2]-H.)

  /// JMP
  fn jump_to_new_location(&mut self, mode: &AddressingMode) {
    self.registers.program_counter = self.get_operand_address(mode);
  }

  /// JSR
  fn jump_to_new_location_saving_return_address(&mut self) {
    // TODO why -1
    self.stack_push_u16(self.registers.program_counter + 2 - 1);
    let address = self.get_operand_address(&AddressingMode::Absolute);
    self.registers.program_counter = address;

    print!("JSR {:02X}", address);
  }

  /// RTS
  fn return_from_subroutine(&mut self) {
    self.registers.program_counter = self.stack_pop_u16() + 1;
  }


  /// Interrupts
  /// BRK
  /// TODO
  // fn force_break(&mut self) {
  //   self.stack_push_u16(self.registers.program_counter.wrapping_add(2));
  //   let mut status = self.registers.status.clone();
  //   status.insert(Flags::B);
  //   status.insert(Flags::U);
  //   self.stack_push(status.bits());
  // }

  /// RTI
  fn return_from_interrupt(&mut self) {
    let status = self.stack_pop();
    self.registers.status = Flags::from_bits(status).expect(&format!("Status {:x} is not valid", status));
    self.registers.status.remove(Flags::B);
    self.registers.status.insert(Flags::U);
    self.registers.program_counter = self.stack_pop_u16();
  }


  /// Other
  /// BIT
  fn test_bits_in_memory_with_accumulator(&mut self, mode: &AddressingMode) {
    let address = self.get_operand_address(mode);
    let data = self.bus.read(address);
    self.registers.status.set(Flags::Z, self.registers.a & data == 0);
    self.registers.status.set(Flags::N, data & 0x80 == 0x80);
    self.registers.status.set(Flags::V, data & 0x40 == 0x40);
  }

}

#[cfg(test)]
mod test {
  // use super::*;
  // use super::status_flags::*;

  // #[test]
  // fn test_0xa9_lda_immidiate_load_data() {
  //   let mut cpu = CPU::new();
  //   cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
  //   assert_eq!(cpu.registers.a, 0x05);
  // }

  // #[test]
  // fn test_0xa9_lda_zero_flag() {
  //   let mut cpu = CPU::new();
  //   cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
  //   assert!(cpu.registers.status.contains(Flags::Z));
  // }

  // #[test]
  // fn test_inx_increment_index_x_by_one() {
  //   let mut cpu = CPU::new();
  //   cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);
  //   assert_eq!(cpu.registers.x, 2);
  // }

  // #[test]
  // fn test_5_ops_working_together() {
  //   let mut cpu = CPU::new();
  //   cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

  //   assert_eq!(cpu.registers.x, 0xc1);
  // }

}
