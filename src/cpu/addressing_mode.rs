
/// CPU 寻址模式
#[derive(Debug)]
pub enum AddressingMode {
  /// 绝对寻址，完整的内存位置用作指令的参数。
  ///
  /// ```
  /// STA $C000 ;store the value in the accumulator at memory location $c000
  /// ```
  Absolute,

  /// 所有支持绝对寻址的指令（跳转指令除外）也可以选择采用单字节地址。
  /// 这种类型的寻址称为“零页”，即只有内存的第一页（前 256 个字节）是可访问的。
  /// 这更快，因为只需要查找一个字节，并且在汇编代码中占用的空间也更少。
  ZeroPage,

  /// 在这种模式下，给出一个零页地址，然后将 X 寄存器的值相加。
  ///
  /// ```
  /// LDX #$01   ;X is $01
  /// LDA #$aa   ;A is $aa
  /// STA $a0,X  ;Store the value of A at memory location $a1
  /// INX        ;Increment X
  /// STA $a0,X  ;Store the value of A at memory location $a2
  /// ```
  ///
  /// 如果加法的结果大于单个字节，则地址回绕。例如：
  ///
  /// ```
  /// LDX #$05
  /// STA $FF,X  ;Store the value of A at memory location $04
  /// ```
  ZeroPageX,

  /// 类似于于 `ZeroPageX`，但只能与 `LDX` 和 `STX` 一起使用。
  ZeroPageY,

  /// 类似于 ZeroPageX 或者 ZeroPageY 的绝对寻址版本
  ///
  /// ```
  /// LDX #$01
  /// STA $0200,X ;store the value of A at memory location $0201
  /// ```
  AbsoluteX,
  AbsoluteY,

  /// 立即寻址并不处理内存地址 —— 这是使用实际值的模式。
  /// 例如，`LDX #$01` 将值 `$01` 加载到 `X` 寄存器中。
  /// 这与零页指令 `LDX $01` 非常不同，后者将内存位置 `$01` 处的值加载到 `X` 寄存器中。
  Immediate,

  /// 相对寻址用于分支指令。
  /// 这些指令采用单个字节，用作与下一条指令的偏移量。
  ///
  /// ```
  ///   LDA #$01
  ///   CMP #$02
  ///   BNE not_equal
  ///   STA $22
  /// not_equal:
  ///   BRK
  /// ```
  ///
  /// hexdump: `a9 01 c9 02 d0 02 85 22 00`
  ///
  /// `A9` 和 `C9` 分别是 `immediate-addressed` 模式下的 LDA 和 CMP 指令。
  /// `01` 和 `02` 分别是这两个指令的参数。
  /// `d0` 是指令 `BNE`，他的参数是 `02`，这个指令的意思是跳过接下来的两个字节（即 STA $22 编译后的 85 22）。
  // Relative,

  /// 一些指令不处理内存位置（例如 INX）。所以称之为隐式寻址，即指令隐含。
  Implicit,

  /// 间接寻址使用绝对地址查找其他地址。
  /// 第一个地址给出地址的最低有效字节，下一个字节给出最高有效字节。
  ///
  /// ```
  /// LDA #$01
  /// STA $f0     ;$f0 is $01
  /// LDA #$cc
  /// STA $f1     ;$f1 is $cc
  /// JMP ($00f0) ;the value of address $00f0 is $cc01, dereferences to $cc01
  /// ```
  Indirect,

  /// 这个有点奇怪。这就像零页、X和间接页之间的交叉。基本上，取零页地址，将X寄存器的值添加到其中，然后使用该值查找两字节地址。例如：
  ///
  /// ```
  /// LDX #$01     ;X is $01
  /// LDA #$05
  /// STA $01      ;$01 is $05
  /// LDA #$07
  /// STA $02      ;$02 is $07
  /// LDY #$0a     ;Y is $0a
  /// STY $0705    ;$0705 is $0a
  /// LDA ($00,X)  ;load A from address $0705 ($00 + $01 is $01, and value of $01, $02 is $05, $07)
  /// ```
  ///
  /// 内存位置$01和$02分别包含值$05和$07。把（$00，X）想象成（$00+X）。在本例中，X是$01，因此简化为$01。
  /// 从这里开始，像标准间接寻址一样，将查找位于$01和$02（05和$07）的两个字节以形成地址$0705。
  /// 这是Y寄存器在前一条指令中存储的地址，因此A寄存器获得与Y相同的值。
  IndexedIndirect,

  /// 与 IndexedIndirect 相似，不过不是将寄存器添加到地址，而是将零页地址解引用，并将寄存器添加到结果地址。
  ///
  /// ```
  /// LDY #$01
  /// LDA #$03
  /// STA $01
  /// LDA #$07
  /// STA $02
  /// LDX #$0a
  /// STX $0704
  /// LDA ($01),Y
  /// ```
  ///
  /// 在这种情况下，($01) 在 $01 和 $02 处查找两个字节：$03 和 $07。这些构成地址 $0703。将 Y 寄存器的值添加到该地址，得到最终地址 $0704。
  IndirectIndexed,
}
