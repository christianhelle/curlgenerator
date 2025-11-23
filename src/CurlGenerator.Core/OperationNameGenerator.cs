using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using Microsoft.OpenApi;

namespace CurlGenerator.Core;

internal interface IOperationNameGenerator
{
    string GetOperationName(
        OpenApiDocument document,
        string path,
        string httpMethod,
        OpenApiOperation operation);
}

public class OperationNameGenerator : IOperationNameGenerator
{
    [ExcludeFromCodeCoverage]
    public bool SupportsMultipleClients => false;

    [ExcludeFromCodeCoverage]
    public string GetClientName(OpenApiDocument document, string path, string httpMethod, OpenApiOperation operation)
    {
        return "ApiClient";
    }

    public string GetOperationName(
        OpenApiDocument document,
        string path,
        string httpMethod,
        OpenApiOperation operation)
    {
        try
        {
            // Try to use operationId if available
            if (!string.IsNullOrWhiteSpace(operation.OperationId))
            {
                return operation.OperationId!
                    .CapitalizeFirstCharacter()
                    .ConvertKebabCaseToPascalCase()
                    .ConvertRouteToCamelCase()
                    .ConvertSpacesToPascalCase()
                    .Prefix(
                        httpMethod
                            .ToLowerInvariant()
                            .CapitalizeFirstCharacter());
            }
            
            // Fallback to generating from path and method
            return httpMethod.CapitalizeFirstCharacter() + 
                   path.ConvertRouteToCamelCase()
                       .ConvertSpacesToPascalCase();
        }
        catch (Exception e)
        {
            Trace.TraceError(e.ToString());
            return httpMethod.CapitalizeFirstCharacter() + 
                   path.ConvertRouteToCamelCase()
                       .ConvertSpacesToPascalCase();
        }
    }

    public bool CheckForDuplicateOperationIds(
        OpenApiDocument document)
    {
        List<string> operationNames = new();
        foreach (var kv in document.Paths)
        {
            foreach (var operations in kv.Value.Operations ?? [])
            {
                var operation = operations.Value;
                operationNames.Add(
                    GetOperationName(
                        document,
                        kv.Key,
                        operations.Key.ToString(),
                        operation));
            }
        }

        return operationNames.Distinct().Count() != operationNames.Count;
    }
}