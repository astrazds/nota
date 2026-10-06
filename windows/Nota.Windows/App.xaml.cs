using Microsoft.UI.Xaml;

namespace Nota.Windows;

public partial class App : Application
{
    private MainWindow? window;

    public App() => InitializeComponent();

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        string? error = null;
        var directory = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "net.astrazds.Nota");
        var arguments = Environment.GetCommandLineArgs().Skip(1).ToArray();
        if (arguments.Length != 0)
        {
            if (arguments.Length == 2 && arguments[0] == "--data-dir" && Path.IsPathFullyQualified(arguments[1]))
                directory = Path.GetFullPath(arguments[1]);
            else
                error = "Start Nota with --data-dir followed by an absolute folder path, or with no arguments.";
        }
        UnhandledException += (_, eventArgs) =>
        {
            try
            {
                Directory.CreateDirectory(directory);
                File.AppendAllText(Path.Combine(directory, "native-error.log"), $"{DateTimeOffset.Now:O}\n{eventArgs.Message}\n{eventArgs.Exception}\n");
            }
            catch (IOException) { }
            catch (UnauthorizedAccessException) { }
        };
        window = new MainWindow(directory, error);
        window.Activate();
    }
}
