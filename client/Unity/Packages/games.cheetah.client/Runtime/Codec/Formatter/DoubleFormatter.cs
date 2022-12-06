using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class DoubleFormatter : UnmanagedFormatter<double>
    {
        public static readonly DoubleFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override unsafe double UncheckedRead(ref CheetahBuffer buffer)
        {
            var read = ULongFormatter.Instance.UncheckedRead(ref buffer);
            return *(double*)&read;
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override unsafe void UncheckedWrite(double value, ref CheetahBuffer buffer)
        {
            ULongFormatter.Instance.UncheckedWrite(*(ulong*)&value, ref buffer);
        }
    }
}