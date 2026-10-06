using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.Web.WebView2.Core;
using Nota.Windows.Interop;

namespace Nota.Windows;

internal sealed class NativePreview(WebView2 view, StackPanel message, TextBlock messageText, Button installButton, string directory, Func<object, Task<Reply>> execute)
{
    private Task? initialization;
    private bool ready;
    private bool allowDocument;
    private string? documentUri;
    private ulong generation;

    public async Task RenderAsync(string html)
    {
        var requested = ++generation;
        initialization ??= InitializeAsync();
        await initialization;
        if (!ready || requested != generation) return;
        documentUri = "data:text/html;charset=utf-8;base64," + Convert.ToBase64String(System.Text.Encoding.UTF8.GetBytes(html));
        allowDocument = true;
        view.CoreWebView2.NavigateToString(html);
        view.Visibility = Visibility.Visible;
        message.Visibility = Visibility.Collapsed;
    }

    private async Task InitializeAsync()
    {
        try
        {
            var options = new CoreWebView2EnvironmentOptions { ScrollBarStyle = CoreWebView2ScrollbarStyle.FluentOverlay };
            var environment = await CoreWebView2Environment.CreateWithOptionsAsync(null, Path.Combine(directory, "preview-cache"), options);
            await view.EnsureCoreWebView2Async(environment);
            var core = view.CoreWebView2;
            var settings = core.Settings;
            settings.IsScriptEnabled = false;
            settings.IsWebMessageEnabled = false;
            settings.AreHostObjectsAllowed = false;
            settings.AreDefaultContextMenusEnabled = false;
            settings.AreDevToolsEnabled = false;
            settings.IsStatusBarEnabled = false;
            settings.IsBuiltInErrorPageEnabled = false;
            settings.IsGeneralAutofillEnabled = false;
            settings.IsPasswordAutosaveEnabled = false;
            settings.AreBrowserAcceleratorKeysEnabled = false;
            core.NavigationStarting += async (_, args) =>
            {
                if (allowDocument && (args.Uri == "about:blank" || args.Uri == documentUri)) { allowDocument = false; return; }
                if (args.Uri.StartsWith("about:blank#", StringComparison.Ordinal) || (documentUri is not null && args.Uri.StartsWith(documentUri + "#", StringComparison.Ordinal))) return;
                args.Cancel = true;
                await OpenExternalAsync(args.Uri, args.IsUserInitiated);
            };
            core.FrameNavigationStarting += (_, args) => args.Cancel = true;
            core.NewWindowRequested += async (_, args) =>
            {
                args.Handled = true;
                await OpenExternalAsync(args.Uri, args.IsUserInitiated);
            };
            core.DownloadStarting += (_, args) => args.Cancel = true;
            core.PermissionRequested += (_, args) => args.State = CoreWebView2PermissionState.Deny;
            core.AddWebResourceRequestedFilter("*", CoreWebView2WebResourceContext.All);
            core.WebResourceRequested += (_, args) =>
            {
                if (args.Request.Uri.StartsWith("data:", StringComparison.OrdinalIgnoreCase) || args.Request.Uri == "about:blank") return;
                args.Response = environment.CreateWebResourceResponse(null, 403, "External resources are disabled", "Content-Type: text/plain");
            };
            core.ProcessFailed += (_, _) => ShowFailure("Preview stopped unexpectedly. Close and reopen Nota to restart it. Your note is still available in Write mode.", false);
            core.NavigationCompleted += (_, args) =>
            {
                if (!args.IsSuccess && args.WebErrorStatus != CoreWebView2WebErrorStatus.OperationCanceled)
                    ShowFailure("Preview could not display this note. Your note is still available in Write mode.", false);
            };
            ready = true;
        }
        catch (Exception exception) when (exception.HResult is unchecked((int)0x80070002) or unchecked((int)0x80070003))
        {
            ShowFailure("Preview needs Microsoft Edge WebView2 Runtime. Install the runtime, then reopen Nota. You can continue writing and saving notes.", true);
        }
        catch (Exception exception)
        {
            ShowFailure($"Preview could not start: {exception.Message} You can continue in Write mode.", false);
        }
    }

    private void ShowFailure(string text, bool install)
    {
        ready = false;
        view.Visibility = Visibility.Collapsed;
        message.Visibility = Visibility.Visible;
        messageText.Text = text;
        installButton.Visibility = install ? Visibility.Visible : Visibility.Collapsed;
    }

    private async Task OpenExternalAsync(string uri, bool userActivated)
    {
        try
        {
            var reply = await execute(new { command = "external_navigation", uri, user_activated = userActivated });
            if (reply.Result is { } result && result.TryGetProperty("uri", out var target) && target.GetString() is { } approved)
                await global::Windows.System.Launcher.LaunchUriAsync(new Uri(approved));
        }
        catch (Exception exception)
        {
            messageText.Text = $"This link could not be opened: {exception.Message}";
            message.Visibility = Visibility.Visible;
        }
    }
}
