using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestShortFormatter : AbstractUnmanagedFormatterTest<short, ShortFormatter>
    {
        protected override short[] GetValues()
        {
            return new[] { short.MinValue, short.MaxValue };
        }
    }
}