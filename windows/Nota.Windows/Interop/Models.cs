using System.Text.Json;

namespace Nota.Windows.Interop;

public sealed record Note(string Id, string Title, string Content, string[] Tags, bool IsPinned, string Created, string LastModified);
public sealed record NoteRow(string Id, string Title, string Preview, string Date, bool IsPinned);
public sealed record DeletedNote(string Id, string Title);
public sealed record Notification(string Message, string Tone);
public sealed record Recovery(string Reason, bool CanRestorePrevious);
public sealed record Prompt(string Title, string Body, string Action);
public sealed record Insets(double Start, double Top, double End, double Bottom);
public sealed record Layout(
    string Axis, bool DrawerFillsWidth, bool FooterStacks, bool ShowWriting, bool ShowPreview,
    double TitlePx, double BodyPx, Insets WritingInsets, Insets PreviewInsets, double SplitGapPx,
    double EditorMinHeightPx, double PreviewMinHeightPx);

public sealed record Frame(
    ulong Revision, ulong EditSequence, Note? SelectedNote, NoteRow[] Rows,
    DeletedNote[] RecentlyDeleted, string SearchInput, string? ActiveTag, string[] Tags, bool PinnedOnly,
    string ViewMode, string Theme, string SaveStatus, string BackupHealth, string BackupDetail,
    Notification? Notification, Recovery? Recovery, Prompt? Confirm,
    string SuggestedExportName, string DataDirectory, Layout Layout,
    string ListTitle, string ListBody, bool DrawerOpen, bool EditingTags, Prompt? TagCleanup,
    string Quarantine, bool FocusTitle);

public sealed record Reply(bool Ok, Frame Frame, JsonElement? Result);
public sealed class NotaException(string code, string message) : Exception(message)
{
    public string Code { get; } = code;
}
