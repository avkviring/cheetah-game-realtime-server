using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestFloatFormatter : AbstractUnmanagedFormatterTest<float, FloatFormatter>
    {
        protected override float[] GetValues()
        {
            return new[] { 3.141f, 2.718f };
        }
    }
}