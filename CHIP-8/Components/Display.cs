using static SDL2.SDL;

namespace CHIP8.Components;

public class Display
{
    private readonly IntPtr _renderer;
    private readonly int _scale;

    public Display(IntPtr renderer, int scale)
    {
        _renderer = renderer;
        _scale = scale;
    }

    public void Render(byte[,] vram)
    {
        SDL_SetRenderDrawColor(_renderer, 0, 0, 0, 255);
        SDL_RenderClear(_renderer);

        SDL_SetRenderDrawColor(_renderer, 0, 255, 0, 255);
        
        for (var row = 0; row < vram.GetLength(0); ++row)
        {
            for (var col = 0; col < vram.GetLength(1); ++col)
            {
                var y = row * _scale;
                var x = col * _scale;

                var (r, g, b) = GetColor(vram[row, col]);

                var rectangle = new SDL_Rect
                {
                    x = x,
                    y = y,
                    w = _scale,
                    h = _scale
                };
                
                SDL_SetRenderDrawColor(_renderer, r, g, b, 255);
                SDL_RenderFillRect(_renderer, ref rectangle);
            }
        }
        
        SDL_RenderPresent(_renderer);
    }

    private static (byte, byte, byte) GetColor(byte pixel)
        => pixel switch
        {
            0 => (0, 0, 0),
            _ => (0, 255, 0)
        };
}
