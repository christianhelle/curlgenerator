using FluentAssertions;
using FluentAssertions.Execution;
using CurlGenerator.Core;
using CurlGenerator.Tests.Resources;

namespace CurlGenerator.Tests;

public class SwaggerPetstoreTests
{
    private const string Https = "https";
    private const string Http = "http";

    private const string HttpsUrlPrefix =
        Https + "://raw.githubusercontent.com/christianhelle/curlgenerator/main/test/OpenAPI/v3.0/";

    private const string HttpUrlPrefix =
        Http + "://raw.githubusercontent.com/christianhelle/curlgenerator/main/test/OpenAPI/v3.0/";

    [Theory]
    [InlineData(Samples.PetstoreJsonV3, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV3, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV2, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV2, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV3, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV3, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV2, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV2, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV3WithDifferentHeaders, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV3WithDifferentHeaders, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV2WithDifferentHeaders, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV2WithDifferentHeaders, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV3WithDifferentHeaders, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV3WithDifferentHeaders, "SwaggerPetstore.yaml")]
    [InlineData(Samples.PetstoreJsonV2WithDifferentHeaders, "SwaggerPetstore.json")]
    [InlineData(Samples.PetstoreYamlV2WithDifferentHeaders, "SwaggerPetstore.yaml")]
    public async Task Can_Generate_Code(Samples version, string filename)
    {
        var generateCode = await GenerateCode(version, filename);
        
        using var scope = new AssertionScope();
        generateCode.Should().NotBeNull();
        generateCode.Files.Should().NotBeNullOrEmpty();
        generateCode.Files
            .All(file => file.Content.Count(c => c == '#') >= 2)
            .Should()
            .BeTrue();
    }

    [Theory]
    [InlineData(HttpsUrlPrefix + "petstore.json")]
    [InlineData(HttpsUrlPrefix + "petstore.yaml")]
    [InlineData(HttpUrlPrefix + "petstore.json")]
    [InlineData(HttpUrlPrefix + "petstore.yaml")]
    [InlineData(HttpsUrlPrefix + "petstore.json")]
    [InlineData(HttpsUrlPrefix + "petstore.yaml")]
    [InlineData(HttpUrlPrefix + "petstore.json")]
    [InlineData(HttpUrlPrefix + "petstore.yaml")]
    public async Task Can_Generate_Code_From_Url(string url)
    {
        var generateCode = await ScriptFileGenerator.Generate(
            new GeneratorSettings
            {
                OpenApiPath = url,
                AuthorizationHeader = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
            });

        using var scope = new AssertionScope();
        generateCode.Should().NotBeNull();
        generateCode.Files.Should().NotBeNullOrEmpty();
        generateCode.Files
            .All(file => file.Content.Count(c => c == '#') >= 2)
            .Should()
            .BeTrue();
    }

    [Theory]
    [InlineData(HttpsUrlPrefix + "petstore.json")]
    [InlineData(HttpsUrlPrefix + "petstore.yaml")]
    [InlineData(HttpUrlPrefix + "petstore.json")]
    [InlineData(HttpUrlPrefix + "petstore.yaml")]
    [InlineData(HttpsUrlPrefix + "petstore.json")]
    [InlineData(HttpsUrlPrefix + "petstore.yaml")]
    [InlineData(HttpUrlPrefix + "petstore.json")]
    [InlineData(HttpUrlPrefix + "petstore.yaml")]
    public async Task Files_Generated_From_Url_Uses_OpenApiPath_Authority_As_For_BaseUrl(string url)
    {
        var generateCode = await ScriptFileGenerator.Generate(
            new GeneratorSettings
            {
                OpenApiPath = url
            });

        generateCode
            .Files
            .All(file => file.Content.Contains(new Uri(url).GetLeftPart(UriPartial.Authority)))
            .Should()
            .BeTrue();
    }

    private static async Task<GeneratorResult> GenerateCode(Samples version, string filename)
    {
        var json = EmbeddedResources.GetSwaggerPetstore(version);
        var swaggerFile = await TestFile.CreateSwaggerFile(json, filename);
        return await ScriptFileGenerator.Generate(
            new GeneratorSettings
            {
                OpenApiPath = swaggerFile,
                AuthorizationHeader = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"
            });
    }
}