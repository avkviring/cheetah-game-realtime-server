using Cheetah.Matches.Relay.Codec.Formatter;

namespace Cheetah.Matches.Relay.Tests.Runtime.Codec.Formatter
{
    public class TestVariableSizeULongFormatter : AbstractVariableSizeFormatterTest<ulong, VariableSizeULongFormatter>
    {
        protected override ulong[] GetValues()
        {
            return new ulong[]
            {
                0,
                258,
                ushort.MaxValue,
                ushort.MaxValue * byte.MaxValue - 1UL,
                uint.MaxValue,
                (ulong)uint.MaxValue * byte.MaxValue * byte.MaxValue - 1UL,
                ulong.MaxValue
            };
        }

        protected override int[] GetSizes()
        {
            return new[]
            {
                1, 2, 3, 4, 5, 7, 9
            };
        }
    }
}