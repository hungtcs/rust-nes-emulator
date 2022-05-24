use bitflags::bitflags;

bitflags! {
  /// [Status Register Flags](http://wiki.nesdev.com/w/index.php/Status_flags) (bit 7 to bit 0)
  ///
  /// | Bit | Flag |    |
  /// |-----|------|-----------------------------------|
  /// |  7   | N   |	Negative                          |
  /// |  6   | V   |	Overflow                          |
  /// |  5   | -   |	ignored                           |
  /// |  4   | B   |	Break                             |
  /// |  3   | D   |	Decimal (use BCD for arithmetics) |
  /// |  2   | I   |	Interrupt (IRQ disable)           |
  /// |  1   | Z   |	Zero                              |
  /// |  0   | C   |	Carry                             |
  pub struct Flags: u8 {
    const C = 0b00000001;
    const Z = 0b00000010;
    const I = 0b00000100;
    const D = 0b00001000;
    const B = 0b00010000;
    const V = 0b01000000;
    const N = 0b10000000;
  }
}
