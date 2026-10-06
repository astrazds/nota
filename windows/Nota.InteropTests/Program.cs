using System.Runtime.InteropServices;
using Nota.Windows.Interop;

if (args.Length > 1) throw new ArgumentException("Usage: Nota.InteropTests [absolute path to nota_ffi.dll]");
if (args.Length == 1)
{
    var dll = Path.GetFullPath(args[0]);
    NativeLibrary.SetDllImportResolver(typeof(NativeSession).Assembly, (name, _, _) => name == "nota_ffi" ? NativeLibrary.Load(dll) : IntPtr.Zero);
}

var directory = Path.Combine(Path.GetTempPath(), "nota-interop-" + Guid.NewGuid().ToString("N"));
Directory.CreateDirectory(directory);
Console.WriteLine($"Testing the actual native DLL with profile {directory}");

static void Check(bool condition, string message)
{
    if (!condition) throw new InvalidOperationException(message);
    Console.WriteLine("PASS " + message);
}

static async Task RejectAsync(NativeSession session, object command, string message)
{
    try { await session.ExecuteAsync(command); }
    catch (NotaException) { Console.WriteLine("PASS " + message); return; }
    throw new InvalidOperationException(message);
}

var opened = await NativeSession.OpenAsync(directory);
string savedId;
await using (var session = opened.Session)
{
    var created = await session.ExecuteAsync(new { command = "new_note" });
    savedId = created.Snapshot.SelectedNote!.Id;
    var sequence = created.Snapshot.EditSequence + 1;
    var nativeText = "😀 hello\r- [ ] A task\r\nLast line\r";
    var edited = await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = sequence, title = "Unicode notebook", content = EditorText.ToCore(nativeText), tags_input = "work, café" });
    Check(edited.Snapshot.SelectedNote?.Content == "😀 hello\n- [ ] A task\nLast line\n", "Windows Return and pasted CRLF persist as Markdown LF paragraphs through the C ABI");
    Check(edited.Snapshot.SelectedNote!.Tags.Contains("café"), "Unicode tags cross the C ABI");
    await RejectAsync(session, new { command = "edit_note", id = savedId, edit_sequence = sequence, title = "Stale", content = "wrong", tags_input = "" }, "Stale edit sequences are rejected");

    var formatted = await session.ExecuteAsync(new { command = "format", content = "😀 hello", start_utf16 = 3, end_utf16 = 8, kind = "bold" });
    Check(formatted.Result!.Value.GetProperty("content").GetString() == "😀 **hello**", "Formatting uses native UTF-16 selection offsets");
    Check(formatted.Result.Value.GetProperty("caret_utf16").GetInt32() <= "😀 **hello**".Length, "Formatting returns a UTF-16 caret inside the result");

    foreach (var (kind, expected) in new[] {
        ("heading", "Before\n## 日本😀\nAfter"),
        ("bullet_list", "Before\n- 日本😀\nAfter"),
        ("link", "Before\n[日本😀](https://example.com)\nAfter")
    })
    {
        var focusFormat = await session.ExecuteAsync(new { command = "format", content = "Before\n日本😀\nAfter", start_utf16 = 7, end_utf16 = 11, kind });
        var result = focusFormat.Result!.Value;
        Check(result.GetProperty("content").GetString() == expected, $"{kind} formats the complete Unicode selection through the C ABI");
        Check(expected[result.GetProperty("caret_utf16").GetInt32()..] == "\nAfter", $"{kind} caret follows the formatted selection");
    }

    var nativeSelection = "😀\r\nhello\rworld";
    var multiline = await session.ExecuteAsync(new { command = "format", content = EditorText.ToCore(nativeSelection), start_utf16 = EditorText.ToCoreOffset(nativeSelection, 4), end_utf16 = EditorText.ToCoreOffset(nativeSelection, 9), kind = "bold" });
    var multilineResult = multiline.Result!.Value.GetProperty("content").GetString()!;
    Check(multilineResult == "😀\n**hello**\nworld", "UTF-16 selection maps across CRLF after an emoji without formatting adjacent paragraphs");
    Check(EditorText.ToNative(multilineResult) == "😀\r**hello**\rworld", "Formatted Markdown returns native TextBox paragraph separators");
    var caret = EditorText.ToNativeOffset(multilineResult, multiline.Result.Value.GetProperty("caret_utf16").GetInt32());
    Check(EditorText.ToNative(multilineResult)[..caret].EndsWith("**hello**", StringComparison.Ordinal), "Formatting caret maps to the end of the selected phrase in native text");
    var paragraphs = await session.ExecuteAsync(new { command = "preview", layout = "reading", title = "", content = EditorText.ToCore("First paragraph\r\rSecond paragraph"), dark = false });
    var paragraphHtml = paragraphs.Result!.Value.GetProperty("html").GetString()!;
    Check(paragraphHtml.Contains("<p>First paragraph</p>") && paragraphHtml.Contains("<p>Second paragraph</p>"), "Windows Return produces separate Markdown paragraphs in shared preview");
    var headingPreview = await session.ExecuteAsync(new { command = "preview", layout = "reading", title = "Notebook", content = "# Notebook\n\nBody\n\n# Another heading", dark = false });
    var headingHtml = headingPreview.Result!.Value.GetProperty("html").GetString()!;
    Check(!headingHtml.Contains("<h1>Notebook</h1>") && headingHtml.Contains("<p>Body</p>") && headingHtml.Contains("<h1>Another heading</h1>"), "Preview leaves the native title in charge and preserves other Markdown content");
    foreach (var (layout, font) in new[] { ("reading", "font:18px/1.9 'Gelasio'"), ("split", "font:15px/1.9 'Gelasio'"), ("split_narrow", "font:14px/1.9 'Gelasio'") })
    {
        var layoutPreview = await session.ExecuteAsync(new { command = "preview", layout, title = "Notebook", content = "Body **text**", dark = false });
        var layoutHtml = layoutPreview.Result!.Value.GetProperty("html").GetString()!;
        Check(layoutHtml.Contains(font) && layoutHtml.Contains("<p>Body <strong>text</strong></p>"), $"{layout} uses its preview type size while retaining Markdown content");
    }

    await session.ExecuteAsync(new { command = "set_view_mode", mode = "preview" });
    var next = await session.ExecuteAsync(new { command = "new_note" });
    Check(next.Snapshot.SelectedNote!.Id != savedId, "New note has a distinct identity");
    Check(next.Snapshot.ViewMode == "write", "New note leaves Preview ready for writing");
    await session.ExecuteAsync(new { command = "toggle_pin", id = savedId });
    var pinned = await session.ExecuteAsync(new { command = "filter_pinned", pinned = true });
    Check(pinned.Snapshot.Rows.Length == 1 && pinned.Snapshot.Rows[0].Id == savedId && pinned.Snapshot.SelectedNote!.Id == next.Snapshot.SelectedNote.Id, "Pinned navigation filters rows without replacing the selected note");
    var allNotes = await session.ExecuteAsync(new { command = "filter_pinned", pinned = false });
    Check(allNotes.Snapshot.Rows.Length == 2, "All notes restores unpinned rows");
    await session.ExecuteAsync(new { command = "toggle_pin", id = savedId });
    var targeted = await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = sequence + 1, title = "Unicode notebook", content = "😀 hello\n- [ ] A task\nIdentity checked", tags_input = "work, café" });
    Check(targeted.Snapshot.SelectedNote?.Id == next.Snapshot.SelectedNote.Id && targeted.Snapshot.SelectedNote.Content == "", "A queued edit targets its explicit note and preserves current selection");
    var selectedTarget = await session.ExecuteAsync(new { command = "select_note", id = savedId });
    Check(selectedTarget.Snapshot.SelectedNote!.Content.Contains("Identity checked"), "An explicit edit updates the intended note");
    await session.ExecuteAsync(new { command = "delete_note", id = next.Snapshot.SelectedNote.Id });
    await session.ExecuteAsync(new { command = "toggle_pin", id = savedId });
    var backupPath = Path.Combine(directory, "export.json");
    var exported = await session.ExecuteAsync(new { command = "export_backup", path = backupPath });
    Check(File.Exists(backupPath), "Backup export writes the chosen file");
    var backup = await File.ReadAllTextAsync(backupPath);
    await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = exported.Snapshot.EditSequence + 1, title = "Changed after export", content = "replacement", tags_input = "" });
    var staged = await session.ExecuteAsync(new { command = "import_backup", json = backup });
    Check(staged.Snapshot.PendingImport is { NotesToReplace: > 0 }, "Backup import previews same-identity replacements");
    var imported = await session.ExecuteAsync(new { command = "confirm_import" });
    Check(imported.Snapshot.SelectedNote?.Id == savedId && imported.Snapshot.SelectedNote.Title == "Unicode notebook", "Confirmed import replaces the selected note with the same identity");

    var html = await session.ExecuteAsync(new { command = "preview", layout = "reading", title = "<script>alert(1)</script>", content = "# A note\n<script>alert(2)</script>\n![remote](https://example.com/image.png)", dark = false });
    var document = html.Result!.Value.GetProperty("html").GetString()!;
    Check(document.Contains("Content-Security-Policy") && !document.Contains("<script>alert(2)</script>"), "Preview has a content policy and sanitizes raw scripts");
    var blocked = await session.ExecuteAsync(new { command = "external_navigation", uri = "file:///C:/Windows/win.ini", user_activated = true });
    Check(blocked.Result!.Value.GetProperty("uri").ValueKind == System.Text.Json.JsonValueKind.Null, "Preview denies local file navigation");
    var automatic = await session.ExecuteAsync(new { command = "external_navigation", uri = "https://example.com", user_activated = false });
    Check(automatic.Result!.Value.GetProperty("uri").ValueKind == System.Text.Json.JsonValueKind.Null, "Preview denies automatic external navigation");

    await session.ExecuteAsync(new { command = "set_theme", theme = "dark" });
    var transitionPath = Path.Combine(directory, "transition.json");
    await session.ExecuteAsync(new { command = "export_transition", path = transitionPath });
    await session.ExecuteAsync(new { command = "delete_note", id = savedId });
    var deleted = await session.ExecuteAsync(new { command = "snapshot" });
    Check(deleted.Snapshot.RecentlyDeleted.Any(note => note.Id == savedId), "Deleted notes remain recoverable");
    var transition = await File.ReadAllTextAsync(transitionPath);
    await RejectAsync(session, new { command = "import_transition", json = transition }, "Complete notebook restore refuses a nonempty profile");
    var destination = await NativeSession.OpenAsync(Path.Combine(directory, "restored"));
    await using (var target = destination.Session)
    {
        var restored = await target.ExecuteAsync(new { command = "import_transition", json = transition });
        Check(restored.Snapshot.Rows.Any(note => note.Id == savedId) && restored.Snapshot.Theme == "dark", "Complete notebook restore preserves notes and theme in an empty profile");
    }
    await session.ExecuteAsync(new { command = "restore_note", id = savedId });
    await session.ExecuteAsync(new { command = "select_note", id = savedId });
    var flushed = await session.ExecuteAsync(new { command = "flush" });
    Check(flushed.Snapshot.SaveStatus == "saved", "Flush acknowledges durable save");
}

var reopened = await NativeSession.OpenAsync(directory);
await using (var session = reopened.Session)
{
    try
    {
        var duplicate = await NativeSession.OpenAsync(directory);
        await duplicate.Session.DisposeAsync();
        throw new InvalidOperationException("A second writer must not open the same profile");
    }
    catch (NotaException exception) when (exception.Code == "profile_in_use")
    {
        Console.WriteLine("PASS Shared core prevents concurrent profile writers");
    }
    var selected = await session.ExecuteAsync(new { command = "select_note", id = savedId });
    Check(selected.Snapshot.SelectedNote?.Title == "Unicode notebook" && selected.Snapshot.SelectedNote.IsPinned, "Saved note and pin survive destroy and reopen");
    Check(selected.Snapshot.Theme == "dark", "Theme survives destroy and reopen");
    var reads = Enumerable.Range(0, 30).Select(_ => session.ExecuteAsync(new { command = "snapshot" }));
    var snapshots = await Task.WhenAll(reads);
    Check(snapshots.All(reply => reply.Ok), "Concurrent managed callers serialize across one native session");

    using (var locked = new FileStream(Path.Combine(directory, "collection.json"), FileMode.Open, FileAccess.Read, FileShare.Read))
    {
        await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = selected.Snapshot.EditSequence + 1, title = "Unicode notebook", content = "Pending retry survives a failed save", tags_input = "work" });
        await RejectAsync(session, new { command = "flush" }, "Flush reports a Windows sharing violation without losing pending changes");
    }
    var retried = await session.ExecuteAsync(new { command = "flush" });
    Check(retried.Snapshot.SaveStatus == "saved" && retried.Snapshot.SelectedNote?.Content == "Pending retry survives a failed save", "A failed flush can be retried after storage becomes writable");
}

var recoveryDirectory = Path.Combine(directory, "recovery");
var recoverable = await NativeSession.OpenAsync(recoveryDirectory);
await using (var session = recoverable.Session)
{
    var created = await session.ExecuteAsync(new { command = "new_note" });
    var id = created.Snapshot.SelectedNote!.Id;
    var previous = await session.ExecuteAsync(new { command = "edit_note", id, edit_sequence = created.Snapshot.EditSequence + 1, title = "Previous saved note", content = "Preserve me", tags_input = "" });
    await session.ExecuteAsync(new { command = "flush" });
    await session.ExecuteAsync(new { command = "edit_note", id, edit_sequence = previous.Snapshot.EditSequence + 1, title = "Current saved note", content = "Newer content", tags_input = "" });
    await session.ExecuteAsync(new { command = "flush" });
}
var corruptBytes = System.Text.Encoding.UTF8.GetBytes("unreadable notebook ♥");
await File.WriteAllBytesAsync(Path.Combine(recoveryDirectory, "collection.json"), corruptBytes);
var recovery = await NativeSession.OpenAsync(recoveryDirectory);
await using (var session = recovery.Session)
{
    Check(recovery.Initial.Snapshot.Recovery is { CanRestorePrevious: true }, "Corrupt storage enters explicit recovery with a previous snapshot");
    await RejectAsync(session, new { command = "new_note" }, "Recovery blocks ordinary mutations");
    var restored = await session.ExecuteAsync(new { command = "restore_previous" });
    Check(restored.Snapshot.Recovery is null && restored.Snapshot.Rows.Any(note => note.Title == "Previous saved note"), "Recovery restores the previous saved notebook");
    Check(Directory.GetFiles(recoveryDirectory, "collection.corrupt-*.json").Any(path => File.ReadAllBytes(path).SequenceEqual(corruptBytes)), "Recovery preserves the exact corrupt bytes");
}

var emptyRecoveryDirectory = Path.Combine(directory, "empty-recovery");
Directory.CreateDirectory(emptyRecoveryDirectory);
await File.WriteAllBytesAsync(Path.Combine(emptyRecoveryDirectory, "collection.json"), corruptBytes);
var emptyRecovery = await NativeSession.OpenAsync(emptyRecoveryDirectory);
await using (var session = emptyRecovery.Session)
{
    Check(emptyRecovery.Initial.Snapshot.Recovery is { CanRestorePrevious: false }, "Recovery accurately reports a missing previous snapshot");
    var empty = await session.ExecuteAsync(new { command = "start_empty" });
    Check(empty.Snapshot.Recovery is null && empty.Snapshot.Rows.Length == 0, "Recovery can start a new empty notebook");
    Check(Directory.GetFiles(emptyRecoveryDirectory, "collection.corrupt-*.json").Any(path => File.ReadAllBytes(path).SequenceEqual(corruptBytes)), "Starting empty preserves the exact corrupt bytes");
}
Console.WriteLine($"All interop checks passed. Test profile retained at {directory}");
