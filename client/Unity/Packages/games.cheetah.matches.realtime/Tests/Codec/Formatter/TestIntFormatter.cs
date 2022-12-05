using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestIntFormatter : AbstractUnmanagedFormatterTest<int, IntFormatter>
    {
        protected override int[] GetValues()
        {
            return new[] { int.MinValue, int.MaxValue };
        }
    }
}