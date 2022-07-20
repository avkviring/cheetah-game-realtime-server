using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestShortFormatter : AbstractUnmanagedFormatterTest<short, ShortFormatter>
    {
        protected override short[] GetValues()
        {
            return new[] { short.MinValue, short.MaxValue };
        }
    }
}