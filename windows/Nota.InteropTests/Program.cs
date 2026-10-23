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
    savedId = created.Frame.SelectedNote!.Id;
    var sequence = created.Frame.EditSequence + 1;
    var nativeText = "😀 hello\r- [ ] A task\r\nLast line\r";
    var edited = await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = sequence, title = "Unicode notebook", content = EditorText.ToCore(nativeText), tags_input = "work, café" });
    Check(edited.Frame.SelectedNote?.Content == "😀 hello\n- [ ] A task\nLast line\n", "Windows Return and pasted CRLF persist as Markdown LF paragraphs through the C ABI");
    Check(edited.Frame.SelectedNote!.Tags.Contains("café"), "Unicode tags cross the C ABI");
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
    var paragraphs = await session.ExecuteAsync(new { command = "preview", title = "", content = EditorText.ToCore("First paragraph\r\rSecond paragraph") });
    var paragraphHtml = paragraphs.Result!.Value.GetProperty("html").GetString()!;
    Check(paragraphHtml.Contains("<p>First paragraph</p>") && paragraphHtml.Contains("<p>Second paragraph</p>"), "Windows Return produces separate Markdown paragraphs in shared preview");
    var headingPreview = await session.ExecuteAsync(new { command = "preview", title = "Notebook", content = "# Notebook\n\nBody\n\n# Another heading" });
    var headingHtml = headingPreview.Result!.Value.GetProperty("html").GetString()!;
    Check(!headingHtml.Contains("<h1>Notebook</h1>") && headingHtml.Contains("<p>Body</p>") && headingHtml.Contains("<h1>Another heading</h1>"), "Preview leaves the native title in charge and preserves other Markdown content");
    foreach (var (width, mode, font) in new[] { (1180.0, "write", "font:18px/1.9 'Gelasio'"), (800.0, "split", "font:15px/1.9 'Gelasio'"), (760.0, "split", "font:14px/1.9 'Gelasio'"), (560.0, "split", "font:14px/1.9 'Gelasio'") })
    {
        await session.ExecuteAsync(new { command = "resize", width_px = width });
        await session.ExecuteAsync(new { command = "set_view_mode", mode });
        var layoutPreview = await session.ExecuteAsync(new { command = "preview", title = "Notebook", content = "Body **text**" });
        var layoutHtml = layoutPreview.Result!.Value.GetProperty("html").GetString()!;
        Check(layoutHtml.Contains(font) && layoutHtml.Contains("<p>Body <strong>text</strong></p>"), $"{width} {mode} uses its preview type size while retaining Markdown content");
        if (width == 560)
        {
            Check(layoutPreview.Frame.Layout.Axis == "vertical" && layoutPreview.Frame.Layout.EditorMinHeightPx == 340 && layoutPreview.Frame.Layout.PreviewMinHeightPx == 500, "560 stacks the preview at the shared 500 minimum");
        }
    }

    await session.ExecuteAsync(new { command = "set_view_mode", mode = "preview" });
    var next = await session.ExecuteAsync(new { command = "new_note" });
    Check(next.Frame.SelectedNote!.Id != savedId, "New note has a distinct identity");
    Check(next.Frame.ViewMode == "write", "New note leaves Preview ready for writing");
    await session.ExecuteAsync(new { command = "toggle_pin", id = savedId });
    var pinned = await session.ExecuteAsync(new { command = "filter_pinned", pinned = true });
    Check(pinned.Frame.Rows.Length == 1 && pinned.Frame.Rows[0].Id == savedId && pinned.Frame.SelectedNote!.Id == next.Frame.SelectedNote.Id, "Pinned navigation filters rows without replacing the selected note");
    var allNotes = await session.ExecuteAsync(new { command = "filter_pinned", pinned = false });
    Check(allNotes.Frame.Rows.Length == 2, "All notes restores unpinned rows");
    await session.ExecuteAsync(new { command = "toggle_pin", id = savedId });
    var targeted = await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = sequence + 1, title = "Unicode notebook", content = "😀 hello\n- [ ] A task\nIdentity checked", tags_input = "work, café" });
    Check(targeted.Frame.SelectedNote?.Id == next.Frame.SelectedNote.Id && targeted.Frame.SelectedNote.Content == "", "A queued edit targets its explicit note and preserves current selection");
    var selectedTarget = await session.ExecuteAsync(new { command = "select_note", id = savedId });
    Check(selectedTarget.Frame.SelectedNote!.Content.Contains("Identity checked"), "An explicit edit updates the intended note");
    await session.ExecuteAsync(new { command = "ask_delete", id = next.Frame.SelectedNote.Id });
    await session.ExecuteAsync(new { command = "accept_confirm" });
    await session.ExecuteAsync(new { command = "toggle_pin", id = savedId });
    var backupPath = Path.Combine(directory, "export.json");
    var exported = await session.ExecuteAsync(new { command = "export_notebook", path = backupPath });
    Check(File.Exists(backupPath), "Backup export writes the chosen file");
    Check(exported.Frame.BackupHealth == "Up to date", "Backup export updates Backup Health after the file is written");
    var backup = await File.ReadAllTextAsync(backupPath);
    Check(backup.Contains("\"kind\": \"nota.desktop_transition\""), "Backup export writes a complete notebook");
    await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = exported.Frame.EditSequence + 1, title = "Changed after export", content = "replacement", tags_input = "" });
    var staged = await session.ExecuteAsync(new { command = "import_notebook", json = backup });
    Check(staged.Frame.Confirm is { Action: "Import", Body: var importBody } && importBody.Contains("Notes to replace: 1"), "Import of a complete file into a nonempty notebook offers a merge");
    var imported = await session.ExecuteAsync(new { command = "accept_confirm" });
    Check(imported.Frame.SelectedNote?.Id == savedId && imported.Frame.SelectedNote.Title == "Unicode notebook", "Confirmed import replaces the selected note with the same identity");

    await session.ExecuteAsync(new { command = "set_view_mode", mode = "write" });
    var html = await session.ExecuteAsync(new { command = "preview", title = "<script>alert(1)</script>", content = "# A note\n<script>alert(2)</script>\n![remote](https://example.com/image.png)" });
    var document = html.Result!.Value.GetProperty("html").GetString()!;
    Check(document.Contains("Content-Security-Policy") && !document.Contains("<script>alert(2)</script>"), "Preview has a content policy and sanitizes raw scripts");
    var blocked = await session.ExecuteAsync(new { command = "external_navigation", uri = "file:///C:/Windows/win.ini", user_activated = true });
    Check(blocked.Result!.Value.GetProperty("uri").ValueKind == System.Text.Json.JsonValueKind.Null, "Preview denies local file navigation");
    var automatic = await session.ExecuteAsync(new { command = "external_navigation", uri = "https://example.com", user_activated = false });
    Check(automatic.Result!.Value.GetProperty("uri").ValueKind == System.Text.Json.JsonValueKind.Null, "Preview denies automatic external navigation");

    await session.ExecuteAsync(new { command = "set_theme", theme = "dark" });
    var transitionPath = Path.Combine(directory, "transition.json");
    await session.ExecuteAsync(new { command = "export_notebook", path = transitionPath });
    await session.ExecuteAsync(new { command = "ask_delete", id = savedId });
    await session.ExecuteAsync(new { command = "accept_confirm" });
    var deleted = await session.ExecuteAsync(new { command = "snapshot" });
    Check(deleted.Frame.RecentlyDeleted.Any(note => note.Id == savedId), "Deleted notes remain recoverable");
    var transition = await File.ReadAllTextAsync(transitionPath);
    var merge = await session.ExecuteAsync(new { command = "import_notebook", json = transition });
    Check(merge.Frame.Confirm is { Action: "Import" }, "A complete file merges when the notebook is not empty");
    await session.ExecuteAsync(new { command = "dismiss_confirm" });
    var destination = await NativeSession.OpenAsync(Path.Combine(directory, "restored"));
    await using (var target = destination.Session)
    {
        var stagedRestore = await target.ExecuteAsync(new { command = "import_notebook", json = transition });
        Check(stagedRestore.Frame.Confirm is { Action: "Restore" }, "An empty notebook offers an exact restore");
        var restored = await target.ExecuteAsync(new { command = "accept_confirm" });
        Check(restored.Frame.Rows.Any(note => note.Id == savedId) && restored.Frame.Theme == "dark", "Exact restore preserves notes and theme in an empty profile");
    }
    await session.ExecuteAsync(new { command = "restore_note", id = savedId });
    await session.ExecuteAsync(new { command = "select_note", id = savedId });
    var flushed = await session.ExecuteAsync(new { command = "flush" });
    Check(flushed.Frame.SaveStatus == "saved", "Flush acknowledges durable save");
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
    Check(selected.Frame.SelectedNote?.Title == "Unicode notebook" && selected.Frame.SelectedNote.IsPinned, "Saved note and pin survive destroy and reopen");
    Check(selected.Frame.Theme == "dark", "Theme survives destroy and reopen");
    var reads = Enumerable.Range(0, 30).Select(_ => session.ExecuteAsync(new { command = "snapshot" }));
    var snapshots = await Task.WhenAll(reads);
    Check(snapshots.All(reply => reply.Ok), "Concurrent managed callers serialize across one native session");

    using (var locked = new FileStream(Path.Combine(directory, "collection.json"), FileMode.Open, FileAccess.Read, FileShare.Read))
    {
        await session.ExecuteAsync(new { command = "edit_note", id = savedId, edit_sequence = selected.Frame.EditSequence + 1, title = "Unicode notebook", content = "Pending retry survives a failed save", tags_input = "work" });
        await RejectAsync(session, new { command = "flush" }, "Flush reports a Windows sharing violation without losing pending changes");
    }
    var retried = await session.ExecuteAsync(new { command = "flush" });
    Check(retried.Frame.SaveStatus == "saved" && retried.Frame.SelectedNote?.Content == "Pending retry survives a failed save", "A failed flush can be retried after storage becomes writable");
}

var recoveryDirectory = Path.Combine(directory, "recovery");
var recoverable = await NativeSession.OpenAsync(recoveryDirectory);
await using (var session = recoverable.Session)
{
    var created = await session.ExecuteAsync(new { command = "new_note" });
    var id = created.Frame.SelectedNote!.Id;
    var previous = await session.ExecuteAsync(new { command = "edit_note", id, edit_sequence = created.Frame.EditSequence + 1, title = "Previous saved note", content = "Preserve me", tags_input = "" });
    await session.ExecuteAsync(new { command = "flush" });
    await session.ExecuteAsync(new { command = "edit_note", id, edit_sequence = previous.Frame.EditSequence + 1, title = "Current saved note", content = "Newer content", tags_input = "" });
    await session.ExecuteAsync(new { command = "flush" });
}
var corruptBytes = System.Text.Encoding.UTF8.GetBytes("unreadable notebook ♥");
await File.WriteAllBytesAsync(Path.Combine(recoveryDirectory, "collection.json"), corruptBytes);
var recovery = await NativeSession.OpenAsync(recoveryDirectory);
await using (var session = recovery.Session)
{
    Check(recovery.Initial.Frame.Recovery is { CanRestorePrevious: true }, "Corrupt storage enters explicit recovery with a previous snapshot");
    await RejectAsync(session, new { command = "new_note" }, "Recovery blocks ordinary mutations");
    var restored = await session.ExecuteAsync(new { command = "restore_previous" });
    Check(restored.Frame.Recovery is null && restored.Frame.Rows.Any(note => note.Title == "Previous saved note"), "Recovery restores the previous saved notebook");
    Check(Directory.GetFiles(recoveryDirectory, "collection.corrupt-*.json").Any(path => File.ReadAllBytes(path).SequenceEqual(corruptBytes)), "Recovery preserves the exact corrupt bytes");
}

var emptyRecoveryDirectory = Path.Combine(directory, "empty-recovery");
Directory.CreateDirectory(emptyRecoveryDirectory);
await File.WriteAllBytesAsync(Path.Combine(emptyRecoveryDirectory, "collection.json"), corruptBytes);
var emptyRecovery = await NativeSession.OpenAsync(emptyRecoveryDirectory);
await using (var session = emptyRecovery.Session)
{
    Check(emptyRecovery.Initial.Frame.Recovery is { CanRestorePrevious: false }, "Recovery accurately reports a missing previous snapshot");
    var empty = await session.ExecuteAsync(new { command = "start_empty" });
    Check(empty.Frame.Recovery is null && empty.Frame.Rows.Length == 0, "Recovery can start a new empty notebook");
    Check(Directory.GetFiles(emptyRecoveryDirectory, "collection.corrupt-*.json").Any(path => File.ReadAllBytes(path).SequenceEqual(corruptBytes)), "Starting empty preserves the exact corrupt bytes");
}
Console.WriteLine($"All interop checks passed. Test profile retained at {directory}");
