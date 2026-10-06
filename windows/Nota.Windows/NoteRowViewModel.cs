using System.ComponentModel;
using System.Runtime.CompilerServices;
using Nota.Windows.Interop;

namespace Nota.Windows;

public sealed class NoteRowViewModel(NoteRow row) : INotifyPropertyChanged
{
    private NoteRow row = row;
    public string Id => row.Id;
    public string DisplayTitle => (row.IsPinned ? "•  " : "") + (string.IsNullOrWhiteSpace(row.Title) ? "Untitled note" : row.Title);
    public string Preview => row.Preview;
    public string Date => row.Date;
    public event PropertyChangedEventHandler? PropertyChanged;

    public void Update(NoteRow value)
    {
        if (row == value) return;
        row = value;
        Changed(nameof(DisplayTitle));
        Changed(nameof(Preview));
        Changed(nameof(Date));
    }

    private void Changed([CallerMemberName] string? property = null) => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(property));
}
