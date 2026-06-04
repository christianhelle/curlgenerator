using FluentAssertions;
using CurlGenerator.Core;
using CurlGenerator.Tests.Resources;

namespace CurlGenerator.Tests;

public class OpenApiDocumentFactoryTests
{
    [Theory]
    [InlineData("https://developers.intellihr.io/docs/v1/swagger.json")] // GZIP encoded
    [InlineData("http://raw.githubusercontent.com/christianhelle/curlgenerator/main/test/OpenAPI/v3.0/petstore.json")]
    public async Task Create_From_Uri_Returns_NotNull(string url)
    {
        (await OpenApiDocumentFactory.CreateAsync(url))
            .Should()
            .NotBeNull();
    }
    
    [Theory]
    [InlineData(Samples.PetstoreJsonV3, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV3, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV2, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV2, "SwaggerPetstore.yaml")]
    public async Task Create_From_File_Returns_NotNull(Samples version, string filename)
    {
        var swaggerFile = await TestFile.CreateSwaggerFile(EmbeddedResources.GetSwaggerPetstore(version), filename);
        (await OpenApiDocumentFactory.CreateAsync(swaggerFile))
            .Should()
            .NotBeNull();
    }
}