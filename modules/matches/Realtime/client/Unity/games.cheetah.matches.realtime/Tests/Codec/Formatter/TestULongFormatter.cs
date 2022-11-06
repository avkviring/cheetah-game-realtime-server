using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestULongFormatter : AbstractUnmanagedFormatterTest<ulong, ULongFormatter>
    {
        protected override ulong[] GetValues()
        {
            return new[] { ulong.MinValue, ulong.MaxValue };
        }
    }
}