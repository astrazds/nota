using System.Collections.ObjectModel;
using Microsoft.UI;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Nota.Windows.Interop;
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
    private bool compactEditor;
    private string? deletedSignature;
    private string? tagSignature;
    private Notification? shownNotification;
    private readonly DispatcherQueueTimer noticeDelay;

    public MainWindow(string dataDirectory, string? error)
    {
        InitializeComponent();
        var glyph = new TextBlock { Text = "0", FontFamily = BodyBox.FontFamily, FontSize = BodyBox.FontSize };
        glyph.Measure(new global::Windows.Foundation.Size(double.PositiveInfinity, double.PositiveInfinity));
        BodyMeasureColumn.MaxWidth = glyph.DesiredSize.Width * 72;
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
        Closed += (_, _) => { savePoll.Stop(); previewDelay.Stop(); noticeDelay.Stop(); PreviewWeb.Close(); };
        AddShortcut(VirtualKey.N, CreateNoteAsync);
        AddShortcut(VirtualKey.F, () => { compactEditor = false; UpdateLayoutMode(); SearchBox.Focus(FocusState.Keyboard); return Task.CompletedTask; });
        AddShortcut(VirtualKey.B, () => FormatAsync("bold"));
        AddShortcut(VirtualKey.I, () => FormatAsync("italic"));
        AddShortcut(VirtualKey.S, async () => await RunAsync(new { command = "flush" }));
    }

    private void AddShortcut(VirtualKey key, Func<Task> action)
    {
        var accelerator = new KeyboardAccelerator { Key = key, Modifiers = VirtualKeyModifiers.Control };
        accelerator.Invoked += async (_, args) => { args.Handled = true; await action(); };
        Root.KeyboardAccelerators.Add(accelerator);
    }

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
        try
        {
            var reply = await RunAsync(command, replaceEditor);
            if (reply is not null) { succeeded = true; compactEditor = true; UpdateLayoutMode(); }
        }
        finally
        {
            transitioning = false;
            EditorPane.IsEnabled = true;
            NewButton.IsEnabled = true;
            NoteList.IsEnabled = true;
        }
        if (succeeded && focusTitle && selectedId is not null)
        {
            TitleBox.Focus(FocusState.Programmatic);
            TitleBox.SelectAll();
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
            ThemeButton.Content = "Theme: " + char.ToUpperInvariant(value.Theme[0]) + value.Theme[1..];
            if (replaceEditor || selectedId != value.SelectedNote?.Id)
            {
                selectedId = value.SelectedNote?.Id;
                TitleBox.Text = value.SelectedNote?.Title ?? "";
                BodyBox.Text = EditorText.ToNative(value.SelectedNote?.Content ?? "");
                TagsBox.Text = string.Join(", ", value.SelectedNote?.Tags ?? []);
                BodyBox.Select(0, 0);
                UpdateCounts();
                SchedulePreview();
            }
            TitleBox.IsEnabled = TagsBox.IsEnabled = BodyBox.IsEnabled = selectedId is not null && value.Recovery is null;
            Toolbar.IsHitTestVisible = selectedId is not null;
            ReconcileRows(value.Rows);
            NoteList.SelectedItem = rows.FirstOrDefault(row => row.Id == selectedId);
            NoteCount.Text = $"Notes   {value.Rows.Length}";
            EmptyNotes.Visibility = rows.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
            EmptyNotes.Text = value.SearchInput.Length > 0 || value.ActiveTag is not null ? "No notes match this search." : "A little room for your thoughts.\nCreate a note to start writing.";
            BackupHealth.Text = value.BackupHealth;
            SaveStatus.Text = value.SaveStatus switch { "saving" => "Saving…", "failed" => "Save failed", _ => "Saved" };
            ToolTipService.SetToolTip(SaveStatus, value.SaveStatus == "failed" ? "Press Ctrl+S to retry saving. Your changes remain in memory." : "Notes are saved on this device");
            ReconcileTags(value);
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

    private void ReconcileTags(Snapshot value)
    {
        var signature = System.Text.Json.JsonSerializer.Serialize(new { value.Tags, value.ActiveTag });
        if (signature == tagSignature) return;
        tagSignature = signature;
        TagFilters.Items.Clear();
        if (value.Tags.Length == 0) return;
        foreach (var filter in new string?[] { null }.Concat(value.Tags))
        {
            var button = new Button { Content = filter is null ? "All notes" : "#" + filter, FontSize = 11, Padding = new Thickness(8, 3, 8, 3), Margin = new Thickness(0, 0, 4, 4), CornerRadius = new CornerRadius(10) };
            if (filter == value.ActiveTag) button.BorderBrush = (Brush)Application.Current.Resources["SignalBrush"];
            button.Click += async (_, _) => await RunAsync(new { command = "filter_tag", tag = filter });
            TagFilters.Items.Add(button);
        }
    }

    private void ReconcileDeleted(Snapshot value)
    {
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

    private Brush DangerBrush() => new SolidColorBrush(Root.ActualTheme == ElementTheme.Dark ? ColorHelper.FromArgb(255, 240, 140, 128) : ColorHelper.FromArgb(255, 175, 52, 43));

    private async void Editor_TextChanged(object sender, TextChangedEventArgs args)
    {
        if (applying || selectedId is null || session is null) return;
        var content = EditorText.ToCore(BodyBox.Text);
        if (snapshot?.SelectedNote is { } note && note.Id == selectedId
            && TitleBox.Text == note.Title && content == EditorText.ToCore(note.Content)
            && TagsBox.Text == string.Join(", ", note.Tags)) return;
        var id = selectedId;
        var sequence = ++editSequence;
        UpdateCounts();
        SchedulePreview();
        await RunAsync(new { command = "edit_note", id, edit_sequence = sequence, title = TitleBox.Text, content, tags_input = TagsBox.Text });
    }

    private void UpdateCounts()
    {
        var text = EditorText.ToCore(BodyBox.Text);
        var words = text.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries).Length;
        var lines = text.Length == 0 ? 0 : text.Count(c => c == '\n') + 1;
        var characters = text.EnumerateRunes().Count();
        WordCount.Text = $"{lines} {(lines == 1 ? "line" : "lines")} · {words} {(words == 1 ? "word" : "words")} · {characters} {(characters == 1 ? "char" : "chars")}";
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
            compactEditor = true;
            UpdateLayoutMode();
        }
    }

    private Task CreateNoteAsync() => TransitionAsync(new { command = "new_note" }, focusTitle: true);
    private async void New_Click(object sender, RoutedEventArgs args) => await CreateNoteAsync();
    private void Back_Click(object sender, RoutedEventArgs args) { compactEditor = false; UpdateLayoutMode(); }

    private void NoteActions_Click(object sender, RoutedEventArgs args)
    {
        if (sender is not Button { Tag: string id } button || snapshot is null) return;
        var flyout = new MenuFlyout();
        var pin = new MenuFlyoutItem { Text = snapshot.Rows.First(row => row.Id == id).IsPinned ? "Unpin note" : "Pin note" };
        pin.Click += async (_, _) => await RunAsync(new { command = "toggle_pin", id });
        var delete = new MenuFlyoutItem { Text = "Move to Recently Deleted" };
        delete.Click += async (_, _) => { if (await ConfirmAsync("Move this note to Recently Deleted?", "You can restore it from the sidebar.", "Move note")) await TransitionAsync(new { command = "delete_note", id }); };
        flyout.Items.Add(pin); flyout.Items.Add(delete); flyout.ShowAt(button);
    }

    private async void ClearDeleted_Click(object sender, RoutedEventArgs args)
    {
        if (await ConfirmAsync("Clear Recently Deleted?", "All notes in Recently Deleted will be permanently removed. This cannot be undone.", "Clear all")) await RunAsync(new { command = "clear_deleted" });
    }

    private async void Format_Click(object sender, RoutedEventArgs args)
    {
        if (sender is Button { Tag: string kind }) await FormatAsync(kind);
    }

    private async Task FormatAsync(string kind)
    {
        if (session is null || selectedId is null || transitioning || snapshot?.ViewMode == "preview") return;
        var nativeContent = BodyBox.Text;
        var content = EditorText.ToCore(nativeContent);
        var id = selectedId;
        var sequence = editSequence;
        var start = EditorText.ToCoreOffset(nativeContent, BodyBox.SelectionStart);
        var end = EditorText.ToCoreOffset(nativeContent, BodyBox.SelectionStart + BodyBox.SelectionLength);
        try
        {
            var reply = await session.ExecuteAsync(new { command = "format", content, start_utf16 = start, end_utf16 = end, kind });
            if (id != selectedId || sequence != editSequence || BodyBox.Text != nativeContent || reply.Result is not { } result) return;
            var formatted = result.GetProperty("content").GetString() ?? "";
            var updated = EditorText.ToNative(formatted);
            var prefix = 0;
            while (prefix < Math.Min(nativeContent.Length, updated.Length) && nativeContent[prefix] == updated[prefix]) prefix++;
            if (prefix > 0 && prefix < nativeContent.Length && char.IsLowSurrogate(nativeContent[prefix])) prefix--;
            var suffix = 0;
            while (suffix < Math.Min(nativeContent.Length, updated.Length) - prefix && nativeContent[^(suffix + 1)] == updated[^(suffix + 1)]) suffix++;
            if (suffix > 0 && char.IsLowSurrogate(nativeContent[nativeContent.Length - suffix])) suffix--;
            BodyBox.Focus(FocusState.Programmatic);
            BodyBox.Select(prefix, nativeContent.Length - prefix - suffix);
            BodyBox.SelectedText = updated.Substring(prefix, updated.Length - prefix - suffix);
            BodyBox.Select(EditorText.ToNativeOffset(formatted, result.GetProperty("caret_utf16").GetInt32()), 0);
        }
        catch (Exception exception) { ShowError(exception.Message); }
    }

    private async void Theme_Click(object sender, RoutedEventArgs args)
    {
        var theme = snapshot?.Theme switch { "system" => "light", "light" => "dark", _ => "system" };
        await RunAsync(new { command = "set_theme", theme });
        deletedSignature = null;
        if (snapshot is not null) ReconcileDeleted(snapshot);
        SchedulePreview();
    }

    private async void Mode_Click(object sender, RoutedEventArgs args)
    {
        if (sender is ToggleButton { Tag: string mode }) await RunAsync(new { command = "set_view_mode", mode });
        SchedulePreview();
    }

    private void ApplyViewMode()
    {
        var mode = snapshot?.ViewMode ?? "write";
        if (Root.ActualWidth < 900 && mode == "split") mode = "write";
        WriteMode.IsChecked = mode == "write"; PreviewMode.IsChecked = mode == "preview"; SplitMode.IsChecked = mode == "split";
        SplitMode.IsEnabled = Root.ActualWidth >= 900;
        BodyBox.Visibility = mode == "preview" ? Visibility.Collapsed : Visibility.Visible;
        Toolbar.Visibility = mode == "preview" ? Visibility.Collapsed : Visibility.Visible;
        PreviewPane.Visibility = mode == "write" ? Visibility.Collapsed : Visibility.Visible;
        WriteColumn.Width = mode == "preview" ? new GridLength(0) : new GridLength(1, GridUnitType.Star);
        PreviewColumn.Width = mode == "write" ? new GridLength(0) : new GridLength(1, GridUnitType.Star);
        PreviewPane.Margin = mode == "split" ? new Thickness(20, 0, 0, 0) : new Thickness(0);
    }

    private void Root_SizeChanged(object sender, SizeChangedEventArgs args) => UpdateLayoutMode();

    private void UpdateLayoutMode()
    {
        var compact = Root.ActualWidth < 720;
        SidebarColumn.Width = compact ? new GridLength(compactEditor ? 0 : 1, compactEditor ? GridUnitType.Pixel : GridUnitType.Star) : new GridLength(288);
        Sidebar.Visibility = compact && compactEditor ? Visibility.Collapsed : Visibility.Visible;
        EditorPane.Visibility = compact && !compactEditor ? Visibility.Collapsed : Visibility.Visible;
        Grid.SetColumnSpan(Sidebar, compact ? 2 : 1);
        BackButton.Visibility = compact ? Visibility.Visible : Visibility.Collapsed;
        WordCount.Visibility = Root.ActualWidth < 520 ? Visibility.Collapsed : Visibility.Visible;
        ApplyViewMode();
    }

    private void ApplyTitleBar()
    {
        var dark = Root.ActualTheme == ElementTheme.Dark;
        var background = dark ? ColorHelper.FromArgb(255, 33, 31, 28) : ColorHelper.FromArgb(255, 240, 237, 230);
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
        var content = EditorText.ToCore(BodyBox.Text);
        try
        {
            var reply = await session.ExecuteAsync(new { command = "preview", title, content, tags = TagsBox.Text.Split(',', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries), dark = Root.ActualTheme == ElementTheme.Dark });
            if (sequence != editSequence || id != selectedId || reply.Result is not { } result) return;
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
