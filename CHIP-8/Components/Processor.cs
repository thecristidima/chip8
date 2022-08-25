// ReSharper disable InconsistentNaming

namespace CHIP8.Components;

public class Processor
{
#region Architecture

    private const int _height = 32;
    private const int _width = 64;
    private const int _redrawTickThreshold = 100; // no idea if this helps, so probably not
    
    private static int _redrawCounter;

    private byte[] Memory { get; } = new byte[4096];
    private byte[] V { get; } = new byte[16];

    private byte Vf
    {
        set => V[15] = value;
    }

    private ushort I { get; set; }
    private ushort[] Stack { get; } = new ushort[16];
    private byte DelayTimer { get; set; }
    private byte SoundTimer { get; set; }
    private ushort ProgramCounter { get; set; } = 0x200;
    private ushort StackPointer { get; set; }
    public byte[,] Vram { get; private set; } = new byte[_height, _width];
    public bool[] PressedKeys { get; set; } = new bool[16];
    public bool Redraw { get; private set; }

    public bool MakeSound => SoundTimer > 0;

    private bool UseOriginalShiftInstructions { get; } // seems to only be useful for TEST_5
    private bool EnableClipping { get; } // same as UseOriginalShiftInstructions

    /*
     * CHIP-8 uses 16 sprites, 0-9 A-F (in this order), each being 5 bytes long
     */
    private readonly byte[] _fontSet =
    {
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80 // F
    };

#endregion

#region SyncStuff

    private readonly object _lock = new();

#endregion

    public Processor(string romPath, bool useOriginalShiftInstructions, bool enableClipping)
    {
        Array.Copy(_fontSet, Memory, _fontSet.Length);

        var fullPath = Path.Combine(Environment.CurrentDirectory, romPath);
        if (!File.Exists(fullPath))
        {
            throw new FileNotFoundException("Cannot load ROM", fullPath);
        }

        using var fileStream = File.OpenRead(fullPath);
        using var binaryReader = new BinaryReader(fileStream);
        var romData = binaryReader.ReadBytes((int) fileStream.Length);

        Array.Copy(romData, 0, Memory, 512, romData.Length);

        UseOriginalShiftInstructions = useOriginalShiftInstructions;
        EnableClipping = enableClipping;
    }

    public void DecrementTimers()
    {
        lock (_lock)
        {
            if (DelayTimer > 0)
            {
                --DelayTimer;
            }

            if (SoundTimer > 0)
            {
                --SoundTimer;
            }
        }
    }

    public void RunCycle()
    {
        Redraw = false;

        var instruction = (ushort) (Memory[ProgramCounter] << 8 | Memory[ProgramCounter + 1]);

        ExecuteInstruction(instruction);

        if (Redraw)
        {
            _redrawCounter = 0;
        }
        else if (++_redrawCounter >= _redrawTickThreshold)
        {
            Redraw = true;
            _redrawCounter = 0;
        }
    }

    private void ExecuteInstruction(ushort instruction)
    {
        var nibbles = (
            (byte) ((instruction & 0xF000) >> 12),
            (byte) ((instruction & 0x0F00) >> 8),
            (byte) ((instruction & 0x00F0) >> 4),
            (byte) ((instruction & 0x000F) >> 0)
        );

        var nnn = (ushort) (instruction & 0x0FFF);
        var kk = (byte) (instruction & 0x00FF);
        var x = nibbles.Item2;
        var y = nibbles.Item3;
        var n = nibbles.Item4;

        var pcAction = nibbles switch
        {
            (0x00, 0x00, 0x0E, 0x00) => Op00E0(),
            (0x00, 0x00, 0x0E, 0x0E) => Op00EE(),
            (0x01, _, _, _) => Op1nnn(nnn),
            (0x02, _, _, _) => Op2nnn(nnn),
            (0x03, _, _, _) => Op3xkk(x, kk),
            (0x04, _, _, _) => Op4xkk(x, kk),
            (0x05, _, _, _) => Op5xy0(x, y),
            (0x06, _, _, _) => Op6xkk(x, kk),
            (0x07, _, _, _) => Op7xkk(x, kk),
            (0x08, _, _, 0x00) => Op8xy0(x, y),
            (0x08, _, _, 0x01) => Op8xy1(x, y),
            (0x08, _, _, 0x02) => Op8xy2(x, y),
            (0x08, _, _, 0x03) => Op8xy3(x, y),
            (0x08, _, _, 0x04) => Op8xy4(x, y),
            (0x08, _, _, 0x05) => Op8xy5(x, y),
            (0x08, _, _, 0x06) => Op8xy6(x, y),
            (0x08, _, _, 0x07) => Op8xy7(x, y),
            (0x08, _, _, 0x0E) => Op8xyE(x, y),
            (0x09, _, _, _) => Op9xy0(x, y),
            (0x0A, _, _, _) => OpAnnn(nnn),
            (0x0B, _, _, _) => OpBnnn(nnn),
            (0x0C, _, _, _) => OpCxkk(x, kk),
            (0x0D, _, _, _) => OpDxyn(x, y, n),
            (0x0E, _, 0x09, 0x0E) => OpEx9E(x),
            (0x0E, _, 0x0A, 0x01) => OpExA1(x),
            (0x0F, _, 0x00, 0x07) => OpFx07(x),
            (0x0F, _, 0x00, 0x0A) => OpFx0A(x),
            (0x0F, _, 0x01, 0x05) => OpFx15(x),
            (0x0F, _, 0x01, 0x08) => OpFx18(x),
            (0x0F, _, 0x01, 0x0E) => OpFx1E(x),
            (0x0F, _, 0x02, 0x09) => OpFx29(x),
            (0x0F, _, 0x03, 0x03) => OpFx33(x),
            (0x0F, _, 0x05, 0x05) => OpFx55(x),
            (0x0F, _, 0x06, 0x05) => OpFx65(x),
            _ => throw new InvalidOperationException($"Unknown instruction {instruction}")
        };

        switch (pcAction)
        {
            case AdvanceAction:
                ProgramCounter += 2;
                break;
            case SkipNextAction:
                ProgramCounter += 4;
                break;
            case JumpToAddressAction jumpAction:
                ProgramCounter = jumpAction.Address;
                break;
        }
    }

#region OpCodes

    /*
     *  00E0 - CLS
     *  Clears the screen
     */
    private ProgramCounterAction Op00E0()
    {
        Vram = new byte[_height, _width];

        return new AdvanceAction();
    }

    /*
     *  00EE - RET
     *  Return from a subroutine
     */
    private ProgramCounterAction Op00EE()
    {
        return new JumpToAddressAction((ushort) (Stack[StackPointer--] + 2));
    }

    /*
     *  1nnn - JMP addr
     *  Set PC to nnn
     */
    private ProgramCounterAction Op1nnn(ushort nnn)
    {
        return new JumpToAddressAction(nnn);
    }

    /*
     *  2nnn - CALL addr
     *  Set PC on top of stack, then set PC to nnn
     */
    private ProgramCounterAction Op2nnn(ushort nnn)
    {
        Stack[++StackPointer] = ProgramCounter;

        return new JumpToAddressAction(nnn);
    }

    /*
     *  3xkk - SE Vx, byte
     *  Skip next instruction if Vx == kk
     */
    private ProgramCounterAction Op3xkk(byte x, byte kk)
    {
        return V[x] == kk ? new SkipNextAction() : new AdvanceAction();
    }

    /*
     *  4xkk - SNE Vx, byte
     *  Skip next instruction if Vx != kk
     */
    private ProgramCounterAction Op4xkk(byte x, byte kk)
    {
        return V[x] != kk ? new SkipNextAction() : new AdvanceAction();
    }

    /*
     *  5xy0 - SE Vx, Vy
     *  Skip next instruction if Vx == Vy 
     */
    private ProgramCounterAction Op5xy0(byte x, byte y)
    {
        return V[x] == V[y] ? new SkipNextAction() : new AdvanceAction();
    }

    /*
     *  6xkk - LD Vx, byte
     *  Set Vx = kk
     */
    private ProgramCounterAction Op6xkk(byte x, byte kk)
    {
        V[x] = kk;

        return new AdvanceAction();
    }

    /*
     *  7xkk - ADD Vx, byte
     *  Set Vx = Vx + kk
     */
    private ProgramCounterAction Op7xkk(byte x, byte kk)
    {
        V[x] += kk;

        return new AdvanceAction();
    }

    /*
     *  8xy0 - LD Vx, Vy
     *  Set Vx = Vy
     */
    private ProgramCounterAction Op8xy0(byte x, byte y)
    {
        V[x] = V[y];

        return new AdvanceAction();
    }

    /*
     *  8xy1 - OR Vx, Vy
     *  Set Vx = Vx OR Vy
     */
    private ProgramCounterAction Op8xy1(byte x, byte y)
    {
        V[x] |= V[y];
        Vf = 0;

        return new AdvanceAction();
    }

    /*
     *  8xy2 - AND Vx, Vy
     *  Set Vx = Vx AND Vy
     */
    private ProgramCounterAction Op8xy2(byte x, byte y)
    {
        V[x] &= V[y];
        Vf = 0;

        return new AdvanceAction();
    }

    /*
     *  8xy3 - XOR Vx, Vy
     *  Set Vx = Vx XOR Vy
     */
    private ProgramCounterAction Op8xy3(byte x, byte y)
    {
        V[x] ^= V[y];
        Vf = 0;

        return new AdvanceAction();
    }

    /*
     *  8xy4 - ADD Vx, Vy
     *  Set Vx = Vx + Vy, Vf = carry
     */
    private ProgramCounterAction Op8xy4(byte x, byte y)
    {
        var setCarry = V[x] + V[y] > 255;

        V[x] = (byte) ((V[x] + V[y]) % 256);

        Vf = setCarry ? (byte) 1 : (byte) 0;

        return new AdvanceAction();
    }

    /*
     *  8xy5 - SUB Vx, Vy
     *  Set Vx = Vx - Vy, Vf = NOT borrow
     */
    private ProgramCounterAction Op8xy5(byte x, byte y)
    {
        var setCarry = V[x] > V[y];

        V[x] -= V[y];

        Vf = setCarry ? (byte) 1 : (byte) 0;

        return new AdvanceAction();
    }

    /*
     *  8xy6 - SHR Vx {, Vy}
     *  Set Vf = least significant bit of Vx, then Vx = Vx >> 1
     *
     *  Note: Original implementation actually set Vx = Vy before
     */
    private ProgramCounterAction Op8xy6(byte x, byte y)
    {
        if (UseOriginalShiftInstructions)
        {
            V[x] = V[y];
        }

        var vf = (byte) (V[x] % 2);

        V[x] >>= 1;

        Vf = vf;

        return new AdvanceAction();
    }

    /*
     *  8xy7 - SUBN Vx, Vy
     *  Set Vx = Vy - Vx, Vf = NOT borrow
     */
    private ProgramCounterAction Op8xy7(byte x, byte y)
    {
        var setCarry = V[y] > V[x];

        V[x] = (byte) (V[y] - V[x]);

        Vf = setCarry ? (byte) 1 : (byte) 0;

        return new AdvanceAction();
    }

    /*
     *  8xyE - SHL Vx {, Vy}
     *  Set Vf = most significant bit of Vx, then Vx = Vx << 1
     *
     *  Note: Original implementation actually set Vx = Vy before
     */
    private ProgramCounterAction Op8xyE(byte x, byte y)
    {
        if (UseOriginalShiftInstructions)
        {
            V[x] = V[y];
        }

        var vf = (V[x] & 0b10000000) != 0 ? (byte) 1 : (byte) 0;

        V[x] <<= 1;

        Vf = vf;

        return new AdvanceAction();
    }

    /*
     *  SNE Vx, Vy
     *  Skip next instruction if Vx != Vy
     */
    private ProgramCounterAction Op9xy0(byte x, byte y)
    {
        return V[x] != V[y] ? new SkipNextAction() : new AdvanceAction();
    }

    /*
     *  LD I, addr
     *  Set I = nnn
     */
    private ProgramCounterAction OpAnnn(ushort nnn)
    {
        I = nnn;

        return new AdvanceAction();
    }

    /*
     *  JMP V0, addr
     *  Set PC = nnn + V0
     */
    private ProgramCounterAction OpBnnn(ushort nnn)
    {
        return new JumpToAddressAction((ushort) (nnn + V[0]));
    }

    /*
     *  RND Vx, byte
     *  Set Vx = random byte AND kk
     */
    private ProgramCounterAction OpCxkk(byte x, byte kk)
    {
        var random = new Random();

        V[x] = (byte) (random.Next(0, 256) & kk);

        return new AdvanceAction();
    }

    /*
     *  DRW Vx, Vy, nibble
     *  Display n-byte sprite starting at memory location I at (Vx, Vy), set Vf = collision
     *
     *  The interpreter reads n bytes from memory, starting at the address stored in I.
     *  These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
     *  Sprites are XORed onto the existing screen.
     *  If this causes pixels to be erased, set Vf = 1, otherwise set Vf = 0.
     *  If the sprite is positioned so part of it is outside the coordinates of the display,
     *  it wraps around tot he opposite side of the screen.
     */
    private ProgramCounterAction OpDxyn(byte x, byte y, byte n)
    {
        Redraw = true;
        var foundCollision = false;

        var startX = V[x] % _width;
        var startY = V[y] % _height;

        for (var idx = 0; idx < n; ++idx)
        {
            var data = Memory[I + idx];
            
            var row = (byte) (startY + idx);

            if (row >= _height && EnableClipping)
            {
                continue;
            }

            row %= _height;

            for (var bitPosition = 0; bitPosition < 8; ++bitPosition)
            {
                var col = (byte) (startX + bitPosition);

                if (col >= _width && EnableClipping)
                {
                    continue;
                }

                col %= _width;

                var newBit = (byte) (data >> (7 - bitPosition) & 1);

                if (newBit == 1 && Vram[row, col] == 1)
                {
                    foundCollision = true;
                }

                Vram[row, col] ^= newBit;
            }
        }

        Vf = (byte) (foundCollision ? 1 : 0);

        return new AdvanceAction();
    }

    /*
     *  SKP Vx
     *  Skip next instruction if key with value Vx is pressed
     */
    private ProgramCounterAction OpEx9E(byte x)
    {
        return PressedKeys[V[x]] ? new SkipNextAction() : new AdvanceAction();
    }

    /*
     *  SKNP Vx
     *  Skip next instruction if key with value Vx is NOT pressed
     */
    private ProgramCounterAction OpExA1(byte x)
    {
        return !PressedKeys[V[x]] ? new SkipNextAction() : new AdvanceAction();
    }

    /*
     *  LD Vx, DT
     *  Set Vx = delay timer
     */
    private ProgramCounterAction OpFx07(byte x)
    {
        V[x] = DelayTimer;

        return new AdvanceAction();
    }

    /*
     *  LD Vx, Key
     *  Set Vx = value of pressed key; If no key is pressed, wait for key to be pressed
     */
    private ProgramCounterAction OpFx0A(byte x)
    {
        var pressedKey = Array.IndexOf(PressedKeys, true);

        if (pressedKey == -1)
        {
            return new WaitForKeyAction();
        }

        V[x] = (byte) pressedKey;

        return new AdvanceAction();
    }

    /*
     *  LD DT, Vx
     *  Set delay timer = Vx
     */
    private ProgramCounterAction OpFx15(byte x)
    {
        DelayTimer = V[x];

        return new AdvanceAction();
    }

    /*
     *  LD ST, Vx
     *  Set sound timer = Vx
     */
    private ProgramCounterAction OpFx18(byte x)
    {
        SoundTimer = V[x];

        return new AdvanceAction();
    }

    /*
     *  ADD I, Vx
     *  Set I = I + Vx
     */
    private ProgramCounterAction OpFx1E(byte x)
    {
        I += V[x];

        return new AdvanceAction();
    }

    /*
     *  LD F, Vx
     *  Set I = location of sprite for digit Vx
     */
    private ProgramCounterAction OpFx29(byte x)
    {
        I = (ushort) (V[x] * 5);

        return new AdvanceAction();
    }

    /*
     *  LD B, Vx 
     *  Store BCD representation of Vx in memory locations I, I+1, I+2
     */
    private ProgramCounterAction OpFx33(byte x)
    {
        Memory[I] = (byte) (V[x] / 100 % 10);
        Memory[I + 1] = (byte) (V[x] / 10 % 10);
        Memory[I + 2] = (byte) (V[x] % 10);

        return new AdvanceAction();
    }

    /*
     *  LD [I], Vx
     *  Store registers V0 through Vx in memory starting at location I
     *  At the end, set I = I + x + 1
     */
    private ProgramCounterAction OpFx55(byte x)
    {
        for (var idx = 0; idx <= x; ++idx)
        {
            Memory[I + idx] = V[idx];
        }

        I += (byte) (x + 1);

        return new AdvanceAction();
    }

    /*
     *  LD Vx, [I]
     *  Read registers V0 through Vx from memory starting at location I
     *  At the end, set I = I + x + 1
     */
    private ProgramCounterAction OpFx65(byte x)
    {
        for (var idx = 0; idx <= x; ++idx)
        {
            V[idx] = Memory[I + idx];
        }

        I += (byte) (x + 1);

        return new AdvanceAction();
    }

#endregion


    private abstract record ProgramCounterAction;

    private record AdvanceAction : ProgramCounterAction;

    private record SkipNextAction : ProgramCounterAction;

    private record WaitForKeyAction : ProgramCounterAction;

    private record JumpToAddressAction(ushort Address) : ProgramCounterAction;
}
