using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestULongFormatter : AbstractUnmanagedFormatterTest<ulong, ULongFormatter>
    {
        protected override ulong[] GetValues()
        {
            return new[] { ulong.MinValue, ulong.MaxValue };
        }
    }
}