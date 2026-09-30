using System.ComponentModel;
using System.Runtime.CompilerServices;
using Nota.Windows.Interop;

namespace Nota.Windows;

public sealed class NoteRowViewModel(NoteRow row) : INotifyPropertyChanged
{
    private NoteRow row = row;
    public string Id => row.Id;
    public string DisplayTitle => (row.IsPinned ? "•  " : "") + (string.IsNullOrWhiteSpace(row.Title) ? "Untitled note" : row.Title);
    public string Detail => $"{row.Date}   {row.Preview}";
    public event PropertyChangedEventHandler? PropertyChanged;

    public void Update(NoteRow value)
    {
        if (row == value) return;
        row = value;
        Changed(nameof(DisplayTitle));
        Changed(nameof(Detail));
    }

    private void Changed([CallerMemberName] string? property = null) => PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(property));
}
