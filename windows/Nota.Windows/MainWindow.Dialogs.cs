using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Windows.Storage.Pickers;

namespace Nota.Windows;

public sealed partial class MainWindow
{
    private async void Tags_Click(object sender, RoutedEventArgs args)
    {
        if (snapshot?.SelectedNote is not { } note) return;
        var field = new TextBox { Header = "Tags, separated by commas", Text = string.Join(", ", note.Tags), MinWidth = 320 };
        Microsoft.UI.Xaml.Automation.AutomationProperties.SetName(field, "Tags, separated by commas");
        var dialog = Dialog("Edit tags", field);
        dialog.PrimaryButtonText = "Save tags";
        dialog.CloseButtonText = "Cancel";
        dialog.DefaultButton = ContentDialogButton.Primary;
        if (await dialog.ShowAsync() != ContentDialogResult.Primary || selectedId != note.Id) return;
        await TransitionAsync(new { command = "edit_note", id = note.Id, edit_sequence = ++editSequence, title = TitleBox.Text, content = Interop.EditorText.ToCore(ReadBody()), tags_input = field.Text });
        SchedulePreview();
    }

    private void Settings_Click(object sender, RoutedEventArgs args)
    {
        var menu = new MenuFlyout { Placement = FlyoutPlacementMode.Bottom };
        foreach (var theme in new[] { "system", "light", "dark" })
        {
            var item = new ToggleMenuFlyoutItem { Text = char.ToUpperInvariant(theme[0]) + theme[1..], IsChecked = snapshot?.Theme == theme };
            item.Click += async (_, _) =>
            {
                await RunAsync(new { command = "set_theme", theme });
                deletedSignature = null;
                if (snapshot is not null) ReconcileDeleted(snapshot);
                SchedulePreview();
            };
            menu.Items.Add(item);
        }
        menu.Items.Add(new MenuFlyoutSeparator());
        var help = new MenuFlyoutItem { Text = "Markdown help" };
        help.Click += Help_Click;
        menu.Items.Add(help);
        var about = new MenuFlyoutItem { Text = "About Nota" };
        about.Click += About_Click;
        menu.Items.Add(about);
        menu.ShowAt((FrameworkElement)sender);
    }

    private void Backup_Click(object sender, RoutedEventArgs args)
    {
        var menu = new MenuFlyout();
        var export = new MenuFlyoutItem { Text = "Export notes backup…" };
        export.Click += async (_, _) => await ExportAsync(false);
        var import = new MenuFlyoutItem { Text = "Import notes backup…" };
        import.Click += Import_Click;
        var complete = new MenuFlyoutItem { Text = "Export complete notebook…" };
        complete.Click += async (_, _) => await ExportAsync(true);
        var restore = new MenuFlyoutItem { Text = "Restore complete notebook…" };
        restore.Click += Restore_Click;
        menu.Items.Add(export);
        menu.Items.Add(import);
        menu.Items.Add(complete);
        menu.Items.Add(restore);
        menu.Items.Add(new MenuFlyoutSeparator());
        var status = new MenuFlyoutItem { Text = "Backup status" };
        status.Click += async (_, _) =>
        {
            var dialog = Dialog("Backup status", snapshot?.BackupHealth ?? "Opening notebook…");
            dialog.CloseButtonText = "Close";
            await dialog.ShowAsync();
        };
        menu.Items.Add(status);
        menu.ShowAt((FrameworkElement)sender);
    }

    private ContentDialog Dialog(string title, object content)
    {
        var dialog = new ContentDialog
        {
            XamlRoot = Root.XamlRoot,
            RequestedTheme = Root.ActualTheme,
            Title = title,
            Content = content is string text ? new TextBlock { Text = text, TextWrapping = TextWrapping.Wrap, MaxWidth = 440 } : content
        };
        var palette = (ResourceDictionary)Application.Current.Resources.ThemeDictionaries[Root.ActualTheme.ToString()];
        foreach (var entry in palette)
        {
            if (entry.Key is string key && (key.StartsWith("AccentButton", StringComparison.Ordinal) || key.StartsWith("ContentDialog", StringComparison.Ordinal)))
                dialog.Resources[key] = entry.Value;
        }
        return dialog;
    }

    private async Task<bool> ConfirmAsync(string title, string message, string action)
    {
        var dialog = Dialog(title, message);
        dialog.PrimaryButtonText = action;
        dialog.CloseButtonText = "Cancel";
        dialog.DefaultButton = ContentDialogButton.Close;
        return await dialog.ShowAsync() == ContentDialogResult.Primary;
    }

    private async void About_Click(object sender, RoutedEventArgs args)
    {
        var dialog = Dialog("About Nota", "A quiet place for your notes.\n\nNota keeps your notebook on this device. Write in Markdown, organize with tags, and export backups you control.\n\nNative Windows edition\nOpen source under the MIT license.\n\nGelasio by the Gelasio Project Authors. Source Sans 3 and Source Code Pro by Adobe. Fonts licensed under the SIL Open Font License.\n\nLucide icons licensed under the ISC license.");
        dialog.CloseButtonText = "Close";
        await dialog.ShowAsync();
    }

    private async void Help_Click(object sender, RoutedEventArgs args)
    {
        var dialog = Dialog("Markdown help", "# Heading\n## Smaller heading\n\n**Bold**    *Italic*    ~~Strikethrough~~\n\n- A list item\n- [ ] A task to do\n- [x] A completed task\n\n[Link text](https://example.com)\n`Inline code`\n\nUse Write to edit, Preview to read, and Split to see both. External images and scripts are blocked in Preview.\n\nCtrl+N  New note\nCtrl+F  Search notes\nCtrl+B  Bold selection\nCtrl+I  Italic selection\nCtrl+S  Save now\nCtrl+Z  Undo");
        dialog.CloseButtonText = "Close";
        await dialog.ShowAsync();
    }

    private async Task ExportAsync(bool transition)
    {
        try
        {
            var picker = new FileSavePicker { SuggestedStartLocation = PickerLocationId.DocumentsLibrary, SuggestedFileName = $"nota-{(transition ? "notebook" : "backup")}-{DateTime.Now:yyyy-MM-dd}" };
            picker.FileTypeChoices.Add("JSON backup", [".json"]);
            WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this));
            var file = await picker.PickSaveFileAsync();
            if (file is not null) await RunAsync(new { command = transition ? "export_transition" : "export_backup", path = file.Path });
        }
        catch (Exception exception) { ShowError(exception.Message); }
    }

    private async Task<string?> PickJsonAsync()
    {
        var picker = new FileOpenPicker { SuggestedStartLocation = PickerLocationId.DocumentsLibrary };
        picker.FileTypeFilter.Add(".json");
        WinRT.Interop.InitializeWithWindow.Initialize(picker, WinRT.Interop.WindowNative.GetWindowHandle(this));
        var file = await picker.PickSingleFileAsync();
        if (file is null) return null;
        return await File.ReadAllTextAsync(file.Path);
    }

    private async void Import_Click(object sender, RoutedEventArgs args)
    {
        try
        {
            var json = await PickJsonAsync();
            if (json is null) return;
            var reply = await RunAsync(new { command = "import_backup", json });
            if (reply?.Snapshot.PendingImport is not { } import) return;
            var confirmed = await ConfirmAsync("Import backup?", $"Notes in this backup: {import.TotalImportedNotes}\n\nNotes to add: {import.NotesToAdd}\nNotes to replace: {import.NotesToReplace}\n\nExisting notes with the same identity will be replaced. Other notes will stay in your notebook.", "Import");
            if (confirmed) await TransitionAsync(new { command = "confirm_import" }, replaceEditor: true);
            else await RunAsync(new { command = "cancel_import" });
        }
        catch (Exception exception) { ShowError(exception.Message); }
    }

    private async void Restore_Click(object sender, RoutedEventArgs args) => await RestoreFileAsync();

    private async Task RestoreFileAsync()
    {
        try
        {
            if (snapshot is not null && (snapshot.Rows.Length > 0 || snapshot.SelectedNote is not null || snapshot.RecentlyDeleted.Length > 0))
            {
                var unavailable = Dialog("Restore requires an empty notebook", "Use Import notes backup to combine notes with this notebook. To restore a complete notebook, open Nota with an empty data folder using --data-dir.");
                unavailable.CloseButtonText = "Close";
                await unavailable.ShowAsync();
                return;
            }
            var json = await PickJsonAsync();
            if (json is null) return;
            if (await ConfirmAsync("Restore this notebook?", "This brings the saved notes, Recently Deleted, and settings into this empty notebook.", "Restore"))
                await TransitionAsync(new { command = "import_transition", json }, replaceEditor: true);
        }
        catch (Exception exception) { ShowError(exception.Message); }
    }

    private async Task CheckRecoveryAsync()
    {
        if (showingRecovery || snapshot?.Recovery is null) return;
        showingRecovery = true;
        try
        {
            while (snapshot?.Recovery is { } recovery)
            {
                Workspace.IsEnabled = false;
                var dialog = Dialog("Your notebook needs recovery", $"{recovery.Reason}\n\nNota will preserve the unreadable file before recovery. You can restore the previous saved notebook, or start an empty notebook.");
                dialog.PrimaryButtonText = "Restore previous";
                dialog.IsPrimaryButtonEnabled = recovery.CanRestorePrevious;
                dialog.SecondaryButtonText = "Start empty";
                dialog.CloseButtonText = "Close Nota";
                dialog.DefaultButton = ContentDialogButton.Close;
                var result = await dialog.ShowAsync();
                if (result == ContentDialogResult.None) { Close(); return; }
                if (result == ContentDialogResult.Secondary && !await ConfirmAsync("Start an empty notebook?", "The unreadable notebook will be preserved on this device. Your new notebook will contain no notes.", "Start empty")) continue;
                var reply = await RunAsync(new { command = result == ContentDialogResult.Primary ? "restore_previous" : "start_empty" }, replaceEditor: true);
                if (reply is null)
                {
                    var failed = Dialog("Recovery did not finish", Notice.Message);
                    failed.CloseButtonText = "Back to recovery";
                    await failed.ShowAsync();
                }
            }
        }
        finally { showingRecovery = false; Workspace.IsEnabled = true; }
    }

    private async void InstallWebView_Click(object sender, RoutedEventArgs args)
        => await global::Windows.System.Launcher.LaunchUriAsync(new Uri("https://developer.microsoft.com/microsoft-edge/webview2/"));
}
