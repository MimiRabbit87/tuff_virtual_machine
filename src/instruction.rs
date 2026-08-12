pub enum Opcode {
    ClearScreen,        // 00E0
    Return,             // 00EE
    Jump(u16),          // 1nnn
    Call(u16),          // 2nnn
    SkipEq(u8, u8),     // 3xkk
    SkipNeq(u8, u8),    // 4xkk
    SkipEqReg(u8, u8),  // 5xy0
    LoadReg(u8, u8),    // 6xkk
    AddImm(u8, u8),     // 7xkk
    LoadRegReg(u8, u8), // 8xy0
    Or(u8, u8),         // 8xy1
    And(u8, u8),        // 8xy2
    Xor(u8, u8),        // 8xy3
    AddReg(u8, u8),     // 8xy4
    Sub(u8, u8),        // 8xy5
    Shr(u8, u8),        // 8xy6
    Subn(u8, u8),       // 8xy7
    Shl(u8, u8),        // 8xyE
    SkipNeqReg(u8, u8), // 9xy0
    LoadI(u16),         // Annn
    JumpOffset(u16),    // Bnnn
    Rand(u8, u8),       // Cxkk
    Draw(u8, u8, u8),   // Dxyn
    SkipKey(u8),        // Ex9E
    SkipNotKey(u8),     // ExA1
    GetDelay(u8),       // Fx07
    WaitKey(u8),        // Fx0A
    SetDelay(u8),       // Fx15
    SetSound(u8),       // Fx18
    AddI(u8),           // Fx1E
    LoadSprite(u8),     // Fx29
    Bcd(u8),            // Fx33
    StoreRegs(u8),      // Fx55
    LoadRegs(u8),       // Fx65
}
