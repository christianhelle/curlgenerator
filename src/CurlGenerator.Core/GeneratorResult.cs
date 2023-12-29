using System.Diagnostics.CodeAnalysis;

namespace CurlGenerator.Core;

[ExcludeFromCodeCoverage]
public record GeneratorResult(IReadOnlyCollection<HttpFile> Files)
{
    public IReadOnlyCollection<HttpFile> Files { get; } = Files;
}