using static SDL2.SDL;

namespace CHIP8.Components;

public class InputHandler
{
    private readonly Dictionary<int, int> _keyMap = new()
    {
        { (int) SDL_Scancode.SDL_SCANCODE_1, 0x01 },
        { (int) SDL_Scancode.SDL_SCANCODE_2, 0x02 },
        { (int) SDL_Scancode.SDL_SCANCODE_3, 0x03 },
        { (int) SDL_Scancode.SDL_SCANCODE_4, 0x0C },
        { (int) SDL_Scancode.SDL_SCANCODE_Q, 0x04 },
        { (int) SDL_Scancode.SDL_SCANCODE_W, 0x05 },
        { (int) SDL_Scancode.SDL_SCANCODE_E, 0x06 },
        { (int) SDL_Scancode.SDL_SCANCODE_R, 0x0D },
        { (int) SDL_Scancode.SDL_SCANCODE_A, 0x07 },
        { (int) SDL_Scancode.SDL_SCANCODE_S, 0x08 },
        { (int) SDL_Scancode.SDL_SCANCODE_D, 0x09 },
        { (int) SDL_Scancode.SDL_SCANCODE_F, 0x0E },
        { (int) SDL_Scancode.SDL_SCANCODE_Z, 0x0A },
        { (int) SDL_Scancode.SDL_SCANCODE_X, 0x00 },
        { (int) SDL_Scancode.SDL_SCANCODE_C, 0x0B },
        { (int) SDL_Scancode.SDL_SCANCODE_V, 0x0F }
    };

    public (bool, bool[]?) Poll()
    {
        while (SDL_PollEvent(out var sdlEvent) == 1)
        {
            if (sdlEvent.type is SDL_EventType.SDL_QUIT)
            {
                return (true, null);
            }
        }

        return (false, GetPressedKeys());
    }

    private unsafe bool[] GetPressedKeys()
    {
        var pressedKeys = new bool[16];
        
        var keyboardState = SDL_GetKeyboardState(out _);
        var state = (byte*) keyboardState.ToPointer();

        foreach (var (scanCode, keyIndex) in _keyMap)
        {
            pressedKeys[keyIndex] = state[scanCode] != 0;
        }
        
        return pressedKeys;
    }
}
