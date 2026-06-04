using System.Diagnostics.CodeAnalysis;

namespace CurlGenerator.Core;

[ExcludeFromCodeCoverage]
public record GeneratorResult(IReadOnlyCollection<ScriptFile> Files)
{
    public IReadOnlyCollection<ScriptFile> Files { get; } = Files;
}