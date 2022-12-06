using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestUIntFormatter : AbstractUnmanagedFormatterTest<uint, UIntFormatter>
    {
        protected override uint[] GetValues()
        {
            return new[] { uint.MinValue, uint.MaxValue };
        }
    }
}