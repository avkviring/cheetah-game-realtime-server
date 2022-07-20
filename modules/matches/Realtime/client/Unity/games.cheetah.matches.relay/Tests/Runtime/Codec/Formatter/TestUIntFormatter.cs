using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestUIntFormatter : AbstractUnmanagedFormatterTest<uint, UIntFormatter>
    {
        protected override uint[] GetValues()
        {
            return new[] { uint.MinValue, uint.MaxValue };
        }
    }
}