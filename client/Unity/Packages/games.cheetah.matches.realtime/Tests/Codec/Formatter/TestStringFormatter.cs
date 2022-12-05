using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestStringFormatter : AbstractFormatterTest<string, StringFormatter>
    {
        protected override string[] GetValues()
        {
            return new[] { "hello", "привет" };
        }
    }
}