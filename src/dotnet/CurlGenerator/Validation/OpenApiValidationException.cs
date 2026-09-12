using System.Diagnostics.CodeAnalysis;

namespace CurlGenerator.Validation;

[ExcludeFromCodeCoverage]
[Serializable]
public class OpenApiValidationException(OpenApiValidationResult validationResult)
    : Exception("OpenAPI validation failed")
{
    public OpenApiValidationResult ValidationResult { get; } = validationResult;
}