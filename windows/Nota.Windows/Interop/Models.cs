using System.Text.Json;

namespace Nota.Windows.Interop;

public sealed record Note(string Id, string Title, string Content, string[] Tags, bool IsPinned, string Created, string LastModified);
public sealed record NoteRow(string Id, string Title, string Preview, string Date, bool IsPinned);
public sealed record DeletedNote(string Id, string Title);
public sealed record Notification(string Message, string Tone);
public sealed record Recovery(string Reason, bool CanRestorePrevious);
public sealed record ImportPreview(int TotalImportedNotes, int NotesToAdd, int NotesToReplace);

public sealed record Snapshot(
    ulong Revision, ulong EditSequence, Note? SelectedNote, NoteRow[] Rows,
    DeletedNote[] RecentlyDeleted, string SearchInput, string? ActiveTag, string[] Tags, bool PinnedOnly,
    string ViewMode, string Theme, string SaveStatus, string BackupHealth,
    Notification? Notification, Recovery? Recovery, ImportPreview? PendingImport, string DataDirectory);

public sealed record Reply(bool Ok, Snapshot Snapshot, JsonElement? Result);
public sealed class NotaException(string code, string message) : Exception(message)
{
    public string Code { get; } = code;
}
