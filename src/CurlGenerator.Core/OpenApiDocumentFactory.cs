using System.Net;
using Microsoft.OpenApi;
using Microsoft.OpenApi.Reader;
using Microsoft.OpenApi.YamlReader;

namespace CurlGenerator.Core;

/// <summary>
/// Creates an <see cref="OpenApiDocument"/> from a specified path or URL.
/// </summary>
public static class OpenApiDocumentFactory
{
    /// <summary>
    /// Creates a new instance of the <see cref="OpenApiDocument"/> class asynchronously.
    /// </summary>
    /// <returns>A new instance of the <see cref="OpenApiDocument"/> class.</returns>
    public static async Task<OpenApiDocument> CreateAsync(string openApiPath)
    {
        var fileInfo = new FileInfo(openApiPath);
        if (IsHttp(openApiPath))
        {
            var settings = new OpenApiReaderSettings
            {
                BaseUrl = new Uri(openApiPath)
            };

            using var content = await GetHttpContent(openApiPath);
            var reader = new OpenApiYamlReader();
            var readResult = await reader.ReadAsync(content, new Uri(openApiPath), settings);
            return readResult.Document!;
        }
        else
        {
            var settings = new OpenApiReaderSettings
            {
                BaseUrl = new Uri($"file://{fileInfo.DirectoryName}{Path.DirectorySeparatorChar}")
            };

            using var stream = File.OpenRead(openApiPath);
            var reader = new OpenApiYamlReader();
            var readResult = await reader.ReadAsync(stream, new Uri($"file://{fileInfo.FullName}"), settings);
            return readResult.Document!;
        }
    }

    /// <summary>
    /// Gets the content of the URI as a string and decompresses it if necessary. 
    /// </summary>
    /// <returns>The content of the HTTP request.</returns>
    private static async Task<Stream> GetHttpContent(string openApiPath)
    {
        var httpMessageHandler = new HttpClientHandler();
        httpMessageHandler.AutomaticDecompression = DecompressionMethods.GZip | DecompressionMethods.Deflate;
        httpMessageHandler.ServerCertificateCustomValidationCallback = (message, cert, chain, errors) => true;
        using var http = new HttpClient(httpMessageHandler);
        var content = await http.GetStreamAsync(openApiPath);
        return content;
    }

    /// <summary>
    /// Determines whether the specified path is an HTTP URL.
    /// </summary>
    /// <param name="path">The path to check.</param>
    /// <returns>True if the path is an HTTP URL, otherwise false.</returns>
    public static bool IsHttp(string path)
    {
        return path.StartsWith("http://", StringComparison.OrdinalIgnoreCase) || path.StartsWith("https://", StringComparison.OrdinalIgnoreCase);
    }
}