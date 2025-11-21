
using CurlGenerator.Validation;
using FluentAssertions;
using Microsoft.OpenApi;
using Microsoft.OpenApi.Reader;

namespace CurlGenerator.Tests;

public class OpenApiValidationResultTests
{
    [Fact]
    public void IsValid_WithNoErrors_ReturnsTrue()
    {
        var diagnostics = new OpenApiDiagnostic();
        var stats = new OpenApiStats();
        var result = new OpenApiValidationResult(diagnostics, stats);

        result.IsValid.Should().BeTrue();
    }

    [Fact]
    public void IsValid_WithErrors_ReturnsFalse()
    {
        var diagnostics = new OpenApiDiagnostic();
        diagnostics.Errors.Add(new OpenApiError("", "Error"));
        var stats = new OpenApiStats();
        var result = new OpenApiValidationResult(diagnostics, stats);

        result.IsValid.Should().BeFalse();
    }

    [Fact]
    public void ThrowIfInvalid_WithNoErrors_DoesNotThrow()
    {
        var diagnostics = new OpenApiDiagnostic();
        var stats = new OpenApiStats();
        var result = new OpenApiValidationResult(diagnostics, stats);

        var action = () => result.ThrowIfInvalid();

        action.Should().NotThrow();
    }

    [Fact]
    public void ThrowIfInvalid_WithErrors_ThrowsException()
    {
        var diagnostics = new OpenApiDiagnostic();
        diagnostics.Errors.Add(new OpenApiError("", "Error"));
        var stats = new OpenApiStats();
        var result = new OpenApiValidationResult(diagnostics, stats);

        var action = () => result.ThrowIfInvalid();

        action.Should().Throw<OpenApiValidationException>();
    }
}
