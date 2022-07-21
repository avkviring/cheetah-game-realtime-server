using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestIntFormatter : AbstractUnmanagedFormatterTest<int, IntFormatter>
    {
        protected override int[] GetValues()
        {
            return new[] { int.MinValue, int.MaxValue };
        }
    }
}