using CHIP8;

if (args.Length < 2)
{
    throw new ArgumentException("Usage: ROM_PATH USE_ORIG_SHIFT_INSTR ENABLE_CLIPPING");
}

var romPath = args[0];
var __ = bool.TryParse(args[1], out var useOriginalShiftInstructions);

var enableClipping = false;
if (args.Length >= 3)
{
    bool.TryParse(args[2], out var enableClippingArg);
    enableClipping = enableClipping || enableClippingArg;
}

var vm = new Chip8Vm(romPath, useOriginalShiftInstructions, enableClipping);
vm.Run();
