pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod trace;

use self::bus::Bus;
use self::trace::trace;
use self::cpu::CPU;
use self::cartridge::Cartridge;

fn main() {
  let bytes: Vec<u8> = std::fs::read("nestest.nes").unwrap();
  let cartridge = Cartridge::new(&bytes).unwrap();

  let bus = Bus::new(cartridge);
  let mut cpu = CPU::new(bus);
  cpu.reset();
  cpu.registers.program_counter = 0xC000;

  cpu.run_with_callback(
    move |cpu| {
      println!("{}", trace(cpu));
    }
  );

}
