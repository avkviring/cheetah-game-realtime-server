using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestVariableSizeUIntFormatter : AbstractVariableSizeFormatterTest<uint, VariableSizeUIntFormatter>
    {
        protected override uint[] GetValues()
        {
            return new uint[]
            {
                0,
                258,
                ushort.MaxValue,
                (uint)(ushort.MaxValue * byte.MaxValue - 1UL),
                uint.MaxValue
            };
        }

        protected override int[] GetSizes()
        {
            return new[]
            {
                1, 2, 3, 4, 5
            };
        }
    }
}