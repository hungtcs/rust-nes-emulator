use crate::cpu::addressing_mode::AddressingMode;
use crate::cpu::CPU;
use crate::cpu::opcodes;
use std::collections::HashMap;

pub fn trace(cpu: &CPU) -> String {
  let ref opscodes: HashMap<u8, &'static opcodes::Opcode> = *opcodes::OPCODES_MAP;

  let code = cpu.bus.read(cpu.registers.program_counter);
  let ops = opscodes.get(&code).expect(&format!("CODE: {:X}", code));

  let begin = cpu.registers.program_counter;
  let mut hex_dump = vec![];
  hex_dump.push(code);

  let (mem_addr, stored_value) = match ops.mode {
      AddressingMode::Immediate | AddressingMode::Implicit => (0, 0),
      _ => {
          let addr = cpu.get_absolute_address(&ops.mode, begin + 1);
          (addr, cpu.bus.read(addr))
      }
  };

  let tmp = match ops.length {
      1 => match ops.code {
          0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
          _ => String::from(""),
      },
      2 => {
          let address: u8 = cpu.bus.read(begin + 1);
          // let value = cpu.bus.read(address));
          hex_dump.push(address);

          match ops.mode {
              AddressingMode::Immediate => format!("#${:02x}", address),
              AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
              AddressingMode::ZeroPageX => format!(
                  "${:02x},X @ {:02x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              AddressingMode::ZeroPageY => format!(
                  "${:02x},Y @ {:02x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              AddressingMode::IndexedIndirect => format!(
                  "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                  address,
                  (address.wrapping_add(cpu.registers.x)),
                  mem_addr,
                  stored_value
              ),
              AddressingMode::IndirectIndexed => format!(
                  "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                  address,
                  (mem_addr.wrapping_sub(cpu.registers.y as u16)),
                  mem_addr,
                  stored_value
              ),
              AddressingMode::Implicit => {
                  // assuming local jumps: BNE, BVS, etc....
                  let address: usize =
                      (begin as usize + 2).wrapping_add((address as i8) as usize);
                  format!("${:04x}", address)
              }

              _ => panic!(
                  "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                  ops.mode, ops.code
              ),
          }
      }
      3 => {
          let address_lo = cpu.bus.read(begin + 1);
          let address_hi = cpu.bus.read(begin + 2);
          hex_dump.push(address_lo);
          hex_dump.push(address_hi);

          let address = cpu.bus.read_u16(begin + 1);

          match ops.mode {
              AddressingMode::Implicit | AddressingMode::Indirect => {
                  if ops.code == 0x6c {
                      //jmp indirect
                      let jmp_addr = if address & 0x00FF == 0x00FF {
                          let lo = cpu.bus.read(address);
                          let hi = cpu.bus.read(address & 0xFF00);
                          (hi as u16) << 8 | (lo as u16)
                      } else {
                          cpu.bus.read_u16(address)
                      };

                      // let jmp_addr = cpu.bus.read_u16(address);
                      format!("(${:04x}) = {:04x}", address, jmp_addr)
                  } else {
                      format!("${:04x}", address)
                  }
              }
              AddressingMode::Absolute => {
                if ops.code == 0x4C || ops.code == 0x20 {
                  format!("${:04x}", address)
                } else {
                  format!("${:04x} = {:02x}", mem_addr, stored_value)
                }
              },
              AddressingMode::AbsoluteX => format!(
                  "${:04x},X @ {:04x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              AddressingMode::AbsoluteY => format!(
                  "${:04x},Y @ {:04x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              _ => panic!(
                  "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                  ops.mode, ops.code
              ),
          }
      }
      _ => String::from(""),
  };

  let hex_str = hex_dump
      .iter()
      .map(|z| format!("{:02x}", z))
      .collect::<Vec<String>>()
      .join(" ");
  let asm_str = format!("{:04x}  {:8} {: >4} {}", begin, hex_str, ops.mnemonic, tmp)
      .trim()
      .to_string();

  format!(
      "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
      asm_str, cpu.registers.a, cpu.registers.x, cpu.registers.y, cpu.registers.status, cpu.registers.stack_pointer,
  )
  .to_ascii_uppercase()
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::bus::Bus;
  use crate::cartridge::test::test_rom;

  #[test]
  fn test_format_trace() {
    let mut bus = Bus::new(test_rom());
    bus.write(100, 0xa2);
    bus.write(101, 0x01);
    bus.write(102, 0xca);
    bus.write(103, 0x88);
    bus.write(104, 0x00);

    let mut cpu = CPU::new(bus);
    cpu.registers.program_counter = 0x64;
    cpu.registers.a = 1;
    cpu.registers.x = 2;
    cpu.registers.y = 3;
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
      result.push(trace(cpu));
    });
    assert_eq!(
      "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
      result[0]
    );
    assert_eq!(
      "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
      result[1]
    );
    assert_eq!(
      "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
      result[2]
    );
  }

  #[test]
  fn test_format_mem_access() {
    let mut bus = Bus::new(test_rom());
    // ORA ($33), Y
    bus.write(100, 0x11);
    bus.write(101, 0x33);

    //data
    bus.write(0x33, 00);
    bus.write(0x34, 04);

    //target cell
    bus.write(0x400, 0xAA);

    let mut cpu = CPU::new(bus);
    cpu.registers.program_counter = 0x64;
    cpu.registers.y = 0;
    let mut result: Vec<String> = vec![];
    cpu.run_with_callback(|cpu| {
      result.push(trace(cpu));
    });
    assert_eq!(
      "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
      result[0]
    );
  }
}
