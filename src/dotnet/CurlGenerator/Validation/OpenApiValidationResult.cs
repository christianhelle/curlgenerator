using Microsoft.OpenApi.Reader;

namespace CurlGenerator.Validation;

public record OpenApiValidationResult(
    OpenApiDiagnostic? Diagnostics,
    OpenApiStats Statistics)
{
    public bool IsValid => Diagnostics is null || Diagnostics.Errors.Count == 0;

    public void ThrowIfInvalid()
    {
        if (!IsValid)
            throw new OpenApiValidationException(this);
    }
}