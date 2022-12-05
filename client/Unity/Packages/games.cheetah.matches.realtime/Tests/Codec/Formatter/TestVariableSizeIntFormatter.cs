using Cheetah.Matches.Realtime.Codec.Formatter;

namespace Cheetah.Matches.Realtime.Tests.Codec.Formatter
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