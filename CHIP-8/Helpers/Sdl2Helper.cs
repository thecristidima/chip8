using static SDL2.SDL;

namespace CHIP8.Helpers;

public static class Sdl2Helper
{
    public static IntPtr InitSdlRenderer(int scale)
    {
        // Initilizes SDL.
        if (SDL_Init(SDL_INIT_VIDEO) < 0)
        {
            throw new Exception($"There was an issue initializing SDL. {SDL_GetError()}");
        }

        // Create a new window given a title, size, and passes it a flag indicating it should be shown.
        var window = SDL_CreateWindow(
            "CHIP-8",
            SDL_WINDOWPOS_UNDEFINED, 
            SDL_WINDOWPOS_UNDEFINED, 
            64 * scale,
            32 * scale, 
            SDL_WindowFlags.SDL_WINDOW_SHOWN);

        if (window == IntPtr.Zero)
        {
            throw new Exception($"There was an issue creating the window. {SDL_GetError()}");
        }

        // Creates a new SDL hardware renderer using the default graphics device with VSYNC enabled.
        var renderer = SDL_CreateRenderer(
            window,
            -1,
            SDL_RendererFlags.SDL_RENDERER_ACCELERATED |
            SDL_RendererFlags.SDL_RENDERER_PRESENTVSYNC);

        if (renderer == IntPtr.Zero)
        {
            throw new Exception($"There was an issue creating the renderer. {SDL_GetError()}");
        }

        return renderer;
    }
}
