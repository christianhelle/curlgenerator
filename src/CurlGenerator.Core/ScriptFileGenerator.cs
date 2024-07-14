namespace CurlGenerator.Core;

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using NSwag;
using NSwag.CodeGeneration.CSharp;

public static class ScriptFileGenerator
{
    private static readonly string LogFilePath = "generator.log";

    public static async Task<GeneratorResult> Generate(GeneratorSettings settings)
    {
        TryLog("Starting generation...");
        TryLog($"Settings: {SerializeObject(settings)}");

        var document = await OpenApiDocumentFactory.CreateAsync(settings.OpenApiPath);
        TryLog($"Document: {SerializeObject(document)}");

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

        TryLog($"Base URL: {baseUrl}");

        return GenerateCode(settings, document, generator, baseUrl);
    }

    private static GeneratorResult GenerateCode(
        GeneratorSettings settings,
        OpenApiDocument document,
        CSharpClientGenerator generator,
        string baseUrl)
    {
        var files = new List<ScriptFile>();
        foreach (var kv in document.Paths)
        {
            TryLog($"Processing path: {kv.Key}");
            foreach (var operations in kv.Value)
            {
                TryLog($"Processing operation: {operations.Key}");

                var operation = operations.Value;
                var verb = operations.Key.CapitalizeFirstCharacter();
                var name = generator
                    .BaseSettings
                    .OperationNameGenerator
                    .GetOperationName(document, kv.Key, verb, operation);

                var filename = !settings.GenerateBashScripts ? $"{name.CapitalizeFirstCharacter()}.ps1" : $"{name.CapitalizeFirstCharacter()}.sh";

                var code = new StringBuilder();
                if (!settings.GenerateBashScripts)
                {
                    code.AppendLine(GenerateRequest(settings, baseUrl, verb, kv, operation));
                }
                else
                {
                    code.AppendLine(GenerateBashRequest(settings, baseUrl, verb, kv, operation));
                }

                TryLog($"Generated code for {filename}:\n{code}");

                files.Add(new ScriptFile(filename, code.ToString()));
            }
        }

        return new GeneratorResult(files);
    }

    private static string GenerateBashRequest(
        GeneratorSettings settings,
        string baseUrl,
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation)
    {
        TryLog($"Generating bash request for operation: {operation.OperationId}");

        var code = new StringBuilder();
        AppendBashSummary(verb, kv, operation, code);
        AppendBashParameters(verb, kv, operation, code);

        var route = kv.Key.Replace("{", "$").Replace("}", null);

        // Add query parameters directly to the URL if there are any
        var queryParams = operation.Parameters
            .Where(p => p.Kind == OpenApiParameterKind.Query)
            .Select(p => $"{p.Name}=${{{p.Name}}}")
            .ToList();

        var queryString = queryParams.Any() ? $"?{string.Join("&", queryParams)}" : string.Empty;
        code.AppendLine($"curl -X {verb.ToUpperInvariant()} \"{baseUrl}{route}{queryString}\" \\");

        code.AppendLine($"  -H \"Accept: application/json\" \\");

        // Determine content type based on consumes or request body
        var contentType = operation.Consumes?.FirstOrDefault()
                          ?? operation.RequestBody?.Content?.Keys.FirstOrDefault()
                          ?? "application/json";

        TryLog($"Content type for operation {operation.OperationId}: {contentType}");
        code.AppendLine($"  -H \"Content-Type: {contentType}\" \\");

        if (!string.IsNullOrWhiteSpace(settings.AuthorizationHeader))
        {
            code.AppendLine($"  -H \"Authorization: {settings.AuthorizationHeader}\" \\");
        }

        if (operation.RequestBody?.Content != null)
        {
            if (contentType == "application/x-www-form-urlencoded" || contentType == "multipart/form-data")
            {
                var formData = operation.RequestBody.Content[contentType].Schema.Properties
                    .Select(p => $"-F \"{p.Key}=${{{p.Key}}}\"");
                foreach (var formField in formData)
                {
                    code.AppendLine(formField + " \\");
                }
            }
            else if (contentType == "application/octet-stream")
            {
                code.AppendLine($"  --data-binary '@filename'");
            }
            else
            {
                var requestBodySchema = operation.RequestBody.Content[contentType].Schema.ActualSchema;
                var requestBodyJson = requestBodySchema?.ToSampleJson()?.ToString() ?? string.Empty;
                code.AppendLine($"  -d '{requestBodyJson}'");
            }
        }
        else
        {
            // Remove the trailing backslash if there is no request body
            code.Length -= 2; // Remove the last backslash and newline
        }

        TryLog($"Generated bash request: {code}");

        return code.ToString();
    }

    private static void AppendBashParameters(
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation,
        StringBuilder code)
    {
        var parameters = operation.Parameters
            .Where(p => p.Kind == OpenApiParameterKind.Path || p.Kind == OpenApiParameterKind.Query || p.Kind == OpenApiParameterKind.Header || p.Kind == OpenApiParameterKind.Cookie)
            .ToArray();

        if (parameters.Length == 0)
        {
            code.AppendLine();
            return;
        }

        code.AppendLine();

        foreach (var parameter in parameters)
        {
            code.AppendLine(
                parameter.Description is null
                    ? $"# {parameter.Kind.ToString().ToLowerInvariant()} parameter: {parameter.Name}"
                    : $"# {parameter.Description}");

            code.AppendLine($"{parameter.Name}=\"\""); // Initialize the parameter
        }

        // Handle form data and file upload fields
        if (operation.RequestBody?.Content != null)
        {
            var contentType = operation.RequestBody.Content.Keys.FirstOrDefault() ?? "application/json";
            TryLog($"Request body content type for operation {operation.OperationId}: {contentType}");
            if (contentType == "application/x-www-form-urlencoded" || contentType == "multipart/form-data")
            {
                var formData = operation.RequestBody.Content[contentType].Schema.Properties
                    .Select(p => $"{p.Key}=\"\"");
                foreach (var formField in formData)
                {
                    code.AppendLine(formField);
                }
            }
        }

        code.AppendLine();
    }

    private static void AppendBashSummary(
        string verb,
        KeyValuePair<string, OpenApiPathItem> kv,
        OpenApiOperation operation,
        StringBuilder code)
    {
        code.AppendLine("#");
        code.AppendLine($"# Request: {verb.ToUpperInvariant()} {kv.Key}");

        if (!string.IsNullOrWhiteSpace(operation.Summary))
        {
            code.AppendLine($"# Summary: {operation.Summary}");
        }

        if (!string.IsNullOrWhiteSpace(operation.Description))
        {
            code.AppendLine($"# Description: {operation.Description}");
        }

        code.AppendLine("#");
    }

    private static void TryLog(string message)
    {
        try
        {
            using var writer = new StreamWriter(LogFilePath, true);
            writer.WriteLine($"{DateTime.Now}: {message}");
        }
        catch
        {
            // Ignore
        }
    }

    private static string SerializeObject(object obj)
    {
        return Newtonsoft.Json.JsonConvert.SerializeObject(obj, Newtonsoft.Json.Formatting.Indented);
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
        var parameterNameMap = AppendParameters(verb, kv, operation, code);

        var url = kv.Key.Replace("{", "$").Replace("}", null);
        if (!settings.GenerateBashScripts)
        {
            if (url.Contains("{") || url.Contains("}"))
            {
                foreach (var parameterName in parameterNameMap)
                {
                    url = url.Replace($"{{{{{parameterName.Key}}}}}", $"${parameterName.Value}");
                }
            }
            else
            {
                if (parameterNameMap.Count > 0)
                {
                    url += "?";
                }

                foreach (var parameterName in parameterNameMap)
                {
                    url += $"{parameterName.Key}=${parameterName.Value}&";
                }

                url = url.Remove(url.Length - 1);
            }
        }

        code.AppendLine($"curl -X {verb.ToUpperInvariant()} {baseUrl}{url} `");

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

    private static Dictionary<string, string> AppendParameters(
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
            return new Dictionary<string, string>();
        }

        code.AppendLine("param(");

        var parameterNameMap = new Dictionary<string, string>();
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
            parameterNameMap[parameter.Name] = parameter.Name;
        }
        code.Remove(code.Length - 5, 3);

        code.AppendLine(")");
        code.AppendLine();

        return parameterNameMap;
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
