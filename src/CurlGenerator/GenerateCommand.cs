using System.Diagnostics;
using System.Linq;
using Azure.Core.Diagnostics;
using CurlGenerator.Core;
using CurlGenerator.Validation;
using Microsoft.OpenApi.Models;
using Microsoft.OpenApi.Readers.Exceptions;
using Spectre.Console;
using Spectre.Console.Cli;

namespace CurlGenerator;

public class GenerateCommand : AsyncCommand<Settings>
{
    private static readonly string Crlf = Environment.NewLine;

    public override async Task<int> ExecuteAsync(CommandContext context, Settings settings)
    {
        if (!settings.NoLogging)
            Analytics.Configure();

        try
        {
            var stopwatch = Stopwatch.StartNew();
            
            // Display banner
            AnsiConsole.MarkupLine($"[bold cyan]cURL Request Generator v{GetType().Assembly.GetName().Version!}[/]");
            AnsiConsole.MarkupLine(
                settings.NoLogging
                    ? "[dim]Support key: Unavailable when logging is disabled[/]"
                    : $"[dim]Support key: {SupportInformation.GetSupportKey()}[/]");
            AnsiConsole.WriteLine();

            if (!settings.SkipValidation)
                await ValidateOpenApiSpec(settings);

            await AcquireAzureEntraIdToken(settings);

            var generatorSettings = new GeneratorSettings
            {
                AuthorizationHeader = settings.AuthorizationHeader,
                OpenApiPath = settings.OpenApiPath,
                ContentType = settings.ContentType,
                BaseUrl = settings.BaseUrl,
                GenerateBashScripts = settings.GenerateBashScripts
            };

            var result = await ScriptFileGenerator.Generate(generatorSettings);
            await Analytics.LogFeatureUsage(settings);

            if (!string.IsNullOrWhiteSpace(settings.OutputFolder) && !Directory.Exists(settings.OutputFolder))
                Directory.CreateDirectory(settings.OutputFolder);

            await Task.WhenAll(
                result.Files.Select(
                    file => File.WriteAllTextAsync(
                        Path.Combine(settings.OutputFolder, file.Filename),
                        file.Content)));

            // Display success summary
            AnsiConsole.WriteLine();
            AnsiConsole.MarkupLine("[bold green]✓ Generation completed successfully![/]");
            AnsiConsole.WriteLine();
            
            // Display output information
            var outputPath = Path.GetFullPath(settings.OutputFolder);
            AnsiConsole.MarkupLine($"[cyan]Output directory:[/] {outputPath}");
            AnsiConsole.MarkupLine($"[cyan]Files generated:[/] {result.Files.Count}");
            AnsiConsole.MarkupLine($"[cyan]Script type:[/] {(settings.GenerateBashScripts ? "Bash (.sh)" : "PowerShell (.ps1)")}");
            AnsiConsole.MarkupLine($"[cyan]Duration:[/] {stopwatch.Elapsed:c}");
            
            // List generated files
            if (result.Files.Count > 0)
            {
                AnsiConsole.WriteLine();
                AnsiConsole.MarkupLine("[bold]Generated files:[/]");
                foreach (var file in result.Files.OrderBy(f => f.Filename))
                {
                    AnsiConsole.MarkupLine($"  [dim]•[/] {file.Filename}");
                }
            }
            
            AnsiConsole.WriteLine();
            return 0;
        }
        catch (OpenApiUnsupportedSpecVersionException exception)
        {
            AnsiConsole.MarkupLine($"{Crlf}[red]Error:{Crlf}{exception.Message}[/]");
            AnsiConsole.MarkupLine(
                $"{Crlf}[yellow]Tips:{Crlf}" +
                $"Consider using the --skip-validation argument.{Crlf}" +
                $"In some cases, the features that are specific to the " +
                $"unsupported versions of OpenAPI specifications aren't really used.{Crlf}" +
                $"This tool uses NSwag libraries to parse the OpenAPI document and " +
                $"Microsoft.OpenApi libraries for validation.{Crlf}{Crlf}[/]");
            return exception.HResult;
        }
        catch (Exception exception)
        {
            if (exception is not OpenApiValidationException)
            {
                AnsiConsole.MarkupLine($"{Crlf}[red]Error:{Crlf}{exception.Message}[/]");
                AnsiConsole.MarkupLine($"[red]Exception:{Crlf}{exception.GetType()}[/]");
                AnsiConsole.MarkupLine($"[yellow]Stack Trace:{Crlf}{exception.StackTrace}[/]");
            }

            await Analytics.LogError(exception, settings);
            return exception.HResult;
        }
    }

    private static async Task AcquireAzureEntraIdToken(Settings settings)
    {
        if (!string.IsNullOrWhiteSpace(settings.AuthorizationHeader) ||
            (string.IsNullOrWhiteSpace(settings.AzureScope) &&
             string.IsNullOrWhiteSpace(settings.AzureTenantId)))
        {
            return;
        }

        try
        {
            AnsiConsole.MarkupLine($"[yellow]Acquiring authorization header from Azure Entra ID...[/]");
            using var listener = AzureEventSourceListener.CreateConsoleLogger();
            var token = await AzureEntraID
                .TryGetAccessTokenAsync(
                    settings.AzureTenantId!,
                    settings.AzureScope!,
                    CancellationToken.None);

            if (!string.IsNullOrWhiteSpace(token))
            {
                settings.AuthorizationHeader = $"Bearer {token}";
                AnsiConsole.MarkupLine($"[green]✓ Successfully acquired access token[/]");
                AnsiConsole.WriteLine();
            }
        }
        catch (Exception exception)
        {
            AnsiConsole.MarkupLine($"{Crlf}[red]Error:{Crlf}{exception.Message}[/]");
        }
    }

    private static async Task ValidateOpenApiSpec(Settings settings)
    {
        var validationResult = await OpenApiValidator.Validate(settings.OpenApiPath!);
        if (!validationResult.IsValid)
        {
            AnsiConsole.MarkupLine($"[red]{Crlf}OpenAPI validation failed:{Crlf}[/]");

            foreach (var error in validationResult.Diagnostics.Errors)
            {
                TryWriteLine(error, "red", "Error");
            }

            foreach (var warning in validationResult.Diagnostics.Warnings)
            {
                TryWriteLine(warning, "yellow", "Warning");
            }

            validationResult.ThrowIfInvalid();
        }

        AnsiConsole.MarkupLine("[bold green]✓ OpenAPI specification validated successfully[/]");
        AnsiConsole.WriteLine();
        AnsiConsole.MarkupLine("[bold]OpenAPI Statistics:[/]");
        AnsiConsole.MarkupLine($"[dim]• Path Items:[/] {validationResult.Statistics.PathItemCount}");
        AnsiConsole.MarkupLine($"[dim]• Operations:[/] {validationResult.Statistics.OperationCount}");
        AnsiConsole.MarkupLine($"[dim]• Parameters:[/] {validationResult.Statistics.ParameterCount}");
        AnsiConsole.MarkupLine($"[dim]• Request Bodies:[/] {validationResult.Statistics.RequestBodyCount}");
        AnsiConsole.MarkupLine($"[dim]• Responses:[/] {validationResult.Statistics.ResponseCount}");
        AnsiConsole.MarkupLine($"[dim]• Schemas:[/] {validationResult.Statistics.SchemaCount}");
        AnsiConsole.WriteLine();
    }

    private static void TryWriteLine(
        OpenApiError error,
        string color,
        string label)
    {
        try
        {
            AnsiConsole.MarkupLine($"[{color}]{label}:{Crlf}{error}{Crlf}[/]");
        }
        catch
        {
            // ignored
        }
    }
}