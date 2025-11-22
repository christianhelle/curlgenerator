
using CurlGenerator.Validation;
using FluentAssertions;
using Microsoft.OpenApi;

namespace CurlGenerator.Tests;

public class OpenApiStatsTests
{
    [Fact]
    public void Visit_FullOpenApiDocument_CorrectlyCountsAllElements()
    {
        // Arrange
        var document = new OpenApiDocument
        {
            Paths = new OpenApiPaths
            {
                ["/test"] = new OpenApiPathItem
                {
                    Operations = new Dictionary<HttpMethod, OpenApiOperation>
                    {
                        [HttpMethod.Get] = new OpenApiOperation
                        {
                            Parameters = [new OpenApiParameter { In = ParameterLocation.Query, Name = "param1" }],
                            RequestBody = new OpenApiRequestBody(),
                            Responses = new OpenApiResponses
                            {
                                ["200"] = new OpenApiResponse
                                {
                                    Headers = new Dictionary<string, IOpenApiHeader>
                                    {
                                        ["X-Rate-Limit"] = new OpenApiHeader()
                                    }
                                }
                            },
                            Callbacks = new Dictionary<string, IOpenApiCallback> { ["onData"] = new OpenApiCallback() }
                        }
                    }
                }
            },
            Components = new OpenApiComponents
            {
                Schemas = new Dictionary<string, IOpenApiSchema> { ["mySchema"] = new OpenApiSchema() },
                Links = new Dictionary<string, IOpenApiLink> { ["myLink"] = new OpenApiLink() }
            }
        };

        var stats = new OpenApiStats();
        var walker = new OpenApiWalker(stats);

        // Act
        walker.Walk(document);

        // Assert
        stats.PathItemCount.Should().Be(1);
        stats.OperationCount.Should().Be(1);
        stats.ParameterCount.Should().Be(1);
        stats.RequestBodyCount.Should().Be(1);
        stats.ResponseCount.Should().Be(1);
        stats.LinkCount.Should().Be(1);
        stats.CallbackCount.Should().Be(1);
        stats.SchemaCount.Should().Be(1);
        stats.HeaderCount.Should().Be(1);

    }
}
