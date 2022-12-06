using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestLongFormatter : AbstractUnmanagedFormatterTest<long, LongFormatter>
    {
        protected override long[] GetValues()
        {
            return new[] { long.MinValue, long.MaxValue };
        }
    }
}