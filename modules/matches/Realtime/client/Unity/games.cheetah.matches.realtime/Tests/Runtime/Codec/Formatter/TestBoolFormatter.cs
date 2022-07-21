using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestBoolFormatter : AbstractUnmanagedFormatterTest<bool, BoolFormatter>
    {
        protected override bool[] GetValues()
        {
            return new[] { true, false };
        }
    }
}