using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
{
    public class TestFloatFormatter : AbstractUnmanagedFormatterTest<float, FloatFormatter>
    {
        protected override float[] GetValues()
        {
            return new[] { 3.141f, 2.718f };
        }
    }
}