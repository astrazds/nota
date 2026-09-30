namespace Nota.Windows.Interop;

public static class EditorText
{
    public static string ToCore(string text) => text.Replace("\r\n", "\n").Replace('\r', '\n');

    public static string ToNative(string text) => ToCore(text).Replace('\n', '\r');

    public static int ToCoreOffset(string nativeText, int utf16Offset)
        => ToCore(nativeText[..Math.Clamp(utf16Offset, 0, nativeText.Length)]).Length;

    public static int ToNativeOffset(string coreText, int utf16Offset)
        => ToNative(coreText[..Math.Clamp(utf16Offset, 0, coreText.Length)]).Length;
}
