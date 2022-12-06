using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestStringFormatter : AbstractFormatterTest<string, StringFormatter>
    {
        protected override string[] GetValues()
        {
            return new[] { "hello", "привет" };
        }
    }
}