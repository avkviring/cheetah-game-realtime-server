using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestIntFormatter : AbstractUnmanagedFormatterTest<int, IntFormatter>
    {
        protected override int[] GetValues()
        {
            return new[] { int.MinValue, int.MaxValue };
        }
    }
}