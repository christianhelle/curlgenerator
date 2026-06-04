
using CurlGenerator.Core;
using FluentAssertions;

namespace CurlGenerator.Tests;

public class StringExtensionsTests
{
    [Theory]
    [InlineData("kebab-case-string", "KebabCaseString")]
    [InlineData("another-kebab-case.string", "AnotherKebabCase_string")]
    [InlineData("single", "Single")]
    public void ConvertKebabCaseToPascalCase_Should_Convert_Correctly(string input, string expected)
    {
        input.ConvertKebabCaseToPascalCase().Should().Be(expected);
    }

    [Theory]
    [InlineData("kebab-case-string", "kebab_case_string")]
    [InlineData("another-kebab-case-string", "another_kebab_case_string")]
    [InlineData("single", "single")]
    public void ConvertKebabCaseToSnakeCase_Should_Convert_Correctly(string input, string expected)
    {
        input.ConvertKebabCaseToSnakeCase().Should().Be(expected);
    }

    [Theory]
    [InlineData("/route/to/resource", "RouteToResource")]
    [InlineData("/another/route/to/another/resource", "AnotherRouteToAnotherResource")]
    [InlineData("/single", "Single")]
    public void ConvertRouteToCamelCase_Should_Convert_Correctly(string input, string expected)
    {
        input.ConvertRouteToCamelCase().Should().Be(expected);
    }

    [Theory]
    [InlineData("string", "String")]
    [InlineData("anotherString", "AnotherString")]
    [InlineData("s", "S")]
    public void CapitalizeFirstCharacter_Should_Capitalize_Correctly(string input, string expected)
    {
        input.CapitalizeFirstCharacter().Should().Be(expected);
    }

    [Theory]
    [InlineData("space separated string", "SpaceSeparatedString")]
    [InlineData("another space separated string", "AnotherSpaceSeparatedString")]
    [InlineData("single", "Single")]
    public void ConvertSpacesToPascalCase_Should_Convert_Correctly(string input, string expected)
    {
        input.ConvertSpacesToPascalCase().Should().Be(expected);
    }

    [Theory]
    [InlineData("string", "prefix", "prefixstring")]
    [InlineData("prefixstring", "prefix", "prefixstring")]
    public void Prefix_Should_Add_Prefix_Correctly(string input, string prefix, string expected)
    {
        input.Prefix(prefix).Should().Be(expected);
    }
}
