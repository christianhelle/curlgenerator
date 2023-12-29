using FluentAssertions;

namespace CurlGenerator.Tests;

public class SupportInformationTests
{
    [Fact]
    public void Should_Return_GetSupportKey()
    {
        SupportInformation
            .GetSupportKey()
            .Should()
            .NotBeNullOrEmpty();
    }
}