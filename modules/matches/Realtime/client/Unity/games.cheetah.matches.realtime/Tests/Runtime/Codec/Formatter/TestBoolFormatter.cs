using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestBoolFormatter : AbstractUnmanagedFormatterTest<bool, BoolFormatter>
    {
        protected override bool[] GetValues()
        {
            return new[] { true, false };
        }
    }
}