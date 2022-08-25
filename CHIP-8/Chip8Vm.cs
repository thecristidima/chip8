using CHIP8.Components;
using CHIP8.Helpers;

namespace CHIP8;

public class Chip8Vm
{
    private const int TimerFrequency = 60;
    private const int CpuFrequency = 500;

    private readonly Processor _cpu;
    private readonly Display _display;
    private readonly InputHandler _inputHandler;

    public Chip8Vm(string romPath, bool useOriginalShiftInstructions, bool enableClipping)
    {
        _cpu = new Processor(romPath, useOriginalShiftInstructions, enableClipping);

        var sdlRenderer = Sdl2Helper.InitSdlRenderer(scale: 10);

        _display = new Display(sdlRenderer, scale: 10);
        _inputHandler = new InputHandler();
    }

    public void Run()
    {
#pragma warning disable CS4014
        RunTimersAsync();
#pragma warning restore CS4014

        while (RunCpu())
        {
        }
    }

    private async Task RunTimersAsync()
    {
        var timer = new PeriodicTimer(TimeSpan.FromMilliseconds(1000.0 / TimerFrequency));

        while (await timer.WaitForNextTickAsync())
        {
            _cpu.DecrementTimers();
        }
    }

    private bool RunCpu()
    {
        var cpuTimer = new PeriodicTimer(TimeSpan.FromMilliseconds(1000.0 / CpuFrequency));

        while (cpuTimer.WaitForNextTickAsync().AsTask().Result)
        {
            var (exit, pressedKeys) = _inputHandler.Poll();

            if (exit)
            {
                return false;
            }

            _cpu.PressedKeys = pressedKeys!;
            _cpu.RunCycle();

            if (_cpu.Redraw)
            {
                _display.Render(_cpu.Vram);
            }

            if (_cpu.MakeSound)
            {
                Console.Beep();
            }
        }

        return true;
    }
}
