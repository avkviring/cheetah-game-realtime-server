using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestBoolFormatter : AbstractUnmanagedFormatterTest<bool, BoolFormatter>
    {
        protected override bool[] GetValues()
        {
            return new[] { true, false };
        }
    }
}