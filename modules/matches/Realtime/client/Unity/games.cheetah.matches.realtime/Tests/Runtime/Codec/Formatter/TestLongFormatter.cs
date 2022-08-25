using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestLongFormatter : AbstractUnmanagedFormatterTest<long, LongFormatter>
    {
        protected override long[] GetValues()
        {
            return new[] { long.MinValue, long.MaxValue };
        }
    }
}