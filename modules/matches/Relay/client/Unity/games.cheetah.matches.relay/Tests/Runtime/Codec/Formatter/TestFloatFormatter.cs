using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestFloatFormatter : AbstractUnmanagedFormatterTest<float, FloatFormatter>
    {
        protected override float[] GetValues()
        {
            return new[] { 3.141f, 2.718f };
        }
    }
}