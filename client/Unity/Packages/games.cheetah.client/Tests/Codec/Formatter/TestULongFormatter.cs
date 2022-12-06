using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestULongFormatter : AbstractUnmanagedFormatterTest<ulong, ULongFormatter>
    {
        protected override ulong[] GetValues()
        {
            return new[] { ulong.MinValue, ulong.MaxValue };
        }
    }
}