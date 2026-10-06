using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.UI;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Input;
using Microsoft.UI.Text;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Documents;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Nota.Windows.Interop;
using Windows.ApplicationModel.DataTransfer;
using Windows.UI.Core;
using Windows.System;
using DispatcherQueueTimer = Microsoft.UI.Dispatching.DispatcherQueueTimer;

namespace Nota.Windows;

public sealed partial class MainWindow : Window
{
    private readonly string directory;
    private readonly string? startupError;
    private readonly ObservableCollection<NoteRowViewModel> rows = [];
    private readonly DispatcherQueueTimer savePoll;
    private readonly DispatcherQueueTimer previewDelay;
    private NativeSession? session;
    private NativePreview? preview;
    private Snapshot? snapshot;
    private string? selectedId;
    private ulong editSequence;
    private bool applying;
    private bool transitioning;
    private bool allowClose;
    private bool closing;
    private bool showingRecovery;
    private Control? drawerPreviousFocus;
    private string? deletedSignature;
    private string? metadataSignature;
    private Notification? shownNotification;
    private readonly DispatcherQueueTimer noticeDelay;
    private readonly Dictionary<VirtualKey, Func<Task>> shortcuts = [];
    private readonly KeyboardHook previewKeyboardCallback;
    private nint previewKeyboardHook;

    public MainWindow(string dataDirectory, string? error)
    {
        InitializeComponent();
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(TitleDragRegion);
        AppWindow.TitleBar.PreferredHeightOption = TitleBarHeightOption.Tall;
        directory = dataDirectory;
        startupError = error;
        NoteList.ItemsSource = rows;
        var icon = Path.Combine(AppContext.BaseDirectory, "Assets", "nota.ico");
        if (File.Exists(icon)) AppWindow.SetIcon(icon);
        AppWindow.Closing += AppWindow_Closing;
        Root.ActualThemeChanged += (_, _) => { ApplyTitleBar(); SchedulePreview(); };
        savePoll = DispatcherQueue.CreateTimer();
        savePoll.Interval = TimeSpan.FromMilliseconds(500);
        savePoll.Tick += async (_, _) =>
        {
            savePoll.Stop();
            await RunAsync(new { command = "snapshot" });
        };
        previewDelay = DispatcherQueue.CreateTimer();
        previewDelay.Interval = TimeSpan.FromMilliseconds(180);
        previewDelay.IsRepeating = false;
        previewDelay.Tick += async (_, _) => await RefreshPreviewAsync();
        noticeDelay = DispatcherQueue.CreateTimer();
        noticeDelay.Interval = TimeSpan.FromSeconds(6);
        noticeDelay.IsRepeating = false;
        noticeDelay.Tick += (_, _) => Notice.IsOpen = false;
        Closed += (_, _) =>
        {
            if (previewKeyboardHook != 0) UnhookWindowsHookEx(previewKeyboardHook);
            previewKeyboardHook = 0;
            savePoll.Stop(); previewDelay.Stop(); noticeDelay.Stop(); PreviewWeb.Close();
        };
        AddShortcut(VirtualKey.N, CreateNoteAsync);
        AddShortcut(VirtualKey.F, () => { OpenLibrary(); SearchBox.Focus(FocusState.Keyboard); return Task.CompletedTask; });
        var dismiss = new KeyboardAccelerator { Key = VirtualKey.Escape };
        dismiss.Invoked += (_, args) => { if (Library.IsPaneOpen) { Library.IsPaneOpen = false; args.Handled = true; } };
        Root.KeyboardAccelerators.Add(dismiss);
        AddShortcut(VirtualKey.B, () => FormatAsync("bold"));
        AddShortcut(VirtualKey.I, () => FormatAsync("italic"));
        AddShortcut(VirtualKey.S, async () => await RunAsync(new { command = "flush" }));
        previewKeyboardCallback = PreviewKeyboard;
        previewKeyboardHook = SetWindowsHookExW(2, previewKeyboardCallback, 0, GetCurrentThreadId());
        if (previewKeyboardHook == 0) ShowError(new Win32Exception(Marshal.GetLastWin32Error()).Message);
    }

    private void AddShortcut(VirtualKey key, Func<Task> action)
    {
        shortcuts.Add(key, action);
        var accelerator = new KeyboardAccelerator { Key = key, Modifiers = VirtualKeyModifiers.Control };
        accelerator.Invoked += async (_, args) => { args.Handled = true; await action(); };
        Root.KeyboardAccelerators.Add(accelerator);
    }

    private nint PreviewKeyboard(int code, nint key, nint flags)
    {
        try
        {
            if (code == 0 && (flags.ToInt64() & (1L << 31)) == 0 && !closing && Workspace.IsEnabled
                && KeyDown(VirtualKey.Control) && !KeyDown(VirtualKey.Shift) && !KeyDown(VirtualKey.Menu)
                && !KeyDown(VirtualKey.LeftWindows) && !KeyDown(VirtualKey.RightWindows)
                && shortcuts.TryGetValue((VirtualKey)key.ToInt32(), out var action))
            {
                var focused = FocusManager.GetFocusedElement(Root.XamlRoot);
                var previewFocused = ReferenceEquals(focused, PreviewWeb);
                if ((flags.ToInt64() & (1L << 30)) != 0
                    && (previewFocused || ((VirtualKey)key.ToInt32() == VirtualKey.N && ReferenceEquals(focused, TitleBox)))) return 1;
                if (previewFocused && DispatcherQueue.TryEnqueue(async () =>
                {
                    try { await action(); }
                    catch (Exception exception) { ShowError(exception.Message); }
                })) return 1;
            }
        }
        catch (Exception exception)
        {
            DispatcherQueue.TryEnqueue(() => ShowError(exception.Message));
        }
        return CallNextHookEx(previewKeyboardHook, code, key, flags);
    }

    private static bool KeyDown(VirtualKey key)
        => (InputKeyboardSource.GetKeyStateForCurrentThread(key) & CoreVirtualKeyStates.Down) != 0;

    private delegate nint KeyboardHook(int code, nint key, nint flags);

    [DllImport("user32.dll", ExactSpelling = true, SetLastError = true)]
    private static extern nint SetWindowsHookExW(int hook, KeyboardHook callback, nint module, uint thread);

    [DllImport("user32.dll", ExactSpelling = true)]
    private static extern nint CallNextHookEx(nint hook, int code, nint key, nint flags);

    [DllImport("user32.dll", ExactSpelling = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnhookWindowsHookEx(nint hook);

    [DllImport("kernel32.dll", ExactSpelling = true)]
    private static extern uint GetCurrentThreadId();

    private async void Root_Loaded(object sender, RoutedEventArgs args)
    {
        var scale = Root.XamlRoot.RasterizationScale;
        var area = DisplayArea.GetFromWindowId(AppWindow.Id, DisplayAreaFallback.Nearest).WorkArea;
        var width = Math.Min((int)Math.Round(1120 * scale), area.Width);
        var height = Math.Min((int)Math.Round(800 * scale), area.Height);
        AppWindow.MoveAndResize(new global::Windows.Graphics.RectInt32(area.X + (area.Width - width) / 2, area.Y + (area.Height - height) / 2, width, height));
        if (startupError is not null) { Workspace.IsEnabled = false; ShowError(startupError); return; }
        Workspace.IsEnabled = false;
        try
        {
            var opened = await NativeSession.OpenAsync(directory);
            session = opened.Session;
            preview = new NativePreview(PreviewWeb, PreviewMessage, PreviewMessageText, InstallWebView, directory, session.ExecuteAsync);
            Apply(opened.Initial.Snapshot);
            Workspace.IsEnabled = true;
            await CheckRecoveryAsync();
            if (snapshot?.Recovery is null) TitleBox.Focus(FocusState.Programmatic);
        }
        catch (Exception exception)
        {
            ShowError(exception is DllNotFoundException or BadImageFormatException
                ? "Nota's shared core could not load. Reinstall the complete Windows application, including nota_ffi.dll."
                : exception.Message);
        }
    }

    private async Task<Reply?> RunAsync(object command, bool replaceEditor = false)
    {
        if (session is null) return null;
        try
        {
            var reply = await session.ExecuteAsync(command);
            Apply(reply.Snapshot, replaceEditor);
            return reply;
        }
        catch (Exception exception)
        {
            ShowError(exception.Message);
            return null;
        }
    }

    private async Task TransitionAsync(object command, bool replaceEditor = false, bool focusTitle = false)
    {
        if (session is null || transitioning || closing) return;
        transitioning = true;
        EditorPane.IsEnabled = false;
        NewButton.IsEnabled = false;
        NoteList.IsEnabled = false;
        var succeeded = false;
        var returnToEditor = Library.IsPaneOpen;
        try
        {
            var reply = await RunAsync(command, replaceEditor);
            if (reply is not null)
            {
                succeeded = true;
                drawerPreviousFocus = null;
                Library.IsPaneOpen = false;
                UpdateLayoutMode();
            }
        }
        finally
        {
            transitioning = false;
            EditorPane.IsEnabled = !Library.IsPaneOpen;
            NewButton.IsEnabled = true;
            NoteList.IsEnabled = true;
        }
        if (succeeded && focusTitle && selectedId is not null)
        {
            TitleBox.Focus(FocusState.Programmatic);
            TitleBox.SelectAll();
        }
        else if (succeeded && returnToEditor && selectedId is not null)
        {
            if (BodyBox.Visibility == Visibility.Visible) BodyBox.Focus(FocusState.Programmatic);
            else TitleBox.Focus(FocusState.Programmatic);
        }
    }

    private void Apply(Snapshot value, bool replaceEditor = false)
    {
        if (snapshot is not null && value.Revision < snapshot.Revision) return;
        snapshot = value;
        editSequence = Math.Max(editSequence, value.EditSequence);
        applying = true;
        try
        {
            Root.RequestedTheme = value.Theme switch { "light" => ElementTheme.Light, "dark" => ElementTheme.Dark, _ => ElementTheme.Default };
            if (SearchBox.Text != value.SearchInput) SearchBox.Text = value.SearchInput;
            if (replaceEditor || selectedId != value.SelectedNote?.Id)
            {
                selectedId = value.SelectedNote?.Id;
                TitleBox.Text = value.SelectedNote?.Title ?? "";
                BodyBox.Document.SetText(TextSetOptions.None, EditorText.ToNative(value.SelectedNote?.Content ?? ""));
                BodyBox.Document.Selection.SetRange(0, 0);
                BodyBox.Document.ClearUndoRedoHistory();
                UpdateCounts();
                SchedulePreview();
            }
            TitleBox.IsEnabled = BodyBox.IsEnabled = selectedId is not null && value.Recovery is null;
            PinButton.IsEnabled = SelectedActionsButton.IsEnabled = selectedId is not null && value.Recovery is null;
            ReconcileMetadata(value);
            NoteDate.Text = value.SelectedNote is { } selected && DateTimeOffset.TryParse(selected.LastModified, out var edited)
                ? "  ·  Edited " + (DateTimeOffset.Now - edited < TimeSpan.FromMinutes(1) ? "just now" : edited.LocalDateTime.Date == DateTime.Today ? "today" : edited.ToLocalTime().ToString("d MMM")) : "";
            var pinned = value.SelectedNote?.IsPinned == true;
            if (pinned) PinButton.Foreground = (Brush)Application.Current.Resources["SignalBrush"];
            else PinButton.ClearValue(Control.ForegroundProperty);
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(PinButton, pinned ? "Unpin note" : "Pin note");
            ToolTipService.SetToolTip(PinButton, pinned ? "Unpin note" : "Pin note");
            AllNotesFilter.IsChecked = !value.PinnedOnly;
            PinnedFilter.IsChecked = value.PinnedOnly;
            ActiveTagFilter.Visibility = value.ActiveTag is null ? Visibility.Collapsed : Visibility.Visible;
            ActiveTagFilter.Content = value.ActiveTag is { } tag ? "#" + tag + " ×" : "";
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(ActiveTagFilter, value.ActiveTag is { } activeTag ? "Clear " + activeTag + " filter" : "Clear tag filter");
            Toolbar.IsHitTestVisible = selectedId is not null;
            ReconcileRows(value.Rows);
            NoteList.SelectedItem = rows.FirstOrDefault(row => row.Id == selectedId);
            NoteCount.Text = value.Rows.Length.ToString();
            EmptyNotes.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
            EmptyNotes.Text = value.SearchInput.Length > 0 || value.ActiveTag is not null || value.PinnedOnly ? "No matching notes." : "No notes yet.";
            SaveStatus.Text = value.SaveStatus switch { "saving" => "Saving…", "failed" => "Save failed", _ => "Saved locally" };
            ToolTipService.SetToolTip(SaveStatus, value.SaveStatus == "failed" ? "Press Ctrl+S to retry saving. Your changes remain in memory." : "Notes are saved on this device");
            ReconcileDeleted(value);
            ApplyViewMode();
            ApplyTitleBar();
            if (value.Notification is { } notification && notification != shownNotification)
            {
                shownNotification = notification;
                Notice.Title = "";
                Notice.Message = notification.Message;
                Notice.Severity = notification.Tone is "error" or "danger" ? InfoBarSeverity.Error : notification.Tone == "success" ? InfoBarSeverity.Success : InfoBarSeverity.Informational;
                Notice.IsOpen = true;
                noticeDelay.Stop();
                if (Notice.Severity != InfoBarSeverity.Error) noticeDelay.Start();
            }
            if (value.SaveStatus == "saving" && !closing) savePoll.Start();
        }
        finally { applying = false; }
    }

    private void ReconcileRows(NoteRow[] values)
    {
        var ids = values.Select(row => row.Id).ToHashSet();
        for (var index = rows.Count - 1; index >= 0; index--) if (!ids.Contains(rows[index].Id)) rows.RemoveAt(index);
        for (var index = 0; index < values.Length; index++)
        {
            var existing = rows.FirstOrDefault(row => row.Id == values[index].Id);
            if (existing is null) rows.Insert(index, new NoteRowViewModel(values[index]));
            else { var old = rows.IndexOf(existing); if (old != index) rows.Move(old, index); existing.Update(values[index]); }
        }
    }

    private void ReconcileMetadata(Snapshot value)
    {
        var tags = value.SelectedNote?.Tags ?? [];
        var signature = System.Text.Json.JsonSerializer.Serialize(tags);
        if (signature == metadataSignature) return;
        metadataSignature = signature;
        MetadataParagraph.Inlines.Clear();
        foreach (var tag in tags)
        {
            var link = new Hyperlink { Foreground = (Brush)Application.Current.Resources["SignalBrush"], TextDecorations = global::Windows.UI.Text.TextDecorations.None };
            link.Inlines.Add(new Run { Text = "#" + tag });
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(link, "Filter notes by " + tag);
            link.Click += async (_, _) =>
            {
                OpenLibrary();
                if (snapshot?.PinnedOnly == true) await RunAsync(new { command = "filter_pinned", pinned = false });
                await RunAsync(new { command = "filter_tag", tag });
                SearchBox.Focus(FocusState.Programmatic);
            };
            MetadataParagraph.Inlines.Add(link);
            MetadataParagraph.Inlines.Add(new Run { Text = "  " });
        }
        var edit = new Hyperlink { Foreground = (Brush)Application.Current.Resources["SignalBrush"], TextDecorations = global::Windows.UI.Text.TextDecorations.None };
        edit.Inlines.Add(new Run { Text = tags.Length == 0 ? "Add tags" : "+" });
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(edit, "Edit tags");
        edit.Click += (_, args) => Tags_Click(edit, args);
        MetadataParagraph.Inlines.Add(edit);
        MetadataParagraph.Inlines.Add(NoteDate);
    }

    private void ReconcileDeleted(Snapshot value)
    {
        DeletedCount.Text = value.RecentlyDeleted.Length.ToString();
        var signature = System.Text.Json.JsonSerializer.Serialize(value.RecentlyDeleted);
        if (signature == deletedSignature) return;
        deletedSignature = signature;
        DeletedRows.Children.Clear();
        ClearDeleted.Visibility = value.RecentlyDeleted.Length == 0 ? Visibility.Collapsed : Visibility.Visible;
        if (value.RecentlyDeleted.Length == 0)
            DeletedRows.Children.Add(new TextBlock { Text = "No deleted notes", FontSize = 12, Opacity = 0.65, Margin = new Thickness(16, 14, 16, 14) });
        foreach (var note in value.RecentlyDeleted)
        {
            var grid = new Grid { Padding = new Thickness(16, 8, 8, 8), ColumnSpacing = 2 };
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            grid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            grid.Children.Add(new TextBlock { Text = string.IsNullOrWhiteSpace(note.Title) ? "Untitled note" : note.Title, FontSize = 12, VerticalAlignment = VerticalAlignment.Center, TextTrimming = TextTrimming.CharacterEllipsis });
            var restore = new Button { Content = "Restore", Style = (Style)Application.Current.Resources["QuietButton"], FontSize = 11, Padding = new Thickness(6) };
            restore.Click += async (_, _) => await TransitionAsync(new { command = "restore_note", id = note.Id });
            Grid.SetColumn(restore, 1); grid.Children.Add(restore);
            var delete = new Button { Content = "Delete", Style = (Style)Application.Current.Resources["QuietButton"], Foreground = DangerBrush(), FontSize = 11, Padding = new Thickness(6) };
            delete.Click += async (_, _) => { if (await ConfirmAsync("Delete this note permanently?", "This cannot be undone. Export a backup first if you want to keep a copy.", "Delete")) await RunAsync(new { command = "permanently_delete", id = note.Id }); };
            Grid.SetColumn(delete, 2); grid.Children.Add(delete);
            DeletedRows.Children.Add(new Border { Child = grid, BorderThickness = new Thickness(0, 0, 0, 1), BorderBrush = new SolidColorBrush(ColorHelper.FromArgb(40, 128, 128, 128)) });
        }
    }

    private Brush DangerBrush() => (Brush)((ResourceDictionary)Application.Current.Resources.ThemeDictionaries[Root.ActualTheme.ToString()])["DangerBrush"];

    private string ReadBody()
    {
        BodyBox.Document.GetText(TextGetOptions.None, out var text);
        return text.EndsWith('\r') ? text[..^1] : text;
    }

    private async void Body_Paste(object sender, TextControlPasteEventArgs args)
    {
        args.Handled = true;
        var id = selectedId;
        var sequence = editSequence;
        var start = BodyBox.Document.Selection.StartPosition;
        var end = BodyBox.Document.Selection.EndPosition;
        try
        {
            var clipboard = Clipboard.GetContent();
            if (!clipboard.Contains(StandardDataFormats.Text)) return;
            var text = EditorText.ToNative(await clipboard.GetTextAsync());
            if (id != selectedId || sequence != editSequence || !BodyBox.IsEnabled) return;
            BodyBox.Document.BeginUndoGroup();
            try
            {
                BodyBox.Document.Selection.SetRange(start, end);
                BodyBox.Document.Selection.SetText(TextSetOptions.None, text);
            }
            finally { BodyBox.Document.EndUndoGroup(); }
            BodyBox.Document.Selection.SetRange(start + text.Length, start + text.Length);
        }
        catch (Exception exception) { ShowError($"The clipboard text could not be pasted. {exception.Message}"); }
    }

    private async void Editor_TextChanged(object sender, RoutedEventArgs args)
    {
        if (applying || selectedId is null || session is null) return;
        var content = EditorText.ToCore(ReadBody());
        if (snapshot?.SelectedNote is { } note && note.Id == selectedId
            && TitleBox.Text == note.Title && content == EditorText.ToCore(note.Content)) return;
        var id = selectedId;
        var sequence = ++editSequence;
        UpdateCounts();
        SchedulePreview();
        await RunAsync(new { command = "edit_note", id, edit_sequence = sequence, title = TitleBox.Text, content, tags_input = string.Join(", ", snapshot?.SelectedNote?.Tags ?? []) });
    }

    private void UpdateCounts()
    {
        var text = EditorText.ToCore(ReadBody());
        var words = System.Text.RegularExpressions.Regex.Replace(text, @"[#*\[\]>]", "").Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries).Length;
        var lines = text.Length == 0 ? 0 : text.Count(c => c == '\n') + 1;
        var characters = text.EnumerateRunes().Count();
        WordCount.Text = $"{words} {(words == 1 ? "word" : "words")}";
        ToolTipService.SetToolTip(WordCount, $"{lines} {(lines == 1 ? "line" : "lines")} · {characters} {(characters == 1 ? "character" : "characters")}");
    }

    private async void Search_TextChanged(object sender, TextChangedEventArgs args)
    {
        if (!applying) await RunAsync(new { command = "search", query = SearchBox.Text });
    }

    private async void NoteList_SelectionChanged(object sender, SelectionChangedEventArgs args)
    {
        if (!applying && NoteList.SelectedItem is NoteRowViewModel row && row.Id != selectedId)
            await TransitionAsync(new { command = "select_note", id = row.Id });
    }

    private void NoteList_ItemClick(object sender, ItemClickEventArgs args)
    {
        if (args.ClickedItem is NoteRowViewModel row && row.Id == selectedId)
        {
            drawerPreviousFocus = null;
            Library.IsPaneOpen = false;
            UpdateLayoutMode();
            if (BodyBox.Visibility == Visibility.Visible) BodyBox.Focus(FocusState.Programmatic);
            else TitleBox.Focus(FocusState.Programmatic);
        }
    }

    private Task CreateNoteAsync() => TransitionAsync(new { command = "new_note" }, focusTitle: true);
    private async void New_Click(object sender, RoutedEventArgs args) => await CreateNoteAsync();
    private void Notes_Click(object sender, RoutedEventArgs args) { if (Library.IsPaneOpen) Library.IsPaneOpen = false; else OpenLibrary(); }
    private void CloseNotes_Click(object sender, RoutedEventArgs args) => Library.IsPaneOpen = false;

    private void OpenLibrary()
    {
        if (Library.IsPaneOpen) return;
        drawerPreviousFocus = FocusManager.GetFocusedElement(Root.XamlRoot) as Control;
        Library.IsPaneOpen = true;
    }

    private void Library_PaneOpening(SplitView sender, object args) => UpdateLayoutMode();
    private void Library_PaneClosed(SplitView sender, object args)
    {
        UpdateLayoutMode();
        if (!transitioning && drawerPreviousFocus is { } previousFocus) previousFocus.Focus(FocusState.Programmatic);
        drawerPreviousFocus = null;
    }

    private void Deleted_Click(object sender, RoutedEventArgs args)
        => DeletedContent.Visibility = DeletedContent.Visibility == Visibility.Visible ? Visibility.Collapsed : Visibility.Visible;

    private async void Filter_Click(object sender, RoutedEventArgs args)
    {
        if (sender is ToggleButton { Tag: string filter })
        {
            await RunAsync(new { command = "filter_tag", tag = (string?)null });
            await RunAsync(new { command = "filter_pinned", pinned = filter == "pinned" });
        }
    }

    private async void ClearTag_Click(object sender, RoutedEventArgs args)
        => await RunAsync(new { command = "filter_tag", tag = (string?)null });

    private async void Pin_Click(object sender, RoutedEventArgs args)
    {
        if (snapshot?.SelectedNote is { } note) await RunAsync(new { command = "toggle_pin", id = note.Id });
    }

    private void SelectedNoteActions_Click(object sender, RoutedEventArgs args)
    {
        if (snapshot?.SelectedNote is { } note && sender is Button button) ShowNoteActions(button, note.Id, note.IsPinned);
    }

    private void NoteActions_Click(object sender, RoutedEventArgs args)
    {
        if (sender is not Button { Tag: string id } button || snapshot is null) return;
        var row = snapshot.Rows.FirstOrDefault(row => row.Id == id);
        if (row is not null) ShowNoteActions(button, id, row.IsPinned);
    }

    private void ShowNoteActions(Button button, string id, bool pinned)
    {
        var flyout = new MenuFlyout { Placement = FlyoutPlacementMode.BottomEdgeAlignedRight };
        var pin = new MenuFlyoutItem { Text = pinned ? "Unpin note" : "Pin note" };
        pin.Click += async (_, _) => await RunAsync(new { command = "toggle_pin", id });
        var title = snapshot?.SelectedNote is { } selected && selected.Id == id
            ? selected.Title : snapshot?.Rows.FirstOrDefault(note => note.Id == id)?.Title;
        var displayTitle = string.IsNullOrWhiteSpace(title) ? "Untitled note" : title;
        var delete = new MenuFlyoutItem { Text = "Move to recently deleted", Foreground = DangerBrush() };
        delete.Click += async (_, _) => { if (await ConfirmAsync("Move to recently deleted?", $"\"{displayTitle}\" can be restored from Recently deleted in Notes.", "Move note")) await TransitionAsync(new { command = "delete_note", id }); };
        flyout.Items.Add(pin);
        if (id == selectedId)
        {
            var tags = new MenuFlyoutItem { Text = "Edit tags" };
            tags.Click += Tags_Click;
            flyout.Items.Add(tags);
            var strike = new MenuFlyoutItem { Text = "Strikethrough" };
            strike.Click += async (_, _) => await FormatAsync("strikethrough", enterWrite: true);
            var table = new MenuFlyoutItem { Text = "Insert table" };
            table.Click += async (_, _) => await FormatAsync("table", enterWrite: true);
            flyout.Items.Add(strike);
            flyout.Items.Add(table);
        }
        flyout.Items.Add(new MenuFlyoutSeparator());
        flyout.Items.Add(delete);
        flyout.ShowAt(button);
    }

    private async void ClearDeleted_Click(object sender, RoutedEventArgs args)
    {
        if (await ConfirmAsync("Clear Recently Deleted?", "All notes in Recently Deleted will be permanently removed. This cannot be undone.", "Clear all")) await RunAsync(new { command = "clear_deleted" });
    }

    private async void Format_Click(object sender, RoutedEventArgs args)
    {
        if (sender is Button { Tag: string kind }) await FormatAsync(kind);
    }

    private async Task FormatAsync(string kind, bool enterWrite = false)
    {
        if (session is null || selectedId is null || transitioning) return;
        if (snapshot?.ViewMode == "preview" && !enterWrite) return;
        var nativeContent = ReadBody();
        var content = EditorText.ToCore(nativeContent);
        var id = selectedId;
        var sequence = editSequence;
        var start = EditorText.ToCoreOffset(nativeContent, BodyBox.Document.Selection.StartPosition);
        var end = EditorText.ToCoreOffset(nativeContent, BodyBox.Document.Selection.EndPosition);
        try
        {
            if (snapshot?.ViewMode == "preview")
            {
                var modeReply = await RunAsync(new { command = "set_view_mode", mode = "write" });
                if (modeReply is null || session is null || transitioning || id != selectedId || sequence != editSequence) return;
            }
            var reply = await session.ExecuteAsync(new { command = "format", content, start_utf16 = start, end_utf16 = end, kind });
            if (id != selectedId || sequence != editSequence || ReadBody() != nativeContent || reply.Result is not { } result) return;
            var formatted = result.GetProperty("content").GetString() ?? "";
            var updated = EditorText.ToNative(formatted);
            var prefix = 0;
            while (prefix < Math.Min(nativeContent.Length, updated.Length) && nativeContent[prefix] == updated[prefix]) prefix++;
            if (prefix > 0 && prefix < nativeContent.Length && char.IsLowSurrogate(nativeContent[prefix])) prefix--;
            var suffix = 0;
            while (suffix < Math.Min(nativeContent.Length, updated.Length) - prefix && nativeContent[^(suffix + 1)] == updated[^(suffix + 1)]) suffix++;
            if (suffix > 0 && char.IsLowSurrogate(nativeContent[nativeContent.Length - suffix])) suffix--;
            BodyBox.Focus(FocusState.Programmatic);
            BodyBox.Document.BeginUndoGroup();
            try
            {
                BodyBox.Document.Selection.SetRange(prefix, nativeContent.Length - suffix);
                BodyBox.Document.Selection.SetText(TextSetOptions.None, updated.Substring(prefix, updated.Length - prefix - suffix));
            }
            finally { BodyBox.Document.EndUndoGroup(); }
            var caret = EditorText.ToNativeOffset(formatted, result.GetProperty("caret_utf16").GetInt32());
            BodyBox.Document.Selection.SetRange(caret, caret);
        }
        catch (Exception exception) { ShowError(exception.Message); }
    }

    private async void Mode_Click(object sender, RoutedEventArgs args)
    {
        if (sender is ToggleButton { Tag: string mode }) await RunAsync(new { command = "set_view_mode", mode });
        SchedulePreview();
    }

    private void ApplyViewMode()
    {
        var mode = snapshot?.ViewMode ?? "write";
        WriteMode.IsChecked = mode == "write"; PreviewMode.IsChecked = mode == "preview"; SplitMode.IsChecked = mode == "split";
        BodyBox.Visibility = mode == "preview" ? Visibility.Collapsed : Visibility.Visible;
        var hasNote = selectedId is not null;
        NoteHeading.Visibility = NoteActions.Visibility = hasNote ? Visibility.Visible : Visibility.Collapsed;
        EmptyEditor.Visibility = hasNote ? Visibility.Collapsed : Visibility.Visible;
        WritingArea.Visibility = hasNote ? Visibility.Visible : Visibility.Collapsed;
        Toolbar.Visibility = mode == "preview" || !hasNote ? Visibility.Collapsed : Visibility.Visible;
        PreviewPane.Visibility = mode == "write" ? Visibility.Collapsed : Visibility.Visible;
        var stacked = mode == "split" && Root.ActualWidth <= 560;
        if (WritingScroll.VerticalScrollMode == ScrollMode.Auto && !stacked)
            WritingScroll.ChangeView(null, 0, null, disableAnimation: true);
        var narrow = Root.ActualWidth <= 760;
        WriteColumn.Width = mode == "preview" ? new GridLength(0) : new GridLength(1, GridUnitType.Star);
        PreviewColumn.Width = mode == "write" || stacked ? new GridLength(0) : new GridLength(1, GridUnitType.Star);
        SplitGapColumn.Width = mode == "split" && !stacked ? new GridLength(narrow ? 18 : 28) : new GridLength(0);
        WriteRow.Height = stacked ? GridLength.Auto : new GridLength(1, GridUnitType.Star);
        PreviewRow.Height = stacked ? GridLength.Auto : new GridLength(0);
        WritingScroll.VerticalScrollBarVisibility = stacked ? ScrollBarVisibility.Auto : ScrollBarVisibility.Disabled;
        WritingScroll.VerticalScrollMode = stacked ? ScrollMode.Auto : ScrollMode.Disabled;
        WritingScroll.VerticalContentAlignment = stacked ? VerticalAlignment.Top : VerticalAlignment.Stretch;
        ScrollViewer.SetVerticalScrollBarVisibility(BodyBox, stacked ? ScrollBarVisibility.Disabled : ScrollBarVisibility.Auto);
        ScrollViewer.SetVerticalScrollMode(BodyBox, stacked ? ScrollMode.Disabled : ScrollMode.Auto);
        BodyBox.MinHeight = stacked ? 340 : 0;
        PreviewPane.Height = stacked ? 525 : double.NaN;
        Grid.SetColumn(PreviewPane, stacked ? 0 : 2);
        Grid.SetRow(PreviewPane, stacked ? 1 : 0);
        PreviewPane.Margin = stacked ? new Thickness(0, 18, 0, 0) : new Thickness(0);
        PreviewPane.BorderThickness = mode == "split" ? stacked ? new Thickness(0, 1, 0, 0) : new Thickness(1, 0, 0, 0) : new Thickness(0);
        PreviewPane.Padding = mode == "split" ? stacked ? new Thickness(0, 24, 0, 0) : new Thickness(narrow ? 18 : 25, 0, 0, 0) : new Thickness(0);
        WritingArea.MaxWidth = mode == "split" ? double.PositiveInfinity : 684;
        WritingArea.Width = mode == "split" ? Root.ActualWidth : Math.Min(684, Root.ActualWidth);
        WritingArea.Padding = Root.ActualWidth <= 560 ? new Thickness(24, 28, 24, 20)
            : mode == "split" ? new Thickness(narrow ? 19 : 27, 31, narrow ? 19 : 27, 24) : new Thickness(42, 31, 42, 24);
        var family = mode == "split" ? "ms-appx:///Assets/Fonts/SourceCodePro-Regular.ttf#Source Code Pro" : ((FontFamily)Application.Current.Resources["ReadingFont"]).Source;
        var fontSize = mode == "split" ? 13 : Root.ActualWidth <= 560 ? 16 : 18;
        BodyBox.Margin = new Thickness(0, Root.ActualWidth <= 560 ? -6 : -7, 0, stacked ? 3 : 0);
        if (BodyBox.FontFamily.Source != family) BodyBox.FontFamily = new FontFamily(family);
        if (BodyBox.FontSize != fontSize) BodyBox.FontSize = fontSize;
        var paragraph = BodyBox.Document.GetDefaultParagraphFormat();
        var lineHeightPoints = (float)(BodyBox.FontSize * (mode == "split" ? 1.85 : 1.8) * 72 / 96);
        if (paragraph.LineSpacingRule != LineSpacingRule.Exactly || Math.Abs(paragraph.LineSpacing - lineHeightPoints) > 0.025)
        {
            paragraph.SetLineSpacing(LineSpacingRule.Exactly, lineHeightPoints);
            paragraph.SpaceBefore = paragraph.SpaceAfter = 0;
            BodyBox.Document.SetDefaultParagraphFormat(paragraph);
        }
    }

    private string PreviewLayout(double width) => snapshot?.ViewMode != "split" ? "reading" : width <= 760 ? "split_narrow" : "split";

    private void Root_SizeChanged(object sender, SizeChangedEventArgs args)
    {
        UpdateLayoutMode();
        if (PreviewLayout(args.PreviousSize.Width) != PreviewLayout(args.NewSize.Width)) SchedulePreview();
    }

    private void UpdateLayoutMode()
    {
        var compact = Root.ActualWidth <= 560;
        TopBar.Height = compact ? 56 : 62;
        TopBar.Padding = new Thickness(compact ? 15 : 23, 0, 0, 0);
        EditorToolsRow.Height = new GridLength(compact ? 46 : 42);
        NoteActions.Margin = new Thickness(0, compact ? 14 : 11, compact ? 12 : 24, 0);
        Library.OpenPaneLength = compact ? Root.ActualWidth : 278;
        EditorPane.Visibility = compact && Library.IsPaneOpen ? Visibility.Collapsed : Visibility.Visible;
        DrawerScrim.Visibility = Library.IsPaneOpen ? Visibility.Visible : Visibility.Collapsed;
        EditorPane.IsEnabled = !Library.IsPaneOpen && !transitioning;
        NotesButton.IsChecked = Library.IsPaneOpen;
        NoteHeading.Padding = compact ? new Thickness(24, 40, 24, 0) : new Thickness(42, 44, 42, 0);
        TitleBox.FontSize = compact ? 31 : Root.ActualWidth <= 760 ? 36 : 39;
        TitleBox.CharacterSpacing = -(int)Math.Round(1250 / TitleBox.FontSize);
        TitleBox.MinHeight = Math.Ceiling(TitleBox.FontSize * 1.2);
        Toolbar.Padding = compact ? new Thickness(17, 23, 17, 4) : new Thickness(36, 26, 36, 8);
        EditorFooter.Padding = compact ? new Thickness(17, 14, 17, 14) : new Thickness(27, 0, 27, 0);
        EditorFooter.Height = compact ? double.NaN : 54;
        EditorFooter.MinHeight = 0;
        EditorFooter.RowDefinitions[0].Height = compact ? GridLength.Auto : new GridLength(1, GridUnitType.Star);
        ViewModes.VerticalAlignment = VerticalAlignment.Center;
        EditorFooter.BorderThickness = compact ? new Thickness(0, 1, 0, 0) : new Thickness(0);
        Grid.SetColumn(Brand, compact ? 1 : 0);
        Grid.SetColumnSpan(Brand, compact ? 1 : 4);
        Brand.Margin = compact ? new Thickness(0) : new Thickness(-23, 0, 0, 0);
        ViewModes.HorizontalAlignment = compact ? HorizontalAlignment.Right : HorizontalAlignment.Center;
        Grid.SetRow(SavedState, compact ? 1 : 0);
        SavedState.Margin = compact ? new Thickness(0, 9, 0, 0) : new Thickness(0);
        SavedState.HorizontalAlignment = compact ? HorizontalAlignment.Left : HorizontalAlignment.Right;
        CaptionInset.Width = new GridLength(AppWindow.TitleBar.RightInset / (Root.XamlRoot?.RasterizationScale ?? 1));
        ApplyViewMode();
    }

    private void ApplyTitleBar()
    {
        var dark = Root.ActualTheme == ElementTheme.Dark;
        var background = dark ? ColorHelper.FromArgb(255, 37, 34, 31) : ColorHelper.FromArgb(255, 253, 252, 249);
        AppWindow.TitleBar.BackgroundColor = AppWindow.TitleBar.ButtonBackgroundColor = background;
        AppWindow.TitleBar.ForegroundColor = AppWindow.TitleBar.ButtonForegroundColor = dark ? Colors.White : Colors.Black;
    }

    private void SchedulePreview() { if (previewDelay is not null) { previewDelay.Stop(); previewDelay.Start(); } }

    private async Task RefreshPreviewAsync()
    {
        if (session is null || preview is null || snapshot?.ViewMode == "write") return;
        var sequence = editSequence;
        var id = selectedId;
        var title = TitleBox.Text;
        var content = EditorText.ToCore(ReadBody());
        var layout = PreviewLayout(Root.ActualWidth);
        try
        {
            var reply = await session.ExecuteAsync(new { command = "preview", title, content, dark = Root.ActualTheme == ElementTheme.Dark, layout });
            if (sequence != editSequence || id != selectedId || layout != PreviewLayout(Root.ActualWidth) || reply.Result is not { } result) return;
            await preview.RenderAsync(result.GetProperty("html").GetString() ?? "");
        }
        catch (Exception exception) { ShowError(exception.Message); }
    }

    private async void AppWindow_Closing(AppWindow sender, AppWindowClosingEventArgs args)
    {
        if (allowClose || session is null) return;
        args.Cancel = true;
        if (closing) return;
        closing = true;
        Workspace.IsEnabled = false;
        savePoll.Stop(); previewDelay.Stop();
        try
        {
            if (snapshot?.Recovery is null) await session.ExecuteAsync(new { command = "flush" });
            await session.DisposeAsync();
            session = null;
            allowClose = true;
            Close();
        }
        catch (Exception exception)
        {
            closing = false;
            Workspace.IsEnabled = true;
            ShowError($"Nota could not save your changes. The window will stay open. {exception.Message} Press Ctrl+S to retry.");
        }
    }

    private void ShowError(string message)
    {
        noticeDelay.Stop();
        Notice.Title = "Something needs attention";
        Notice.Message = message;
        Notice.Severity = InfoBarSeverity.Error;
        Notice.IsOpen = true;
    }
}
