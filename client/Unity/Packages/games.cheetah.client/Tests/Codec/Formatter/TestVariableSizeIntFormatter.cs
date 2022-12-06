using Games.Cheetah.Client.Codec.Formatter;

namespace Games.Cheetah.Client.Tests.Codec.Formatter
{
    public class TestVariableSizeIntFormatter : AbstractVariableSizeFormatterTest<int, VariableSizeIntFormatter>
    {
        protected override int[] GetValues()
        {
            return new[]
            {
                -120,
                short.MinValue,
                short.MaxValue,
                int.MinValue,
                int.MaxValue
            };
        }

        protected override int[] GetSizes()
        {
            return new[]
            {
                1, 3, 3, 5, 5
            };
        }
    }
}