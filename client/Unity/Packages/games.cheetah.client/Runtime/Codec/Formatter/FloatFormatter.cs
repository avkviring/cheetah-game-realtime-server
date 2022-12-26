using System.Runtime.CompilerServices;
using Games.Cheetah.Client.Types;
using Games.Cheetah.Client.Types.Field;

namespace Games.Cheetah.Client.Codec.Formatter
{
    public sealed class FloatFormatter : UnmanagedFormatter<float>
    {
        public static readonly FloatFormatter Instance = new();

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override float UncheckedRead(ref NetworkBuffer buffer)
        {
            return StaticUncheckedRead(ref buffer);
        }


        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public override void UncheckedWrite(float value, ref NetworkBuffer buffer)
        {
            StaticUncheckedWrite(value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe void StaticUncheckedWrite(float value, ref NetworkBuffer buffer)
        {
            UIntFormatter.StaticUncheckedWrite(*(uint*)&value, ref buffer);
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        public static unsafe float StaticUncheckedRead(ref NetworkBuffer buffer)
        {
            var read = UIntFormatter.StaticUncheckedRead(ref buffer);
            return *(float*)&read;
        }
    }
}