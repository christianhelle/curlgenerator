
using CurlGenerator.Core;
using FluentAssertions;
using Microsoft.OpenApi;

namespace CurlGenerator.Tests;

public class OperationNameGeneratorTests
{
    [Fact]
    public void GetOperationName_ValidInput_ReturnsExpectedName()
    {
        var generator = new OperationNameGenerator();
        var document = new OpenApiDocument();
        document.Paths = new OpenApiPaths();
        document.Paths.Add("/my-path", new OpenApiPathItem
        {
            Operations = new Dictionary<HttpMethod, OpenApiOperation>
            {
                { HttpMethod.Get, new OpenApiOperation { OperationId = "my-operation" } }
            }
        });

        var operation = document.Paths["/my-path"].Operations![HttpMethod.Get];

        var result = generator.GetOperationName(document, "/my-path", "get", operation);

        result.Should().Be("GetMyOperation");
    }

    [Fact]
    public void GetOperationName_WithException_ReturnsFallbackName()
    {
        var generator = new OperationNameGenerator();
        var document = new OpenApiDocument();
        document.Paths = new OpenApiPaths();
        document.Paths.Add("/my-path", new OpenApiPathItem
        {
            Operations = new Dictionary<HttpMethod, OpenApiOperation>
            {
                { HttpMethod.Get, new OpenApiOperation { OperationId = null } }
            }
        });

        var operation = document.Paths["/my-path"].Operations![HttpMethod.Get];

        var result = generator.GetOperationName(document, "/my-path", "get", operation);

        result.Should().Be("GetMy-path");
    }

    [Fact]
    public void CheckForDuplicateOperationIds_NoDuplicates_ReturnsFalse()
    {
        var generator = new OperationNameGenerator();
        var document = new OpenApiDocument();
        document.Paths = new OpenApiPaths();
        document.Paths.Add("/my-path", new OpenApiPathItem
        {
            Operations = new Dictionary<HttpMethod, OpenApiOperation>
            {
                { HttpMethod.Get, new OpenApiOperation { OperationId = "my-operation" } }
            }
        });
        document.Paths.Add("/my-other-path", new OpenApiPathItem
        {
            Operations = new Dictionary<HttpMethod, OpenApiOperation>
            {
                { HttpMethod.Get, new OpenApiOperation { OperationId = "my-other-operation" } }
            }
        });

        var result = generator.CheckForDuplicateOperationIds(document);

        result.Should().BeFalse();
    }

    [Fact]
    public void CheckForDuplicateOperationIds_WithDuplicates_ReturnsTrue()
    {
        var generator = new OperationNameGenerator();
        var document = new OpenApiDocument();
        document.Paths = new OpenApiPaths();
        document.Paths.Add("/my-path", new OpenApiPathItem
        {
            Operations = new Dictionary<HttpMethod, OpenApiOperation>
            {
                { HttpMethod.Get, new OpenApiOperation { OperationId = "my-operation" } }
            }
        });
        document.Paths.Add("/my-other-path", new OpenApiPathItem
        {
            Operations = new Dictionary<HttpMethod, OpenApiOperation>
            {
                { HttpMethod.Get, new OpenApiOperation { OperationId = "my-operation" } }
            }
        });

        var result = generator.CheckForDuplicateOperationIds(document);

        result.Should().BeTrue();
    }
}
