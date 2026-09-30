using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Threading.Channels;
using Microsoft.Win32.SafeHandles;

namespace Nota.Windows.Interop;

public sealed class NativeSession : IAsyncDisposable
{
    private static readonly JsonSerializerOptions Json = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        PropertyNameCaseInsensitive = true
    };
    private sealed record Work(byte[] Request, bool Open, TaskCompletionSource<Reply> Completion);
    private readonly Channel<Work> queue = Channel.CreateUnbounded<Work>(new UnboundedChannelOptions { SingleReader = true });
    private readonly Task worker;
    private int disposed;

    private NativeSession() => worker = Task.Run(Run);

    public static async Task<(NativeSession Session, Reply Initial)> OpenAsync(string directory)
    {
        var session = new NativeSession();
        try
        {
            var reply = await session.Enqueue(new { data_directory = directory }, true);
            return (session, reply);
        }
        catch
        {
            await session.DisposeAsync();
            throw;
        }
    }

    public Task<Reply> ExecuteAsync(object request) => Enqueue(request, false);

    private Task<Reply> Enqueue(object request, bool open)
    {
        ObjectDisposedException.ThrowIf(Volatile.Read(ref disposed) != 0, this);
        var completion = new TaskCompletionSource<Reply>(TaskCreationOptions.RunContinuationsAsynchronously);
        if (!queue.Writer.TryWrite(new Work(JsonSerializer.SerializeToUtf8Bytes(request, Json), open, completion)))
            completion.SetException(new ObjectDisposedException(nameof(NativeSession)));
        return completion.Task;
    }

    private async Task Run()
    {
        SessionHandle? handle = null;
        Exception? fault = null;
        try
        {
            await foreach (var work in queue.Reader.ReadAllAsync())
            {
                if (fault is not null)
                {
                    work.Completion.SetException(fault);
                    continue;
                }
                try
                {
                    if (work.Open && NativeMethods.AbiVersion() != 1)
                        throw new NotaException("abi_version", "This Nota application and its core have incompatible versions. Reinstall Nota.");
                    NativeBuffer buffer = default;
                    int status;
                    try
                    {
                        if (work.Open)
                        {
                            status = NativeMethods.Open(work.Request, (nuint)work.Request.Length, out var pointer, out buffer);
                            if (pointer != IntPtr.Zero) handle = new SessionHandle(pointer);
                        }
                        else
                        {
                            if (handle is null || handle.IsInvalid) throw new NotaException("session_closed", "The notebook session is not open.");
                            status = NativeMethods.Execute(handle, work.Request, (nuint)work.Request.Length, out buffer);
                        }
                        if (buffer.Length > int.MaxValue) throw new NotaException("response_size", "The notebook response is too large.");
                        var text = buffer.Pointer == IntPtr.Zero ? "" : Marshal.PtrToStringUTF8(buffer.Pointer, (int)buffer.Length) ?? "";
                        using var document = JsonDocument.Parse(text);
                        var root = document.RootElement;
                        if (status != 0 || !root.GetProperty("ok").GetBoolean())
                        {
                            var error = root.GetProperty("error");
                            var exception = new NotaException(error.GetProperty("code").GetString() ?? "core_error", error.GetProperty("message").GetString() ?? "The operation failed.");
                            if (status == 2) fault = exception;
                            throw exception;
                        }
                        work.Completion.SetResult(JsonSerializer.Deserialize<Reply>(text, Json) ?? throw new JsonException("Empty notebook response."));
                    }
                    finally
                    {
                        NativeMethods.BufferFree(buffer);
                    }
                }
                catch (Exception exception)
                {
                    if (exception is DllNotFoundException or EntryPointNotFoundException or BadImageFormatException or JsonException)
                        fault = exception;
                    work.Completion.TrySetException(exception);
                }
            }
        }
        finally
        {
            handle?.Dispose();
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (Interlocked.Exchange(ref disposed, 1) == 0) queue.Writer.TryComplete();
        await worker;
    }

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct NativeBuffer
    {
        public readonly IntPtr Pointer;
        public readonly nuint Length;
    }

    private sealed class SessionHandle : SafeHandleZeroOrMinusOneIsInvalid
    {
        public SessionHandle(IntPtr pointer) : base(true) => SetHandle(pointer);
        protected override bool ReleaseHandle() { NativeMethods.Destroy(handle); return true; }
    }

    private static class NativeMethods
    {
        [DllImport("nota_ffi", EntryPoint = "nota_abi_version", CallingConvention = CallingConvention.Cdecl)]
        internal static extern uint AbiVersion();
        [DllImport("nota_ffi", EntryPoint = "nota_open", CallingConvention = CallingConvention.Cdecl)]
        internal static extern int Open(byte[] options, nuint length, out IntPtr session, out NativeBuffer reply);
        [DllImport("nota_ffi", EntryPoint = "nota_execute", CallingConvention = CallingConvention.Cdecl)]
        internal static extern int Execute(SessionHandle session, byte[] request, nuint length, out NativeBuffer reply);
        [DllImport("nota_ffi", EntryPoint = "nota_buffer_free", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void BufferFree(NativeBuffer buffer);
        [DllImport("nota_ffi", EntryPoint = "nota_destroy", CallingConvention = CallingConvention.Cdecl)]
        internal static extern void Destroy(IntPtr session);
    }
}
