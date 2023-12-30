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
        AppendParameters(verb, kv, operation, code);

        var route = kv.Key.Replace("{", "$").Replace("}", null);
        code.AppendLine($"curl -X {verb.ToUpperInvariant()} {baseUrl}{route} `");

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

    private static void AppendParameters(
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation,
        StringBuilder code)
    {
        var parameters = operation
            .Parameters
            .Where(c => c.Kind is OpenApiParameterKind.Path or OpenApiParameterKind.Query)
            .ToArray();

        if (parameters.Length == 0)
        {
            code.AppendLine();
            return;
        }

        code.AppendLine("param(");
        
        foreach (var parameter in parameters)
        {
            code.AppendLine(
                parameter.Description is null
                    ? $"""
                         [Parameter(Mandatory=$True)]
                         [String] ${parameter.Name},
                       """
                    : $"""
                          <# {parameter.Description} #>
                          [Parameter(Mandatory=$True)]
                          [String] ${parameter.Name},
                       """);
            code.AppendLine();
        }
        code.Remove(code.Length - 5, 3);

        code.AppendLine(")");
        code.AppendLine();
    }

    private static void AppendSummary(
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation,
        StringBuilder code)
    {
        code.AppendLine("<#");
        code.AppendLine($"  Request: {verb.ToUpperInvariant()} {kv.Key}");

        if (!string.IsNullOrWhiteSpace(operation.Summary))
        {
            code.AppendLine($"  Summary: {operation.Summary}");
        }

        if (!string.IsNullOrWhiteSpace(operation.Description))
        {
            code.AppendLine($"  Description: {operation.Description}");
        }

        code.AppendLine("#>");
    }
}