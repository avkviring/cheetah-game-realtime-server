using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestStringFormatter : AbstractFormatterTest<string, StringFormatter>
    {
        protected override string[] GetValues()
        {
            return new[] { "hello", "привет" };
        }
    }
}