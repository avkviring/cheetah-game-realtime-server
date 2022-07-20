using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestLongFormatter : AbstractUnmanagedFormatterTest<long, LongFormatter>
    {
        protected override long[] GetValues()
        {
            return new[] { long.MinValue, long.MaxValue };
        }
    }
}