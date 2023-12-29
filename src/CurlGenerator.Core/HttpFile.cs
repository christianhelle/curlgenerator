using System.Diagnostics.CodeAnalysis;

namespace CurlGenerator.Core;

[ExcludeFromCodeCoverage]
public record ScriptFile(string Filename, string Content)
{
    public string Filename { get; } = Filename;
    public string Content { get; } = Content;
}