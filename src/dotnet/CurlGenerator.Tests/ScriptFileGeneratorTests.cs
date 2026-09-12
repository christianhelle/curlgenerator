
using CurlGenerator.Core;
using CurlGenerator.Tests.Resources;
using FluentAssertions;
using FluentAssertions.Execution;

namespace CurlGenerator.Tests;

public class ScriptFileGeneratorTests
{
    [Fact]
    public async Task Generate_WithBashScripts_GeneratesShFiles()
    {
        var json = EmbeddedResources.GetSwaggerPetstore(Samples.PetstoreJsonV3);
        var swaggerFile = await TestFile.CreateSwaggerFile(json, "SwaggerPetstore.json");

        var result = await ScriptFileGenerator.Generate(
            new GeneratorSettings
            {
                OpenApiPath = swaggerFile,
                GenerateBashScripts = true
            });

        using var scope = new AssertionScope();
        result.Should().NotBeNull();
        result.Files.Should().NotBeNullOrEmpty();
        result.Files.Should().OnlyContain(f => f.Filename.EndsWith(".sh"));
    }

    [Fact]
    public async Task Generate_WithNoServers_UsesBaseUrl()
    {
        var json = EmbeddedResources.GetSwaggerPetstore(Samples.PetstoreJsonV3);
        var swaggerFile = await TestFile.CreateSwaggerFile(json, "SwaggerPetstore.json");

        var result = await ScriptFileGenerator.Generate(
            new GeneratorSettings
            {
                OpenApiPath = swaggerFile,
                BaseUrl = "http://my-custom-base-url.com"
            });

        using var scope = new AssertionScope();
        result.Should().NotBeNull();
        result.Files.Should().NotBeNullOrEmpty();
        result.Files.Should().Contain(f => f.Content.Contains("http://my-custom-base-url.com"));
    }
}
