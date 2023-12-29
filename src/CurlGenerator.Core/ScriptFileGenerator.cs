using System.Text;
using NSwag;
using NSwag.CodeGeneration.CSharp;

namespace CurlGenerator.Core;

public static class ScriptFileGenerator
{
    public static async Task<GeneratorResult> Generate(GeneratorSettings settings)
    {
        var document = await OpenApiDocumentFactory.CreateAsync(settings.OpenApiPath);
        var generator = new CSharpClientGenerator(document, new CSharpClientGeneratorSettings());
        generator.BaseSettings.OperationNameGenerator = new OperationNameGenerator();

        var baseUrl = settings.BaseUrl + document.Servers?.FirstOrDefault()?.Url;
        if (!Uri.IsWellFormedUriString(baseUrl, UriKind.Absolute) &&
            settings.OpenApiPath.StartsWith("http", StringComparison.OrdinalIgnoreCase))
        {
            baseUrl = new Uri(settings.OpenApiPath)
                          .GetLeftPart(UriPartial.Authority) +
                      baseUrl;
        }

        return settings.OutputType == OutputType.OneRequestPerFile
            ? GenerateMultipleFiles(settings, document, generator, baseUrl)
            : GenerateSingleFile(settings, document, baseUrl);
    }

    private static GeneratorResult GenerateSingleFile(
        GeneratorSettings settings,
        OpenApiDocument document,
        string baseUrl)
    {
        var code = new StringBuilder();

        foreach (var kv in document.Paths)
        {
            foreach (var operations in kv.Value)
            {
                code.AppendLine(
                    GenerateRequest(
                        settings,
                        baseUrl,
                        operations.Key.CapitalizeFirstCharacter(),
                        kv,
                        operations.Value));
            }
        }

        return new GeneratorResult(
            new[] { new ScriptFile("Requests.ps1", code.ToString()) });
    }

    private static GeneratorResult GenerateMultipleFiles(
        GeneratorSettings settings,
        OpenApiDocument document,
        CSharpClientGenerator generator,
        string baseUrl)
    {
        var files = new List<ScriptFile>();
        foreach (var kv in document.Paths)
        {
            foreach (var operations in kv.Value)
            {
                var operation = operations.Value;
                var verb = operations.Key.CapitalizeFirstCharacter();
                var name = generator
                    .BaseSettings
                    .OperationNameGenerator
                    .GetOperationName(document, kv.Key, verb, operation);
                var filename = $"{name.CapitalizeFirstCharacter()}.ps1";

                var code = new StringBuilder();
                code.AppendLine(GenerateRequest(settings, baseUrl, verb, kv, operation));

                files.Add(new ScriptFile(filename, code.ToString()));
            }
        }

        return new GeneratorResult(files);
    }

    private static string GenerateRequest(
        GeneratorSettings settings,
        string baseUrl,
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation)
    {
        var code = new StringBuilder();
        AppendSummary(verb, kv, operation, code);
        code.AppendLine($"curl -X {verb.ToUpperInvariant()} {baseUrl}{kv.Key} `");
        code.AppendLine($"  -H 'Accept: {settings.ContentType}' `");
        code.AppendLine($"  -H 'Content-Type: {settings.ContentType}' `");

        if (!string.IsNullOrWhiteSpace(settings.AuthorizationHeader))
        {
            code.AppendLine($"  -H 'Authorization: {settings.AuthorizationHeader}' `");
        }

        var contentType = operation.RequestBody?.Content?.Keys
            ?.FirstOrDefault(c => c.Contains(settings.ContentType));

        if (operation.RequestBody?.Content is null || contentType is null)
            return code.ToString();

        var requestBody = operation.RequestBody;
        var requestBodySchema = requestBody.Content[contentType].Schema.ActualSchema;
        var requestBodyJson = requestBodySchema?.ToSampleJson()?.ToString() ?? string.Empty;

        code.AppendLine($"  -d '{requestBodyJson}'");
        return code.ToString();
    }

    private static void AppendSummary(
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation,
        StringBuilder code)
    {
        const int padding = 2;
        const string summary = "### Summary: ";
        const string description = "### Description: ";

        var request = $"### Request: {verb.ToUpperInvariant()} {kv.Key}";
        var length = request.Length + padding;
        length = Math.Max(
            length,
            Math.Max(
                (operation.Summary?.Length ?? 0) + summary.Length + padding,
                (operation.Description?.Length ?? 0) + description.Length + padding));

        for (var i = 0; i < length; i++)
        {
            code.Insert(0, "#");
        }

        code.AppendLine();
        code.AppendLine(request);

        if (!string.IsNullOrWhiteSpace(operation.Summary))
        {
            code.AppendLine($"{summary}{operation.Summary}");
        }

        if (!string.IsNullOrWhiteSpace(operation.Description))
        {
            code.AppendLine($"{description}{operation.Description}");
        }

        for (var i = 0; i < length; i++)
        {
            code.Insert(code.Length, "#");
        }

        code.AppendLine(Environment.NewLine);
    }
}