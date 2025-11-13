using System.Diagnostics;
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
            Analytics.Configure();        try
        {
            var stopwatch = Stopwatch.StartNew();
            
            // Display improved header
            DisplayHeader(settings);
            
            // Display configuration
            DisplayConfiguration(settings);

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
                Directory.CreateDirectory(settings.OutputFolder);            await Task.WhenAll(
                result.Files.Select(
                    file => File.WriteAllTextAsync(
                        Path.Combine(settings.OutputFolder, file.Filename),
                        file.Content)));

            DisplayResults(result, stopwatch.Elapsed, settings);
            return 0;
        }
        catch (OpenApiUnsupportedSpecVersionException exception)
        {
            var escapedMessage = exception.Message.Replace("[", "[[").Replace("]", "]]");
            AnsiConsole.MarkupLine($"{Crlf}[red]Error:{Crlf}{escapedMessage}[/]");
            AnsiConsole.MarkupLine(
                $"{Crlf}[yellow]Tips:{Crlf}" +
                $"Consider using the --skip-validation argument.{Crlf}" +
                $"In some cases, the features that are specific to the " +
                $"unsupported versions of OpenAPI specifications aren't really used.{Crlf}" +
                $"This tool uses Microsoft.OpenApi libraries for both parsing and validation.{Crlf}{Crlf}[/]");
            return exception.HResult;
        }
        catch (Exception exception)
        {
            if (exception is not OpenApiValidationException)
            {
                // Escape markup characters in exception messages
                var escapedMessage = exception.Message.Replace("[", "[[").Replace("]", "]]");
                var escapedStackTrace = exception.StackTrace?.Replace("[", "[[").Replace("]", "]]") ?? "";
                
                AnsiConsole.MarkupLine($"{Crlf}[red]Error:{Crlf}{escapedMessage}[/]");
                AnsiConsole.MarkupLine($"[red]Exception:{Crlf}{exception.GetType()}[/]");
                AnsiConsole.MarkupLine($"[yellow]Stack Trace:{Crlf}{escapedStackTrace}[/]");
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
        }        try
        {
            AnsiConsole.MarkupLine("[green]🔐 Acquiring authorization header from Azure Entra ID...[/]");
            using var listener = AzureEventSourceListener.CreateConsoleLogger();
            var token = await AzureEntraID
                .TryGetAccessTokenAsync(
                    settings.AzureTenantId!,
                    settings.AzureScope!,
                    CancellationToken.None);

            if (!string.IsNullOrWhiteSpace(token))
            {
                settings.AuthorizationHeader = $"Bearer {token}";
                AnsiConsole.MarkupLine("[green]✅ Successfully acquired access token[/]");
            }
        }
        catch (Exception exception)
        {
            var escapedMessage = exception.Message.Replace("[", "[[").Replace("]", "]]");
            AnsiConsole.MarkupLine($"{Crlf}[red]Error:{Crlf}{escapedMessage}[/]");
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

            validationResult.ThrowIfInvalid();        }

        DisplayOpenApiStatistics(validationResult.Statistics);
    }    private static void DisplayOpenApiStatistics(OpenApiStats statistics)
    {
        var statsTable = new Table()
            .BorderColor(Color.Blue)
            .AddColumn(new TableColumn("[bold]Component[/]").LeftAligned())
            .AddColumn(new TableColumn("[bold]Count[/]").RightAligned());

        statsTable.AddRow("📝 Path Items", $"[blue]{statistics.PathItemCount}[/]");
        statsTable.AddRow("⚙️  Operations", $"[blue]{statistics.OperationCount}[/]");
        statsTable.AddRow("📝 Parameters", $"[blue]{statistics.ParameterCount}[/]");
        statsTable.AddRow("📦 Request Bodies", $"[blue]{statistics.RequestBodyCount}[/]");
        statsTable.AddRow("📋 Responses", $"[blue]{statistics.ResponseCount}[/]");
        statsTable.AddRow("🔗 Links", $"[blue]{statistics.LinkCount}[/]");
        statsTable.AddRow("📞 Callbacks", $"[blue]{statistics.CallbackCount}[/]");
        statsTable.AddRow("📝 Schemas", $"[blue]{statistics.SchemaCount}[/]");

        AnsiConsole.Write(new Panel(statsTable)
            .Header("[bold blue]📊 OpenAPI Statistics[/]")
            .BorderColor(Color.Blue)
            .Padding(1, 0));
        AnsiConsole.WriteLine();
    }

    private static void TryWriteLine(
        OpenApiError error,
        string color,
        string label)
    {
        try
        {
            // Escape markup characters in the error message to prevent markup interpretation
            // Square brackets and curly braces need escaping for Spectre.Console markup
            var escapedError = error.ToString()
                .Replace("[", "[[")
                .Replace("]", "]]")
                .Replace("{", "{{")
                .Replace("}", "}}");
            AnsiConsole.MarkupLine($"[{color}]{label}:{Crlf}{escapedError}{Crlf}[/]");
        }
        catch
        {
            // ignored
        }
    }

    private static void DisplayHeader(Settings settings)
    {
        // Create a fancy header panel
        var version = typeof(GenerateCommand).Assembly.GetName().Version!.ToString();
        var headerText = new Text($"🔧 cURL Request Generator v{version}", new Style(Color.Green, decoration: Decoration.Bold));
        
        var panel = new Panel(headerText)
            .BorderColor(Color.Green)
            .Padding(1, 0)
            .Expand();
            
        AnsiConsole.Write(panel);
        AnsiConsole.WriteLine();
        
        // Support key info
        var supportKey = settings.NoLogging 
            ? "[yellow]⚠️  Unavailable when logging is disabled[/]"
            : $"[green]🔑 Support key: {SupportInformation.GetSupportKey()}[/]";
        AnsiConsole.MarkupLine(supportKey);
        AnsiConsole.WriteLine();
    }

    private static void DisplayConfiguration(Settings settings)
    {
        var configTable = new Table()
            .BorderColor(Color.Grey)
            .AddColumn(new TableColumn("[bold]Setting[/]").LeftAligned())
            .AddColumn(new TableColumn("[bold]Value[/]").LeftAligned());

        configTable.AddRow("📁 OpenAPI Source", $"[cyan]{settings.OpenApiPath}[/]");
        configTable.AddRow("📂 Output Folder", $"[cyan]{settings.OutputFolder}[/]");
        configTable.AddRow("🌐 Content Type", $"[cyan]{settings.ContentType}[/]");
        
        if (!string.IsNullOrWhiteSpace(settings.BaseUrl))
            configTable.AddRow("🔗 Base URL", $"[cyan]{settings.BaseUrl}[/]");
            
        if (settings.GenerateBashScripts)
            configTable.AddRow("🐚 Bash Scripts", "[green]✓ Enabled[/]");
            
        if (settings.SkipValidation)
            configTable.AddRow("⚠️  Validation", "[yellow]⚠️  Skipped[/]");

        if (!string.IsNullOrWhiteSpace(settings.AuthorizationHeader))
        {
            var authHeader = settings.AuthorizationHeader.Length > 50 
                ? settings.AuthorizationHeader[..47] + "..." 
                : settings.AuthorizationHeader;
            configTable.AddRow("🔐 Authorization", $"[dim]{authHeader}[/]");
        }

        AnsiConsole.Write(new Panel(configTable)
            .Header("[bold yellow]📋 Configuration[/]")
            .BorderColor(Color.Yellow)
            .Padding(1, 0));
        AnsiConsole.WriteLine();
    }

    private static void DisplayResults(GeneratorResult result, TimeSpan elapsed, Settings settings)
    {
        // Create a results table
        var resultsTable = new Table()
            .BorderColor(Color.Green)
            .AddColumn(new TableColumn("[bold]Metric[/]").LeftAligned())
            .AddColumn(new TableColumn("[bold]Value[/]").LeftAligned());

        resultsTable.AddRow("📄 Files Generated", $"[green]{result.Files.Count}[/]");
        resultsTable.AddRow("⏱️  Duration", $"[green]{elapsed.TotalMilliseconds:F0}ms[/]");
        resultsTable.AddRow("📁 Output Location", $"[cyan]{Path.GetFullPath(settings.OutputFolder)}[/]");

        if (result.Files.Any())
        {
            AnsiConsole.Write(new Panel(resultsTable)
                .Header("[bold green]✅ Generation Complete[/]")
                .BorderColor(Color.Green)
                .Padding(1, 0));

            // List generated files
            AnsiConsole.MarkupLine("[bold yellow]📁 Generated Files:[/]");
            foreach (var file in result.Files)
            {
                var fileSize = System.Text.Encoding.UTF8.GetByteCount(file.Content);
                var fileSizeText = fileSize switch
                {
                    < 1024 => $"{fileSize} bytes",
                    < 1024 * 1024 => $"{fileSize / 1024.0:F1} KB",
                    _ => $"{fileSize / (1024.0 * 1024.0):F1} MB"
                };
                // Escape markup characters in filename
                var escapedFilename = file.Filename.Replace("[", "[[").Replace("]", "]]");
                AnsiConsole.MarkupLine($"  📝 [cyan]{escapedFilename}[/] [dim]({fileSizeText})[/]");
            }
        }
        else
        {
            AnsiConsole.Write(new Panel(resultsTable)
                .Header("[bold yellow]⚠️  Generation Complete (No Files)[/]")
                .BorderColor(Color.Yellow)
                .Padding(1, 0));
        }
        
        AnsiConsole.WriteLine();
        AnsiConsole.MarkupLine("[bold green]🎉 Done![/]");
    }
}