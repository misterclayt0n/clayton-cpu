#[derive(Debug)]
struct CPU {
    registers: [u8; 16],
    position_in_memory: usize,
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
}

impl CPU {
    fn new() -> CPU {
        CPU {
            registers: [0; 16],
            memory: [0; 4096],
            position_in_memory: 0,
            stack: [0; 16],
            stack_pointer: 0,
        }
    }

    fn load_program(&mut self, program: &[u8], start_address: usize) {
        self.memory[start_address..(start_address + program.len())].copy_from_slice(program);
    }

    fn read_opcode(&self) -> u16 {
        let p = self.position_in_memory;
        let op_byte1 = self.memory[p] as u16;
        let op_byte2 = self.memory[p + 1] as u16;

        op_byte1 << 8 | op_byte2
    }

    fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.position_in_memory += 2;

            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;

            let nnn = opcode & 0x0FFF;
            let kk = (opcode & 0x00FF) as u8;

            match (c, x, y, d) {
                (0, 0, 0, 0) => {
                    println!("terminating execution.");
                    return;
                }
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x1, _, _, _) => self.jmp(nnn),
                (0x2, _, _, _) => self.call(nnn),
                (0x3, _, _, _) => self.se(x, kk),
                (0x4, _, _, _) => self.sne(x, kk),
                (0x6, _, _, _) => self.ld(x, kk),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                (0x8, _, _, 0x5) => self.sub_xy(x, y),
                (0x8, _, _, 0x2) => self.and_xy(x, y),
                (0x8, _, _, 0x1) => self.or_xy(x, y),
                (0x8, _, _, 0x3) => self.xor_xy(x, y),
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sub_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, borrow) = arg1.overflowing_sub(arg2);
        self.registers[x as usize] = val;

        if borrow {
            self.registers[0xF] = 0;
        } else {
            self.registers[0xF] = 1;
        }
    }

    fn and_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        self.registers[x as usize] = arg1 & arg2;
    }

    fn or_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        self.registers[x as usize] = arg1 | arg2;
    }

    fn xor_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        self.registers[x as usize] = arg1 ^ arg2;
    }

    fn jmp(&mut self, addr: u16) {
        self.position_in_memory = addr as usize;
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("stack overflow");
        }

        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = addr as usize;
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("stack underflow");
        }

        self.stack_pointer -= 1;
        let addr = self.stack[self.stack_pointer];
        self.position_in_memory = addr as usize;
    }

    fn ld(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    fn se(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.position_in_memory += 2;
        }
    }

    fn sne(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.position_in_memory += 2;
        }
    }
}

fn main() {
    let mut cpu = CPU::new();

    let program: Vec<u8> = vec![
        0x60, 0x05, // LD V0, 5
        0x61, 0x0A, // LD V1, 10
        0x80, 0x15, // SUB V0, V1
        0x80, 0x14, // ADD V0, V1
        0x80, 0x12, // AND V0, V1
        0x80, 0x11, // OR V0, V1
        0x80, 0x13, // XOR V0, V1
        0x30, 0x00, // SE V0, 0
        0x40, 0x00, // SNE V0, 0
        0x12, 0x00, // JMP 0x200 (endless loop for demonstration)
    ];

    cpu.load_program(&program, 0x000);
    cpu.run();

    println!("Resultado do registrador V0: {}", cpu.registers[0]);
}
