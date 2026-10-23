// Generated from the commands! inventory. Do not edit by hand.
namespace Nota.Windows.Interop;

public static class Commands
{
    public const int SearchDebounceMs = 200;

    public static object Snapshot() => new { command = "snapshot" };
    public static object NewNote() => new { command = "new_note" };
    public static object Flush() => new { command = "flush" };
    public static object Resize(double widthPx) => new { command = "resize", width_px = widthPx };
    public static object SetViewMode(string mode) => new { command = "set_view_mode", mode };
    public static object SetTheme(string theme) => new { command = "set_theme", theme };
    public static object ToggleTheme() => new { command = "toggle_theme" };
    public static object ToggleNavigation() => new { command = "toggle_navigation" };
    public static object SelectNote(string id) => new { command = "select_note", id };
    public static object EditNote(string id, ulong editSequence, string title, string content, string tagsInput) => new { command = "edit_note", id, edit_sequence = editSequence, title, content, tags_input = tagsInput };
    public static object Search(string query) => new { command = "search", query };
    public static object FilterTag(string? tag) => new { command = "filter_tag", tag };
    public static object FilterPinned(bool pinned) => new { command = "filter_pinned", pinned };
    public static object TogglePin(string id) => new { command = "toggle_pin", id };
    public static object AskDelete(string id) => new { command = "ask_delete", id };
    public static object AskPermanentDelete(string id) => new { command = "ask_permanent_delete", id };
    public static object AskClearDeleted() => new { command = "ask_clear_deleted" };
    public static object AcceptConfirm() => new { command = "accept_confirm" };
    public static object DismissConfirm() => new { command = "dismiss_confirm" };
    public static object RestoreNote(string id) => new { command = "restore_note", id };
    public static object ImportNotebook(string json) => new { command = "import_notebook", json };
    public static object ExportNotebook(string path) => new { command = "export_notebook", path };
    public static object RestorePrevious() => new { command = "restore_previous" };
    public static object StartEmpty() => new { command = "start_empty" };
    public static object Format(string content, int startUtf16, int endUtf16, string kind) => new { command = "format", content, start_utf16 = startUtf16, end_utf16 = endUtf16, kind };
    public static object Preview(string title, string content) => new { command = "preview", title, content };
    public static object ExternalNavigation(string uri, bool userActivated) => new { command = "external_navigation", uri, user_activated = userActivated };
    public static object ApplyTagCleanup() => new { command = "apply_tag_cleanup" };
    public static object AcceptTagSuggestion(string input) => new { command = "accept_tag_suggestion", input };
    public static object StartEditTags() => new { command = "start_edit_tags" };
    public static object FinishEditTags() => new { command = "finish_edit_tags" };
    public static object DismissNotification(ulong generation) => new { command = "dismiss_notification", generation };
}
