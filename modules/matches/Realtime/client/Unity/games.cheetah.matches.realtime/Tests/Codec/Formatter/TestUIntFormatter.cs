using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestUIntFormatter : AbstractUnmanagedFormatterTest<uint, UIntFormatter>
    {
        protected override uint[] GetValues()
        {
            return new[] { uint.MinValue, uint.MaxValue };
        }
    }
}