using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestShortFormatter : AbstractUnmanagedFormatterTest<short, ShortFormatter>
    {
        protected override short[] GetValues()
        {
            return new[] { short.MinValue, short.MaxValue };
        }
    }
}